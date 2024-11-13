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
    
    // CP/M 'bios' containing just the subroutine for 0x0005
    let cpm_bios_program = vec![
        // start by jumping to 0x100
        0xC3, 0x00, 0x01, // JMP 0x0100
        0x00, // NOP
        0x00, // NOP

        // 0x05 subroutine:
        //  if C == 9:
        //      output string starting at (DE) until $ character is found
        //  if C == 2:
        //      output single character in E
        
        0x79, // MOV A,C
        0xFE, 0x09, // CPI 9
        0xC2, 0x17, 0x00, // JNZ not_nine
        // str_loop:
        0x1A, // LDAX DE
        0x13, // INX DE
        0xFE, 0x24, // CPI '$'
        0xCA, 0x1F, 0x00, // JZ done
        0xD3, 0x00, // OUT 0
        0xC3, 0x0B, 0x00, // JMP str_loop
        // not_nine:
        0xFE, 0x02, // CPI 2
        0xC2, 0x1F, 0x00, // JNZ done
        0x7B, // MOV A,E
        0xD3, 0x00, // OUT 0
        // done:
        0xC9, // RET
    ];
    let program = include_bytes!("TST8080.COM");
    let program = Vec::from(program);

    cpu.load_to_memory(cpm_bios_program, 0).unwrap();
    cpu.load_to_memory(program, 0x100).unwrap();
    cpu.set_port_handler_fn(port_handler);

    while cpu.is_running() {
        cpu.execute_next().unwrap();
    }
}
