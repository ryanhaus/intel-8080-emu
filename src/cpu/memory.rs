/*
 * memory.rs - Holds all code pertaining to the memory of the system
 */

// Memory struct - holds all memory and has functions for accessing and
// modifying the memory
pub struct Memory {
    data: [u8; 0x10000], // supports 64KB of memory
}

impl Memory {
    // creates a new empty instance of Memory
    pub fn new() -> Self {
        Self {
            data: [0; 0x10000],
        }
    }

    // gets a value at a certain memory address
    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    // mutably gets a value at a certain memory address
    pub fn get_mut(&mut self, addr: u16) -> &mut u8 {
        &mut self.data[addr as usize]
    }

    // writes a value to a certain memory address
    pub fn write(&mut self, addr: u16, value: u8) {
        *self.get_mut(addr) = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_read_write() {
        use rand::prelude::*;

        let mut memory = Memory::new();

        // write a random value to each address, then read it back and make
        // sure it is the same as what was written
        let mut written_values = [0u8; 0x10000];

        for i in 0..0x10000 {
            let value = rand::random::<u8>();

            written_values[i] = value;
            memory.write(i as u16, value);
        }

        for i in 0..0x10000 {
            assert_eq!(
                written_values[i],
                memory.read(i as u16)
            );
        }
    }
}
