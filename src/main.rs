mod cpu;
use cpu::instruction::*;
use cpu::registers::*;
use cpu::*;

fn main() {
    let mut cpu = Cpu::new();

    let program = include_bytes!("8080tests.bin");
    let program = Vec::from(program);

    cpu.load_to_memory(program, 0).unwrap();

    while cpu.is_running() {
        cpu.execute_next().unwrap();
    }

    println!("{cpu:#X?}\n");
}
