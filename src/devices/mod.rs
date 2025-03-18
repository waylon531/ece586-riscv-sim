mod framebuffer;
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
    pub fn into_device(self) -> Result<Device,DeviceConfigError> {
        match self.name.as_str() {
            "serial" => Ok(serial::Serial::from_config(self.options)?),
            "fb" | "framebuffer" => Ok(framebuffer::Framebuffer::from_config(self.options)?),
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

pub enum Device {
    ByteDevice(Box<dyn ByteDevice>),
    HalfwordDevice(Box<dyn HalfwordDevice>),
    WordDevice(Box<dyn WordDevice>),

}
use Device::*;
impl Device {
    /// Read a byte from the device
    pub fn read_byte(&self, addr: u32) -> Result<i8,Box<dyn Error>> {
        match self {
            ByteDevice(dev) => dev.read_byte(addr),
            WordDevice(dev) => {
                // Which byte in the word to select
                let offset = addr & 0x11;
                let word = dev.read_word(addr)?;
                Ok(((word >> (offset*8)) & 0xFF) as i8)
            },
            _ => unimplemented!()
        }
    }
    /// Store a byte in the device
    pub fn store_byte(&self, addr: u32, data: u8) -> Result<(),Box<dyn Error>> {
        match self {
            ByteDevice(dev) => dev.store_byte(addr,data),
            WordDevice(dev) => {
                let offset = addr & 0x11;
                // Turn on all bits in the byte to be modified
                let word = dev.read_word(addr)? | (0xFF << offset);
                // Mask the bits so the byte is now at the correct value
                let word = word as u32 & ((data as u32) << offset);
                // Write back the correct value
                dev.store_word(addr,word)

            },
            _ => unimplemented!(),
        }
    }
    /// Read a 16-bit halfword from the device
    pub fn read_halfword(&self, addr: u32) -> Result<i16,Box<dyn Error>> {
        match self {
            ByteDevice(dev) => {
                Ok((dev.read_byte(addr)? as u16
                    + ((dev.read_byte(addr.overflowing_add(1).0)? as u16) << 8)) as i16)
            },
            _ => unimplemented!()
        }
    }
    /// Store a 16-bit halfword in the device
    pub fn store_halfword(&self, addr: u32, data: u16) -> Result<(),Box<dyn Error>> {
        match self {
            ByteDevice(dev) => {
                dev.store_byte(addr,data as u8)?;
                dev.store_byte(addr.overflowing_add(1).0,(data >> 8) as u8)?;
                Ok(())
            },
            _ => unimplemented!()
        }
    }
    /// Read a 32-bit word from the device
    pub fn read_word(&self, addr: u32) -> Result<i32,Box<dyn Error>> {
        match self {
            ByteDevice(dev) => {
                Ok((dev.read_byte(addr)? as u32
                    + ((dev.read_byte(addr.overflowing_add(1).0)? as u32) << 8)
                    + ((dev.read_byte(addr.overflowing_add(2).0)? as u32) << 16)
                    + ((dev.read_byte(addr.overflowing_add(3).0)?  as u32) << 24)) as i32)
            },
            WordDevice(dev) => dev.read_word(addr),
            _ => unimplemented!()
        }

    }
    /// Store a 32-bit word in the device
    pub fn store_word(&self, addr: u32, data: u32) -> Result<(),Box<dyn Error>> {
        match self {
            ByteDevice(dev) => {
                dev.store_byte(addr,data as u8)?;
                dev.store_byte(addr.overflowing_add(1).0,(data >> 8) as u8)?;
                dev.store_byte(addr.overflowing_add(2).0,(data >> 16) as u8)?;
                dev.store_byte(addr.overflowing_add(3).0,(data >> 24) as u8)?;
                Ok(())
            },
            WordDevice(dev) => dev.store_word(addr,data),
            _ => unimplemented!()
        }
    }
    /// The memory range associated with a device
    pub fn memory_range(&self) -> ops::RangeInclusive<u32> {
        match self {
            ByteDevice(dev) => dev.memory_range(),
            HalfwordDevice(dev) => dev.memory_range(),
            WordDevice(dev) => dev.memory_range(),
        }
    }
}
