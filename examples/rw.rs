//! See `examples/ftdi.rs` for connection information.
//!
//! **Note:** This is a destructive example that will write your EEPROM.

use eeprom25aa02e48::{Eeprom25aa02e48, PAGE_SIZE};
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

    let mut page: [u8; PAGE_SIZE as usize] = [0; PAGE_SIZE as usize];
    const BYTE_ADDR: u8 = 0x10;
    const PAGE_ADDR: u8 = BYTE_ADDR / PAGE_SIZE;
    assert!(BYTE_ADDR % PAGE_ADDR == 0);
    println!("Reading page");
    eeprom
        .read(BYTE_ADDR, &mut page)
        .expect("Failed to read page");

    print!("Page data:");
    page.iter().for_each(|x| print!(" {:02X}", x));
    println!();

    // invert all page bytes
    println!("Inverting page bytes");
    page.iter_mut().for_each(|x| *x ^= 0xFF);
    print!("Page data:");
    page.iter().for_each(|x| print!(" {:02X}", x));
    println!();
    eeprom
        .write_page(BYTE_ADDR, &page)
        .expect("Failed to write page");

    // read the data again
    println!("Reading page");
    eeprom
        .read(BYTE_ADDR, &mut page)
        .expect("Failed to read page");

    print!("Page data:");
    page.iter().for_each(|x| print!(" {:02X}", x));
    println!();
}
