/*
 * debug_menu.rs - Contains all code related to the debug menu and GUI
 * See the imgui-rs crate: https://github.com/imgui-rs/imgui-rs
 * Also referenced: https://github.com/imgui-rs/imgui-examples/blob/main/examples/support/mod.rs
 */
pub mod cpu_output;
pub mod registers_view;

use glium::Surface;
use imgui::{Context, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::winit::event::{Event, WindowEvent};
use imgui_winit_support::winit::event_loop::EventLoop;
use imgui_winit_support::winit::window::WindowAttributes;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;

// initializes an ImGui window
pub fn init_imgui(title: &str, ui_f: impl Fn(&mut Ui)) {
    let mut imgui = Context::create();

    let event_loop = EventLoop::new().unwrap();

    let window_attributes = WindowAttributes::default().with_title(title);

    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .set_window_builder(window_attributes)
        .build(&event_loop);

    let mut renderer = Renderer::new(&mut imgui, &display).unwrap();

    let mut platform = WinitPlatform::new(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

    let mut last_frame = Instant::now();

    event_loop
        .run(move |event, window_target| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }

            Event::AboutToWait => {
                platform.prepare_frame(imgui.io_mut(), &window).unwrap();
                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let ui = imgui.frame();

                ui_f(ui);

                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                platform.prepare_render(ui, &window);

                let draw_data = imgui.render();

                if draw_data.draw_lists_count() > 0 {
                    renderer.render(&mut target, draw_data).unwrap();
                }

                target.finish().unwrap();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    display.resize((new_size.width, new_size.height));
                }

                platform.handle_event(imgui.io_mut(), &window, &event);
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => window_target.exit(),

            event => {
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
        })
        .unwrap();
}
