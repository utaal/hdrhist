`hdrhist` is a small footprint [hdr histogram](https://hdrhistogram.github.io/HdrHistogram/) in rust. It collects `u64` samples in the full `u64` value range with precision of 5 most significant bits. You can add new samples in `O(1)` time (a handful of cycles), and it will never reallocate.

[![Crates.io](https://img.shields.io/crates/v/hdrhist.svg)](https://crates.io/crates/hdrhist) [![Documentation](https://docs.rs/hdrhist/badge.svg)](https://docs.rs/hdrhist)

## Adding the dependency

Add the following to your `[dependecies]` in `Cargo.toml`.

```
hdrhist = "^<version>"
```

where `<version>` is the desired version of the library. The latest version is [![Crates.io](https://img.shields.io/crates/v/hdrhist.svg)](https://crates.io/crates/hdrhist).

## Documentation

The reference documentation is at [docs.rs/hdrhist](https://docs.rs/hdrhist).

