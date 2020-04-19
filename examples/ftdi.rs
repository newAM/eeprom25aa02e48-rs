extern crate eeprom25aa02e48;
extern crate embedded_hal;
extern crate ftdi_embedded_hal as hal;

use eeprom25aa02e48::Eeprom25aa02e48;

use crate::hal::x232h::FTx232H;

fn main() {
    let dev = FTx232H::init(0x0403, 0x6014).unwrap();
    let spi = dev.spi(hal::spi::SpiSpeed::CLK_1MHz).unwrap();
    let cs = dev.ph0().unwrap();

    let mut eeprom = Eeprom25aa02e48::new(spi, cs);

    let mut mac: [u8; 6] = [0; 6];

    eeprom.read_eui48(&mut mac).unwrap();

    println!(
        "MAC address: {:X?}:{:X?}:{:X?}:{:X?}:{:X?}:{:X?}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
}
