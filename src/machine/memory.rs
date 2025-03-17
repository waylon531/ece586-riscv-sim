pub struct Memory {
    memory: Box<[u8]>,
    memory_top: u32
}
impl Memory {
    pub fn read_instruction_bytes(&self, addr: u32) -> Result<&[u8], ExecutionError> {
        // Error out if the address is not aligned on a 32-bit boundary
        if addr & 0b11 != 0 {
            Err(ExecutionError::InstructionAddressMisaligned(addr))
        // If the memory top is zero then assume we are using the full 4GB address space as memory
        } else if self.memory_top == 0 || addr.overflowing_add(4).0 <= self.memory_top {
            Ok(&self.memory[addr as usize .. addr as usize + 4])
        } else {
            Err(ExecutionError::InstructionAccessFault(addr))
        }
    }
    pub fn new(memmap: Box<[u8]>, memory_top: u32) -> Self {
        Memory {
            memory: memmap,    
            // The top of memory, points right above the last usable address
            memory_top: memory_top
        }
    }
    pub fn read_byte(&self, addr: u32) -> Result<i8, ExecutionError> {
        //TODO in future: check if this is a memory mapped device
        if addr < self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize] as i8)
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn read_word(&self, addr: u32) -> Result<u32, ExecutionError> {
        //TODO in future: check if this is a memory mapped device
        if addr.saturating_add(4) <= self.memory_top || self.memory_top == 0 {
            Ok(self.memory[addr as usize] as u32
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u32) << 8)
                + ((self.memory[addr.overflowing_add(2).0 as usize] as u32) << 16)
                + ((self.memory[addr.overflowing_add(3).0 as usize] as u32) << 24))
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn read_halfword(&self, addr: u32) -> Result<i16, ExecutionError> {
        //TODO in future: check if this is a memory mapped device
        if addr.saturating_add(2) <= self.memory_top || self.memory_top == 0 {
            Ok((self.memory[addr as usize] as u16
                + ((self.memory[addr.overflowing_add(1).0 as usize] as u16) << 8)) as i16)
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn store_byte(&mut self, data: u8, addr: u32) -> Result<(), ExecutionError> {
        if addr < self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    pub fn store_halfword(&mut self, data: u16, addr: u32) -> Result<(), ExecutionError> {
        if addr.saturating_add(2) <= self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
    
    pub fn store_word(&mut self, data: u32, addr: u32) -> Result<(), ExecutionError> {
        if addr.saturating_add(4) <= self.memory_top || self.memory_top == 0 {
            self.memory[addr as usize] = data as u8;
            self.memory[addr.overflowing_add(1).0 as usize] = (data >> 8) as u8;
            self.memory[addr.overflowing_add(2).0 as usize] = (data >> 16) as u8;
            self.memory[addr.overflowing_add(3).0 as usize] = (data >> 24) as u8;
            Ok(())
        } else {
            Err(ExecutionError::LoadAccessFault(addr))
        }
    }
}