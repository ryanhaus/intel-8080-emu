mod cp_m;
mod cpu;
mod debug_menu;

use cpu::registers::*;
use cpu::*;
use debug_menu::*;
use std::sync::{Arc, Mutex};
use std::{env, fs, thread};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} program", args[0]);
        return;
    }

    let program_name = &args[1];
    let program = fs::read(program_name).unwrap();

    let cpu_output_str = Arc::new(Mutex::new(String::new()));
    let cpu_output_str_thr = cpu_output_str.clone();

    thread::spawn(move || {
        let mut cpu = Cpu::new();
        cp_m::add_cpm_bdos(&mut cpu);
        cpu.set_pc(0x100).unwrap();

        cpu.load_to_memory(program, 0x100).unwrap();
        cpu.set_port_handler_fn(move |port, value| {
            let port = u8::try_from(port).unwrap();
            let value = u8::try_from(value).unwrap();

            match port {
                0 => {
                    let character = value as char;

                    let cpu_output_str = Arc::clone(&cpu_output_str_thr);
                    let mut out_str = cpu_output_str.lock().unwrap();
                    (*out_str).push(character);

                    // also print
                    print!("{character}");
                }

                _ => {}
            }
        });

        while cpu.is_running() {
            cpu.execute_next().unwrap();
        }

        println!();
    });

    init_debug_menu(|ui| {
        ui.window("Output")
            .size([300.0, 110.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let cpu_output_str = Arc::clone(&cpu_output_str);
                let out_str = cpu_output_str.lock().unwrap();

                ui.text_wrapped(out_str.clone());
            });
    });
}
