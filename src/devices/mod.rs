use std::collections::HashMap;
use std::ops;
#[derive(Clone,Debug)]
pub struct DeviceConfig {
    name: String,
    options: HashMap<String,String>
}

// These should return Results, I'm not sure what error though
pub trait Device {
    type Error;
    /// Read a byte from the device
    fn read_byte(&self, addr: u32) -> Result<i8,Self::Error>;
    /// Store a byte in the device
    fn store_byte(&self, addr: u32, data: u8) -> Result<(),Self::Error>;
    /// The memory range associated with a device
    fn memory_range(&self) -> ops::Range<u32>;
    /// Create a new copy of this device
    fn initialize(init: DeviceConfig) -> Self;
}
