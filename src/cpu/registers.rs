/*
 * register_array.rs - contains code relating to the register array
 * see Intel 8080 datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

use std::{
    convert::{From, TryFrom},
    error::Error,
};

// RegisterArray struct - contains all register values
pub struct RegisterArray {
    program_counter: u16, // 16-bit program counter
    stack_pointer: u16,   // 16-bit stack pointer

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
        }
    }

    // Writes a given value to the given register
    pub fn write_reg(&mut self, register: Register, value: RegisterValue) -> Result<(), String> {
        use Register::*;

        match register {
            B => self.reg_b = value.try_into()?,
            C => self.reg_c = value.try_into()?,
            D => self.reg_d = value.try_into()?,
            E => self.reg_e = value.try_into()?,
            H => self.reg_h = value.try_into()?,
            L => self.reg_l = value.try_into()?,
            W => self.reg_w = value.try_into()?,
            Z => self.reg_z = value.try_into()?,
            BC => (self.reg_b, self.reg_c) = separate_values(value.into()),
            DE => (self.reg_d, self.reg_e) = separate_values(value.into()),
            HL => (self.reg_h, self.reg_l) = separate_values(value.into()),
            WZ => (self.reg_w, self.reg_z) = separate_values(value.into()),

            // unsupported register: throw an error
            _ => {
                return Err(format!(
                    "Cannot write to {}",
                    register.get_human_readable_name()
                ))
            }
        };

        // if this point is reached, write was successful
        Ok(())
    }
}

// Register enum - contains all possible registers that can be referenced
#[derive(Clone, Copy)]
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
}

impl Register {
    // Returns a string literal with a human-readable name for each register
    pub fn get_human_readable_name(&self) -> &'static str {
        use Register::*;

        match self {
            PC => "PC",
            SP => "SP",
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
}

// RegisterValue enum - could either be a 8-bit or 16-bit integer value
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RegisterValue {
    Integer8(u8),
    Integer8Pair(u8, u8),
    Integer16(u16),
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
            Integer8Pair(higher, lower) => combine_values(higher, lower),
            Integer16(value) => value,
        }
    }
}

// Helper function that combines two 8-bit values together to make a single
// 16-bit value, primarily used for making register pairs out of two
// registers. The first parameter will be the 'higher' register, and the
// second parameter is the 'lower' register, i.e., (B, C) -> BC.
fn combine_values(higher: u8, lower: u8) -> u16 {
    let (higher, lower) = (higher as u16, lower as u16);

    (higher << 8) + lower
}

// Helper function that is essentially the inverse of the above combine_values,
// takes in a 16-bit value and returns a tuple with two 8-bit values. The first
// value in the tuple is the 'higher' value, and the second value is the 'lower'
// value.
fn separate_values(value: u16) -> (u8, u8) {
    let higher = ((value >> 8) & 0xFF) as u8;
    let lower = (value & 0xFF) as u8;

    (higher, lower)
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
        let registers_16 = [BC, DE, HL, WZ];

        // write the 8 values
        registers_16
            .iter()
            .zip(rand_values.iter())
            .for_each(|(reg, val)| {
                reg_array
                    .write_reg(*reg, RegisterValue::from(*val))
                    .unwrap()
            });

        // read back the 8 values, ensure they are equal to what was written
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