use rand::distributions::{Normal, Distribution};

fn main() {
    let normal = Normal::new(20.0, 5.0);
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
    for (v, p, c) in hist2.ccdf() {
        println!("{}\t{}\t{}", v, p, c);
    }
    eprintln!("summary_string\n{}", hist2.summary_string());

    let mut hist3 = hdrhist::HDRHist::new();
    for x in 1024..2049 { hist3.add_value(x); }
    for (v, p, c) in hist3.ccdf() {
        println!("{}\t{}\t{}", v, p, c);
    }
    for (v, p) in hist3.ccdf_upper_bound() {
        println!("{}\t{}", v, p);
    }
    for (v, p) in hist3.ccdf_lower_bound() {
        println!("{}\t{}", v, p);
    }
}
