mod cpu;

use cpu::registers::*;
use cpu::*;

fn port_handler(port: RegisterValue, value: RegisterValue) {
    // println!("Port write occured: {value:X?} written to port {port:X?}");

    // terminal example: say terminal out is port 0
    if port == RegisterValue::from(0u8) {
        let value = u8::try_from(value).unwrap();
        let character = value as char;

        print!("{character}");
        // print!("Terminal output: {character} ({value:02X})\n");
    }
}

fn main() {
    let mut cpu = Cpu::new();
    cpu.set_pc(0x100).unwrap();

    let program = include_bytes!("TST8080.COM");
    let program = Vec::from(program);

    cpu.load_to_memory(program, 0x100).unwrap();
    cpu.set_port_handler_fn(port_handler);

    while cpu.is_running() {
        cpu.execute_next().unwrap();
    }

    println!("DONE EXECUTION, TOTAL CYCLES: {}", cpu.get_total_cycles());
}
