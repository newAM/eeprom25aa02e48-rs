//! # FTDI Example
//!
//! This example uses a FT232H USB to SPI device to develop the drivers
//! without the use of a microcontroller.
//!
//! One-time device setup instructions can be found in the [libftd2xx crate].
//!
//! With the [adafruit FT232H breakout] create the following connections:
//!
//! * Connect SCK to D0
//! * Connect MOSI to D1
//! * Connect MISO to D2
//! * Connect CS to D3
//! * Connect Vdd to 3.3V or 5V
//! * Connect Vss to GND
//!
//! Run the example with `cargo run --example ftdi`.
//!
//! [adafruit FT232H breakout]: https://www.adafruit.com/product/2264
//! [libftd2xx crate]: https://github.com/newAM/libftd2xx-rs/

use eeprom25aa02e48::Eeprom25aa02e48;
use embedded_hal::spi::Polarity;
use ftd2xx_embedded_hal::Ft232hHal;

fn main() {
    let dev = Ft232hHal::new()
        .expect("Failed to open FT232H")
        .init_default()
        .expect("Failed to initialize MPSSE");
    let mut spi = dev.spi().unwrap();
    spi.set_clock_polarity(Polarity::IdleLow);
    let cs = dev.ad3();

    let mut eeprom = Eeprom25aa02e48::new(spi, cs);
    let mac: [u8; 6] = eeprom.read_eui48().unwrap();

    println!(
        "MAC address: {:02X?}:{:02X?}:{:02X?}:{:02X?}:{:02X?}:{:02X?}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
}
