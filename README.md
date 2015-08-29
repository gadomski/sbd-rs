# sbd-rs

Native rust library to read and write Iridium Short Burt Data (SBD) messages.

[![Build Status](https://travis-ci.org/gadomski/sbd-rs.svg?branch=master)](https://travis-ci.org/gadomski/sbd-rs)
[![](http://meritbadge.herokuapp.com/sbd)](https://crates.io/crates/sbd)


## Installation

Include the following in your `Cargo.toml`:

```toml
[dependencies]
sbd = "0.0.2"
```

## Documentation

The documentation includes examples, which should be used as a reference when developing an application with **sbd**.
To build the documentation, first clone the git repository, then build the documentation and open it:

```bash
$ git clone https://github.com/gadomski/sbd-rs
$ cd sbd-rs
$ cargo doc --open
```

This will open the documentation in your browser.


## What is Iridium? What is SBD?

Iridium can refer to both a [satellite network](https://en.wikipedia.org/wiki/Iridium_satellite_constellation) and the [private company](https://en.wikipedia.org/wiki/Iridium_Communications) that manages that network.
Iriduim supports a variety of services, including phone communications, dial-up modems, and (our area of interest) a type of "packet" communication called Short Burst Data (SBD).
SBD messages are composed of header information, including time of transmission and modem number, and a payload, which is an arbitrary set of bytes.
SBD payloads can be restricted to as little as 340 bytes per message, or can allow 1960 bytes per message or more.

Iridium modems with SBD capabilities can both send messages (called Mobile Originated or MO messages) and receive messages (called Mobile Terminated or MT).
At this time this library focuses on Mobile Originated (MO) messages, as the author has no use for MT messages.
Pull requests for MT support are welcome.

Mobile Originated (MO) messages are sent from an Iridium modem to the Iridium "gateway", which processes messages and forwards them on for delivery to one more more endpoints.
Endpoints are configured through a web interface, and can come in one of two flavors: email endpoints, where the message is sent as a MIME email attachment, and DirectIP endpoints, where the messages are delivered over a TCP connection to an IP address and port.

This library includes facilities to read SBD messages stored on the filesystem, and to parse SBD messages coming in on a `TcpStream`.
At this time, no binary executables are included in this package — it is a library only.


## License

This code is available under the MIT license, a full text of which can be found in `LICENSE.txt` inside this repository.


## Contributing

Please open issues and/or pull requests through the [github interface](https://github.com/gadomski/sbd-rs/issues).


## Authors

This code was cobbled together by Pete Gadomski <pete.gadomski@gmail.com>.
