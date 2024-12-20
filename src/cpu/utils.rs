/*
 * utils.rs - contains helper functions for other files
 */

// helper function to return an array of all bits in a u8, with [0] being the MSB
pub fn get_bits(x: u8) -> [u8; 8] {
    let mut bits = [0u8; 8];

    for i in 0..8 {
        bits[7 - i] = (x >> i) & 1;
    }

    bits
}

// helper function to turn an array of 8 bits into a u8, with [0] being the MSB
pub fn from_bits(bits: [u8; 8]) -> u8 {
    let mut x = 0;

    for (i, bit) in bits.iter().enumerate() {
        let multiplier = 0x80 >> i;

        x += bit * multiplier;
    }

    x
}

// Helper function that combines two 8-bit values together to make a single
// 16-bit value, primarily used for making register pairs out of two
// registers. The first parameter will be the 'higher' register, and the
// second parameter is the 'lower' register, i.e., (B, C) -> BC.
pub fn combine_values(higher: u8, lower: u8) -> u16 {
    let (higher, lower) = (higher as u16, lower as u16);

    (higher << 8) + lower
}

// Helper function that is essentially the inverse of the above combine_values,
// takes in a 16-bit value and returns a tuple with two 8-bit values. The first
// value in the tuple is the 'higher' value, and the second value is the 'lower'
// value.
pub fn separate_values(value: u16) -> (u8, u8) {
    let higher = ((value >> 8) & 0xFF) as u8;
    let lower = (value & 0xFF) as u8;

    (higher, lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utils_get_bits() {
        assert_eq!(get_bits(0b11001100), [1, 1, 0, 0, 1, 1, 0, 0]);

        assert_eq!(get_bits(0b01011010), [0, 1, 0, 1, 1, 0, 1, 0]);
    }

    #[test]
    fn utils_from_bits() {
        assert_eq!(from_bits([1, 1, 0, 0, 1, 1, 0, 0]), 0b11001100u8);

        assert_eq!(from_bits([0, 1, 0, 1, 1, 0, 1, 0]), 0b01011010u8);
    }

    #[test]
    fn utils_combine_values() {
        assert_eq!(combine_values(0xAB, 0xCD), 0xABCDu16);
    }

    #[test]
    fn utils_separate_values() {
        assert_eq!(separate_values(0xABCD), (0xAB, 0xCD));
    }
}
