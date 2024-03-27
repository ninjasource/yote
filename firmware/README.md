# Yote - Year of the Ear

Experiments with the nrf5340 Audio Dk, CS47L63 DSP chip and Yote experimental hearing aid

See README.md in root repo for more info

## Prerequisites

`cargo install probe-rs --features="cli"`

If probe-rs does not work then a known working version can be found here:
https://github.com/probe-rs/probe-rs.git#ff2370fb

## To run

NOTE on naming:
All the examples prefixed with "yote" are for the custom built hearing aid device. Everything else is for the nRF5340 Audio-dk

IMPORTANT: be sure to set the appropriate runner in .cargo/config.toml

```
cargo run --bin blinky --release
```