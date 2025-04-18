# Intel 8080 Emulator
This project is an emulator for the Intel 8080 microprocessor (or, more accurately, a system containing an Intel 8080 and 64KB of memory).

## Running
To run the project, run `cargo r -- [ROM file]`.<br/>
Running `cargo r` or `cargo r -- --help` will show a help menu as well.<br/>
For example, to run the TST8080.COM ROM:<br/>
```
$ cargo r -- roms/TST8080.COM
MICROCOSM ASSOCIATES 8080/8085 CPU DIAGNOSTIC
 VERSION 1.0  (C) 1980

 CPU IS OPERATIONAL
```
Tests can be run with `cargo t`.

## Future ideas
- I think it would be a good idea to take advantage of Rust's traits for things such as instructions or sources to instructions. The current method of doing things is a little bit messy.
- It would also be cool to get some actual programs such as Space Invaders or CP/M running on this implementation.
- I would be interested in making another version of this written in Verilog, and getting it running on an FPGA dev board.
- Or, as an intermediate step, getting this running on a microcontroller such as RP2040 or STM32.
- I would also like to emulate some other microprocessors/systems such as the 6502, or some game consoles such as the GameBoy.
