//! See `examples/ftdi.rs` for connection information.
//!
//! This dumps the entire EEPROM contents as hex.

use eeprom25aa02e48::Eeprom25aa02e48;
use embedded_hal::spi::Polarity;
use ftdi_embedded_hal::{
    libftd2xx::{self, Ft232h},
    FtHal, OutputPin, Spi,
};

fn hexdump(buf: &[u8]) {
    let width: usize = format!("{:X}", buf.len())
        .chars()
        .count()
        .checked_sub(1)
        .unwrap_or(1);

    let mut ascii_row: String = String::with_capacity(16);

    for (idx, byte) in buf.iter().enumerate() {
        if idx % 16 == 0 {
            print!("{:0width$X} ", idx, width = width);
            ascii_row = String::with_capacity(16);
        } else if idx % 8 == 0 {
            print!(" ");
        }

        print!(" {:02X}", byte);

        if 32 <= *byte && *byte <= 126 {
            ascii_row.push((*byte).into());
        } else {
            ascii_row.push('.');
        }

        if (idx + 1) % 16 == 0 && idx != 0 {
            println!("  {}", ascii_row);
        }
    }
}

fn main() {
    let device: Ft232h = libftd2xx::Ftdi::new().unwrap().try_into().unwrap();
    let hal_dev: FtHal<Ft232h> = FtHal::init_default(device).unwrap();

    let mut spi: Spi<Ft232h> = hal_dev.spi().unwrap();
    spi.set_clock_polarity(Polarity::IdleLow);
    let cs: OutputPin<Ft232h> = hal_dev.ad3().unwrap();

    let mut eeprom = Eeprom25aa02e48::new(spi, cs);

    let mut all_data: [u8; 256] = [0; 256];
    eeprom
        .read(0x00, &mut all_data)
        .expect("Failed to read data");

    hexdump(&all_data);
}
