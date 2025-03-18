use std::{collections::VecDeque, fs::{self, File}};

use serde_json::de::Read;



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
  file: File,
  fd: u32,
  flags: u32
}

pub struct FileDescriptorTable {
  freed_fds: VecDeque<u32>,
  file_descriptors: Vec<FileDescriptor>
}
impl FileDescriptorTable {
  pub fn new() -> Self {
    FileDescriptorTable {
      freed_fds : VecDeque::new(),
      file_descriptors: Vec::new()
    }
  }
  fn assignfd(&mut self) -> u32 {
    self.freed_fds.pop_front().unwrap_or(self.file_descriptors.len() as u32 + 2)
  }
  pub fn open(&mut self, filename: &Vec<u8>, flags: u32) -> Result<i32,ReadFileError> {
    let fname = match String::from_utf8(filename.to_vec()) {
      Ok(s) => s,
      Err(e) => { return Err(ReadFileError::ParseError(e.to_string())); }
    };
    let fd = self.assignfd();
    let f = match has_flag(flags, OpenFlags::OCreat) {
      True => File::create(fname)?,
      False => File::open(fname)?
    };
    self.file_descriptors.push(FileDescriptor { file: f, fd: fd, flags: flags});
    Ok(fd as i32)
  }
  pub fn close(&mut self, fd:u32) -> i32 {
    let idx = self.get_idx(fd);
    // if we can't find the file then error
    // TODO: REFACTOR THIS TO USE RUST'S INBUILT RESULT SYSTEM
    if(idx<0) { return -1 };
    let i = idx as usize;
    self.freed_fds.push_back(self.file_descriptors[i].fd);
    self.file_descriptors.swap_remove(i);
    return 0;

  }
  pub fn get_file(&self, fd:u32) -> Option<&File> {
    let idx = self.get_idx(fd);
    if(idx<0) { return None };
    return Some(&self.file_descriptors[idx as usize].file);
  }
  pub fn get_idx(&self, fd:u32) -> i32 {
    match self.file_descriptors.iter().position(|f| f.fd == fd) {
        Some(idx) => idx as i32,
        None => -1
    }
  }
}

fn has_flag(flags: u32, flag: OpenFlags) -> bool {
    flags & flag as u32 != 0
}
