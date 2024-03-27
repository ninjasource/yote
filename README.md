# Yote - Year of the Ear

Experiments with the nrf5340 Audio Dk, CS47L63 DSP chip and Yote experimental hearing aid
For Rust firmware, open the `firmware` folder instead of working from this folder

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

## Troubleshooting

You may get an "Arm specific error at runtime and the most likely cause is that you ran probe-run with the `--erase-all` flag set.
Check the notes in `.cargo/config.toml`


Here are some commands you may find useful when working with the nrf5340 audio dk. 
You will need to install the nrfjprog command line tools from Nordic beforhand.

```
nrfjprog -i
nrfjprog --eraseall --snr 1050175639 (locks up)
nrfjprog --recover --coprocessor CP_APPLICATION --snr 1050175639
nrfjprog --recover --snr 1050175639 (same as application above)
nrfjprog --recover --coprocessor CP_NETWORK --snr 1050175639 (locks up)
```

Here is some code that you can use to flash the device without probe-run:

```
#! /usr/bin/env bash

cargo objcopy --release -- -O ihex test.hex
nrfjprog -f NRF53 --erasepage 0x00000000-0x00100000
nrfjprog -f NRF53 --program test.hex --debugreset
rm test.hex
```

## How it works

This demo uses the mcu to generate a tone and plays it into the cs47l63 dsp chip. 
Before it can do that it has to configure the dsp chip correctly by writing to various registers over spi.
There are multiple tasks running simultaneously so we need a mechanism to share the spi bus between them.

The demo runs three main tasks simultaneously: 
1. Handle interrupts from the DSP chip which indicates that long running tasks on the DSP have been completed
2. Handle button presses to control playback and volume
3. Generate a waveform in real time and push the data to the DSP chip over the I2S sound bus

## Project structure

The `cs47l63` module should be in its own crate and is not specific to this demo. 
It is responsible for booting up the cs47l63 chip correctly and providing a bunch of register address and constants to be used by the application.
The `hw_codec` constains a collection of helper functions that make calls to the `cs47l63` driver to configure it.
The `main.rs` application should use `dsp` module to exclusively communicate with the `cs47l63` even though, in theory, it could communicate directly with it through the `shared_bus` module.

# Chapro compilation

CC=arm-none-eabi-gcc
AR=arm-none-eabi-ar
CFLAGS = -W -Wall -Os -mthumb -ffunction-sections -fdata-sections -mcpu=cortex-m33 -DBR_ARMEL_CORTEXM_GCC -I$(INCDIR) -Wno-unused-local-typedef -fPIC -mfpu=fpv5-sp-d16 -mfloat-abi=hard

# Notes on Power Consumption

Here are some of the things that seem to affect power consumption:

Logging: No effect unless a debugger is attached - then huge effect
Core clock 64Mhz -> 128Mhz and Flash Cache enabled - 200uA consumption
Setting all peripheral pins to Input Low on startup - no effect, they seem to already default to their lowest power setting anyway

