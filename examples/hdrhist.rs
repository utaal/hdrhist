extern crate hdrhist;
extern crate rand;

use rand::distributions::{Normal, Distribution};

fn main() {
    let normal = Normal::new(20.0, 3.0);
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

    eprintln!("summary {:#?}", hist.summary().collect::<Vec<_>>());
    eprintln!("summary_string\n{}", hist.summary_string());
}
