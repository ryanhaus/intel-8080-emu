/*
 * register_array.rs - contains code relating to the register array
 * see Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

use super::utils;
use std::{
    convert::{From, TryFrom},
    error::Error,
};

// RegisterArray struct - contains all register values
#[derive(Debug)]
pub struct RegisterArray {
    program_counter: u16, // 16-bit program counter
    stack_pointer: u16,   // 16-bit stack pointer

    // 8-bit registers from the ALU that are used to form the PSW
    reg_a: u8,
    reg_f: u8,

    // 8-bit general purpose registers
    // also can be used as 16-bit registers BC, DE, HL
    reg_b: u8,
    reg_c: u8,
    reg_d: u8,
    reg_e: u8,
    reg_h: u8,
    reg_l: u8,

    // 8-bit temporary registers
    reg_w: u8,
    reg_z: u8,
}

impl RegisterArray {
    // Creates a new instance of RegisterArray, with all values set to 0
    pub fn new() -> Self {
        Self {
            program_counter: 0,
            stack_pointer: 0,
            reg_a: 0,
            reg_f: 0,
            reg_b: 0,
            reg_c: 0,
            reg_d: 0,
            reg_e: 0,
            reg_h: 0,
            reg_l: 0,
            reg_w: 0,
            reg_z: 0,
        }
    }

    // Reads the value of the given register
    pub fn read_reg(&self, register: Register) -> RegisterValue {
        use Register::*;
        use RegisterValue::*;

        match register {
            PC => Integer16(self.program_counter),
            SP => Integer16(self.stack_pointer),
            B => Integer8(self.reg_b),
            C => Integer8(self.reg_c),
            D => Integer8(self.reg_d),
            E => Integer8(self.reg_e),
            H => Integer8(self.reg_h),
            L => Integer8(self.reg_l),
            W => Integer8(self.reg_w),
            Z => Integer8(self.reg_z),
            BC => Integer8Pair(self.reg_b, self.reg_c),
            DE => Integer8Pair(self.reg_d, self.reg_e),
            HL => Integer8Pair(self.reg_h, self.reg_l),
            WZ => Integer8Pair(self.reg_w, self.reg_z),
            PSW => Integer8Pair(self.reg_a, self.reg_f),
        }
    }

    // Writes a given value to the given register
    pub fn write_reg(&mut self, register: Register, value: RegisterValue) -> Result<(), String> {
        use Register::*;

        match register {
            PC => self.program_counter = value.into(),
            SP => self.stack_pointer = value.into(),
            B => self.reg_b = value.try_into()?,
            C => self.reg_c = value.try_into()?,
            D => self.reg_d = value.try_into()?,
            E => self.reg_e = value.try_into()?,
            H => self.reg_h = value.try_into()?,
            L => self.reg_l = value.try_into()?,
            W => self.reg_w = value.try_into()?,
            Z => self.reg_z = value.try_into()?,
            BC => (self.reg_b, self.reg_c) = utils::separate_values(value.into()),
            DE => (self.reg_d, self.reg_e) = utils::separate_values(value.into()),
            HL => (self.reg_h, self.reg_l) = utils::separate_values(value.into()),
            WZ => (self.reg_w, self.reg_z) = utils::separate_values(value.into()),
            PSW => (self.reg_a, self.reg_f) = utils::separate_values(value.into()),
        };

        // if this point is reached, write was successful
        Ok(())
    }
}

// Register enum - contains all possible registers that can be referenced
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Register {
    PC, // 16-bit program counter
    SP, // 16-bit stack pointer

    // 8-bit general purpose registers
    B,
    C,
    D,
    E,
    H,
    L,
    W, // temporary
    Z, // temporary

    // 16-bit register pairs
    BC,
    DE,
    HL,
    WZ, // temporary
    PSW,
}

impl Register {
    // Returns a string literal with a human-readable name for each register
    pub fn get_human_readable_name(&self) -> &'static str {
        use Register::*;

        match self {
            PC => "PC",
            SP => "SP",
            A => "A",
            B => "B",
            C => "C",
            D => "D",
            E => "E",
            H => "H",
            L => "L",
            W => "W",
            Z => "Z",
            BC => "BC",
            DE => "DE",
            HL => "HL",
            WZ => "WZ",
        }
    }

    pub fn from_reg_id(id: u8) -> Result<Self, String> {
        use Register::*;

        match id {
            0b000 => Ok(B),
            0b001 => Ok(C),
            0b010 => Ok(D),
            0b011 => Ok(E),
            0b100 => Ok(H),
            0b101 => Ok(L),
            0b110 => Err(String::from("Attempt to read from the register M, which is not an actual register. Read from memory instead")),
            0b111 => Err(String::from("Attempt to read from register A, which is contained in the ALU, not the register array")),
            _ => Err(format!("Unknown register ID: {id}")),
        }
    }

    pub fn from_rp_id(id: u8) -> Result<Self, String> {
        use Register::*;

        match id {
            0b00 => Ok(Register::BC),
            0b01 => Ok(Register::DE),
            0b10 => Ok(Register::HL),
            0b11 => Ok(Register::SP), // can sometimes refer to PSW
            _ => Err(format!("Unknown register pair ID: {id}")),
        }
    }

    // returns the number of bytes taken up by self
    pub fn n_bytes(&self) -> usize {
        use Register::*;

        match self {
            B | C | D | E | H | L | W | Z => 1,
            PC | SP | BC | DE | HL | WZ | PSW => 2,
        }
    }
}

// RegisterValue enum - could either be a 8-bit or 16-bit integer value
#[derive(Debug, Clone, Copy)]
pub enum RegisterValue {
    Integer8(u8),
    Integer8Pair(u8, u8),
    Integer16(u16),
}

impl PartialEq<RegisterValue> for RegisterValue {
    fn eq(&self, other: &RegisterValue) -> bool {
        let val_lhs = u16::from(*self);
        let val_rhs = u16::from(*other);

        val_lhs == val_rhs
    }
}

impl From<u8> for RegisterValue {
    fn from(value: u8) -> Self {
        Self::Integer8(value)
    }
}

impl From<u16> for RegisterValue {
    fn from(value: u16) -> Self {
        Self::Integer16(value)
    }
}

impl TryFrom<RegisterValue> for u8 {
    type Error = &'static str;

    fn try_from(reg_val: RegisterValue) -> Result<u8, Self::Error> {
        use RegisterValue::*;

        match reg_val {
            Integer8(value) => Ok(value),
            _ => Err("Only an 8-bit register can be converted to u8"),
        }
    }
}

impl From<RegisterValue> for u16 {
    fn from(reg_val: RegisterValue) -> u16 {
        use RegisterValue::*;

        match reg_val {
            Integer8(value) => value as u16,
            Integer8Pair(higher, lower) => utils::combine_values(higher, lower),
            Integer16(value) => value,
        }
    }
}

impl RegisterValue {
    // returns the number of bytes occupied by a value of this RegisterValue type
    pub fn n_bytes(&self) -> usize {
        use RegisterValue::*;

        match self {
            Integer8(_) => 1,
            Integer8Pair(_, _) => 2,
            Integer16(_) => 2,
        }
    }

    // tries to add two RegisterValues together. consumes self and rhs
    pub fn try_add(self, rhs: Self) -> Result<Self, String> {
        use RegisterValue::*;

        // if either operand is an Integer8Pair, convert it to an Integer16 and
        // re-call this method recursively
        if let Integer8Pair(higher, lower) = self {
            let merged = Integer16(utils::combine_values(higher, lower));
            return merged.try_add(rhs);
        }

        if let Integer8Pair(higher, lower) = rhs {
            let merged = Integer16(utils::combine_values(higher, lower));
            return merged.try_add(rhs);
        }

        if matches!(self, Integer8(_)) && matches!(rhs, Integer8(_)) {
            // adding an Integer8 with an Integer8
            let val_lhs = u8::try_from(self)?;
            let val_rhs = u8::try_from(rhs)?;

            Ok(Integer8(val_lhs + val_rhs))
        } else if matches!(self, Integer16(_)) && matches!(rhs, Integer16(_)) {
            // adding an Integer16 with an Integer16
            let val_lhs = u16::from(self);
            let val_rhs = u16::from(rhs);

            Ok(Integer16(val_lhs + val_rhs))
        } else {
            Err(format!("Could not add RegisterValues {self:?} and {rhs:?}"))
        }
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_array_read_write() {
        use rand::prelude::*;
        use Register::*;

        let mut reg_array = RegisterArray::new();

        // random reads + writes
        // generate random values for each of the 8 8-bit registers
        let rand_values: [u8; 8] = rand::random();
        let registers_8 = [B, C, D, E, H, L, W, Z];

        // write the 8 values
        registers_8
            .iter()
            .zip(rand_values.iter())
            .for_each(|(reg, val)| {
                reg_array
                    .write_reg(*reg, RegisterValue::from(*val))
                    .unwrap()
            });

        // read back the 8 values, ensure they are equal to what was written
        registers_8
            .iter()
            .zip(rand_values.iter())
            .for_each(|(reg, val)| assert_eq!(reg_array.read_reg(*reg), RegisterValue::from(*val)));

        // generate random values for each of the 6 16-bit registers
        let rand_values: [u16; 6] = rand::random();
        let registers_16 = [PC, SP, BC, DE, HL, WZ];

        // write the 6 values
        registers_16
            .iter()
            .zip(rand_values.iter())
            .for_each(|(reg, val)| {
                reg_array
                    .write_reg(*reg, RegisterValue::from(*val))
                    .unwrap()
            });

        // read back the 6 values, ensure they are equal to what was written
        registers_16
            .iter()
            .zip(rand_values.iter())
            .for_each(|(reg, val)| assert_eq!(u16::from(reg_array.read_reg(*reg)), *val));

        // verify that the register pairs write to the 8-bit registers correctly
        let registers_and_pairs = [(BC, (B, C)), (DE, (D, E)), (HL, (H, L)), (WZ, (W, Z))];

        registers_and_pairs
            .map(|(a, (b, c))| {
                (
                    reg_array.read_reg(a).into(),
                    (reg_array.read_reg(b).into(), reg_array.read_reg(c).into()),
                )
            })
            .into_iter()
            .for_each(|(reg16, (reg8_high, reg8_low)): (u16, (u16, u16))| {
                assert_eq!(reg16, (reg8_high << 8) + reg8_low)
            });
    }
}
