/*
 * cpu_output.rs - Debug menu window showing the CPU
 * output string (port 0)
 */

use imgui::*;
use crate::cpu::*;

pub fn add_cpu_output(ui: &Ui, out_str: &str) {
    ui.window("Output")
        .size([300.0, 110.0], imgui::Condition::FirstUseEver)
        .build(|| {
            ui.text_wrapped(out_str);
        });
}
