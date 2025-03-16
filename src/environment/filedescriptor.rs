use std::fs::{self, File};

use crate::ReadFileError;

pub enum OpenFlags {
  OAccMode = 0o00000003,
  ORdOnly = 0o00000000,
  OWrOnly = 0o00000001,
  ORdWr = 0o00000002,
  OCreat = 0o00000100,
  OExcl = 0o00000200,
  ONoCtty = 0o00000400,
  OTrunc = 0o00001000,
  OAppend = 0o00002000,
  ONonBlock = 0o00004000,
  ODSync = 0o00010000,
  FAsync = 0o00020000,
  ODirect = 0o00040000,
  OLargeFile = 0o00100000,
  ODirectory = 0o00200000,
  ONoFollow = 0o00400000,
  ONoAtime = 0o01000000,
  OCloExec = 0o02000000,
  OSync = 0o04000000,
  OPath = 0o010000000,
  OTmpFile = 0o020000000
}

pub struct FileDescriptor {
  f: File,
  flags: u32
}

pub struct FileDescriptorTable {
  file_descriptors: Vec<FileDescriptor>
}
impl FileDescriptorTable {
  pub fn new() -> Self {
    FileDescriptorTable {
      file_descriptors: Vec::new()
    }
  }
  pub fn open(&mut self, filename: &Vec<u8>, flags: u32 ) -> Result<i32,ReadFileError> {
    Ok(0)
  }
}

impl FileDescriptor {
  
  pub fn new(filename: &str, flags: u32) -> Result<Self,ReadFileError> {
    Ok(FileDescriptor { 
      f: File::create(filename)?,
      flags: flags
    })
  }
  fn has_flag(&mut self, flag: OpenFlags) -> bool {
    self.flags & flag as u32 != 0
  }
}
