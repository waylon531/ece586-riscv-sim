mod serial;

use std::collections::HashMap;
use std::num;
use std::ops;
use std::error::Error;

use thiserror;
#[derive(Clone,Debug)]
pub struct DeviceConfig {
    name: String,
    options: HashMap<String,String>
}
impl DeviceConfig {
    pub fn from_str(s: &str) -> Result<DeviceConfig,DeviceConfigError>{
        let mut iter = s.split(',');
        let name = iter.next().ok_or(DeviceConfigError::NameNotFound(s.to_string()))?.to_string();
        let mut options = HashMap::new();
        for entry in iter {
            let (lhs,rhs) = entry.split_once('=').ok_or(DeviceConfigError::UnparseableOption(entry.to_string()))?;
            options.insert(lhs.to_string(),rhs.to_string());
        }
        Ok(DeviceConfig { name, options })
    }
    pub fn into_device(self) -> Result<Box<dyn Device>,DeviceConfigError> {
        match self.name.as_str() {
            "serial" => Ok(serial::Serial::from_config(self.options)?),
            _ => Err(DeviceConfigError::InvalidDeviceName(self.name))

        }
    }
}
#[derive(Debug,thiserror::Error)]
pub enum DeviceConfigError {
    #[error("Empty device name in `{0}`")]
    NameNotFound(String),
    #[error("Unable to parse option: `{0}`")]
    UnparseableOption(String),
    #[error("Invalid option: `{0}`")]
    InvalidOption(String),
    #[error("Device name not recognized: `{0}`")]
    InvalidDeviceName(String),
    #[error("Failed to parse number: {0}")]
    NumberParseError(#[from] num::ParseIntError),
    #[error("{0}")]
    Custom(String),

}

pub trait ByteDevice {
    /// Read a byte from the device
    fn read_byte(&self, addr: u32) -> Result<i8,Box<dyn Error>>;
    /// Store a byte in the device
    fn store_byte(&self, addr: u32, data: u8) -> Result<(),Box<dyn Error>>;
    /// The memory range associated with a device
    fn memory_range(&self) -> ops::RangeInclusive<u32>;
}
pub trait HalfwordDevice {
    /// Read a 16-bit halfword from the device
    fn read_halfword(&self, addr: u32) -> Result<i16,Box<dyn Error>>;
    /// Store a 16-bit halfword in the device
    fn store_halfword(&self, addr: u32, data: u16) -> Result<(),Box<dyn Error>>;
    /// The memory range associated with a device
    fn memory_range(&self) -> ops::RangeInclusive<u32>;
}
pub trait WordDevice {
    /// Read a 32-bit word from the device
    fn read_word(&self, addr: u32) -> Result<i32,Box<dyn Error>>;
    /// Store a 32-bit word in the device
    fn store_word(&self, addr: u32, data: u32) -> Result<(),Box<dyn Error>>;
    /// The memory range associated with a device
    fn memory_range(&self) -> ops::RangeInclusive<u32>;
}

pub trait Device {
    /// Read a byte from the device
    fn read_byte(&self, addr: u32) -> Result<i8,Box<dyn Error>>;
    /// Store a byte in the device
    fn store_byte(&self, addr: u32, data: u8) -> Result<(),Box<dyn Error>>;
    /// Read a 16-bit halfword from the device
    fn read_halfword(&self, addr: u32) -> Result<i16,Box<dyn Error>>;
    /// Store a 16-bit halfword in the device
    fn store_halfword(&self, addr: u32, data: u16) -> Result<(),Box<dyn Error>>;
    /// Read a 32-bit word from the device
    fn read_word(&self, addr: u32) -> Result<i32,Box<dyn Error>>;
    /// Store a 32-bit word in the device
    fn store_word(&self, addr: u32, data: u32) -> Result<(),Box<dyn Error>>;
    /// The memory range associated with a device
    fn memory_range(&self) -> ops::RangeInclusive<u32>;
}
// This impl lets you read and write entire words to a device without having to
// manually implement these methods
impl<T: ByteDevice> Device for T {
    /// Read a byte from the device
    fn read_byte(&self, addr: u32) -> Result<i8,Box<dyn Error>> {
        self.read_byte(addr)
    }
    /// Store a byte in the device
    fn store_byte(&self, addr: u32, data: u8) -> Result<(),Box<dyn Error>> {
        self.store_byte(addr,data)
    }
    /// Read a 16-bit halfword from the device
    fn read_halfword(&self, addr: u32) -> Result<i16,Box<dyn Error>> {
        Ok((self.read_byte(addr)? as u16
                + ((self.read_byte(addr.overflowing_add(1).0)? as u16) << 8)) as i16)
    }
    /// Store a 16-bit halfword in the device
    fn store_halfword(&self, addr: u32, data: u16) -> Result<(),Box<dyn Error>> {
        self.store_byte(addr,data as u8)?;
        self.store_byte(addr.overflowing_add(1).0,(data >> 8) as u8)?;
        Ok(())
    }
    /// Read a 32-bit word from the device
    fn read_word(&self, addr: u32) -> Result<i32,Box<dyn Error>> {
        Ok((self.read_byte(addr)? as u32
            + ((self.read_byte(addr.overflowing_add(1).0)? as u32) << 8)
            + ((self.read_byte(addr.overflowing_add(2).0)? as u32) << 16)
            + ((self.read_byte(addr.overflowing_add(3).0)?  as u32) << 24)) as i32)

    }
    /// Store a 32-bit word in the device
    fn store_word(&self, addr: u32, data: u32) -> Result<(),Box<dyn Error>> {
        self.store_byte(addr,data as u8)?;
        self.store_byte(addr.overflowing_add(1).0,(data >> 8) as u8)?;
        self.store_byte(addr.overflowing_add(2).0,(data >> 16) as u8)?;
        self.store_byte(addr.overflowing_add(3).0,(data >> 24) as u8)?;
        Ok(())
    }
    /// The memory range associated with a device
    fn memory_range(&self) -> ops::RangeInclusive<u32> {
        self.memory_range()
    }
    

}
