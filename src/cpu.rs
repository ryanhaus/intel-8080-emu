pub mod registers;
pub mod alu;

use registers::*;
use alu::*;

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
            alu: ALU::new()
        }
    }
}
