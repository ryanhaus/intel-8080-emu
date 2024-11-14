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

// macro to help with debug output
const DEBUG_OUTPUT: bool = true;

macro_rules! dbg_print {
    ( $x:expr ) => {
        if DEBUG_OUTPUT {
            print!($x);
        }
    };
}
macro_rules! dbg_println {
    ( $x:expr ) => {
        if DEBUG_OUTPUT {
            println!($x);
        }
    };
}

// PortHandlerFn - type of a function that handles a port write
// when a port gets written to, this function gets called with the port as the
// first argument, and the value written as the second.
type PortHandlerFn = fn(RegisterValue, RegisterValue);

// Cpu struct - holds all components of the CPU and has I/O functions
#[derive(Debug)]
pub struct Cpu {
    running: bool,
    interrupts_enabled: bool,
    reg_array: RegisterArray,
    alu: Alu,
    memory: Memory,
    ports: [RegisterValue; 0x100],
    port_handler_fn: Option<PortHandlerFn>,
}

impl Cpu {
    // creates a new empty instance of the Cpu struct
    pub fn new() -> Self {
        Self {
            running: true,
            interrupts_enabled: true,
            reg_array: RegisterArray::new(),
            alu: Alu::new(),
            memory: Memory::new(),
            ports: [RegisterValue::from(0u8); 256],
            port_handler_fn: None,
        }
    }

    // returns whether or not the CPU is running
    pub fn is_running(&self) -> bool {
        self.running
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
        let new_pc_val = u16::from(pc_val).wrapping_add(pc_inc_val);
        self.reg_array
            .write_reg(Register::PC, RegisterValue::from(new_pc_val))?;

        // return the read value
        dbg_println!("read_next: Read {value:X?} from {pc_val:X?}, updated PC to {new_pc_val:X?}");
        Ok(value)
    }

    // decodes the instruction at the current program counter into an Instruction enum
    fn decode_next_instruction(&mut self) -> Result<Instruction, String> {
        let instruction = self.read_next(MemorySize::Integer8)?;
        let instruction = Instruction::decode(instruction)?;

        dbg_println!("decode_next_instruction: {instruction:?}");

        Ok(instruction)
    }

    // evaluates the value of a InstructionSource into a RegisterValue
    fn evaluate_source(&mut self, source: InstructionSource) -> Result<RegisterValue, String> {
        use InstructionSource::*;

        let value = match source {
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

            // if the source is a sum of InstructionSources
            Sum(source1, source2) => {
                let val_src1 = self.evaluate_source(*source1)?;
                let val_src2 = self.evaluate_source(*source2)?;

                val_src1.try_add(val_src2)
            }

            // if the source is a value
            Value(value) => Ok(value),
        };

        value
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

                // if the register being written to is the PSW, the A register
                // and flags must be updated as well
                if matches!(register, registers::Register::PSW) {
                    let value = u16::from(value);
                    let a = (value >> 8) as u8;
                    let a = RegisterValue::from(a);

                    let f = (value & 0xFF) as u8;
                    let f = RegisterValue::from(f);

                    self.alu.write_accumulator(a)?;
                    self.alu.write_flags(AluFlags::from_f(f)?);
                }
            }

            // if the source is the accumulator (A register)
            Accumulator => {
                self.alu.write_accumulator(value)?;
            }

