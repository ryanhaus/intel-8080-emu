mod cpu;
use cpu::*;
use cpu::instruction::*;
use cpu::registers::*;

fn main() {
    let mut cpu = Cpu::new();

    for i in 0..=255 {
        let opcode = i as u8;
        let decoded =
            Instruction::decode(RegisterValue::from(opcode));

        match decoded {
            Ok(instr) => {
                println!("{i:08b} ({i:02X}): {instr:?}");
            }

            Err(message) => {
                println!("{i:08b} ({i:02X}): ERROR {message}");
            }
        }
    }
}
