//! `hdrhist` is a small footprint [hdr histogram](https://hdrhistogram.github.io/HdrHistogram/).
//!
//! It collects `u64` samples in the full `u64` value range with precision of 5 most significant bits. You can add new samples in `O(1)` time (a handful of cycles), and it will never reallocate.

#![deny(missing_docs)]

const HDHISTOGRAM_BITS: usize = 4;

/// An hdr histogram that collects `u64` sample with 5 bit precision.
#[derive(Clone)]
pub struct HDRHist {
    counts: [[u64; 1 << HDHISTOGRAM_BITS]; 64 - HDHISTOGRAM_BITS + 1],
}

impl HDRHist {
    /// Construct an empty hdr histogram.
    pub fn new() -> Self {
        HDRHist {
            counts: [[0u64; 1 << HDHISTOGRAM_BITS]; 64 - HDHISTOGRAM_BITS + 1],
        }
    }

    /// Add a sample to the histogram.
    ///
    /// This is guaranteed to be constant time and never re-allocates.
    pub fn add_value(&mut self, value: u64) {
        let msb: usize = 64usize - value.leading_zeros() as usize;
        let index = msb.saturating_sub(HDHISTOGRAM_BITS);
        let low_bits = value >> (index.saturating_sub(1)) & ((1 << HDHISTOGRAM_BITS) - 1);
        self.counts[index][low_bits as usize] += 1;
    }

    /// Combine this histogram with another. This doesn't normalize, and only adds the per-bucket
    /// counts. Only use it for histograms that have captured a comparable number of samples.
    pub fn combined(mut self, other: Self) -> Self {
        for (bs, bo) in self.counts.iter_mut().zip(other.counts.iter()).flat_map(
            |(ss, os)| ss.iter_mut().zip(os.iter())) {

            *bs += bo;
        }
        self
    }

