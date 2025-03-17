mod multiply;
mod divide;

use super::ParseError;

#[derive(Debug)]
pub enum ExtOpcode {

}
#[derive(Debug)]
pub enum ExtInsttype {
    

}

#[derive(Debug)]
pub enum ExtSubfields {

}

#[derive(Debug)]
pub enum ExtInst {

}

pub fn try_opcode(opcode: u8) -> Result<ExtOpcode,ParseError> {
    match opcode {
        _ => Err(ParseError::InvalidOpcode(opcode))
    }
}
pub fn try_insttype(opcode:ExtOpcode ) -> ExtInsttype {
    match opcode {
        // No external instruction types are yet defined.
        _ => unreachable!()
    }
}

pub fn try_subfields(iword: u32, insttype:ExtInsttype) ->  ExtSubfields {
    match insttype {
        _ => unreachable!()
    }
}

