/*
 * cpu.rs - Contains all code relating to the Cpu struct
 * See Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

pub mod alu;
pub mod instruction;
pub mod memory;
pub mod registers;
mod utils;

use alu::*;
use instruction::*;
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
    fn read_next(&mut self, size: MemorySize) -> Result<RegisterValue, String> {
        // get the current value of the program counter
        let pc_val = self.reg_array.read_reg(Register::PC);

        // get the value in memory at the program counter
        let value = self.memory.read(pc_val, size)?;

        // increment the program counter by an appropriate amount
        let pc_inc_val = size.n_bytes() as u16;
        let new_pc_val = u16::from(pc_val) + pc_inc_val;
        self.reg_array
            .write_reg(Register::PC, RegisterValue::from(new_pc_val))?;

        // return the read value
        Ok(value)
    }

    // decodes the instruction at the current program counter into an Instruction enum
    fn decode_next_instruction(&mut self) -> Result<Instruction, String> {
        let instruction = self.read_next(MemorySize::Integer8)?;
        Instruction::decode(instruction)
    }

    // evaluates the value of a InstructionSource into a RegisterValue
    fn evaluate_source(&mut self, source: InstructionSource) -> Result<RegisterValue, String> {
        use InstructionSource::*;

        match source {
            // if the source value is contained in memory
            Memory(memory_source, size) => {
                use MemorySource::*;

                match memory_source {
                    // if the memory source contains the address directly
                    Address(addr) => self.memory.read(addr, size),

                    // if the memory source address is contained in a register
                    Register(register) => {
                        let addr = self.reg_array.read_reg(register);

                        self.memory.read(addr, size)
                    }

                    // if the memory source address is the program counter
                    // this will also increase the program counter by an
                    // appropriate amount
                    ProgramCounter => self.read_next(size),
                }
            }

            // if the source value is contained in a register
            Register(register) => Ok(self.reg_array.read_reg(register)),

            // if the source is contained in the accumulator (A register)
            Accumulator => Ok(self.alu.accumulator()),
        }
    }

    // writes a RegisterValue to an InstructionSource
    fn write_to_source(
        &mut self,
        source: InstructionSource,
        value: RegisterValue,
    ) -> Result<(), String> {
        use InstructionSource::*;

        match source {
            // if the source is contained in memory
            Memory(memory_source, size) => {
                use MemorySource::*;

                // make sure the RegisterValue is the same size as the MemorySize
                if value.n_bytes() != size.n_bytes() {
                    return Err(format!("Attempted to write a value of size {} to a memory address expecting size {}", value.n_bytes(), size.n_bytes()));
                }

                // figure out where to write to
                let addr = match memory_source {
                    Address(addr) => addr,
                    Register(register) => self.reg_array.read_reg(register),
                    ProgramCounter => {
                        return Err(String::from("Attempted to write to value at PC"))
                    }
                };

                // write to the address
                self.memory.write(addr, value)?;
            }

            // if the source is a register
            Register(register) => {
                self.reg_array.write_reg(register, value)?;
            }

            // if the source is the accumulator (A register)
            Accumulator => {
                self.alu.write_accumulator(value)?;
            }
        }

        Ok(())
    }
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
        assert_eq!(
            cpu.read_next(MemorySize::Integer16).unwrap(),
            RegisterValue::from(0x0123u16)
        );
        assert_eq!(
            cpu.read_next(MemorySize::Integer8).unwrap(),
            RegisterValue::from(0x67u8)
        );
        assert_eq!(
            cpu.read_next(MemorySize::Integer8).unwrap(),
            RegisterValue::from(0x45u8)
        );

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
            .evaluate_source(InstructionSource::Memory(
                MemorySource::Address(RegisterValue::from(0x1000u16)),
                MemorySize::Integer16,
            ))
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
            .evaluate_source(InstructionSource::Memory(
                MemorySource::Register(Register::HL),
                MemorySize::Integer16,
            ))
            .unwrap();
        assert_eq!(value, RegisterValue::from(0x1234u16));

        // set PC to 0x1000, read from PC
        cpu.reg_array
            .write_reg(Register::PC, RegisterValue::from(0x1000u16))
            .unwrap();
        let value = cpu
            .evaluate_source(InstructionSource::Memory(
                MemorySource::ProgramCounter,
                MemorySize::Integer16,
            ))
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

    #[test]
    fn cpu_write_to_instruction_source() {
        let mut cpu = Cpu::new();

        // write a dummy value to some InstructionSources and read them back,
        // verifying that they are the same value
        // write to a single register
        cpu.write_to_source(
            InstructionSource::Register(Register::B),
            RegisterValue::from(0xFFu8),
        )
        .unwrap();

        assert_eq!(
            cpu.evaluate_source(InstructionSource::Register(Register::B))
                .unwrap(),
            RegisterValue::from(0xFFu8)
        );

        // write to a register pair
        cpu.write_to_source(
            InstructionSource::Register(Register::DE),
            RegisterValue::from(0x1234u16),
        )
        .unwrap();

        assert_eq!(
            cpu.evaluate_source(InstructionSource::Register(Register::DE))
                .unwrap(),
            RegisterValue::from(0x1234u16)
        );

        // write to memory: 8-bit explicit address
        cpu.write_to_source(
            InstructionSource::Memory(
                MemorySource::Address(RegisterValue::from(0x1000u16)),
                MemorySize::Integer8,
            ),
            RegisterValue::from(0xABu8),
        )
        .unwrap();

        assert_eq!(
            cpu.evaluate_source(InstructionSource::Memory(
                MemorySource::Address(RegisterValue::from(0x1000u16)),
                MemorySize::Integer8
            ))
            .unwrap(),
            RegisterValue::from(0xABu8)
        );

        // write to memory: 16-bit explicit address
        cpu.write_to_source(
            InstructionSource::Memory(
                MemorySource::Address(RegisterValue::from(0x2000u16)),
                MemorySize::Integer16,
            ),
            RegisterValue::from(0xABCDu16),
        )
        .unwrap();

        assert_eq!(
            cpu.evaluate_source(InstructionSource::Memory(
                MemorySource::Address(RegisterValue::from(0x2000u16)),
                MemorySize::Integer16
            ))
            .unwrap(),
            RegisterValue::from(0xABCDu16)
        );

        // write to memory: 16-bit from register
        cpu.write_to_source(
            InstructionSource::Memory(MemorySource::Register(Register::DE), MemorySize::Integer16),
            RegisterValue::from(0x1234u16),
        )
        .unwrap();

        assert_eq!(
            cpu.evaluate_source(InstructionSource::Memory(
                MemorySource::Register(Register::DE),
                MemorySize::Integer16
            ))
            .unwrap(),
            RegisterValue::from(0x1234u16)
        );
    }
}
