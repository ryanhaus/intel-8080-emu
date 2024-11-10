/*
 * alu.rs - contains code relating to the arithmetic & logic unit (ALU)
 * see the datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */
use super::instruction::InstructionCondition;
use super::registers::RegisterValue;

// AluFlags struct - holds the 5 ALU flags
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AluFlags {
    pub zero: bool,
    pub sign: bool,
    pub parity: bool, // even parity
    pub carry: bool,
    pub aux_carry: bool, // aka half carry
}

impl AluFlags {
    // creates a new instance of AluFlags with all values defaulting to false
    pub fn new() -> Self {
        Self {
            zero: false,
            sign: false,
            parity: false,
            carry: false,
            aux_carry: false,
        }
    }

    // creates a new instance of AluFlags with given flag values
    pub fn from_bools(zero: bool, sign: bool, parity: bool, carry: bool, aux_carry: bool) -> Self {
        Self {
            zero,
            sign,
            parity,
            carry,
            aux_carry,
        }
    }

    // evaluates an InstructionCondition based on the flags
    pub fn evaluate_condition(&self, condition: InstructionCondition) -> bool {
        use InstructionCondition::*;

        match condition {
            NotZero => !self.zero,
            Zero => self.zero,
            NoCarry => !self.carry,
            Carry => self.carry,
            ParityOdd => !self.parity,
            ParityEven => self.parity,
            Plus => !self.sign,
            Minus => self.sign,
        }
    }
}

// AluOperation - what operation the Alu should perform, as well as the data
// to be used in the operation
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AluOperation {
    Add(RegisterValue, RegisterValue),
    AddCarry(RegisterValue, RegisterValue),
    Sub(RegisterValue, RegisterValue),
    SubBorrow(RegisterValue, RegisterValue),
    Increment(RegisterValue),
    Decrement(RegisterValue),
    DecimalAdjust(RegisterValue),
    BitwiseAnd(RegisterValue, RegisterValue),
    BitwiseXor(RegisterValue, RegisterValue),
    BitwiseOr(RegisterValue, RegisterValue),
    Comparison(RegisterValue, RegisterValue),
    RotateLeft(RegisterValue),
    RotateRight(RegisterValue),
    RotateLeftThroughCarry(RegisterValue),
    RotateRightThroughCarry(RegisterValue),
    Complement(RegisterValue),
    SetCarry,
    ComplementCarry,
}

// Alu struct - holds the registers inside of the ALU, has functions that
// perform ALU operations
pub struct Alu {
    accumulator: RegisterValue, // 8-bit accumulator register
    flags: AluFlags,            // 5-bit flags register
}

impl Alu {
    // creates a new empty instance of Alu
    pub fn new() -> Self {
        Self {
            accumulator: RegisterValue::from(0u8),
            flags: AluFlags::new(),
        }
    }

    // returns the value of the accumulator register
    pub fn accumulator(&self) -> RegisterValue {
        self.accumulator
    }

    // writes a value to the accumulator register
    pub fn write_accumulator(&mut self, value: RegisterValue) -> Result<(), String> {
        if value.n_bytes() != 1 {
            return Err(format!(
                "Attempted to write a value of size {} to the accuulator register",
                value.n_bytes()
            ));
        }

        self.accumulator = value;

        Ok(())
    }

    // returns the flags of the alu
    pub fn flags(&self) -> AluFlags {
        self.flags
    }

