# REMU

This program will read in either a memory map, specified in a custom format, or
a flat binary file. By default the return value from the executed code will be
returned as a shell status code, but this can be suppressed with a flag.


## Debugger
There is an included debugger, and this can be accessed by either:
- Starting the program with the `--single-step` flag, which will instantly enter
  the debugger.
- Pressing Ctrl-C during execution will drop you into the debugger. Pressing
  Ctrl-C in the debugger will exit the program.
- Hitting a hardcoded EBREAK will enter the debugger.

Here is a listing of the debugger commands.
```
PEEK    [format]    <addr/reg>      # Read data at a memory location or from a register
                                    # Valid formats are /x (hex), /u (unsigned),
                                    # /i (integer), and /b (binary)
POKE    <addr/reg>      <data>      # Modify data at a memory location or in a register
WATCH   [format]    <addr/reg>      # Read data every time control is returned
                                    # to the debugger
RMWATCH             <addr/reg>      # Stop watching a variable
STEP    [count]                     # Step once, or the given number of times
BREAK   [address]                   # Set a breakpoint at the given address
RMBRK   [address/num]               # Remove a breakpoint at the given address
                                    # or by breakpoint index
LSBRK                               # List out all breakpoints
CONTINUE                            # Return control to the program and run
                                    # until a breakpoint is hit
RUN                                 # Synonym for CONTINUE
EXIT                                # Close the emulator
HELP                                # Show this help message
```

## Building And Running

Use `cargo build` for a debug build and `cargo build --release` for a
release-optimized build. You can run the program with `cargo run -- [FLAGS]` or,
once the program is compiled, you can run the compiled binary in
`target/(release/debug)/ece586-riscv-sim`.

## Demos

There are a handful of demos in the `programs/` directory. There is an included
Makefile and the default target, ran with `make`, will compile all demo
programs. You can then run them with `cargo run --release -- <OPTIONS> FILE.MEM`.
If you want to write your own programs then the Makefile should be able to
compile any .c or .s files and convert them to the required memory map format.

- checker.mem
  - Prints an alternating black and white checkerboard pattern to the
    framebuffer.
  - `cargo run --release -- --device fb checker.mem`
- jpg\_decode/jpg\_decode.mem
  - Decompresses and displays a jpg stored in .rodata
  - `cargo run --release -- --device fb --device serial jpg_decode/jpg_decode.mem`
- echo.mem
  - Reads input and displays it back to you on the serial terminal.
  - `cargo run --release -- --device serial echo.mem`

## Virtual Hardware Devices

There are two virtual hardware devices, a serial port and framebuffer. These
devices are IO mapped and every device must start in the highest 16th of memory,
an address of the form 0xF000000. Every device is configurable, and options are
given in the form:

```
-device name,option1=foo,option2=bar
```

#### Serial
The serial port by default has a base address of `0xF000003F8`. Writing and
reading are both instantaneous, but to make sure there is data available to read
and not just garbage you should poll `(base + 5) & 1`. The lowest bit in the line
status register will go high when there is data available.

You can connect to the serial device with `screen`. REMU will print which pts
device to connect to on startup.

##### Options
- `address`
  - Set the base address to an alternative of your choice.

#### Framebuffer
The framebuffer is linear and by default starts at address `0xFF000000`. The
framebuffer consists of 32-bit 0RGB values and has a resolution of 160x144.

##### Options
- `address`
  - Set the base address to an alternative of your choice.


## Supported Extensions

- RV32I
- M

## USAGE
```
Usage: ece586-riscv-sim [OPTIONS] [FILE]

Arguments:
  [FILE]  [default: program.mem]

Options:
  -v, --verbose                        
  -q, --quiet                          Don't print registers when the program finishes execution
      --single-step                    Start the program in the debugger with the ability to single-step
  -a, --starting-addr <STARTING_ADDR>  [default: 0]
  -s, --stack-addr <STACK_ADDR>        
  -m, --memory-top <MEMORY_TOP>        [default: 65536]
  -W                                   Enable the webui
  -d, --dump-to <DUMP_TO>              Dump machine state to filename DUMP_TO when finished
      --dump-fmt <DUMP_FMT>            [default: txt] [possible values: json, txt]
      --suppress-status                Suppress exit code returned from emulated program
      --device <DEVICE>                Enable a specific device. Format is `--device NAME,opt=foo,opt2=foo2`
  -h, --help                           Print help
  -V, --version                        Print version

```
