/* struct to represent command sent to machine  */
enum control_code {
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