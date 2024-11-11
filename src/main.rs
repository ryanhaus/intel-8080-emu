mod cpu;
use cpu::instruction::*;
use cpu::registers::*;
use cpu::*;

macro_rules! execute {
    ($cpu:expr, $instr:expr) => {
        ($cpu).execute(
            Instruction::decode(
                RegisterValue::from($instr)
            ).unwrap()
        ).unwrap()
    }
}

fn main() {
    let mut cpu = Cpu::new();

    execute!(cpu, 0x3Cu8);

    println!("{cpu:#X?}");
}
