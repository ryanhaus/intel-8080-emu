/*
 * memory.rs - Holds all code pertaining to the memory of the system
 */
use super::registers::*;

// MemorySize enum - represents a size of memory in bits
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MemorySize {
    Integer8,
    Integer16,
}

impl MemorySize {
    // returns the amount of bytes that would be occupied by a value of the size
    // indicated by self
    pub fn n_bytes(&self) -> usize {
        use MemorySize::*;
        match self {
            Integer8 => 1,
            Integer16 => 2,
        }
    }
}

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
    pub fn read(&self, addr: RegisterValue, size: MemorySize) -> Result<RegisterValue, String> {
        // convert addr to usize for array access
        let addr = u16::from(addr) as usize;

        // read the value
        use MemorySize::*;
        let value = match size {
            // read a single 8-bit integer from memory
            Integer8 => RegisterValue::from(self.data[addr]),

            // read little-endian 16-bit integer from memory
            Integer16 => {
                // make sure this won't read outside of memory
                if addr == 0xFFFF {
                    return Err(String::from("Attempt to read outside of memory"));
                }

                // read little-endian 16-bit integer from memory
                let lower = self.data[addr] as u16;
                let higher = self.data[addr + 1] as u16;
                let value = (higher << 8) + lower;
                RegisterValue::from(value)
            }
        };

        // return the value
        Ok(value)
    }

    // writes a RegisterValue to memory at the given address
    pub fn write(&mut self, addr: RegisterValue, value: RegisterValue) -> Result<(), String> {
        // convert addr to usize for array access
        let addr = u16::from(addr) as usize;

        // write the value
        use RegisterValue::*;
        match value {
            // 16-bit write
            Integer16(_) | Integer8Pair(_, _) => {
                // make sure this won't write outside of memory
                if addr == 0xFFFF {
                    return Err(String::from("Attempt to write outside of memory"));
                }

                // convert value to u16
                let value = u16::from(value);

                // write little-endian 16-bit integer to memory
                let lower = (value & 0xFF) as u8;
                let higher = (value >> 8) as u8;

                self.data[addr] = lower;
                self.data[addr + 1] = higher;
            }

            // 8-bit write
            Integer8(_) => {
                // convert value to u8
                let value = u8::try_from(value)?;

                // write single 8-bit integer to memory
                self.data[addr] = value;
            }
        }

        Ok(())
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
                memory
                    .read(RegisterValue::from(i as u16), MemorySize::Integer8)
                    .unwrap()
            );
        }
    }

    #[test]
    fn memory_read_write_16bit() {
        let mut memory = Memory::new();

        // write 0xABCD to address 0x1000, ensure that it will be read back correctly
        // both when reading a single 16-bit value and two 8-bit values
        memory
            .write(
                RegisterValue::from(0x1000u16),
                RegisterValue::from(0xABCDu16),
            )
            .unwrap();

        assert_eq!(
            memory
                .read(RegisterValue::from(0x1000u16), MemorySize::Integer16)
                .unwrap(),
            RegisterValue::from(0xABCDu16)
        );

        assert_eq!(
            memory
                .read(RegisterValue::from(0x1000u16), MemorySize::Integer8)
                .unwrap(),
            RegisterValue::from(0xCDu8)
        );

        assert_eq!(
            memory
                .read(RegisterValue::from(0x1001u16), MemorySize::Integer8)
                .unwrap(),
            RegisterValue::from(0xABu8)
        );
    }

    #[test]
    #[should_panic]
    fn memory_attempt_write_outside_bounds() {
        let mut memory = Memory::new();

        // try to write a 16-bit value to address 0xFFFF, which would write
        // outside of memory and should return an Err, which upon .unwrap()
        // will panic
        memory
            .write(
                RegisterValue::from(0xFFFFu16),
                RegisterValue::from(0xABCDu16),
            )
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn memory_attempt_read_outside_bounds() {
        let memory = Memory::new();

        // try to read a 16-bit value from address 0xFFFF, which would read
        // outside of memory and should return an Err, which upon .unwrap()
        // will panic
        memory
            .read(RegisterValue::from(0xFFFFu16), MemorySize::Integer16)
            .unwrap();
    }
}
