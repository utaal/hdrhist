use hdrhist::HDRHist;

use std::io;
use std::io::prelude::*;

#[derive(Debug)]
struct HdrHistError {
    str: String,
}

fn main() -> Result<(), HdrHistError> {
    let mut hist = HDRHist::new();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let val: u64 = line?.parse()?;
        hist.add_value(val);
    }

    for (value, prob, count) in hist.ccdf() {
        println!("{}\t{}\t{}", value, prob, count);
    }

    Ok(())
}

impl std::convert::From<io::Error> for HdrHistError {
    fn from(f: io::Error) -> Self {
        HdrHistError { str: format!("io error: {:?}", f) }
    }
}

impl std::convert::From<std::num::ParseIntError> for HdrHistError {
    fn from(f: std::num::ParseIntError) -> Self {
        HdrHistError { str: format!("parse error: {:?}", f) }
    }
}
