use crate::register::Register;

struct Machine {
    // Maybe this should be on the heap
    // How is memory mapped? Is there max 64K? Or is that just the size of program
    // we can load?
    memory: [u8; 64*1024],
    // Store x1-x31
    // x0 is always 0, no reason to store
    registers: [u32;31],
    pc: u32
}
impl Machine {
    pub fn new(starting_addr: u32, stack_addr: u32) -> Self{
        let mut m = Machine {
                    memory: [0; 64*1024], 
                    registers: [0;31],
                    pc: starting_addr
        };
        m.set_reg(Register::SP,stack_addr);
        m

    }
    pub fn set_reg(&mut self,reg: Register, value: u32) {
        let reg_num = reg.to_num();
        // Writes to the zero register are NOPs
        if reg_num == 0 {
            return
        } else {
            self.registers[reg_num - 1] = value;
        }

    }
}
