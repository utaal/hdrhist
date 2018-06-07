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
}
