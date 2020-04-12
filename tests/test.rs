extern crate eeprom25x;
extern crate embedded_hal_mock as hal;
use eeprom25x::{Eeprom25x, Error};
use hal::pin::Mock as PinMock;
use hal::spi::Mock as SpiMock;

#[test]
fn address_not_page_aligned() {
    let mut eeprom = Eeprom25x::new(SpiMock::new(&[]), PinMock::new(&[]));
    let data: [u8; 16] = [0; 16];
    match eeprom.write_page(7, data) {
        Err(Error::AddressNotPageAligned) => (),
        _ => panic!("AddressNotPageAligned not returned."),
    }
}
