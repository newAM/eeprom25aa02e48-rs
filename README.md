![Maintenance](https://img.shields.io/badge/maintenance-as--is-yellow.svg)
[![crates.io](https://img.shields.io/crates/v/eeprom25aa02e48.svg)](https://crates.io/crates/eeprom25aa02e48)
[![docs.rs](https://docs.rs/eeprom25aa02e48/badge.svg)](https://docs.rs/eeprom25aa02e48/)
[![Build Status](https://github.com/newAM/eeprom25aa02e48-rs/workflows/CI/badge.svg)](https://github.com/newAM/eeprom25aa02e48-rs/actions)

# eeprom25aa02e48

Inspired by [eeprom24x-rs], this is a driver for the [Microchip 25AA02E48]
SPI EEPROM, based on the [`embedded-hal`] traits.

This EEPROM is unique because it has an EUI-48 MAC address programmed into
the EEPROM, which is convient for creating internet connected devices
with valid MAC addresses.

## FTDI Example

The FTDI example uses a FT232H USB to SPI device to develop the drivers
without the use of a microcontroller.

One-time device setup instructions can be found in the [libftd2xx crate].

With the [adafruit FT232H breakout] create the following connections:

* Connect SCK to D0
* Connect MOSI to D1
* Connect MISO to D2
* Connect CS to D3
* Connect Vdd to 3.3V or 5V
* Connect Vss to GND

Run the example with `cargo run --example ftdi`.

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
[adafruit FT232H breakout]: https://www.adafruit.com/product/2264
[eeprom24x-rs]: https://github.com/eldruin/eeprom24x-rs
[libftd2xx crate]: https://github.com/newAM/libftd2xx-rs/
[Microchip 25AA02E48]: http://ww1.microchip.com/downloads/en/DeviceDoc/25AA02E48-25AA02E64-2K-SPI-Bus-Serial-EEPROM-Data%20Sheet_DS20002123G.pdf
