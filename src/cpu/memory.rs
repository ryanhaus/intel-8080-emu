/*
 * memory.rs - Holds all code pertaining to the memory of the system
 */
use super::registers::*;

// Memory struct - holds all memory and has functions for accessing and
// modifying the memory
pub struct Memory {
    data: [u8; 0x10000], // supports 64KB of memory
}

impl Memory {
    // creates a new empty instance of Memory
    pub fn new() -> Self {
        Self { data: [0; 0x10000] }
    }

    // reads a RegisterValue from the given address
    pub fn read(&self, addr: RegisterValue, read_16: bool) -> RegisterValue {
        // convert addr to usize for array access
        let addr = u16::from(addr) as usize;

        // read the value
        if read_16 {
            // read little-endian 16-bit integer from memory
            let lower = self.data[addr % 0x10000] as u16;
            let higher = self.data[(addr + 1) % 0x10000] as u16;
            let value = (higher << 8) + lower;
            RegisterValue::from(value)
        } else {
            // read single 8-bit integer from memory
            RegisterValue::from(self.data[addr])
        }
    }

    // writes a RegisterValue to memory at the given address
    pub fn write(&mut self, addr: RegisterValue, value: RegisterValue) {
        // convert addr to usize for array access
        let addr = u16::from(addr) as usize;

        // write the value
        use RegisterValue::*;
        match value {
            // 16-bit write
            Integer16(_) | Integer8Pair(_, _) => {
                let value = u16::from(value);

                // write little-endian 16-bit integer to memory
                let lower = (value & 0xFF) as u8;
                let higher = (value >> 8) as u8;

                self.data[addr % 0x10000] = lower;
                self.data[(addr + 1) % 0x10000] = higher;
            }

            // 8-bit write
            Integer8(_) => {
                let value = u8::try_from(value).unwrap();

                // write single 8-bit integer to memory
                self.data[addr % 0x10000] = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_read_write_8bit_random() {
        use rand::prelude::*;

        let mut memory = Memory::new();

        // write a random value to each address, then read it back and make
        // sure it is the same as what was written
        let mut written_values = [RegisterValue::from(0u8); 0x10000];

        for i in 0..0x10000 {
            let value = rand::random::<u8>();
            let value = RegisterValue::from(value);

            written_values[i] = value;
            memory.write(RegisterValue::from(i as u16), value);
        }

        for i in 0..0x10000 {
            assert_eq!(
                written_values[i],
                memory.read(RegisterValue::from(i as u16), false)
            );
        }
    }

    #[test]
    fn memory_read_write_16bit() {
        let mut memory = Memory::new();

        // write 0xABCD to address 0x1000, ensure that it will be read back correctly
        // both when reading a single 16-bit value and two 8-bit values
        memory.write(
            RegisterValue::from(0x1000u16),
            RegisterValue::from(0xABCDu16),
        );

        assert_eq!(
            memory.read(RegisterValue::from(0x1000u16), true),
            RegisterValue::from(0xABCDu16)
        );

        assert_eq!(
            memory.read(RegisterValue::from(0x1000u16), false),
            RegisterValue::from(0xCDu8)
        );

        assert_eq!(
            memory.read(RegisterValue::from(0x1001u16), false),
            RegisterValue::from(0xABu8)
        );
    }
}
