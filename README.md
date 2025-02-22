# RISCV-SIM

This program will read in either a memory map, specified in a custom format, or
a flat binary file. By default the return value from the executed code will be
returned as a shell status code, but this can be suppressed with a flag.

Starting the program in single-step mode will return control to a debugger after
every cycle. Alternately, starting without single-step will return control to
the debugger if an EBREAK instruction is encountered.

## Building And Running

Use `cargo build` for a debug build and `cargo build --release` for a
release-optimized build. You can run the program with `cargo run -- [FLAGS]` or,
once the program is compiled, you can run the compiled binary in
`target/(release/debug)/ece586-riscv-sim`.

## Virtual Hardware Devices

These should have their own docs in the docs/ folder. Each one will have a
configurable base memory offset.

Planned devices are:

- PCM audio device
- Serial port
- PS/2 Keyboard
- Framebuffer

## Supported Extensions

- RV32I

## USAGE
```
Usage: ece586-riscv-sim [OPTIONS] [FILE]

Arguments:
  [FILE]  [default: program.mem]

Options:
  -v, --verbose
      --single-step
  -a, --starting-addr <STARTING_ADDR>  [default: 0]
  -s, --stack-addr <STACK_ADDR>        [default: 65536]
  -m, --memory-top <MEMORY_TOP>        [default: 65536]
  -h, --help                           Print help
  -V, --version                        Print version
```
