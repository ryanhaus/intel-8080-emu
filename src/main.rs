mod cpu;
use cpu::instruction::*;
use cpu::registers::*;
use cpu::*;

fn port_handler(port: RegisterValue, value: RegisterValue) {
    //println!("Port write occured: {value:X?} written to port {port:X?}");
    
    // terminal example: say terminal out is port 0
    if port == RegisterValue::from(0u8) {
        let value = u8::try_from(value).unwrap();
        let character = value as char;

        print!("{character}");
        //print!("Terminal output: {character} ({value:02X})\n");
    }
}

fn main() {
    let mut cpu = Cpu::new();
    
    // CP/M 'bios' containing just a jump to start the program
    let cpm_bios_program = vec![
        // start by jumping to 0x100
        0xC3, 0x00, 0x01, // JMP 0x0100
    ];
    let program = include_bytes!("8080PRE.COM");
    let program = Vec::from(program);

    cpu.load_to_memory(cpm_bios_program, 0).unwrap();
    cpu.load_to_memory(program, 0x100).unwrap();
    cpu.set_port_handler_fn(port_handler);

    while cpu.is_running() {
        cpu.execute_next().unwrap();
    }
}