    /// Output the complementary cumulative distribution function (ccdf) of the samples
    /// 
    /// Returns an iterator over increasing sample values such that, for every triple
    /// `(value, prob, count)`, `prob` is the ratio of samples >= `value`, and
    /// `count` is the number of samples >= the current `value` and < the next `value`.
    pub fn ccdf<'a>(&'a self) -> impl Iterator<Item=(u64, f64, u64)>+'a {
        let flattened = self.counts.iter().enumerate().flat_map(move |(index, bucket)| {
            bucket.iter().enumerate().map(move |(sub, count)| (index, sub, count))
        });
        let (first, last, total) = if let Some((first, _)) = flattened.clone().enumerate().find(|&(_, (_, _, c))| *c > 0) {
            let last = flattened.clone().enumerate().fold(0, |acc, (i, (_, _, c))| {
                if *c > 0 {
                    i
                } else {
                    acc
                }
            });
            let total: u64 = self.counts.iter().map(|x| x.iter().sum::<u64>()).sum();
            (first, last, total)
        } else { (2, 0, 0) };

        let mut sum: u64 = 0;
        flattened.take(last + 2).skip(first).map(move |(index, sub, count)| {
            let value = if index > 0 {
                (1u64 << (index + HDHISTOGRAM_BITS - 1)).saturating_add((sub as u64) << (index - 1))
            } else {
                sub as u64
            };
            let fraction = (total - sum) as f64 / (total as f64);
            sum += count;
            (value, fraction, *count)
        })
    }

    /// Outputs an upper bound of the complementary cumulative distribution function (ccdf) of the samples.
    ///
    /// You can use these points to plot with linear interpolation and never report intermediate
    /// sample values that underestimate quantiles. In other words, all the actual quantile values will be below the
    /// reported curve.
    pub fn ccdf_upper_bound<'a>(&'a self) -> impl Iterator<Item=(u64, f64)>+'a {
        let mut ccdf = self.ccdf();
        let mut cur_f = ccdf.next().map(|(_, f, _)| f);
        ccdf.map(move |(v, f, _)| {
            let prev_f = cur_f.unwrap();
            cur_f = Some(f);
            (v, prev_f)
        })
    }

    /// Outputs an upper bound of the complementary cumulative distribution function (ccdf) of the samples.
    ///
    /// You can use these points to plot with linear interpolation and never report intermediate
    /// sample values that overestimate quantiles. In other words, all the actual quantile values will be above the
    /// reported curve.
    pub fn ccdf_lower_bound<'a>(&'a self) -> impl Iterator<Item=(u64, f64)>+'a {
        let mut ccdf = self.ccdf();
        let mut cur_v = ccdf.next().map(|(v, _, _)| v);
        ccdf.map(move |(v, f, _)| {
            let prev_v = cur_v.unwrap();
            cur_v = Some(v);
            (prev_v, f)
        })
    }

    /// Output estimated quantiles as (quantile, lower_bound, upper_bound) pairs
    ///
    /// Each quantile's value is somewhere >= lower_bound and < upper_bound
    pub fn quantiles<'a>(&'a self, quantiles: impl Iterator<Item=f64>+'a) -> impl Iterator<Item=(f64, u64, u64)>+'a {
        let mut ccdf = self.ccdf();
        let mut prev_v = 0;
        let (mut cur_f, mut cur_v) = (1.0, 0);
        quantiles.map(move |p| {
            let pf = 1f64 - p;
            while cur_f > pf {
                if let Some((v, f, _)) = ccdf.next() {
                    prev_v = cur_v;
                    cur_f = f;
                    cur_v = v;
                } else {
                    break;
                }
            }
            (p, prev_v, cur_v)
        })
    }

    /// Output a summary of estimated quantiles
    pub fn summary<'a>(&'a self) -> impl Iterator<Item=(f64, u64, u64)>+'a {
        let summary_quantiles = [0.25, 0.50, 0.75, 0.95, 0.99, 0.999, 1.0].iter().map(|x| *x);
        self.quantiles(summary_quantiles)
    }

    /// Output a text summary of estimated quantiles
    pub fn summary_string(&self) -> String {
        let mut values_lower: Vec<String> = vec!["╭ ".to_string()];
        let mut values_upper: Vec<String> = vec!["| ".to_string()];
        let mut points: Vec<String> =       vec!["╰ ".to_string()];
        for (p, lower, upper) in self.summary() {
            if p == 0.25 {
                points.push("[".to_string());
            } else if p == 0.95 {
                points.push("]".to_string());
            } else if p < 0.95 {
                points.push(" ".to_string());
            } else {
                points.push("-".to_string());
            }
            values_lower.push(" ".to_string());
            values_upper.push(" ".to_string());
            if p < 0.95 {
                points.push(format!("    {:<5}    ", p));
            } else if p != 1.0 {
                points.push(format!("--- {:<5} ---", p));
            } else {
                points.push("---| max     ".to_string());
            }
            values_lower.push(format!(" {:^11.6e} ", lower as f64));
            values_upper.push(format!(" {:^11.6e} ", upper as f64));
        }
        let mut res = values_lower;
        res.push("╮\n".to_string());
        res.extend(values_upper.into_iter());
        res.push("|\n".to_string());
        res.extend(points.into_iter());
        res.push("╯".to_string());
        res.join("")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_panic_on_empty_hist() {
        let hist = crate::HDRHist::new();
        assert_eq!(hist.ccdf().next(), None);
        assert_eq!(hist.summary().last(), Some((1.0, 0, 0)));
        assert!(hist.summary_string().len() > 0);
    }

    #[test]
    fn single_value() {
        let mut hist = crate::HDRHist::new();
        hist.add_value(1_000_000);
        let ccdf = hist.ccdf().collect::<Vec<_>>();
        assert_eq!(
            ccdf.iter().find(|(v, _, _)| v >= &1_000_000).map(|(_, p, _)| p),
            Some(&0.0));
        assert_eq!(
            ccdf.iter().take_while(|(v, _, _)| v < &1_000_000).last().map(|(_, p, _)| p),
            Some(&1.0));
    }

    #[test]
    fn quartiles() {
        let mut hist = crate::HDRHist::new();
        // [--------+---------+--------]
        // |      |      |      |      |
        for _ in 0..1_000_000 {
            hist.add_value(100_000_000);
            hist.add_value(200_000_000);
            hist.add_value(300_000_000);
        }
        let quantiles = hist.quantiles([0.25, 0.5, 0.75, 1.0].iter().cloned()).collect::<Vec<_>>();
        dbg!(&quantiles);
        for (p, v) in [
            (0.25, 100_000_000),
            (0.5,  200_000_000),
            (0.75, 300_000_000),
            (1.0,  300_000_000),
        ].iter() {
            assert_eq!(
                quantiles.iter().find(|(f, _, _)| f == p)
                    .map(|(_, a, b)| a <= v && b > v),
                Some(true));
        }
    }

    // test based on https://github.com/utaal/hdrhist-rust/issues/3
    #[test]
    fn boundary_test() {
        let mut hist = crate::HDRHist::new();
        for _ in 0..2400 { hist.add_value(1); }
        for _ in 0..200  { hist.add_value(2); }
        for _ in 0..7400 { hist.add_value(3); }
        hist.add_value(5);
        assert_eq!(
            hist.summary().collect::<Vec<_>>(),
            vec![(0.25, 2, 3),
                (0.5, 3, 4),
                (0.75, 3, 4),
                (0.95, 3, 4),
                (0.99, 3, 4),
                (0.999, 3, 4),
                (1.0, 5, 6)]);
    }
}
