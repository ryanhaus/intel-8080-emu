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
    }
}

fn main() {
    let mut cpu = Cpu::new();

    //let program = include_bytes!("8080tests.bin");
    //let program = Vec::from(program);
    let program = vec![
        0x01, 0x06, 0x00, // LXI BC, 6
        0xC3, 0x14, 0x00, // JMP loop
        // data:
        0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21, 0x0A,
        // loop:
        0x0A, // LDAX BC
        0x03, // INX BC
        0xD3, 0x00, // OUT 0
        0x79, // MOV A, C
        0xFE, 0x14, // CPI 20
        0xC2, 0x14, 0x00, // JNZ loop
        0x76, // HLT
    ];

    cpu.load_to_memory(program, 0).unwrap();
    cpu.set_port_handler_fn(port_handler);

    while cpu.is_running() {
        cpu.execute_next().unwrap();
    }

    // println!("{cpu:#X?}\n");
}
