/*
 * cpu.rs - Contains all code relating to the Cpu struct
 * See Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

pub mod alu;
pub mod memory;
pub mod registers;
mod utils;

use alu::*;
use memory::*;
use registers::*;

// Cpu struct - holds all components of the CPU and has I/O functions
pub struct Cpu {
    reg_array: RegisterArray,
    alu: Alu,
    memory: Memory,
}

impl Cpu {
    // creates a new empty instance of the Cpu struct
    pub fn new() -> Self {
        Self {
            reg_array: RegisterArray::new(),
            alu: Alu::new(),
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
    fn decode_next_instruction(&mut self) -> Result<Instruction, String> {
        let instruction = self.read_next(false)?;
        Cpu::decode_instruction(instruction)
    }

    // decodes a given instruction as a RegisterValue into an Instruction enum
    fn decode_instruction(instruction: RegisterValue) -> Result<Instruction, String> {
        // convert the instruction into an array of bits for the match
        let instruction: u8 = instruction.try_into()?;
        let instruction_bits = utils::get_bits(instruction);

        // find helpful selection values
        let rp = (instruction & 0b0011_0000) >> 4; // instruction[5:4]
        let ddd = (instruction & 0b0011_1000) >> 3; // instruction[5:3]
        let (alu, cc, n) = (ddd, ddd, ddd); // instruction[5:3]
        let sss = instruction & 0b0000_0111; // instruction[2:0]

        // determine what the instruction is
        match instruction_bits {
            [0, 0, 0, 0, 0, 0, 0, 0] => Ok(Instruction::Nop),
            _ => Err(String::from(
                "Unknown/unsupported instruction: {instruction}",
            )),
        }
    }

    // evaluates the value of a InstructionSource into a RegisterValue
    fn evaluate_source(&mut self, source: InstructionSource) -> Result<RegisterValue, String> {
        use InstructionSource::*;

        match source {
            // if the source value is contained in memory
            Memory(memory_source) => {
                use MemorySource::*;

                match memory_source {
                    // if the memory source contains the address directly
                    Address(addr, read_16) => self.memory.read(addr, read_16),

                    // if the memory source address is contained in a register
                    Register(register, read_16) => {
                        let addr = self.reg_array.read_reg(register);

                        self.memory.read(addr, read_16)
                    }

                    // if the memory source address is the program counter
                    // this will also increase the program counter by an
                    // appropriate amount
                    ProgramCounter(read_16) => self.read_next(read_16),
                }
            }

            // if the source value is contained in a register
            Register(register) => Ok(self.reg_array.read_reg(register)),

            // if the source is contained in the accumulator (A register)
            Accumulator => Ok(self.alu.accumulator()),
        }
    }
}

// MemorySource enum - represents a source of something in memory
#[derive(Debug, PartialEq)]
enum MemorySource {
    Address(RegisterValue, bool),
    Register(Register, bool),
    ProgramCounter(bool),
}

// InstructionSource enum - represents the source of data to be passed to an
// instruction, will be converted into a RegisterValue during execution
#[derive(Debug, PartialEq)]
enum InstructionSource {
    Memory(MemorySource),
    Register(Register),
    Accumulator,
}

// Instruction enum - represents a single instruction and all data required
// to execute it
#[derive(Debug, PartialEq)]
enum Instruction {
    Nop,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_read_next_value() {
        let mut cpu = Cpu::new();

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

    #[test]
    fn cpu_evaluate_instruction_source() {
        let mut cpu = Cpu::new();

        // write some dummy values into memory/registers, then attempt to evaluate
        // InstructionSource instances to see if they return the correct values
        // write 0x1234 to 0x1000
        cpu.memory
            .write(
                RegisterValue::from(0x1000u16),
                RegisterValue::from(0x1234u16),
            )
            .unwrap();

        // read from explicit memory address
        let value = cpu
            .evaluate_source(InstructionSource::Memory(MemorySource::Address(
                RegisterValue::from(0x1000u16),
                true,
            )))
            .unwrap();
        assert_eq!(value, RegisterValue::from(0x1234u16));

        // write 0x1000 to HL, evaluate HL register
        cpu.reg_array
            .write_reg(Register::HL, RegisterValue::from(0x1000u16))
            .unwrap();
        let value = cpu
            .evaluate_source(InstructionSource::Register(Register::HL))
            .unwrap();
        assert_eq!(value, RegisterValue::Integer8Pair(0x10, 0x00));

        // read the value at the memory address in HL (0x1000)
        let value = cpu
            .evaluate_source(InstructionSource::Memory(MemorySource::Register(
                Register::HL,
                true,
            )))
            .unwrap();
        assert_eq!(value, RegisterValue::from(0x1234u16));

        // set PC to 0x1000, read from PC
        cpu.reg_array
            .write_reg(Register::PC, RegisterValue::from(0x1000u16))
            .unwrap();
        let value = cpu
            .evaluate_source(InstructionSource::Memory(MemorySource::ProgramCounter(
                true,
            )))
            .unwrap();
        assert_eq!(value, RegisterValue::from(0x1234u16));

        // perform a dummy ALU operation and make sure the Accumulator is resolved
        // to the correct value
        cpu.alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(1u8),
                RegisterValue::from(2u8),
            ))
            .unwrap()
            .unwrap();

        let value = cpu.evaluate_source(InstructionSource::Accumulator).unwrap();

        assert_eq!(value, RegisterValue::from(3u8));
    }
}
