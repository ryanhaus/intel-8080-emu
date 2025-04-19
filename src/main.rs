use clap::Parser;
mod arguments;
mod cpu;
mod cp_m;
mod debug_menu;

use cpu::*;
use debug_menu::*;
use std::sync::{Arc, Mutex};
use std::{fs, thread};

fn main() {
    let args = arguments::Args::parse();

    let program = fs::read(args.program).unwrap();

    let cpu_output_str = Arc::new(Mutex::new(String::new())); // string containing the output of
                                                              // the cpu through port 0
    let cpu_output_str_thr = cpu_output_str.clone(); // clone to be passed to the thread

    let cpu = Cpu::new();
    let cpu = Arc::new(Mutex::new(cpu));
    let cpu_thr = cpu.clone();

    let sim_handler = move || {
        let cpu_arc = Arc::clone(&cpu_thr);
        let mut cpu = cpu_arc.lock().unwrap();

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
    };

    if args.debug {
        thread::spawn(sim_handler);

        init_imgui("Intel 8080 Emulator", |ui| {
            let cpu_arc = Arc::clone(&cpu);
            let mut cpu = cpu_arc.lock().unwrap();

            let cpu_output_str = Arc::clone(&cpu_output_str);
            let out_str = cpu_output_str.lock().unwrap();

            cpu_output::add_cpu_output(ui, &*out_str);
            registers_view::add_registers_view(ui, &cpu.reg_array);
        });
    } else {
        sim_handler();
    }
}
