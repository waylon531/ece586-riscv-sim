/* struct to represent command sent to machine  */
enum control_code {
  RUN,
  STOP,
  STEP,
  RESET,
  LOAD,
  POKE,
  POKEREG,
  SETBREAK,
  WATCH,
  WATCHREG,
  JMP
}
struct control_cmd {
  

}