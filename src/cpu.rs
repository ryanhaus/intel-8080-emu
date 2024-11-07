/*
 * cpu.rs - Contains all code relating to the CPU struct
 * See Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

pub mod alu;
pub mod registers;
pub mod memory;

use alu::*;
use registers::*;
use memory::*;

// CPU struct - holds all components of the CPU and has I/O functions
pub struct CPU {
    reg_array: RegisterArray,
    alu: ALU,
}

impl CPU {
    // creates a new empty instance of the CPU struct
    pub fn new() -> Self {
        Self {
            reg_array: RegisterArray::new(),
            alu: ALU::new(),
        }
    }

    // decodes the instruction at the current program counter into an Instruction enum
    fn decode_next_instruction(&self) -> Instruction {
        todo!()
    }
}

// Instruction enum - represents a single instruction and all data required
// to execute it
enum Instruction {
    ALUInstruction(ALUOperation),
}
