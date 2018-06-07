const HDHISTOGRAM_BITS: usize = 4;

#[derive(Clone, Debug)]
pub struct HDRHist {
    counts: Vec<[u64; 1 << HDHISTOGRAM_BITS]>,
}

impl HDRHist {
    pub fn add_value(&mut self, value: u64) {
        let msb: usize = 64usize - value.leading_zeros() as usize;
        let index = msb.saturating_sub(HDHISTOGRAM_BITS);
        let low_bits = value >> (index.saturating_sub(1)) & ((1 << HDHISTOGRAM_BITS) - 1);
        self.counts[index][low_bits as usize] += 1;
    }
}

impl HDRHist {
    /// New HDRHist
    pub fn new() -> Self {
        HDRHist {
            counts: vec![[0u64; 1 << HDHISTOGRAM_BITS]; 64 - HDHISTOGRAM_BITS + 1],
        }
    }

    /// Output the complementary cumulative distribution function (ccdf) of the samples
    /// 
    /// Returns an iterator over increasing sample values such that, for every triple
    /// `(value, prob, count)`, `prob` is the ratio of samples >= `value`, and
    /// `count` is the nubmer of samples >= the previous `value` and < the current `value`.
    pub fn ccdf<'a>(&'a self) -> impl Iterator<Item=(u64, f64, u64)>+'a {
        let flattened = self.counts.iter().enumerate().flat_map(move |(index, bucket)| {
            bucket.iter().enumerate().map(move |(sub, count)| (index, sub, count))
        });
        let (first, _) = flattened.clone().enumerate().find(|&(_, (_, _, c))| *c > 0).expect("no values in histogram");
        let last = flattened.clone().enumerate().fold(0, |acc, (i, (_, _, c))| {
            if *c > 0 {
                i
            } else {
                acc
            }
        });

        let total: u64 = self.counts.iter().map(|x| x.iter().sum::<u64>()).sum();
        let mut sum: u64 = 0;

        flattened.take(last + 2).skip(first).map(move |(index, sub, count)| {
            let value = if index > 0 {
                (1u64 << (index + HDHISTOGRAM_BITS - 1)).saturating_add((sub as u64 + 1) << (index - 1))
            } else {
                sub as u64 + 1
            };
            let fraction = (total + 1 - sum) as f64 / ((total + 1) as f64);
            sum += count;
            (value, fraction, *count)
        })
    }

    /// Output a summary of the samples' cdf as (quantile, value) pairs
    ///
    /// Quantiles are estimated
    pub fn summary<'a>(&'a self) -> impl Iterator<Item=(f64, u64)>+'a {
        let mut ccdf = self.ccdf();
        [0.75, 0.50, 0.25, 0.05, 0.01, 0.001, 0.0].into_iter().map(move |p| {
            let (value, _, _) = if *p == 0.0 {
                ccdf.by_ref().last().expect("invalid ccdf")
            } else {
                ccdf.by_ref().find(|&(_, fraction, _)| fraction <= *p).expect("invalid ccdf")
            };
            (1f64 - p, value)
        })
    }

    /// Output a summary of the samples' cdf as (quantile, value) pairs
    ///
    /// Quantiles are estimated
    pub fn summary_string(&self) -> String {
        let mut values: Vec<String> = vec!["╭ ".to_string()];
        let mut points: Vec<String> = vec!["╰ ".to_string()];
        for (p, v) in self.summary() {
            if p == 0.25 {
                points.push("[".to_string());
            } else if p == 0.95 {
                points.push("]".to_string());
            } else if p < 0.95 {
                points.push(" ".to_string());
            } else {
                points.push("-".to_string());
            }
            values.push(" ".to_string());
            if p < 0.95 {
                points.push(format!("    {:<5}    ", p));
            } else if p != 1.0 {
                points.push(format!("--- {:<5} ---", p));
            } else {
                points.push("---| max     ".to_string());
            }
            values.push(format!(" {:^11.6e} ", v as f64));
        }
        values.push("╮\n".to_string());
        values.extend(points.into_iter());
        values.push("╯".to_string());
        values.join("")
    }
}
