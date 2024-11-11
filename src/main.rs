mod cpu;
use cpu::instruction::*;
use cpu::registers::*;
use cpu::*;

fn main() {
    let mut cpu = Cpu::new();

    cpu.load_to_memory(vec![0x01, 0xCD, 0xAB, 0xC6, 0x33, 0xD6, 0x03, 0x76], 0)
        .unwrap();

    for _ in 0..100 {
        cpu.execute_next().unwrap();
    }

    println!("{cpu:#X?}");
}
