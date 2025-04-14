/*
 * cp_m.rs - Contains code related to implementing functions
 * in CP/M.
 */
use crate::cpu::*;
use crate::cpu::registers::*;
use crate::cpu::memory::*;

pub fn add_cpm_bdos(cpu: &mut Cpu) {

    cpu.add_subroutine_handler(0x0005u16, |cpu| {
        // 0x05 subroutine:
        //  if C == 9:
        //      output string starting at (DE) until $ character is found
        //  if C == 2:
        //      output single character in E

        match cpu.reg_array.read_reg(Register::C) {
            // C == 9: output string starting at (DE) until $ character is found
            RegisterValue::Integer8(9) => {
                // store mutable copy of DE, which will be used as the pointer
                let mut str_pointer = cpu.reg_array.read_reg(Register::DE);

                // until a $ character is found, output the string
                loop {
                    // get the current character
                    let current_char = cpu.memory.read(str_pointer, MemorySize::Integer8).unwrap();

                    // break condition: character is '$'
                    if current_char == RegisterValue::from(b'$') {
                        break;
                    }

                    // write to port, increase pointer
                    cpu.write_to_port(RegisterValue::from(0u8), current_char).unwrap();
                    str_pointer = str_pointer.try_add(RegisterValue::from(1u16)).unwrap();;
                }
            }

            // C == 2: output single character in E
            RegisterValue::Integer8(2) => {
                let e_val = cpu.reg_array.read_reg(Register::E);
                cpu.write_to_port(RegisterValue::from(0u8), e_val).unwrap();
            }

            // otherwise, do nothing
            _ => {}
        }
    });
}
