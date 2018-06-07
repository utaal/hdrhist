const HDHISTOGRAM_BITS: usize = 4;

fn main() {
    for value in 0u64..4096 {
        let msb: usize = 64usize - value.leading_zeros() as usize;
        let index = msb.saturating_sub(HDHISTOGRAM_BITS);
        let low_bits = value >> (index.saturating_sub(1)) & ((1 << HDHISTOGRAM_BITS) - 1);
        println!("{} [{:b}] msb={} index={} low_bits={:b}", value, value, msb, index, low_bits);
    }
}
