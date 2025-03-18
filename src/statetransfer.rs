/* struct to represent command sent to machine  */
use crate::register;
pub enum ControlCode {
  RUN,
  STOP,
  STEP,
  RESET,
  LOAD {
    file: &'static str,
  },
  POKE {
    address: u32,
    value: u32 
  },
  POKEREG {
    register: register::Register,
    value: u32
  },
  SETBREAK {
    address: u32,
  },
  WATCH {
    address: u32,
  },
  WATCHREG {
    register:register::Register
  },
  JMP {
    address: u32
  },
  SPEED {
    speed: Speed
  }
}
pub enum Speed {
  FAST,
  SLOW
}
#[derive(Clone)]
pub struct MachineState {
  pub pc: u32,
  pub registers: [u32;31],
  pub cur_inst: String,
  pub memory_changes: Vec<(u32,u32)>,
  pub cycle: u128
}
impl MachineState {
  pub fn empty() -> MachineState {
    MachineState { pc: 0, registers: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0], cur_inst: "".to_string(), memory_changes: Vec::new() , cycle:0}
  }
}