    // evaluates a given AluOperation, updates flags & internal registers,
    // and returns the result
    pub fn evaluate(&mut self, operation: AluOperation) -> Result<Option<RegisterValue>, String> {
        use AluOperation::*;

        // convert the arguments in the AluOperation from RegisterValues
        // to u8, u16, depending on what the operation is
        let mut x = None;
        let mut y = None;
        let mut x16 = None;

        match operation {
            Add(a, b)
            | AddCarry(a, b)
            | Sub(a, b)
            | SubBorrow(a, b)
            | BitwiseAnd(a, b)
            | BitwiseXor(a, b)
            | BitwiseOr(a, b)
            | Comparison(a, b) => {
                x = Some(a.try_into()?);
                y = Some(b.try_into()?);
            }
            Increment(a)
            | Decrement(a)
            | DecimalAdjust(a)
            | RotateLeft(a)
            | RotateRight(a)
            | RotateLeftThroughCarry(a)
            | RotateRightThroughCarry(a)
            | Complement(a) => {
                use RegisterValue::*;
                match a {
                    Integer8(_) => {
                        x = Some(a.try_into()?);
                    }

                    Integer8Pair(_, _) | Integer16(_) => {
                        x16 = Some(a.into());
                    }
                }
            }

            _ => {}
        }

        // perform the operation
        let result = match operation {
            Add(_, _) => Some(self.add(x.unwrap(), y.unwrap(), false).into()),
            AddCarry(_, _) => Some(self.add(x.unwrap(), y.unwrap(), true).into()),
            Sub(_, _) => Some(self.sub(x.unwrap(), y.unwrap(), false).into()),
            SubBorrow(_, _) => Some(self.sub(x.unwrap(), y.unwrap(), true).into()),
            Increment(_) => Some({
                if let Some(x) = x {
                    self.inc_dec(x, true).into()
                } else if let Some(x16) = x16 {
                    self.inc_dec16(x16, true).into()
                } else {
                    return Err(String::from("Could not increment: {x:?}"));
                }
            }),
            Decrement(_) => Some({
                if let Some(x) = x {
                    self.inc_dec(x, false).into()
                } else if let Some(x16) = x16 {
                    self.inc_dec16(x16, false).into()
                } else {
                    return Err(String::from("Could not decrement: {x:?}"));
                }
            }),
            DecimalAdjust(_) => Some(self.decimal_adjust(x.unwrap()).into()),
            BitwiseAnd(_, _) => Some(self.bitwise_and(x.unwrap(), y.unwrap()).into()),
            BitwiseXor(_, _) => Some(self.bitwise_xor(x.unwrap(), y.unwrap()).into()),
            BitwiseOr(_, _) => Some(self.bitwise_or(x.unwrap(), y.unwrap()).into()),
            Comparison(_, _) => {
                self.sub(x.unwrap(), y.unwrap(), false);
                None // comparison doesn't modify any registers
            }
            RotateLeft(_) => Some(self.rotate(x.unwrap(), false, false).into()),
            RotateRight(_) => Some(self.rotate(x.unwrap(), true, false).into()),
            RotateLeftThroughCarry(_) => Some(self.rotate(x.unwrap(), false, true).into()),
            RotateRightThroughCarry(_) => Some(self.rotate(x.unwrap(), true, true).into()),
            Complement(_) => Some(self.complement(x.unwrap()).into()),
            SetCarry => {
                self.flags.carry = true;
                None
            }
            ComplementCarry => {
                self.flags.carry = !self.flags.carry;
                None
            }
        };

        // set the accumulator register (if applicable), return result
        if let Some(result) = result {
            self.accumulator = result;
        }

        Ok(result)
    }

    // performs addition, and updates internal registers & flags, returns result
    fn add(&mut self, x: u8, mut y: u8, use_carry: bool) -> u8 {
        // if the carry is used, apply that to y
        if use_carry {
            let carry = self.flags.carry as u8;
            y = y.wrapping_add(carry);
        }

        // find result, set flags
        let result = x.wrapping_add(y);

        self.flags.zero = (result == 0);
        self.flags.sign = (result & 0x80 != 0);
        self.flags.parity = (result.count_ones() % 2 == 0);
        self.flags.carry = (x.checked_add(y) == None);

        // auxiliary carry has to be found manually
        let x_lower = x & 0xF;
        let y_lower = y & 0xF;
        let lower_sum = x_lower + y_lower;
        self.flags.aux_carry = (lower_sum & 0x10 > 0);

        result
    }

    // performs subtraction, and updates internal registers & flags, returns result
    fn sub(&mut self, x: u8, mut y: u8, use_carry: bool) -> u8 {
        // if the carry is used, apply that to y
        if use_carry {
            let carry = self.flags.carry as u8;
            y = y.wrapping_add(carry); // the carry is added here since y is subtracted
        }

        // find result, set flags
        let result = x.wrapping_sub(y);

        self.flags.zero = (result == 0);
        self.flags.sign = (result & 0x80 != 0);
        self.flags.parity = (result.count_ones() % 2 == 0);
        self.flags.carry = (x.checked_sub(y) == None);

        // auxiliary carry has to be found manually
        let x_lower = x & 0xF;
        let y_lower = y & 0xF;
        self.flags.aux_carry = (x_lower.checked_sub(y_lower) == None);

        result
    }

