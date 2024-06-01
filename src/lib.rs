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
//! # use embedded_hal_mock::eh1 as hal;
//! # let spi = hal::spi::Mock::new(&[
//! #   hal::spi::Transaction::transaction_start(),
//! #   hal::spi::Transaction::write_vec(vec![instruction::READ, EUI48_MEMORY_ADDRESS]),
//! #   hal::spi::Transaction::transfer_in_place(vec![0; 6], vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
//! #   hal::spi::Transaction::transaction_end(),
//! # ]);
//! use eeprom25aa02e48::Eeprom25aa02e48;
//!
//! let mut eeprom = Eeprom25aa02e48::new(spi);
//! let eui48: [u8; 6] = eeprom.read_eui48()?;
//! # assert_eq!(eui48, [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
//! # let mut spi = eeprom.free(); spi.done();
//! # Ok::<(), embedded_hal::spi::ErrorKind>(())
//! ```
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [eeprom24x-rs]: https://github.com/eldruin/eeprom24x-rs
//! [Microchip 25AA02E48]: http://ww1.microchip.com/downloads/en/DeviceDoc/25AA02E48-25AA02E64-2K-SPI-Bus-Serial-EEPROM-Data%20Sheet_DS20002123G.pdf
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![no_std]

use embedded_hal::spi::Operation;

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
pub struct Eeprom25aa02e48<SPI> {
    spi: SPI,
}

impl<SPI> Eeprom25aa02e48<SPI>
where
    SPI: embedded_hal::spi::SpiDevice,
{
    /// Creates a new driver from a SPI bus.
    ///
    /// # Example
    ///
    /// The `spi` variables in this example will be provided by your
    /// device-specific hal crate.
    ///
    /// ```
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// # let mut spi = eeprom.free(); spi.done();
    /// ```
    #[inline]
    pub fn new(spi: SPI) -> Self {
        Eeprom25aa02e48 { spi }
    }

    /// Free the SPI bus from the device.
    ///
    /// # Example
    ///
    /// ```
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// let mut spi = eeprom.free();
    /// # spi.done();
    /// ```
    #[inline]
    pub fn free(self) -> SPI {
        self.spi
    }

    /// Context manager to ensure the write latch is always disabled after an operation.
    #[inline(always)]
    fn with_write_latch(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), SPI::Error> {
        self.spi.write(&[instruction::WREN])?;
        let result = self.spi.transaction(operations);
        // write latch automatically resets on successful write
        if result.is_err() {
            self.spi.write(&[instruction::WRDI])?;
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
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::transaction_start(),
    /// #   hal::spi::Transaction::write_vec(vec![instruction::READ, 0x00]),
    /// #   hal::spi::Transaction::transfer_in_place(vec![0x00; 64], vec![0x00; 64]),
    /// #   hal::spi::Transaction::transaction_end(),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut some_big_buf: [u8; 1024] = [0; 1024];
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// // read 64 bytes starting at EEPROM address 0x00
    /// eeprom.read(0x00, &mut some_big_buf[..64])?;
    /// # let mut spi = eeprom.free(); spi.done();
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
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
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut some_big_buf: [u8; 1024] = [0; 1024];
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// eeprom.read(0x0, &mut some_big_buf)?;
    /// # let mut spi = eeprom.free(); spi.done();
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    pub fn read(&mut self, address: u8, buf: &mut [u8]) -> Result<(), SPI::Error> {
        if buf.is_empty() {
            Ok(())
        } else {
            // buffer is too large
            assert!(buf.len() <= 256);
            let cmd: [u8; 2] = [instruction::READ, address];
            self.spi
                .transaction(&mut [Operation::Write(&cmd), Operation::TransferInPlace(buf)])
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
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::transaction_start(),
    /// #   hal::spi::Transaction::write_vec(vec![instruction::WREN]),
    /// #   hal::spi::Transaction::transaction_end(),
    /// #   hal::spi::Transaction::transaction_start(),
    /// #   hal::spi::Transaction::write_vec(vec![instruction::WRITE, 0x10]),
    /// #   hal::spi::Transaction::write_vec(vec![0x12; 16]),
    /// #   hal::spi::Transaction::transaction_end(),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let data: [u8; 16] = [0x12; 16];
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// eeprom.write_page(0x10, &data)?;
    /// # let mut spi = eeprom.free(); spi.done();
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// # Panics
    ///
    /// The data length must be less than or equal to the page size (16).
    ///
    /// ```should_panic
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::digital::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let data: [u8; 17] = [0x00; 17];
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// eeprom.write_page(0, &data)?;
    /// # let mut spi = eeprom.free(); spi.done();
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// The address must be page aligned.
    ///
    /// ```should_panic
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[]);
    /// # let pin = hal::digital::Mock::new(&[]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let data: [u8; 16] = [0x00; 16];
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// eeprom.write_page(1, &data)?;
    /// # let mut spi = eeprom.free(); spi.done();
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    pub fn write_page(&mut self, address: u8, data: &[u8]) -> Result<(), SPI::Error> {
        assert!(address % PAGE_SIZE == 0);
        if data.is_empty() {
            Ok(())
        } else {
            assert!(data.len() <= PAGE_SIZE as usize);
            let cmd: [u8; 2] = [instruction::WRITE, address];
            self.with_write_latch(&mut [Operation::Write(&cmd), Operation::Write(data)])
        }
    }

    /// Read the EUI-48 MAC address from the EEPROM.
    ///
    /// # Example
    ///
    /// ```
    /// # use eeprom25aa02e48::{instruction, EUI48_MEMORY_ADDRESS};
    /// # use embedded_hal_mock::eh1 as hal;
    /// # let spi = hal::spi::Mock::new(&[
    /// #   hal::spi::Transaction::transaction_start(),
    /// #   hal::spi::Transaction::write_vec(vec![instruction::READ, EUI48_MEMORY_ADDRESS]),
    /// #   hal::spi::Transaction::transfer_in_place(vec![0; 6], vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]),
    /// #   hal::spi::Transaction::transaction_end(),
    /// # ]);
    /// use eeprom25aa02e48::Eeprom25aa02e48;
    ///
    /// let mut eeprom = Eeprom25aa02e48::new(spi);
    /// let eui48: [u8; 6] = eeprom.read_eui48()?;
    /// # let mut spi = eeprom.free(); spi.done();
    /// # assert_eq!(eui48, [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC]);
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    pub fn read_eui48(&mut self) -> Result<[u8; EUI48_BYTES], SPI::Error> {
        let mut eui48: [u8; EUI48_BYTES] = [0; EUI48_BYTES];
        self.read(EUI48_MEMORY_ADDRESS, &mut eui48)?;
        Ok(eui48)
    }
}
