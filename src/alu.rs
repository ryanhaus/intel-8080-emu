/*
 * alu.rs - contains code relating to the arithmetic & logic unit (ALU)
 * see the datasheet: https://deramp.com/downloads/intel/8080%20Data%20Sheet.pdf
 */

// ALUFlags struct - holds the 5 ALU flags
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ALUFlags {
    zero: bool,
    sign: bool,
    parity: bool, // even parity
    carry: bool,
    aux_carry: bool, // aka half carry
}

impl ALUFlags {
    // creates a new instance of ALUFlags with all values defaulting to false
    pub fn new() -> Self {
        Self {
            zero: false,
            sign: false,
            parity: false,
            carry: false,
            aux_carry: false,
        }
    }

    // creates a new instance of ALUFlags with given flag values
    pub fn from_bools(zero: bool, sign: bool, parity: bool, carry: bool, aux_carry: bool) -> Self {
        Self {
            zero,
            sign,
            parity,
            carry,
            aux_carry,
        }
    }
}

// ALUOperation - what operation the ALU should perform, as well as the data
// to be used in the operation
pub enum ALUOperation {
    Add(u8, u8),
    AddCarry(u8, u8),
    Sub(u8, u8),
    SubBorrow(u8, u8),
}

// ALU struct - holds the registers inside of the alu, has functions that
// perform ALU operations
pub struct ALU {
    accumulator: u8,           // 8-bit accumulator register
    temporary_accumulator: u8, // 8-bit temporary accumulator register
    flags: ALUFlags,           // 5-bit flags register
    temporary_register: u8,    // 8-bit temporary register
}

impl ALU {
    // creates a new empty instance of ALU
    pub fn new() -> Self {
        Self {
            accumulator: 0,
            temporary_accumulator: 0,
            flags: ALUFlags::new(),
            temporary_register: 0,
        }
    }

    // returns the flags of the alu
    pub fn flags(&self) -> ALUFlags {
        self.flags
    }

    // evaluates a given ALUOperation, updates flags & internal registers,
    // and returns the result
    pub fn evaluate(&mut self, operation: ALUOperation) -> u8 {
        use ALUOperation::*;

        match operation {
            Add(x, y) => self.add(x, y, false),
            AddCarry(x, y) => self.add(x, y, true),
            Sub(x, y) => self.sub(x, y, false),
            SubBorrow(x, y) => self.sub(x, y, true),
        }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alu_add() {
        let mut alu = ALU::new();

        // test some hand-picked values for adding
        // 0 + 0
        let result = alu.evaluate(ALUOperation::Add(0, 0));
        assert_eq!(result, 0);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(true, false, true, false, false)
        );

        // 13 + 7
        let result = alu.evaluate(ALUOperation::Add(13, 7));
        assert_eq!(result, 20);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, false, true, false, true)
        );

        // 255 + 2 (carry occurs)
        let result = alu.evaluate(ALUOperation::Add(255, 2));
        assert_eq!(result, 1);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, false, false, true, true)
        );

        // 127 + 1
        let result = alu.evaluate(ALUOperation::Add(127, 1));
        assert_eq!(result, 128);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, true, false, false, true)
        );
    }

    #[test]
    fn alu_add_with_carry() {
        let mut alu = ALU::new();

        // test some hand-picked values for adding with carry
        // 240 + 16
        let result = alu.evaluate(ALUOperation::AddCarry(240, 16));
        assert_eq!(result, 0);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(true, false, true, true, false)
        );

        // 1 + 1 (should also add the carry flag to make 3)
        let result = alu.evaluate(ALUOperation::AddCarry(1, 1));
        assert_eq!(result, 3);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, false, true, false, false)
        );
    }

    #[test]
    fn alu_sub() {
        let mut alu = ALU::new();

        // test some hand-picked values for subtraction
        // 7 - 3
        let result = alu.evaluate(ALUOperation::Sub(7, 3));
        assert_eq!(result, 4);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, false, false, false, false)
        );

        // 12 - 24
        let result = alu.evaluate(ALUOperation::Sub(12, 24));
        assert_eq!(result, 12u8.wrapping_neg());
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, true, false, true, false)
        );

        // 1 - 2
        let result = alu.evaluate(ALUOperation::Sub(1, 2));
        assert_eq!(result, 1u8.wrapping_neg());
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, true, true, true, true)
        );
    }

    #[test]
    fn alu_sub_with_borrow() {
        let mut alu = ALU::new();

        // test some hand-picked values for subtraction with carry
        // 1 - 2
        let result = alu.evaluate(ALUOperation::SubBorrow(1, 2));
        assert_eq!(result, 1u8.wrapping_neg());
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, true, true, true, true)
        );

        // 100 - 49 (should also include borrow to make 50)
        let result = alu.evaluate(ALUOperation::SubBorrow(100, 49));
        assert_eq!(result, 50);
        assert_eq!(
            alu.flags(),
            ALUFlags::from_bools(false, false, false, false, false)
        );
    }
}
