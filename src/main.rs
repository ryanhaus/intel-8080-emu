mod cpu;
use cpu::instruction::*;
use cpu::registers::*;
use cpu::*;

fn port_handler(port: RegisterValue, value: RegisterValue) {
    println!("Port write occured: {value:X?} written to port {port:X?}");
}

fn main() {
    let mut cpu = Cpu::new();

    //let program = include_bytes!("8080tests.bin");
    //let program = Vec::from(program);
    let program = vec![0x3E, 0x48, 0xD3, 0x00, 0x3E, 0x69, 0xD3, 0x00, 0x76];

    cpu.load_to_memory(program, 0).unwrap();
    cpu.set_port_handler_fn(port_handler);

    while cpu.is_running() {
        cpu.execute_next().unwrap();
    }

    println!("{cpu:#X?}\n");
}
