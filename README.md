This is a quick and dirty implementation of [Improv](https://www.improv-wifi.com) in Rust.

I basically just use it right now to bootstrap connected devices I'm developing that are running the
Serial C++ code, in order to exercise the web-flasher-based provisioning without having to go
clicky-clicky-clicky.

# Usage

To use this, you can either import the library (see [main.rs](src/main.rs) for an example), or run
it directly using cargo:

```bash
cargo run -- /dev/tty.usb-serial01 myssid hunter2
```
