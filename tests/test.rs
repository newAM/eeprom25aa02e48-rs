use eeprom25aa02e48::{
    Eeprom25aa02e48, EUI48_BYTES, EUI48_MEMORY_ADDRESS, INSTRUCTION_READ, INSTRUCTION_WRITE,
    PAGE_SIZE,
};
use embedded_hal_mock as hal;
use hal::pin::{Mock as PinMock, State as PinState, Transaction as PinTransaction};
use hal::spi::{Mock as SpiMock, Transaction as SpiTransaction};

#[test]
#[should_panic]
fn address_not_page_aligned() {
    let mut eeprom = Eeprom25aa02e48::new(SpiMock::new(&[]), PinMock::new(&[]));
    let data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
    eeprom.write_page(7, &data).unwrap();
}

#[test]
fn write_page() {
    let address: u8 = PAGE_SIZE as u8;
    let mut data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
    for i in 0..data.len() {
        data[i] = (PAGE_SIZE - i) as u8;
    }
    let mut eeprom = Eeprom25aa02e48::new(
        SpiMock::new(&[
            SpiTransaction::write(vec![INSTRUCTION_WRITE, address]),
            SpiTransaction::write(data.to_vec()),
        ]),
        PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]),
    );

    eeprom.write_page(address, &data).unwrap();
}

#[test]
fn write_byte() {
    let address: u8 = 7;
    let data: u8 = 0xAF;
    let mut eeprom = Eeprom25aa02e48::new(
        SpiMock::new(&[SpiTransaction::write(vec![
            INSTRUCTION_WRITE,
            address,
            data,
        ])]),
        PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]),
    );

    eeprom.write_byte(address, data).unwrap();
}

#[test]
#[should_panic]
fn address_invalid() {
    let mut eeprom = Eeprom25aa02e48::new(SpiMock::new(&[]), PinMock::new(&[]));
    let mut data: [u8; 2] = [0; 2];
    eeprom.read_data(0xFF, &mut data).unwrap();
}

#[test]
fn read_data() {
    let address: u8 = 0xFF;
    let output: u8 = 0xAF;
    let mut eeprom = Eeprom25aa02e48::new(
        SpiMock::new(&[
            SpiTransaction::write(vec![INSTRUCTION_READ, address]),
            SpiTransaction::transfer(vec![0], vec![output]),
        ]),
        PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]),
    );
    let mut data: [u8; 1] = [0; 1];
    eeprom.read_data(address, &mut data).unwrap();
    assert_eq!(data[0], output);
}

#[test]
fn read_eui48() {
    let dummy_mac: [u8; EUI48_BYTES] = [0xFF; EUI48_BYTES];
    let mut eeprom = Eeprom25aa02e48::new(
        SpiMock::new(&[
            SpiTransaction::write(vec![INSTRUCTION_READ, EUI48_MEMORY_ADDRESS]),
            SpiTransaction::transfer(vec![0; EUI48_BYTES], dummy_mac.to_vec()),
        ]),
        PinMock::new(&[
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]),
    );
    let mut mac: [u8; EUI48_BYTES] = [0; EUI48_BYTES];
    eeprom.read_eui48(&mut mac).unwrap();
    assert_eq!(mac, dummy_mac);
}
