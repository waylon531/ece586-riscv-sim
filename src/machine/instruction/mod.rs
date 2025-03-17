pub struct Opcode {
    opcode: u8,
    name: String,
    format: InstructionType
}

pub enum IT_T {
    R,
    I,
    S,
    B,
    J,
    U
}

struct InstructionType {
    fields: Vec<RawField>,
    name: String,
}
impl InstructionType {
    pub fn new(&mut self, name: &str) -> InstructionType {
        InstructionType{ fields: Vec::new(), name: str::to_string(name) }
    }
}


//struct Opcode {

//}


struct RawField {

    extract: fn(u32) -> u32,
}
impl RawField {
    pub fn new(&mut self, f: fn(u32) -> u32) {
        self.extract = f;
    }
} 
*/