            _ => {
                return Err(String::from(
                    "Tried to write to an unwriteable InstructionSource variant",
                ));
            }
        }

        Ok(())
    }

    // updates the PSW (processor status word), which is equivalent to { A, F }
    fn update_status_word(&mut self) -> Result<(), String> {
        let a = u8::try_from(self.alu.accumulator())?;
        let flags = self.alu.flags();

        // F is equivalent to SZ0A0P1C
        let f_bits = [
            flags.sign as u8,
            flags.zero as u8,
            0,
            flags.aux_carry as u8,
            0,
            flags.parity as u8,
            1,
            flags.carry as u8,
        ];

        // form F from bits
        let f = utils::from_bits(f_bits);

        // form PSW
        let psw = utils::combine_values(a, f);
        let psw = RegisterValue::from(psw);

        // update the PSW
        self.reg_array.write_reg(Register::PSW, psw)?;

        Ok(())
    }

    // loads a vector of u8s to memory
    pub fn load_to_memory(&mut self, data: Vec<u8>, start_addr: u16) -> Result<(), String> {
        let writes = data.iter().enumerate().map(|(i, val)| {
            (
                RegisterValue::from(i as u16 + start_addr),
                RegisterValue::from(*val),
            )
        });

        for (addr, val) in writes {
            self.memory.write(addr, val)?;
        }

        Ok(())
    }

    // executes an instruction
    pub fn execute(&mut self, instruction: Instruction) -> Result<(), String> {
        // make sure to update the status word before anything
        self.update_status_word()?;

        println!("{:X?}: {instruction:?}", u16::from(self.reg_array.read_reg(Register::PC)) - 1);

        // handle the instruction
        use Instruction::*;
        match instruction {
            // no operation, do nothing
            Nop => {}

            // loads a value from the immediate address to the destination
            Load(dest) => {
                let addr = self.read_next(MemorySize::Integer16)?;

                let imm_size = MemorySize::from_bytes(dest.n_bytes()?)?;
                let imm_val = self.memory.read(addr, imm_size)?;

                dbg_println!("execute (Load): ({addr:X?}) = {imm_val:X?} -> {dest:?}");

                self.write_to_source(dest, imm_val)?;
            }

            // stores a value to an immediate address
            Store(source) => {
                let src_size = MemorySize::from_bytes(source.n_bytes()?)?;
                let addr = self.read_next(MemorySize::Integer16)?;
                let dest = InstructionSource::Memory(MemorySource::Address(addr), src_size);
                let value = self.evaluate_source(source)?;
                dbg_println!("execute (Store): {value:X?} -> {dest:?}");

                self.write_to_source(dest, value)?;
            }

            // increments a value
            Increment(source) => {
                let src_size = MemorySize::from_bytes(source.n_bytes()?)?;
                let rhs = match src_size {
                    MemorySize::Integer8 => RegisterValue::from(1u8),
                    MemorySize::Integer16 => RegisterValue::from(1u16),
                };

                let sum = InstructionSource::Sum(
                    Box::new(source.clone()),
                    Box::new(InstructionSource::Value(rhs)),
                );

                let result = self.evaluate_source(sum)?;

                dbg_println!("execute (Increment): {result:X?} -> {source:?}");

                self.write_to_source(source, result)?;
            }

            // decrements a value
            Decrement(source) => {
                let src_size = MemorySize::from_bytes(source.n_bytes()?)?;
                let rhs = match src_size {
                    MemorySize::Integer8 => RegisterValue::from(1u8.wrapping_neg()),
                    MemorySize::Integer16 => RegisterValue::from(1u16.wrapping_neg()),
                };

                let sum = InstructionSource::Sum(
                    Box::new(source.clone()),
                    Box::new(InstructionSource::Value(rhs)),
                );

                let result = self.evaluate_source(sum)?;

                dbg_println!("execute (Decrement): {result:X?} -> {source:?}");

                self.write_to_source(source, result)?;
            }

            // moves a value to another place
            Move(dest, source) => {
                let src_val = self.evaluate_source(source)?;

                dbg_println!("execute (Move): {src_val:X?} -> {dest:?}");

                self.write_to_source(dest, src_val)?;
            }

            // ALU operations
            RotateLeft(_)
            | RotateRight(_)
            | RotateLeftThroughCarry(_)
            | RotateRightThroughCarry(_)
            | DecimalAdjust(_)
            | Complement(_)
            | Add(_, _)
            | AddWithCarry(_, _)
            | Subtract(_, _)
            | SubtractWithBorrow(_, _)
            | BitwiseAnd(_, _)
            | BitwiseXor(_, _)
            | BitwiseOr(_, _)
            | Comparison(_, _)
            | SetCarry
            | ComplementCarry => {
                let alu_op = AluOperation::from_instruction(self, instruction)?;

                dbg_print!("execute (ALU operation): evaluating {alu_op:X?}; ");

                let result = self.alu.evaluate(alu_op)?;

                if let Some(result) = result {
                    dbg_print!("{result:X?} -> A; ");
                } else {
                    dbg_print!("no result; ");
                }

                let flags = self.alu.flags();
                dbg_println!("flags: {flags:?}");
            }

            // DAD (Double Byte Add)
            DoubleByteAdd(rp) => {
                // HL <- HL + RP
                // Carry flag affected
                let hl_val = self.reg_array.read_reg(Register::HL);
                let hl_val = u16::from(hl_val);

                let rp_val = self.evaluate_source(rp)?;
                let rp_val = u16::from(rp_val);

                let new_hl = RegisterValue::from(hl_val.wrapping_add(rp_val));
                self.reg_array.write_reg(Register::HL, new_hl)?;

                let new_carry = (hl_val.checked_add(rp_val) == None);
                if new_carry {
                    self.alu.evaluate(AluOperation::SetCarry)?;
                } else if !self.alu.flags().carry {
                    self.alu.evaluate(AluOperation::ComplementCarry)?;
                }
            }

            // conditional return
            ReturnConditional(condition) => {
                if self.alu.flags().evaluate_condition(condition) {
                    // pop into PC
                    let new_pc = self.pop_from_stack(MemorySize::Integer16)?;
                    dbg_println!("execute (ReturnConditional): {new_pc:X?} -> PC");
                    self.reg_array.write_reg(Register::PC, new_pc)?;
                } else {
                    dbg_println!("execute (ReturnConditional): branch not taken");
                }
            }

            // halt the processor
            Halt => {
                dbg_println!("execute (Halt): halted the processor");
                self.running = false;
            }

            // stack pop
            StackPop(dest) => {
                let pop_size = MemorySize::from_bytes(dest.n_bytes()?)?;
                let stack_val = self.pop_from_stack(pop_size)?;
                dbg_println!("execute (StackPop): {stack_val:X?} -> {dest:?}");

                self.write_to_source(dest, stack_val)?;
            }

            // conditional jump
            JumpConditional(condition) => {
                let addr = self.read_next(MemorySize::Integer16)?;
                if self.alu.flags().evaluate_condition(condition) {
                    dbg_println!("execute (JumpConditional): branch taken, {addr:X?} -> PC");
                    self.reg_array.write_reg(Register::PC, addr)?;
                } else {
                    dbg_println!("execute (JumpConditional): branch not taken");
                }
            }

            // unconditional jump
            Jump => {
                let addr = self.read_next(MemorySize::Integer16)?;

                dbg_println!("execute (Jump): {addr:X?} -> PC");

                self.reg_array.write_reg(Register::PC, addr)?;
            }

            // conditional call
            CallConditional(condition) => {
                let addr = self.read_next(MemorySize::Integer16)?;
                if self.alu.flags().evaluate_condition(condition) {
                    dbg_println!(
                        "execute (CallConditional): branch taken, PC -> stack, {addr:X?} -> PC"
                    );

                    let pc_val = self.reg_array.read_reg(Register::PC);
                    self.push_to_stack(pc_val)?;

                    self.reg_array.write_reg(Register::PC, addr)?;
                } else {
                    dbg_println!("execute (CallConditional): branch not taken");
                }
            }

            // push to stack
            StackPush(source) => {
                let value = self.evaluate_source(source)?;
                dbg_println!("execute (StackPush): {value:X?} -> stack");

                self.push_to_stack(value)?;
            }

            // reset
            Reset(n) => {
                let n = self.evaluate_source(n)?;
                let n = u16::from(n);
                let new_pc = RegisterValue::from(n * 8);
                dbg_println!("execute (Reset): {new_pc:X?} -> PC");

                self.reg_array.write_reg(Register::PC, new_pc)?;
            }

            // unconditional return
            Return => {
                // pop into PC
                let new_pc = self.pop_from_stack(MemorySize::Integer16)?;
                dbg_println!("execute (Return): {new_pc:X?} -> PC");
                self.reg_array.write_reg(Register::PC, new_pc)?;
            }

            // unconditional call
            Call => {
                let addr = self.read_next(MemorySize::Integer16)?;
                // CP/M BDOS subroutine
                if addr == RegisterValue::from(0x0005u16) {
                    dbg_println!("execute (Call): Calling CP/M BDOS subroutine");
                    self.cpm_bdos_subroutine()?;
                }
                else {
                    dbg_println!("execute (Call): PC -> stack, {addr:X?} -> PC");

                    let pc_val = self.reg_array.read_reg(Register::PC);
                    self.push_to_stack(pc_val)?;

                    self.reg_array.write_reg(Register::PC, addr)?;
                }
            }

            // IO output
            IoOut => {
                let port = self.read_next(MemorySize::Integer8)?;
                let a_val = self.alu.accumulator();

                self.write_to_port(port, a_val)?;
            }

            // IO input
            IoIn => {
                let port = self.read_next(MemorySize::Integer8)?;
                let port_val = self.read_port(port)?;

                self.alu.write_accumulator(port_val)?;
            }

            // exchange instruction
            Exchange(src_a, src_b) => {
                let val_a = self.evaluate_source(src_a.clone())?;
                let val_b = self.evaluate_source(src_b.clone())?;

                dbg_println!(
                    "execute (Exchange): {val_a:X?} -> {src_b:?}, {val_b:X?} -> {src_a:?}"
                );

                self.write_to_source(src_a, val_b)?;
                self.write_to_source(src_b, val_a)?;
            }

            // disable interrupts
            DisableInterrupts => {
                dbg_println!("execute (DisableInterrupts): interrupts disabled");
                self.interrupts_enabled = false;
            }

            // enable interrupts
            EnableInterrupts => {
                dbg_println!("execute (EnableInterrupts): interrupts enabled");
                self.interrupts_enabled = true;
            }
        }

        dbg_println!("");

        // update the status word again
        self.update_status_word()?;

        // if we get here, execution was ok
        Ok(())
    }

    // executes the next instruction in memory
    pub fn execute_next(&mut self) -> Result<(), String> {
        if self.running {
            let instruction = self.decode_next_instruction();
            self.execute(instruction?)
        } else {
            Ok(())
        }
    }

    // pushes a value to the stack
    pub fn push_to_stack(&mut self, value: RegisterValue) -> Result<(), String> {
        // get the size of the value
        let value_size = value.n_bytes() as u16;

        // decrease SP by the size of the value
        let sp_decrement = RegisterValue::from(value_size.wrapping_neg());
        let mut sp_val = self.reg_array.read_reg(Register::SP);
        sp_val = sp_val.try_add(sp_decrement)?;
        self.reg_array.write_reg(Register::SP, sp_val)?;

        // write value to (SP)
        self.memory.write(sp_val, value)?;

        Ok(())
    }

    // pops a value from the stack, returns it
    pub fn pop_from_stack(&mut self, size: MemorySize) -> Result<RegisterValue, String> {
        // read from SP
        let mut sp_val = self.reg_array.read_reg(Register::SP);
        let value = self.memory.read(sp_val, size)?;

        // increase SP by size
        let value_size = size.n_bytes() as u16;
        let sp_increment = RegisterValue::from(value_size);
        sp_val = sp_val.try_add(sp_increment)?;
        self.reg_array.write_reg(Register::SP, sp_val)?;

        // return the value
        Ok(value)
    }

    // writes a value to a port
    pub fn write_to_port(
        &mut self,
        port: RegisterValue,
        value: RegisterValue,
    ) -> Result<(), String> {
        if port.n_bytes() != 1 || value.n_bytes() != 1 {
            return Err(String::from("Port IDs and port values are 8 bits"));
        }

        let port_id = u8::try_from(port)? as usize;
        self.ports[port_id] = value;
        
        // if there is a port handler function, call it
        if let Some(port_handler_fn) = self.port_handler_fn {
            port_handler_fn(port, value);
        }

        Ok(())
    }

    // reads a value from a port
    pub fn read_port(&self, port: RegisterValue) -> Result<RegisterValue, String> {
        if port.n_bytes() != 1 {
            return Err(String::from("Port IDs are 8 bits"));
        }

        let port_id = u8::try_from(port)? as usize;
        Ok(self.ports[port_id])
    }

    // sets the port write handler function
    pub fn set_port_handler_fn(&mut self, port_handler_fn: PortHandlerFn) {
        self.port_handler_fn = Some(port_handler_fn);
    }

    // performs the CP/M BDOS subroutine (0x0005)
    fn cpm_bdos_subroutine(&mut self) -> Result<(), String> {
        // 0x05 subroutine:
        //  if C == 9:
        //      output string starting at (DE) until $ character is found
        //  if C == 2:
        //      output single character in E

        match self.reg_array.read_reg(Register::C) {
            // C == 9: output string starting at (DE) until $ character is found
            RegisterValue::Integer8(9) => {
                // store mutable copy of DE, which will be used as the pointer
                let mut str_pointer = self.reg_array.read_reg(Register::DE).clone();

                // until a $ character is found, output the string
                loop {
                    // get the current character
                    let current_char = self.memory.read(str_pointer, MemorySize::Integer8)?;

                    // break condition: character is '$'
                    if current_char == RegisterValue::from('$' as u8) {
                        break;
                    }

                    // write to port, increase pointer
                    self.write_to_port(RegisterValue::from(0u8), current_char)?;
                    str_pointer = str_pointer.try_add(RegisterValue::from(1u16))?;
                }
            }

            // C == 2: output single character in E
            RegisterValue::Integer8(2) => {
                let e_val = self.reg_array.read_reg(Register::E);
                self.write_to_port(RegisterValue::from(0u8), e_val)?;
            }

            // otherwise, do nothing
            _ => {}
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

    #[test]
    fn cpu_stack_push_pop() {
        let mut cpu = Cpu::new();

        // set stack pointer to 0x1000 to start
        cpu.reg_array
            .write_reg(Register::SP, RegisterValue::from(0x1000u16))
            .unwrap();

        // push some arbitrary values, make sure they pop back off the same
        cpu.push_to_stack(RegisterValue::from(0x1234u16)).unwrap();
        cpu.push_to_stack(RegisterValue::from(0xAAu8)).unwrap();
        cpu.push_to_stack(RegisterValue::from(0x5678u16)).unwrap();

        assert_eq!(
            cpu.pop_from_stack(MemorySize::Integer16).unwrap(),
            RegisterValue::from(0x5678u16)
        );

        assert_eq!(
            cpu.pop_from_stack(MemorySize::Integer8).unwrap(),
            RegisterValue::from(0xAAu8)
        );

        assert_eq!(
            cpu.pop_from_stack(MemorySize::Integer16).unwrap(),
            RegisterValue::from(0x1234u16)
        );
    }
}
