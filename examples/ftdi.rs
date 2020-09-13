use eeprom25aa02e48::Eeprom25aa02e48;

use embedded_hal::spi::Polarity;
use ftd2xx_embedded_hal::Ft232hHal;

fn main() {
    let dev = Ft232hHal::new().unwrap();
    let mut spi = dev.spi().unwrap();
    spi.set_clock_polarity(Polarity::IdleLow);
    let cs = dev.ad3();

    let mut eeprom = Eeprom25aa02e48::new(spi, cs);

    let mut mac: [u8; 6] = [0; 6];

    eeprom.read_eui48(&mut mac).unwrap();

    println!(
        "MAC address: {:02X?}:{:02X?}:{:02X?}:{:02X?}:{:02X?}:{:02X?}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
}
