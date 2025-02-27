# Debug Language

This document describes the commands for the debugger built-in to the simulator.
All commands are case insensitive

```
PEEK    [format]    <addr/reg>      # Read data at a memory location or from a register
                                    # NOTE:  s0 shows the integer in s0
                                    #       [s0] dereferences s0 and shows memory contents
POKE    <data>      <addr/reg>      # Modify data at a memory location or in a register
WATCH   [format]    <addr/reg>      # Read data every time control is returned
                                    # to the debugger
STEP    [count]                     # Step once, or the given number of times
BREAK   [address]                   # Set a breakpoint at the given address
RMBRK   [address/num]               # Remove a breakpoint at the given address
                                    # or by breakpoint index
LSBRK                               # List out all breakpoints
CONTINUE                            # Return control to the program and run
                                    # until a breakpoint is hit
RUN                                 # Synonym for CONTINUE
HELP                                # Show this help message
```
