/*
 * registers_view.rs - Debug menu window that displays
 * the current register values.
 */

use imgui::*;
use crate::cpu::*;
use registers::*;
use strum::IntoEnumIterator;

pub fn add_registers_view(ui: &Ui, reg_array: &RegisterArray) {
    ui.window("Registers View")
        .size([300.0, 110.0], Condition::FirstUseEver)
        .build(|| {
            let flags = TableFlags::ROW_BG
                | TableFlags::RESIZABLE
                | TableFlags::BORDERS_H
                | TableFlags::BORDERS_V;

            if let Some(_t) = ui.begin_table_with_sizing(
                "Registers View",
                2,
                flags,
                [300.0, 100.0],
                0.0,
            ) {
                ui.table_setup_column("Register");
                ui.table_setup_column("Value");

                ui.table_setup_scroll_freeze(2, 1);

                ui.table_headers_row();

                for register in Register::iter() {
                    ui.table_next_row();

                    ui.table_set_column_index(0);
                    ui.text(format!("{:?}", register));

                    ui.table_set_column_index(1);
                    let reg_value = reg_array.read_reg(register);

                    match reg_value {
                        RegisterValue::Integer8(reg_value) => {
                            ui.text(format!("{:02X?}", reg_value));
                        }

                        RegisterValue::Integer16(reg_value) => {
                            ui.text(format!("{:04X?}", reg_value));
                        }

                        RegisterValue::Integer8Pair(reg_value_h, reg_value_l) => {
                            ui.text(format!("{:02X?}{:02X?}", reg_value_h, reg_value_l));
                        }
                    }
                }
            }
        });
}
