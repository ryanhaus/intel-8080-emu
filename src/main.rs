mod cpu;
use cpu::instruction::*;
use cpu::registers::*;
use cpu::*;

fn main() {
    let mut cpu = Cpu::new();

    let program = vec![
        0x3E, 0x05, 0x3D, 0x4F, 0x78, 0xC6, 0x05, 0x47, 0x79, 0xFE, 0x00, 0xC2, 0x02, 0x00, 0x76,
    ];

    cpu.load_to_memory(program, 0).unwrap();

    for _ in 0..100 {
        cpu.execute_next().unwrap();
        println!("{cpu:#X?}\n");
    }
}