    // performs 8-bit increment/decrement operations, and updates internal registers
    // and flags, returns result
    fn inc_dec(&mut self, x: u8, increment: bool) -> u8 {
        // the increment/decrement operations do NOT modify the carry flag,
        // so store a copy of the present value to be written back to it after the operation
        let carry_flag_copy = self.flags.carry;
        let result = if increment {
            self.add(x, 1, false)
        } else {
            self.sub(x, 1, false)
        };

        // write back the original carry flag
        self.flags.carry = carry_flag_copy;

        result
    }

    // performs 16-bit increment/decrement operations, does not update any flags
    fn inc_dec16(&mut self, x: u16, increment: bool) -> u16 {
        // no flags are updated by 16-bit operations, just return the result
        if increment {
            x.wrapping_add(1)
        } else {
            x.wrapping_sub(1)
        }
    }

    // performs a 'decimal adjustment', i.e., an eight-bit number is "adjusted
    // to form two four-bit Binary-Coded-Decimal digits"
    fn decimal_adjust(&mut self, mut x: u8) -> u8 {
        // per the datasheet, the adjustment process is as follows: (see pg. 4-8)
        // 1. If the value of the least significant 4 bits of the [register] is
        //    greater than 9 or if the [auxiliary carry] flag is set, 6 is
        //    added to the [register].
        // 2. If the value of the most significant 4 bits of the [register] is
        //    now greater than 9, or if the [carry] flag is set, 6 is added to
        //    the most significant 4 bits of the [register].

        // step 1
        let lower_bits = x & 0xF;
        if lower_bits > 9 || self.flags.aux_carry {
            x = self.add(x, 6, false);
        }

        // step 2
        let higher_bits = (x & 0xF0) >> 4;
        if higher_bits > 9 || self.flags.carry {
            x = self.add(x, 0x60, false);
        }

        x
    }

    // performs a logical bitwise AND between two numbers
    fn bitwise_and(&mut self, x: u8, y: u8) -> u8 {
        let result = x & y;

        self.flags.zero = (result == 0);
        self.flags.sign = (result & 0x80 != 0);
        self.flags.parity = (result.count_ones() % 2 == 0);
        self.flags.carry = false;
        self.flags.aux_carry = false;

        result
    }

    // performs a logical bitwise XOR between two numbers
    fn bitwise_xor(&mut self, x: u8, y: u8) -> u8 {
        let result = x ^ y;

        self.flags.zero = (result == 0);
        self.flags.sign = (result & 0x80 != 0);
        self.flags.parity = (result.count_ones() % 2 == 0);
        self.flags.carry = false;
        self.flags.aux_carry = false;

        result
    }

    // performs a logical bitwise OR between two numbers
    fn bitwise_or(&mut self, x: u8, y: u8) -> u8 {
        let result = x | y;

        self.flags.zero = (result == 0);
        self.flags.sign = (result & 0x80 != 0);
        self.flags.parity = (result.count_ones() % 2 == 0);
        self.flags.carry = false;
        self.flags.aux_carry = false;

        result
    }

    // performs a bit rotation (different from a shift) in either direction
    fn rotate(&mut self, x: u8, right: bool, through_carry: bool) -> u8 {
        // store copy of current carry flag for rotation through carry
        let carry_copy = self.flags.carry;

        let mut result = if right {
            // carry flag becomes old LSB
            self.flags.carry = (x & 0x01) != 0;

            // rotate
            x.rotate_right(1)
        } else {
            // carry flag becomes old MSB
            self.flags.carry = (x & 0x80) != 0;

            // rotate
            x.rotate_left(1)
        };

        // handle rotate through carry
        if through_carry {
            if right {
                // if rotating right, new MSB becomes old carry
                result &= 0b0111_1111; // clear MSB
                result |= (carry_copy as u8) << 7; // set MSB to old carry
            } else {
                // if rotating left, new LSB becomes old carry
                result &= 0b1111_1110; // clear LSB
                result |= carry_copy as u8; // set LSB to old carry
            }
        }

        result
    }

    fn complement(&mut self, x: u8) -> u8 {
        !x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alu_accumulator_is_set() {
        let mut alu = Alu::new();

        // add 1+1, ensure accumulator AND return value value are 2
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(1u8),
                RegisterValue::from(1u8),
            ))
            .unwrap();

