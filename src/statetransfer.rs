/* struct to represent command sent to machine  */
enum ControlCode {
  RUN,
  STOP,
  STEP,
  RESET,
  LOAD {
    file: String,
  },
  POKE {
    
  },
  POKEREG,
  SETBREAK,
  WATCH,
  WATCHREG,
  JMP
}