#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod actions;
mod config;
mod controller;
#[cfg(feature = "diagnostics")]
mod diagnostics;
mod input;
mod input_windows;
mod installation;
mod model;
mod notices;
mod platform;
mod session;
mod settings;
mod storage;
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    controller::run()
}
#[cfg(feature = "render-check")]
mod render_check {
    use crate::{App, model::View, session::Session};
    use slint::ComponentHandle;
    use std::{cell::RefCell, rc::Rc, time::Duration};

    fn resize(ui: &App, width: f32, height: f32) {
        // Winit's logical-size conversion uses the OS scale, whereas this check
        // can intentionally override Slint's scale without changing Windows.
        let scale = ui.window().scale_factor();
        ui.window().set_size(slint::PhysicalSize::new(
            (width * scale).round() as u32,
            (height * scale).round() as u32,
        ));
    }

    // Developer-only snapshots from the actual Slint renderer, not browser mockups.
    pub fn schedule(ui: &App, model: Rc<RefCell<Session>>, output: std::path::PathBuf) {
        std::fs::create_dir_all(&output).expect("snapshot directory");
        let readme = std::env::args().any(|a| a == "--readme");
        if readme {
            let mut session = model.borrow_mut();
            let m = &mut session.model;
            for key in [
                "Roll", "Pitch", "Yaw", "Throttle", "Gun", "Missile", "Target", "Radar", "Flare",
                "View", "Gear",
            ] {
                let i = crate::actions::ACTIONS
                    .iter()
                    .position(|a| a.key == key)
                    .unwrap();
                m.navigate(View::Controls(Some(i)));
                let ticket = m.start_capture().unwrap();
                m.complete_demo_capture(ticket);
            }
            m.device_names
                .insert("Demo stick".into(), "Flight stick".into());
            m.device_names
                .insert("Demo throttle".into(), "Throttle".into());
        }
        next(ui.as_weak(), model, output, 0, readme);
    }

    pub fn layout_check(ui: &App, output: std::path::PathBuf) {
        std::fs::create_dir_all(&output).expect("layout check directory");
        ui.set_demo(true);
        ui.set_loaded(true);
        ui.set_setup(true);
        ui.set_step(4);
        ui.set_action_label("Gun".into());
        ui.set_instruction("Press a button or hat direction.".into());
        ui.set_axis(false);
        ui.set_art(5);
        resize(ui, 1016., 700.);
        layout_next(ui.as_weak(), output, 0, Vec::new());
    }
    fn layout_next(
        weak: slint::Weak<App>,
        output: std::path::PathBuf,
        index: usize,
        mut measurements: Vec<(f32, f32)>,
    ) {
        slint::Timer::single_shot(Duration::from_millis(200), move || {
            let Some(ui) = weak.upgrade() else { return };
            let snapshot = ui.window().take_snapshot().expect("layout snapshot");
            image::save_buffer(
                output.join(format!("layout-{index}.png")),
                snapshot.as_bytes(),
                snapshot.width(),
                snapshot.height(),
                image::ColorType::Rgba8,
            )
            .expect("layout snapshot output");
            measurements.push((ui.get_setup_panel_x(), ui.get_setup_panel_width()));
            if index == 3 {
                std::fs::write(
                    output.join("measurements.json"),
                    serde_json::to_vec_pretty(&measurements).unwrap(),
                )
                .unwrap();
                for &(x, width) in &measurements {
                    if width < 300.
                        || (x - measurements[0].0).abs() >= 1.
                        || (width - measurements[0].1).abs() >= 1.
                    {
                        eprintln!("Setup columns changed or collapsed: {measurements:?}");
                        std::process::exit(1);
                    }
                }
                let _ = slint::quit_event_loop();
                return;
            }
            match index {
                0 => {
                    ui.set_manual_visible(true);
                }
                1 => {
                    ui.set_bound(true);
                    ui.set_binding_label("Saitek Pro Flight X-56 Rhino Stick · Button19".into());
                }
                _ => {
                    ui.set_manual_visible(false);
                    ui.set_shared_warning("This input is also assigned to another control".into());
                }
            }
            layout_next(weak, output, index + 1, measurements);
        });
    }

    fn next(
        weak: slint::Weak<App>,
        model: Rc<RefCell<Session>>,
        output: std::path::PathBuf,
        index: usize,
        readme: bool,
    ) {
        slint::Timer::single_shot(Duration::from_millis(200), move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            if index > 0 {
                let snapshot = ui.window().take_snapshot().expect("render snapshot");
                image::save_buffer(
                    output.join(format!("native-{}.png", index)),
                    snapshot.as_bytes(),
                    snapshot.width(),
                    snapshot.height(),
                    image::ColorType::Rgba8,
                )
                .expect("write snapshot");
            }
            if index == 5 {
                let _ = slint::quit_event_loop();
                return;
            }
            {
                let mut session = model.borrow_mut();
                let m = &mut session.model;
                match index {
                    0 => {
                        m.navigate(View::Setup(if readme { 1 } else { 0 }));
                        resize(&ui, 1100.0, 700.0);
                    }
                    1 => {
                        m.navigate(View::Setup(2));
                        m.pedals = true;
                        resize(&ui, 960.0, 640.0);
                    }
                    2 => {
                        m.navigate(View::Setup(4));
                        resize(&ui, 1100.0, 700.0);
                    }
                    3 => {
                        m.navigate(View::Controls(Some(if readme { 1 } else { 0 })));
                        let ticket = m.start_capture().unwrap();
                        m.complete_demo_capture(ticket);
                    }
                    _ => {
                        m.navigate(View::Controls(None));
                        ui.invoke_ask_discard();
                    }
                }
                crate::controller::refresh(&ui, &session, true);
                if readme {
                    // Render production chrome while the snapshot-only process
                    // continues using sample data with its input worker disabled.
                    ui.set_demo(false);
                    ui.set_connection_status("2 controllers connected".into());
                }
            }
            next(weak, model, output, index + 1, readme);
        });
    }
}
