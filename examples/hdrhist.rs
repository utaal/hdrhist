extern crate hdrhist;
extern crate rand;

use rand::Rng;
use rand::RngCore;

use rand::distributions::{Normal, Poisson, Distribution};

fn main() {
    let normal = Normal::new(10.0, 3.0);
    let mut rng = rand::thread_rng();

    let mut hist = hdrhist::HDRHist::new();

    for _ in 0..1000000000 {
        let val = normal.sample(&mut rng) * 1_000_000_000f64;
        if val >= 0f64 {
            hist.add_value(val as u64);
        }
    }

    for (v, p, c) in hist.ccdf() {
        println!("{}\t{}\t{}", v, p, c);
    }
}
