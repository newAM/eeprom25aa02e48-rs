//! TODO, docstring
#![deny(missing_docs, unsafe_code)]
#![no_std]

use embedded_hal as hal;

use hal::blocking::spi;
use hal::digital::v2::OutputPin;

/// Read instruction.
pub const INSTRUCTION_READ: u8 = 0x03;
/// Write instruction.
pub const INSTRUCTION_WRITE: u8 = 0x02;
/*
const INSTRUCTION_WRDI: u8 = 0x04;
const INSTRUCTION_WREN: u8 = 0x06;
const INSTRUCTION_RDSR: u8 = 0x05;
const INSTRUCTION_WRSR: u8 = 0x01;
*/

/// EEPROM page size in bytes.
pub const PAGE_SIZE: usize = 16;
/// Maximum EEPROM address.
pub const MAX_ADDR: usize = 0xFF;

/// Eeprom25aa02e48 driver
#[derive(Default)]
pub struct Eeprom25aa02e48<SPI, CS> {
    /// SPI device.
    spi: SPI,

    /// GPIO for chip select.
    cs: CS,
}

/// Eeprom25aa02e48 error type.
#[derive(Debug)]
pub enum Error<SpiError, PinError> {
    /// SPI bus error wrapper.
    Spi(SpiError),
    /// GPIO pin error wrapper.
    Pin(PinError),
    /// Address is not page aligned.
    AddressNotPageAligned,
    /// Address is invalid.
    AddressInvalid,
}

impl<SPI, CS, SpiError, PinError> Eeprom25aa02e48<SPI, CS>
where
    SPI: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
{
    /// Creates a new `Eeprom25aa02e48` driver from a SPI peripheral and a chip
    /// select digital I/O pin.
    pub fn new(spi: SPI, cs: CS) -> Self {
        Eeprom25aa02e48 { spi: spi, cs: cs }
    }

    fn chip_enable(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.cs.set_low().map_err(Error::Pin)
    }

    fn chip_disable(&mut self) -> Result<(), Error<SpiError, PinError>> {
        self.cs.set_high().map_err(Error::Pin)
    }

    /// Read from the EEPROM.
    /// The size of the `data` buffer determines the number of bytes read.
    pub fn read_data(
        &mut self,
        address: u8,
        data: &mut [u8],
    ) -> Result<(), Error<SpiError, PinError>> {
        if address as usize + data.len() - 1 > MAX_ADDR {
            return Err(Error::AddressInvalid);
        }
        let cmd: [u8; 2] = [INSTRUCTION_READ, address];
        self.chip_enable()?;
        self.spi.write(&cmd).map_err(Error::Spi)?;
        self.spi.transfer(data).map_err(Error::Spi)?;
        self.chip_disable()
    }

    /// Write a byte to the EEPROM.
    pub fn write_byte(&mut self, address: u8, data: u8) -> Result<(), Error<SpiError, PinError>> {
        let cmd: [u8; 3] = [INSTRUCTION_WRITE, address, data];
        self.chip_enable()?;
        self.spi.write(&cmd).map_err(Error::Spi)?;
        self.chip_disable()
    }

    /// Write a page to the EEPROM.
    ///
    /// *Note*: The address must be page aligned.
    pub fn write_page(
        &mut self,
        address: u8,
        data: [u8; PAGE_SIZE],
    ) -> Result<(), Error<SpiError, PinError>> {
        if address % PAGE_SIZE as u8 != 0 {
            return Err(Error::AddressNotPageAligned);
        }
        let cmd: [u8; 2] = [INSTRUCTION_WRITE, address];
        self.chip_enable()?;
        self.spi.write(&cmd).map_err(Error::Spi)?;
        self.spi.write(&data).map_err(Error::Spi)?;
        self.chip_disable()
    }
}
