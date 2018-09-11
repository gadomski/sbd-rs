# sbd-rs

Native rust library to read and write Iridium Short Burt Data (SBD) messages, and an executable that exposes some of that library's functionality.
Documentation is available [online](https://docs.rs/sbd), and they include some background on Iridum and its Short Burst Data (SBD) services.

[![Build Status](https://travis-ci.org/gadomski/sbd-rs.svg?branch=master)](https://travis-ci.org/gadomski/sbd-rs)
[![Crates.io](http://meritbadge.herokuapp.com/sbd)](https://crates.io/crates/sbd)
[![Documentation](https://docs.rs/sbd/badge.svg)](https://docs.rs/sbd)

## Building the executable

To build the `sbd` executable, you need [rust](https://www.rust-lang.org/downloads.html).
Once you have rust, simply:

```bash
cargo install sbd
```
### Using `sbd serve` as a daemon

The executable includes a couple of powers, including a "run-forever" server for receiving Iridium SBD DirectIP messages and storing those messages to the filesystem.
This `sbd serve` power *does not* include any sort of daemonization; you have to roll your own.
I personally use [supervisord](http://supervisord.org/).

## Using the library

Include the following in your `Cargo.toml`:

```toml
[dependencies]
sbd = "0.2"
```

## Contributing

Please open issues and/or pull requests through the [github interface](https://github.com/gadomski/sbd-rs/issues).

## Authors

This code was cobbled together by Pete Gadomski <pete@gadom.ski>.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
