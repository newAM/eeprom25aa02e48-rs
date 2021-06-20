//! Inspired by [eeprom24x-rs], this is a driver for the [Microchip 25AA02E48]
//! SPI EEPROM, based on the [`embedded-hal`] traits.
//!
//! This EEPROM is unique because it has an EUI-48 MAC address programmed into
//! the EEPROM, which is convenient for creating internet connected devices
//! with valid MAC addresses.
//!
//! # Example
//!
//! ```
//! # use eeprom25aa02e48::{instruction, EUI48_MEMORY_ADDRESS};
//! # use embedded_hal_mock as hal;
//! # let spi = hal::spi::Mock::new(&[
//! #   hal::spi::Transaction::write(vec![instruction::READ, EUI48_MEMORY_ADDRESS]),
//! #   hal::spi::Transaction::transfer(vec![0; 6], vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
//! # ]);
//! # let pin = hal::pin::Mock::new(&[
//! #    hal::pin::Transaction::set(hal::pin::State::Low),
//! #    hal::pin::Transaction::set(hal::pin::State::High),
//! # ]);
//! use eeprom25aa02e48::Eeprom25aa02e48;
//!
//! let mut eeprom = Eeprom25aa02e48::new(spi, pin);
//! let eui48: [u8; 6] = eeprom.read_eui48()?;
//! # assert_eq!(eui48, [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
//! # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
//! ```
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [eeprom24x-rs]: https://github.com/eldruin/eeprom24x-rs
//! [Microchip 25AA02E48]: http://ww1.microchip.com/downloads/en/DeviceDoc/25AA02E48-25AA02E64-2K-SPI-Bus-Serial-EEPROM-Data%20Sheet_DS20002123G.pdf
#![doc(html_root_url = "https://docs.rs/eeprom25aa02e48/0.2.0")]
#![deny(missing_docs, unsafe_code)]
#![no_std]

use embedded_hal as hal;

use hal::blocking;
use hal::digital::v2::OutputPin;

/// EEPROM instructions.
pub mod instruction {
    /// Read data from memory array beginning at selected address.
    pub const READ: u8 = 0x03;
    /// Write data to memory array beginning at selected address.
    pub const WRITE: u8 = 0x02;
    /// Reset the write enable latch (disable write operations).
    pub const WRDI: u8 = 0x04;
    /// Set the write enable latch (enable write operations).
    pub const WREN: u8 = 0x06;
    /// Read STATUS register.
    pub const RDSR: u8 = 0x05;
    /// Write STATUS register.
    pub const WRSR: u8 = 0x01;
}

/// Number of bytes in an EUI48 MAC address.
pub const EUI48_BYTES: usize = 6;
/// EPPROM memory address of the EUI48 address.
pub const EUI48_MEMORY_ADDRESS: u8 = 0xFA;
/// EEPROM page size in bytes.
pub const PAGE_SIZE: u8 = 16;

/// Microchip 25AA02E48 driver.
#[derive(Default)]
pub struct Eeprom25aa02e48<SPI, CS> {
    /// SPI device.
    spi: SPI,
    /// GPIO for chip select.
    cs: CS,
}

/// Error type.
#[derive(Debug)]
pub enum Error<SpiError, PinError> {
    /// SPI bus error wrapper.
    Spi(SpiError),
    /// GPIO pin error wrapper.
    Pin(PinError),
}