        assert_eq!(result.unwrap(), RegisterValue::from(2u8));
        assert_eq!(alu.accumulator(), RegisterValue::from(2u8));
    }

    #[test]
    fn alu_add() {
        let mut alu = Alu::new();

        // test some hand-picked values for adding
        // 0 + 0
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(0u8),
                RegisterValue::from(0u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, false, false)
        );

        // 13 + 7
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(13u8),
                RegisterValue::from(7u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(20u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, true, false, true)
        );

        // 255 + 2 (carry occurs)
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(255u8),
                RegisterValue::from(2u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(1u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, true, true)
        );

        // 127 + 1
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(127u8),
                RegisterValue::from(1u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(128u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, false, false, true)
        );
    }

    #[test]
    fn alu_add_with_carry() {
        let mut alu = Alu::new();

        // test some hand-picked values for adding with carry
        // 240 + 16
        let result = alu
            .evaluate(AluOperation::AddCarry(
                RegisterValue::from(240u8),
                RegisterValue::from(16u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, true, false)
        );

        // 1 + 1 (should also add the carry flag to make 3)
        let result = alu
            .evaluate(AluOperation::AddCarry(
                RegisterValue::from(1u8),
                RegisterValue::from(1u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(3u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, true, false, false)
        );
    }

    #[test]
    fn alu_sub() {
        let mut alu = Alu::new();

        // test some hand-picked values for subtraction
        // 7 - 3
        let result = alu
            .evaluate(AluOperation::Sub(
                RegisterValue::from(7u8),
                RegisterValue::from(3u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(4u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // 12 - 24
        let result = alu
            .evaluate(AluOperation::Sub(
                RegisterValue::from(12u8),
                RegisterValue::from(24u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(12u8.wrapping_neg()));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, false, true, false)
        );

        // 1 - 2
        let result = alu
            .evaluate(AluOperation::Sub(
                RegisterValue::from(1u8),
                RegisterValue::from(2u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(1u8.wrapping_neg()));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, true, true, true)
        );
    }

    #[test]
    fn alu_sub_with_borrow() {
        let mut alu = Alu::new();

        // test some hand-picked values for subtraction with carry
        // 1 - 2
        let result = alu
            .evaluate(AluOperation::SubBorrow(
                RegisterValue::from(1u8),
                RegisterValue::from(2u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(1u8.wrapping_neg()));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, true, true, true)
        );

        // 100 - 49 (should also include borrow to make 50)
        let result = alu
            .evaluate(AluOperation::SubBorrow(
                RegisterValue::from(100u8),
                RegisterValue::from(49u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(50u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );
    }

    #[test]
    fn alu_increment() {
        let mut alu = Alu::new();

        // test some hand-picked values for increment
        // increment 15
        let result = alu
            .evaluate(AluOperation::Increment(RegisterValue::from(15u8)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(16u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, true)
        );

        // increment 255 (overflow, but carry flag should NOT be updated)
        let result = alu
            .evaluate(AluOperation::Increment(RegisterValue::from(255u8)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, false, true)
        );
    }

    #[test]
    fn alu_decrement() {
        let mut alu = Alu::new();

        // test some hand-picked values for decrement
        // decrement 16
        let result = alu
            .evaluate(AluOperation::Decrement(RegisterValue::from(16u8)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(15u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, true, false, true)
        );

        // decrement 0 (overflow)
        let result = alu
            .evaluate(AluOperation::Decrement(RegisterValue::from(0u8)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(255u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, true, false, true)
        );
    }

    #[test]
    fn alu_increment_16bit() {
        let mut alu = Alu::new();

        // test some hand-picked values for 16-bit increment
        // increment 255
        let result = alu
            .evaluate(AluOperation::Increment(RegisterValue::from(255u16)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(256u16));
        assert_eq!(
            alu.flags(),
            AluFlags::new() // all flags should be 0
        );

        // increment 65535 (overflow)
        let result = alu
            .evaluate(AluOperation::Increment(RegisterValue::from(65535u16)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0u16));
        assert_eq!(
            alu.flags(),
            AluFlags::new() // all flags should be 0
        );
    }

    #[test]
    fn alu_decrement_16bit() {
        let mut alu = Alu::new();

        // test some hand-picked values for 16-bit decrement
        // decrement 256
        let result = alu
            .evaluate(AluOperation::Decrement(RegisterValue::from(256u16)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(255u16));
        assert_eq!(
            alu.flags(),
            AluFlags::new() // all flags should be 0
        );

        // decrement 0 (overflow)
        let result = alu
            .evaluate(AluOperation::Decrement(RegisterValue::from(0u16)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(65535u16));
        assert_eq!(
            alu.flags(),
            AluFlags::new() // all flags should be 0
        );
    }

    #[test]
    fn alu_decimal_adjustment() {
        let mut alu = Alu::new();

        // test some hand-picked values for decimal adjustment
        // add 0x5 and 0x3, then decimal adjust
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(0x5u8),
                RegisterValue::from(0x3u8),
            ))
            .unwrap()
            .unwrap();
        let result = alu
            .evaluate(AluOperation::DecimalAdjust(RegisterValue::from(result)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x8u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // add 0x15 and 0x27, then decimal adjust
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(0x15u8),
                RegisterValue::from(0x27u8),
            ))
            .unwrap()
            .unwrap();
        let result = alu
            .evaluate(AluOperation::DecimalAdjust(RegisterValue::from(result)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x42u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, true, false, true)
        );

        // add 0x99 and 0x01, then decimal adjust
        let result = alu
            .evaluate(AluOperation::Add(
                RegisterValue::from(0x99u8),
                RegisterValue::from(0x01u8),
            ))
            .unwrap()
            .unwrap();
        let result = alu
            .evaluate(AluOperation::DecimalAdjust(RegisterValue::from(result)))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x00u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, true, false)
        );
    }

    #[test]
    fn alu_bitwise_and() {
        let mut alu = Alu::new();

        // test some hand-picked values for bitwise AND
        // bitwise AND 0x37 and 0xF0
        let result = alu
            .evaluate(AluOperation::BitwiseAnd(
                RegisterValue::from(0x37u8),
                RegisterValue::from(0xF0u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x30u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, true, false, false)
        );

        // bitwise AND 0xFF and 0x00
        let result = alu
            .evaluate(AluOperation::BitwiseAnd(
                RegisterValue::from(0xFFu8),
                RegisterValue::from(0x00u8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x00u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, false, false)
        );
    }

    #[test]
    fn alu_bitwise_xor() {
        let mut alu = Alu::new();

        // test some hand-picked values for bitwise XOR
        // bitwise XOR 0x55 and 0xFF
        let result = alu
            .evaluate(AluOperation::BitwiseXor(
                RegisterValue::from(0x55u8),
                RegisterValue::from(0xFFu8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0xAAu8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, true, false, false)
        );

        // bitwise XOR 0xAB and 0xAB
        let result = alu
            .evaluate(AluOperation::BitwiseXor(
                RegisterValue::from(0xABu8),
                RegisterValue::from(0xABu8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x00u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, false, false)
        );
    }

    #[test]
    fn alu_bitwise_or() {
        let mut alu = Alu::new();

        // test some hand-picked values for bitwise OR
        // bitwise OR 0x55 and 0xAA
        let result = alu
            .evaluate(AluOperation::BitwiseOr(
                RegisterValue::from(0x55u8),
                RegisterValue::from(0xAAu8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0xFFu8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, true, false, false)
        );

        // bitwise OR 0x3A and 0x4A
        let result = alu
            .evaluate(AluOperation::BitwiseOr(
                RegisterValue::from(0x3Au8),
                RegisterValue::from(0x4Au8),
            ))
            .unwrap();
        assert_eq!(result.unwrap(), RegisterValue::from(0x7Au8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );
    }

    #[test]
    fn alu_comparison() {
        let mut alu = Alu::new();

        // test some hand-picked values for comparison
        // compare 5 and 5
        let result = alu
            .evaluate(AluOperation::Comparison(
                RegisterValue::from(5u8),
                RegisterValue::from(5u8),
            ))
            .unwrap();
        assert_eq!(result, None);
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(true, false, true, false, false)
        );

        // compare 6 and 5
        let result = alu
            .evaluate(AluOperation::Comparison(
                RegisterValue::from(6u8),
                RegisterValue::from(5u8),
            ))
            .unwrap();
        assert_eq!(result, None);
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // compare 4 and 5
        let result = alu
            .evaluate(AluOperation::Comparison(
                RegisterValue::from(4u8),
                RegisterValue::from(5u8),
            ))
            .unwrap();
        assert_eq!(result, None);
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, true, true, true, true)
        );
    }

    #[test]
    fn alu_rotate_left() {
        let mut alu = Alu::new();

        // test some hand-picked values for left rotation
        // ensure that 8 rotations will result in the original number
        // rotate 1 left 8 times
        let mut result = RegisterValue::from(1u8);
        for _ in 0..8 {
            result = alu
                .evaluate(AluOperation::RotateLeft(result))
                .unwrap()
                .unwrap();
        }
        assert_eq!(result, RegisterValue::from(1u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, true, false)
        );

        // rotate 0x55 left once
        let result = alu
            .evaluate(AluOperation::RotateLeft(RegisterValue::from(0x55u8)))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0xAAu8));
        assert_eq!(
            alu.flags(),
            // even though the sign bit is 1 and the parity is even, only
            // the carry flag is modified during a rotation
            AluFlags::from_bools(false, false, false, false, false)
        );
    }

    #[test]
    fn alu_rotate_right() {
        let mut alu = Alu::new();

        // test some hand-picked values for right rotation
        // ensure that 8 rotations will result in the original number
        // rotate 1 right 8 times
        let mut result = RegisterValue::from(1u8);
        for _ in 0..8 {
            result = alu
                .evaluate(AluOperation::RotateRight(result))
                .unwrap()
                .unwrap();
        }
        assert_eq!(result, RegisterValue::from(1u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // rotate 0xAA right once
        let result = alu
            .evaluate(AluOperation::RotateRight(RegisterValue::from(0xAAu8)))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0x55u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );
    }

    #[test]
    fn alu_rotate_left_through_carry() {
        let mut alu = Alu::new();

        // test some hand-picked values for left rotation through carry
        // ensure that 9 rotations will result in the same number
        // rotate 1 left 9 times through carry
        let mut result = RegisterValue::from(1u8);
        for _ in 0..9 {
            result = alu
                .evaluate(AluOperation::RotateLeftThroughCarry(result))
                .unwrap()
                .unwrap();
        }
        assert_eq!(result, RegisterValue::from(1u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // rotate 0xAA left through carry
        let result = alu
            .evaluate(AluOperation::RotateLeftThroughCarry(RegisterValue::from(
                0xAAu8,
            )))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0x54u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, true, false)
        );

        // rotate 0x54 left through carry
        let result = alu
            .evaluate(AluOperation::RotateLeftThroughCarry(RegisterValue::from(
                0x54u8,
            )))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0xA9u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );
    }

    #[test]
    fn alu_rotate_right_through_carry() {
        let mut alu = Alu::new();

        // test some hand-picked values for right rotation through carry
        // ensure that 9 rotations will result in the same number
        // rotate 1 right 9 times through carry
        let mut result = RegisterValue::from(1u8);
        for _ in 0..9 {
            result = alu
                .evaluate(AluOperation::RotateRightThroughCarry(result))
                .unwrap()
                .unwrap();
        }
        assert_eq!(result, RegisterValue::from(1u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // rotate 0x55 right through carry
        let result = alu
            .evaluate(AluOperation::RotateRightThroughCarry(RegisterValue::from(
                0x55u8,
            )))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0x2Au8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, true, false)
        );

        // rotate 0x2A left through carry
        let result = alu
            .evaluate(AluOperation::RotateRightThroughCarry(RegisterValue::from(
                0x2Au8,
            )))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0x95u8));
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );
    }

    #[test]
    fn alu_complement() {
        let mut alu = Alu::new();

        // test some hand-picked values for complement (bitwise NOT)
        // complement 0x55
        let result = alu
            .evaluate(AluOperation::Complement(RegisterValue::from(0x55u8)))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0xAAu8));
        assert_eq!(
            alu.flags(),
            AluFlags::new() // no flags should be updated
        );

        // complement 0xF0
        let result = alu
            .evaluate(AluOperation::Complement(RegisterValue::from(0xF0u8)))
            .unwrap()
            .unwrap();
        assert_eq!(result, RegisterValue::from(0x0Fu8));
        assert_eq!(
            alu.flags(),
            AluFlags::new() // no flags should be updated
        );
    }

    #[test]
    fn alu_set_and_complement_carry() {
        let mut alu = Alu::new();

        // test out setting and complementing the carry flag
        // set the carry
        alu.evaluate(AluOperation::SetCarry).unwrap();
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, true, false)
        );

        // complement the carry
        alu.evaluate(AluOperation::ComplementCarry).unwrap();
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, false, false)
        );

        // complement the carry again
        alu.evaluate(AluOperation::ComplementCarry).unwrap();
        assert_eq!(
            alu.flags(),
            AluFlags::from_bools(false, false, false, true, false)
        );
    }
}
