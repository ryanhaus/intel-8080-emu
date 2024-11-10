/*
 * instruction.rs - contains code relating to instruction decoding and the
 * Instruction enum
 * See the Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

use super::memory::*;
use super::registers::*;
use super::utils;

// MemorySource enum - represents a source of something in memory
#[derive(Debug, PartialEq)]
pub enum MemorySource {
    Address(RegisterValue),
    Register(Register),
    ProgramCounter,
}

// InstructionSource enum - represents the source of data to be passed to an
// instruction, will be converted into a RegisterValue during execution
#[derive(Debug, PartialEq)]
pub enum InstructionSource {
    Memory(MemorySource, MemorySize),
    Register(Register),
    Accumulator,
}

impl InstructionSource {
    // returns an InstructionSource based on its ID
    pub fn from_id(id: u8) -> Result<Self, String> {
        match id {
            0b000..=0b101 => Ok(InstructionSource::Register(Register::from_reg_id(id)?)),
            0b110 => Ok(InstructionSource::Memory(
                MemorySource::Register(Register::HL),
                MemorySize::Integer8,
            )),
            0b111 => Ok(InstructionSource::Accumulator),
            _ => Err(format!("Unknown InstructionSource ID: {id}")),
        }
    }
}

// Instruction enum - represents a single instruction and all data required
// to execute it
#[derive(Debug, PartialEq)]
pub enum Instruction {
    Add(InstructionSource, InstructionSource),
    AddWithCarry(InstructionSource, InstructionSource),
    Subtract(InstructionSource, InstructionSource),
    SubtractWithBorrow(InstructionSource, InstructionSource),
    BitwiseAnd(InstructionSource, InstructionSource),
    BitwiseXor(InstructionSource, InstructionSource),
    BitwiseOr(InstructionSource, InstructionSource),
    Comparison(InstructionSource, InstructionSource),
    Nop,
}

impl Instruction {
    // decodes a given instruction as a RegisterValue into an Instruction enum
    pub fn decode(instruction: RegisterValue) -> Result<Instruction, String> {
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
            // NOP
            [0, 0, 0, 0, 0, 0, 0, 0] => Ok(Instruction::Nop),

            // A <- A [ALU operation] SSS
            [1, 0, _, _, _, _, _, _] => {
                let src_a = InstructionSource::Accumulator;
                let src_b = InstructionSource::from_id(sss)?;

                Instruction::alu_instr_from_id(alu, src_a, src_b)
            }

            // unknown/unsupported instruction code
            _ => Err(String::from(
                "Unknown/unsupported instruction: {instruction:08b}",
            )),
        }
    }

    fn alu_instr_from_id(
        alu: u8,
        src_a: InstructionSource,
        src_b: InstructionSource,
    ) -> Result<Self, String> {
        match alu {
            0 => Ok(Instruction::Add(src_a, src_b)),
            1 => Ok(Instruction::AddWithCarry(src_a, src_b)),
            2 => Ok(Instruction::Subtract(src_a, src_b)),
            3 => Ok(Instruction::SubtractWithBorrow(src_a, src_b)),
            4 => Ok(Instruction::BitwiseAnd(src_a, src_b)),
            5 => Ok(Instruction::BitwiseXor(src_a, src_b)),
            6 => Ok(Instruction::BitwiseOr(src_a, src_b)),
            7 => Ok(Instruction::Comparison(src_a, src_b)),
            _ => Err(format!("Unknown ALU operation ID: {alu}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruction_decode() {
        // ADD B
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1000_0000u8)).unwrap(),
            Instruction::Add(
                InstructionSource::Accumulator,
                InstructionSource::Register(Register::B)
            )
        );

        // ADC C
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1000_1001u8)).unwrap(),
            Instruction::AddWithCarry(
                InstructionSource::Accumulator,
                InstructionSource::Register(Register::C)
            )
        );

        // SUB D
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1001_0010u8)).unwrap(),
            Instruction::Subtract(
                InstructionSource::Accumulator,
                InstructionSource::Register(Register::D)
            )
        );

        // SBB E
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1001_1011u8)).unwrap(),
            Instruction::SubtractWithBorrow(
                InstructionSource::Accumulator,
                InstructionSource::Register(Register::E)
            )
        );

        // ANA H
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1010_0100u8)).unwrap(),
            Instruction::BitwiseAnd(
                InstructionSource::Accumulator,
                InstructionSource::Register(Register::H)
            )
        );

        // XRA L
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1010_1101u8)).unwrap(),
            Instruction::BitwiseXor(
                InstructionSource::Accumulator,
                InstructionSource::Register(Register::L)
            )
        );

        // ORA M
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1011_0110u8)).unwrap(),
            Instruction::BitwiseOr(
                InstructionSource::Accumulator,
                InstructionSource::Memory(
                    MemorySource::Register(Register::HL),
                    MemorySize::Integer8
                )
            )
        );

        // CMP A
        assert_eq!(
            Instruction::decode(RegisterValue::from(0b1011_1111u8)).unwrap(),
            Instruction::Comparison(
                InstructionSource::Accumulator,
                InstructionSource::Accumulator
            )
        );
    }
}