impl<SPI, CS, SpiError, PinError> Eeprom25aa02e48<SPI, CS>
where
    SPI: blocking::spi::Transfer<u8, Error = SpiError> + blocking::spi::Write<u8, Error = SpiError>,
    CS: OutputPin<Error = PinError>,
{
    /// Creates a new driver from a SPI bus and a chip select digital I/O pin.
    ///
    /// # Safety
    ///
    /// The chip select pin must be high before being passed to this function.
    ///
    /// # Example
    ///
    /// The `spi` and `pin` variables in this example will be provided by your
    /// device-specific hal crate.
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let mut pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    /// use embedded_hal::digital::v2::OutputPin;
    ///
    /// pin.set_high()?;
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// # Ok::<(), hal::MockError>(())
    /// ```
    pub fn new(spi: SPI, cs: CS) -> Self {
        Eeprom25aa02e48 { spi, cs }
    }

    /// Free the SPI bus and CS pin from the W5500.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// let (spi, pin) = eeprom.free();
    /// ```
    pub fn free(self) -> (SPI, CS) {
        (self.spi, self.cs)
    }

    /// Context manager to ensure CS is always set high after an operation.
    #[inline(always)]
    fn with_chip_enable<T, E, F>(&mut self, mut f: F) -> Result<T, E>
    where
        F: FnMut(&mut SPI) -> Result<T, E>,
        E: core::convert::From<Error<SpiError, PinError>>,
    {
        self.cs.set_low().map_err(Error::Pin)?;
        let result = f(&mut self.spi);
        self.cs.set_high().map_err(Error::Pin)?;
        result
    }

    /// Context manager to ensure the write latch is always disabled after an operation.
    #[inline(always)]
    fn with_write_latch<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnMut(&mut SPI) -> Result<T, E>,
        E: core::convert::From<Error<SpiError, PinError>>,
    {
        self.with_chip_enable(|spi| spi.write(&[instruction::WREN]).map_err(Error::Spi))?;
        let result = self.with_chip_enable(f);
        // write latch automatically resets on successful write
        if result.is_err() {
            self.with_chip_enable(|spi| spi.write(&[instruction::WRDI]).map_err(Error::Spi))?;
        }
        result
    }

    /// Read from the EEPROM.
    ///
    /// # Arguments
    ///
    /// * `address` - A byte address from 0x00 to 0xFF.
    /// * `buf` - Buffer to read data into.
    ///   The size of the buffer determines the number of bytes read.
    ///
    /// # Example
    ///
    /// ```
    /// # use eeprom25aa02e48::instruction;
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![instruction::READ, 0x00]),
    /// #   hal::spi::Transaction::transfer(vec![0x00; 64], vec![0x00; 64]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut some_big_buf: [u8; 1024] = [0; 1024];
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// // read 64 bytes starting at EEPROM address 0x00
    /// eeprom.read(0x00, &mut some_big_buf[..64])?;
    /// # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
    /// ```
    ///
    /// # Safety
    ///
    /// If the buffer length plus address exceeds the maximum address of `0xFF`
    /// the address counter will roll over to `0x00`.
    ///
    /// # Panics
    ///
    /// The length of the buf may not exceed 256.
    ///
    /// ```should_panic
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut some_big_buf: [u8; 1024] = [0; 1024];
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// eeprom.read(0x0, &mut some_big_buf)?;
    /// # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
    /// ```
    pub fn read(&mut self, address: u8, buf: &mut [u8]) -> Result<(), Error<SpiError, PinError>> {
        if buf.is_empty() {
            Ok(())
        } else {
            // buffer is too large
            assert!(buf.len() <= 256);
            let cmd: [u8; 2] = [instruction::READ, address];
            self.with_chip_enable(|spi| {
                spi.write(&cmd).map_err(Error::Spi)?;
                spi.transfer(buf).map_err(Error::Spi)?;
                Ok(())
            })
        }
    }

    /// Writes up to a page of data to the EEPROM.
    ///
    /// # Arguments
    ///
    /// * `address` - A byte address from 0x00 to 0xFF.
    /// * `data` - Data to write, must be less than or equal to the page size in length.
    ///
    /// # Example
    ///
    /// Write to the second page (page 1).
    ///
    /// ```
    /// # use eeprom25aa02e48::instruction;
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![instruction::WREN]),
    /// #   hal::spi::Transaction::write(vec![instruction::WRITE, 0x10]),
    /// #   hal::spi::Transaction::write(vec![0x12; 16]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let data: [u8; 16] = [0x12; 16];
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// eeprom.write_page(0x10, &data)?;
    /// # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
    /// ```
    ///
    /// # Panics
    ///
    /// The data length must be less than or equal to the page size (16).
    ///
    /// ```should_panic
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let data: [u8; 17] = [0x00; 17];
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// eeprom.write_page(0, &data)?;
    /// # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
    /// ```
    ///
    /// The address must be page aligned.
    ///
    /// ```should_panic
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::pin::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let data: [u8; 16] = [0x00; 16];
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// eeprom.write_page(1, &data)?;
    /// # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
    /// ```
    pub fn write_page(
        &mut self,
        address: u8,
        data: &[u8],
    ) -> Result<(), Error<SpiError, PinError>> {
        assert!(address % PAGE_SIZE == 0);
        if data.is_empty() {
            Ok(())
        } else {
            assert!(data.len() <= PAGE_SIZE as usize);
            let cmd: [u8; 2] = [instruction::WRITE, address];
            self.with_write_latch(|spi| {
                spi.write(&cmd).map_err(Error::Spi)?;
                spi.write(data).map_err(Error::Spi)?;
                Ok(())
            })
        }
    }

    /// Read the EUI-48 MAC address from the EEPROM.
    ///
    /// # Example
    ///
    /// ```
    /// # use eeprom25aa02e48::{instruction, EUI48_MEMORY_ADDRESS};
    /// # use embedded_hal_mock as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::write(vec![instruction::READ, EUI48_MEMORY_ADDRESS]),
    /// #   hal::spi::Transaction::transfer(vec![0; 6], vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
    /// # ]);
    /// # let pin = hal::pin::Mock::new(&[
    /// #    hal::pin::Transaction::set(hal::pin::State::Low),
    /// #    hal::pin::Transaction::set(hal::pin::State::High),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut eeprom = Eeprom25aa02e48::new(spi, pin);
    /// let eui48: [u8; 6] = eeprom.read_eui48()?;
    /// # assert_eq!(eui48, [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
    /// # Ok::<(), eeprom25aa02e48::Error<_, _>>(())
    /// ```
    pub fn read_eui48(&mut self) -> Result<[u8; EUI48_BYTES], Error<SpiError, PinError>> {
        let mut eui48: [u8; EUI48_BYTES] = [0; EUI48_BYTES];
        self.read(EUI48_MEMORY_ADDRESS, &mut eui48)?;
        Ok(eui48)
    }
}
