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

## Example

```rust
use eeprom25aa02e48::Eeprom25aa02e48;

let mut eeprom = Eeprom25aa02e48::new(spi, pin);
let eui48: [u8; 6] = eeprom.read_eui48()?;
```

[`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
[eeprom24x-rs]: https://github.com/eldruin/eeprom24x-rs
[Microchip 25AA02E48]: http://ww1.microchip.com/downloads/en/DeviceDoc/25AA02E48-25AA02E64-2K-SPI-Bus-Serial-EEPROM-Data%20Sheet_DS20002123G.pdf
