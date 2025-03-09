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
use ftdi_embedded_hal::{
    FtHal, SpiDevice,
    libftd2xx::{self, Ft232h},
};

fn main() {
    let device: Ft232h = libftd2xx::Ftdi::new().unwrap().try_into().unwrap();
    let hal_dev: FtHal<Ft232h> = FtHal::init_default(device).unwrap();

    let mut spi: SpiDevice<Ft232h> = hal_dev.spi_device(3).unwrap();
    spi.set_clock_polarity(Polarity::IdleLow);

    let mut eeprom = Eeprom25aa02e48::new(spi);
    let mac: [u8; 6] = eeprom.read_eui48().unwrap();

    println!(
        "MAC address: {:02X?}:{:02X?}:{:02X?}:{:02X?}:{:02X?}:{:02X?}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
}
