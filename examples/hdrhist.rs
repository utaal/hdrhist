extern crate hdrhist;
extern crate rand;
extern crate textplots;

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

    eprintln!("plot"); 
    use textplots::{Chart, Plot};
    let data: Vec<_> = hist.ccdf().map(|(v, p, _)| (v as f32, p as f32)).collect();
    let interpolated = textplots::utils::interpolate(&data[..]);
    Chart::new(180, 60, 0.0, 8e10).lineplot(interpolated).display();

    let mut hist2 = hdrhist::HDRHist::new();
    for _ in 0..1000000000 {
        hist2.add_value(1000000);
    }
    eprintln!("summary_string\n{}", hist2.summary_string());
}
