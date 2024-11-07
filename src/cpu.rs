/*
 * cpu.rs - Contains all code relating to the CPU struct
 * See Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

pub mod alu;
pub mod memory;
pub mod registers;

use alu::*;
use memory::*;
use registers::*;

// CPU struct - holds all components of the CPU and has I/O functions
pub struct CPU {
    reg_array: RegisterArray,
    alu: ALU,
    memory: Memory,
}

impl CPU {
    // creates a new empty instance of the CPU struct
    pub fn new() -> Self {
        Self {
            reg_array: RegisterArray::new(),
            alu: ALU::new(),
            memory: Memory::new(),
        }
    }

    // reads a RegisterValue at the current program counter, also increments
    // the program counter by an appropriate amount
    fn read_next(&mut self, read_16: bool) -> Result<RegisterValue, String> {
        // get the current value of the program counter
        let pc_val = self.reg_array.read_reg(Register::PC);

        // get the value in memory at the program counter
        let value = self.memory.read(pc_val, read_16)?;

        // increment the program counter by an appropriate amount
        let pc_inc_val = if read_16 { 2u16 } else { 1u16 };
        let new_pc_val = u16::from(pc_val) + pc_inc_val;
        self.reg_array
            .write_reg(Register::PC, RegisterValue::from(new_pc_val))?;

        // return the read value
        Ok(value)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_read_next_value() {
        let mut cpu = CPU::new();

        // write some values to program memory, modify PC, and read those
        // values back and make sure they are the same values that were written
        cpu.memory
            .write(
                RegisterValue::from(0x1000u16),
                RegisterValue::from(0x0123u16),
            )
            .unwrap();

        cpu.memory
            .write(
                RegisterValue::from(0x1002u16),
                RegisterValue::from(0x4567u16),
            )
            .unwrap();

        // set PC to 0x1000
        cpu.reg_array
            .write_reg(Register::PC, RegisterValue::from(0x1000u16))
            .unwrap();

        // should have been written little-endian
        assert_eq!(cpu.read_next(true).unwrap(), RegisterValue::from(0x0123u16));
        assert_eq!(cpu.read_next(false).unwrap(), RegisterValue::from(0x67u8));
        assert_eq!(cpu.read_next(false).unwrap(), RegisterValue::from(0x45u8));

        assert_eq!(
            cpu.reg_array.read_reg(Register::PC),
            RegisterValue::from(0x1004u16)
        );
    }
}
