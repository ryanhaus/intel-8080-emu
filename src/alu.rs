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
            Add(x, y) => self.add(x, y),
        }
    }

    // performs addition, and updates internal registers & flags, returns result
    fn add(&mut self, x: u8, y: u8) -> u8 {
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
}
