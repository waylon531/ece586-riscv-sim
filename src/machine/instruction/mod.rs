mod parse;
pub use parse::ParseError;

// Declare extensions here
mod integer_base;

pub struct Opcode {
    opcode: u8,
    name: String,
    format: Format
}

enum FormatName {
    RV32I(integer_base::RV32IFormatName)
}
pub struct Format {
    fields: Vec<Field>,
    name: FormatName,
}


pub struct Field {
    // a field is specified by an array of start and end positions. usually, there's only one, but sometimes things are chopped up
    sections: Vec<(u8,u8)>,
    name: FieldName
}

enum FieldName {
    RV32I(integer_base::RV32IFieldName)
}