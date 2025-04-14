mod cpu;
mod debug_menu;

use std::{env,fs};
use cpu::registers::*;
use cpu::*;
use debug_menu::*;

use std::thread;

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
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} program", args[0]);
        return;
    }
    
    let program_name = &args[1];
    let program = fs::read(program_name).unwrap();

    thread::spawn(|| {
        let mut cpu = Cpu::new();
        cpu.set_pc(0x100).unwrap();

        cpu.load_to_memory(program, 0x100).unwrap();
        cpu.set_port_handler_fn(port_handler);

        while cpu.is_running() {
            cpu.execute_next().unwrap();
        }

        println!();
    });

    init_debug_menu(|ui| {
        ui.window("Intel 8080 Emulator")
            .size([300.0, 110.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text_wrapped("Hello, world!");
            });
    });
}
