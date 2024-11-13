/*
 * instruction.rs - contains code relating to instruction decoding and the
 * Instruction enum
 * See the Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

use super::memory::*;
use super::registers::*;
use super::utils;

// MemorySource enum - represents a source of something in memory
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemorySource {
    Address(RegisterValue),
    Register(Register),
    ProgramCounter,
}

// InstructionSource enum - represents the source of data to be passed to an
// instruction, will be converted into a RegisterValue during execution
#[derive(Debug, PartialEq, Clone)]
pub enum InstructionSource {
    Memory(MemorySource, MemorySize),
    Register(Register),
    Accumulator,
    Sum(Box<Self>, Box<Self>),
    Value(RegisterValue),
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

    // returns the number of bytes taken up by self
    pub fn n_bytes(&self) -> Result<usize, String> {
        use InstructionSource::*;

        match &self {
            Memory(_, size) => Ok(size.n_bytes()),
            Register(reg) => Ok(reg.n_bytes()),
            Accumulator => Ok(1),
            _ => Err(String::new()),
        }
    }
}

// InstructionCondition enum - represents a condition that is used by an
// instruction during execution
#[derive(Debug, PartialEq)]
pub enum InstructionCondition {
    NotZero,
    Zero,
    NoCarry,
    Carry,
    ParityOdd,
    ParityEven,
    Plus,
    Minus,
}

impl InstructionCondition {
    // returns an InstructionCondition based on its ID
    pub fn from_id(id: u8) -> Result<Self, String> {
        use InstructionCondition::*;

        match id {
            0b000 => Ok(NotZero),
            0b001 => Ok(Zero),
            0b010 => Ok(NoCarry),
            0b011 => Ok(Carry),
            0b100 => Ok(ParityOdd),
            0b101 => Ok(ParityEven),
            0b110 => Ok(Plus),
            0b111 => Ok(Minus),
            _ => Err(format!("Unknown InstructionCondition ID: {id}")),
        }
    }
}

// Instruction enum - represents a single instruction and all data required
// to execute it
#[derive(Debug, PartialEq)]
pub enum Instruction {
    Nop,
    Load(InstructionSource),
    Store(InstructionSource),
    Increment(InstructionSource),
    Decrement(InstructionSource),
    Move(InstructionSource, InstructionSource),
    RotateLeft(InstructionSource),
    RotateLeftThroughCarry(InstructionSource),
    RotateRight(InstructionSource),
    RotateRightThroughCarry(InstructionSource),
    DecimalAdjust(InstructionSource),
    Complement(InstructionSource),
    SetCarry,
    ComplementCarry,
    Halt,
    Add(InstructionSource, InstructionSource),
    AddWithCarry(InstructionSource, InstructionSource),
    Subtract(InstructionSource, InstructionSource),
    SubtractWithBorrow(InstructionSource, InstructionSource),
    BitwiseAnd(InstructionSource, InstructionSource),
    BitwiseXor(InstructionSource, InstructionSource),
    BitwiseOr(InstructionSource, InstructionSource),
    Comparison(InstructionSource, InstructionSource),
    ReturnConditional(InstructionCondition),
    StackPop(InstructionSource),
    JumpConditional(InstructionCondition),
    Jump,
    CallConditional(InstructionCondition),
    StackPush(InstructionSource),
    Reset(InstructionSource),
    Return,
    Call,
    IoOut,
    IoIn,
    Exchange(InstructionSource, InstructionSource),
    DisableInterrupts,
    EnableInterrupts,
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
        // https://en.wikipedia.org/wiki/Intel_8080
        match instruction_bits {
            // NOP
            [0, 0, 0, 0, 0, 0, 0, 0] => Ok(Instruction::Nop),

            // LXI rp, data: RP <- immediate
            [0, 0, _, _, 0, 0, 0, 1] => Ok(Instruction::Move(
                InstructionSource::Register(Register::from_rp_id(rp)?),
                InstructionSource::Memory(MemorySource::ProgramCounter, MemorySize::Integer16),
            )),

            // SHLD addr: (addr) <- HL
            [0, 0, 1, 0, 0, 0, 1, 0] => Ok(Instruction::Store(InstructionSource::Register(
                Register::HL,
            ))),

            // STA addr: (addr) <- A
            [0, 0, 1, 1, 0, 0, 1, 0] => Ok(Instruction::Store(InstructionSource::Accumulator)),

            // STAX rp: (RP) <- A [BC or DE only]
            [0, 0, _, _, 0, 0, 1, 0] => {
                let reg_pair = Register::from_rp_id(rp)?;

                if reg_pair != Register::BC && reg_pair != Register::DE {
                    return Err(String::from("STAX [rp] is only valid for BC or DE"));
                }

                Ok(Instruction::Move(
                        InstructionSource::Memory(
                            MemorySource::Register(reg_pair),
                            MemorySize::Integer16
                        ),
                        InstructionSource::Register(reg_pair)
                ))
            }

            // INX rp: RP <- RP + 1
            [0, 0, _, _, 0, 0, 1, 1] => Ok(Instruction::Increment(InstructionSource::Register(
                Register::from_rp_id(rp)?,
            ))),

            // INR ddd: DDD <- DDD + 1
            [0, 0, _, _, _, 1, 0, 0] => {
                Ok(Instruction::Increment(InstructionSource::from_id(ddd)?))
            }

            // DCR ddd: DDD <- DDD - 1
            [0, 0, _, _, _, 1, 0, 1] => {
                Ok(Instruction::Decrement(InstructionSource::from_id(ddd)?))
            }

            // MVI ddd, data: DDD <- immediate
            [0, 0, _, _, _, 1, 1, 0] => Ok(Instruction::Move(
                InstructionSource::from_id(ddd)?,
                InstructionSource::Memory(MemorySource::ProgramCounter, MemorySize::Integer8),
            )),

            // DAD rp: HL <- HL + RP
            [0, 0, _, _, 1, 0, 0, 1] => Ok(Instruction::Move(
                InstructionSource::Register(Register::HL),
                InstructionSource::Sum(
                    Box::new(InstructionSource::Register(Register::HL)),
                    Box::new(InstructionSource::Register(Register::from_rp_id(rp)?)),
                ),
            )),

            // LHLD addr: HL <- (addr)
            [0, 0, 1, 0, 1, 0, 1, 0] => {
                Ok(Instruction::Load(InstructionSource::Register(Register::HL)))
            }

            // LDA addr, A <- (addr)
            [0, 0, 1, 1, 1, 0, 1, 0] => Ok(Instruction::Load(InstructionSource::Accumulator)),

            // LDAX rp: A <- (RP) [BC or DE only]
            [0, 0, _, _, 1, 0, 1, 0] => {
                let reg_pair = Register::from_rp_id(rp)?;

                if reg_pair != Register::BC && reg_pair != Register::DE {
                    return Err(String::from("LDAX [rp] is only valid for BC or DE"));
                }

                Ok(Instruction::Move(
                        InstructionSource::Accumulator,
                        InstructionSource::Memory(
                            MemorySource::Register(reg_pair),
                            MemorySize::Integer8
                        )
                ))
            }

            // DCX rp: RP <- RP - 1
            [0, 0, _, _, 1, 0, 1, 1] => Ok(Instruction::Decrement(InstructionSource::Register(
                Register::from_rp_id(rp)?,
            ))),

            // RLC: rotate A left through carry
            [0, 0, 0, 0, 0, 1, 1, 1] => Ok(Instruction::RotateLeftThroughCarry(
                InstructionSource::Accumulator,
            )),

            // RRC: rotate A right through carry
            [0, 0, 0, 0, 1, 1, 1, 1] => Ok(Instruction::RotateRightThroughCarry(
                InstructionSource::Accumulator,
            )),

            // RAL: rotate A left
            [0, 0, 0, 1, 0, 1, 1, 1] => Ok(Instruction::RotateLeft(InstructionSource::Accumulator)),

            // RAL: rotate A right
            [0, 0, 0, 1, 1, 1, 1, 1] => {
                Ok(Instruction::RotateRight(InstructionSource::Accumulator))
            }

            // DAA: decimal adjust A
            [0, 0, 1, 0, 0, 1, 1, 1] => {
                Ok(Instruction::DecimalAdjust(InstructionSource::Accumulator))
            }

            // CMA: complement A
            [0, 0, 1, 0, 1, 1, 1, 1] => Ok(Instruction::Complement(InstructionSource::Accumulator)),

            // STC: set carry
            [0, 0, 1, 1, 0, 1, 1, 1] => Ok(Instruction::SetCarry),

            // CMC: complement carry
            [0, 0, 1, 1, 1, 1, 1, 1] => Ok(Instruction::ComplementCarry),

            // HLT: halt
            [0, 1, 1, 1, 0, 1, 1, 0] => Ok(Instruction::Halt),

            // MOV ddd,sss: DDD <- SSS
            [0, 1, _, _, _, _, _, _] => Ok(Instruction::Move(
                InstructionSource::from_id(ddd)?,
                InstructionSource::from_id(sss)?,
            )),

            // A <- A [ALU operation] SSS
            [1, 0, _, _, _, _, _, _] => {
                let src_a = InstructionSource::Accumulator;
                let src_b = InstructionSource::from_id(sss)?;

                Instruction::alu_instr_from_id(alu, src_a, src_b)
            }

            // Rcc: if cc true, return
            [1, 1, _, _, _, 0, 0, 0] => Ok(Instruction::ReturnConditional(
                InstructionCondition::from_id(cc)?,
            )),

            // POP rp: pops value from stack into RP
            [1, 1, _, _, 0, 0, 0, 1] => {
                let mut reg_pair = Register::from_rp_id(rp)?;

                // for stack operations, SP gets swapped with the PSW
                if reg_pair == Register::SP {
                    reg_pair = Register::PSW;
                }

                Ok(Instruction::StackPop(InstructionSource::Register(reg_pair)))
            }

            // Jcc addr: if cc true, jump to addr (PC <- addr)
            [1, 1, _, _, _, 0, 1, 0] => Ok(Instruction::JumpConditional(
                InstructionCondition::from_id(cc)?,
            )),

            // JMP addr: PC <- addr
            [1, 1, 0, 0, 0, 0, 1, 1] => Ok(Instruction::Jump),

            // Ccc addr: if cc true, call addr (push PC, set PC <- addr)
            [1, 1, _, _, _, 1, 0, 0] => Ok(Instruction::CallConditional(
                InstructionCondition::from_id(cc)?,
            )),

            // PUSH rp: push RP into stack
            [1, 1, _, _, 0, 1, 0, 1] => {
                let mut reg_pair = Register::from_rp_id(rp)?;

                // for stack operations, SP gets swapped with the PSW
                if reg_pair == Register::SP {
                    reg_pair = Register::PSW;
                }

                Ok(Instruction::StackPush(InstructionSource::Register(
                    reg_pair,
                )))
            }

            // A <- A [ALU operation] immediate
            [1, 1, _, _, _, 1, 1, 0] => {
                let src_a = InstructionSource::Accumulator;
                let src_b =
                    InstructionSource::Memory(MemorySource::ProgramCounter, MemorySize::Integer8);

                Instruction::alu_instr_from_id(alu, src_a, src_b)
            }

            // RST n: pushes PC to stack, PC <- n * 8
            [1, 1, _, _, _, 1, 1, 1] => Ok(Instruction::Reset(InstructionSource::Value(
                RegisterValue::from(n as u16),
            ))),

            // RET: PC <- (SP)
            [1, 1, 0, 0, 1, 0, 0, 1] => Ok(Instruction::Return),

            // CALL addr: pushes PC to stack, PC <- addr
            [1, 1, 0, 0, 1, 1, 0, 1] => Ok(Instruction::Call),

            // OUT port: Port <- A
            [1, 1, 0, 1, 0, 0, 1, 1] => Ok(Instruction::IoOut),

            // IN port: A <- Port
            [1, 1, 0, 1, 1, 0, 1, 1] => Ok(Instruction::IoIn),

            // XTHL: HL <-> (SP)
            [1, 1, 1, 0, 0, 0, 1, 1] => Ok(Instruction::Exchange(
                InstructionSource::Register(Register::HL),
                InstructionSource::Memory(
                    MemorySource::Register(Register::SP),
                    MemorySize::Integer16,
                ),
            )),

            // PCHL: PC <-> HL
            [1, 1, 1, 0, 1, 0, 0, 1] => Ok(Instruction::Exchange(
                InstructionSource::Register(Register::PC),
                InstructionSource::Register(Register::HL),
            )),

            // XCHG: HL <-> DE
            [1, 1, 1, 0, 1, 0, 1, 1] => Ok(Instruction::Exchange(
                InstructionSource::Register(Register::HL),
                InstructionSource::Register(Register::DE),
            )),

            // DI: disable interrupts
            [1, 1, 1, 1, 0, 0, 1, 1] => Ok(Instruction::DisableInterrupts),

            // SPHL: SP <- HL
            [1, 1, 1, 1, 1, 0, 0, 1] => Ok(Instruction::Move(
                InstructionSource::Register(Register::SP),
                InstructionSource::Register(Register::HL),
            )),

            // EI: enable interrupts
            [1, 1, 1, 1, 1, 0, 1, 1] => Ok(Instruction::EnableInterrupts),

            // unknown/unsupported instruction code
            _ => Err(format!(
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
    use super::super::registers::Register::*;
    use super::*;
    use Instruction::*;
    use InstructionSource::*;

    macro_rules! instr_decode {
        ($instr:expr) => {
            Instruction::decode(RegisterValue::from($instr)).unwrap()
        };
    }

    #[test]
    fn instruction_decode() {}
}
