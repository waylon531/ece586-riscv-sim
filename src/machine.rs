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
        unimplemented!()
    }
}

//Should all the ABI register names go in here too?
enum Register {
    Zero,
    RA,
    SP,
    GP,
    TP,
    T0,T1,T2,T3,T4,T5,T6,
    // These are two names for the same reg
    S0,FP,
    S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,
    A0,A1,A2,A3,A4,A5,A6,A7,
}
impl Register {
    fn to_num(&self) -> usize {
        unimplemented!()
    }
    pub fn from_num(num: usize) -> Register {
        unimplemented!()
    }
}
