# Intel 8080 Emulator
This project is an emulator for the Intel 8080 microprocessor (or, more accurately, a system containing an Intel 8080 and 64KB of memory).

## Running
To run the project, run `cargo r -- [ROM file]`.<br/>
For example, to run the TST8080.COM ROM:<br/>
```
$ cargo r -- roms/TST8080.COM
MICROCOSM ASSOCIATES 8080/8085 CPU DIAGNOSTIC
 VERSION 1.0  (C) 1980

 CPU IS OPERATIONAL
```
Tests can be run with `cargo t`.
