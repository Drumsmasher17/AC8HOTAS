use crate::{
    App, ControlRow,
    actions::{self, ACTIONS},
    input::{self, Request},
    model::View,
    session::{Pending, Session},
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Duration};

pub fn refresh(ui: &App, s: &Session, demo: bool) {
    let m = &s.model;
    let selected = m.selected();
    ui.set_demo(demo);
    ui.set_loaded(demo || s.installation.is_some());
    ui.set_setup(matches!(m.view, View::Setup(_)));
    ui.set_step(if let View::Setup(n) = m.view {
        n as i32
    } else {
        0
    });
    ui.set_selected(selected.map_or(-1, |i| i as i32));
    ui.set_assigned(m.assigned_count() as i32);
    ui.set_dirty(m.dirty());
    ui.set_assigned_only(m.assigned_only);
    if ui.get_query().as_str() != m.query {
        ui.set_query(m.query.clone().into());
    }
    ui.set_message(m.message.clone().into());
    ui.set_details(s.error.clone().into());
    ui.set_conflict(s.conflict);
    ui.set_connection_status(if !s.catalog.catalog_ok {
        "Detecting controllers…".into()
    } else {
        format!("{} controllers connected", s.catalog.devices.len()).into()
    });
    ui.set_manual_visible(s.capture_hint);
    ui.set_manual_enabled(s.catalog.catalog_ok && !s.catalog.devices.is_empty());
    ui.set_shared_warning(s.shared_warning().into());
    ui.set_pedals(m.pedals);
    ui.set_listening(m.capture.is_some());
    if let Some(i) = selected {
        let a = ACTIONS[i];
        let b = m.bindings[i].as_ref();
        ui.set_action_label(a.label.into());
        ui.set_instruction(actions::instruction(i, m.pedals).into());
        ui.set_bound(b.is_some());
        ui.set_binding_label(if b.is_some() {
            m.binding_label(i).into()
        } else {
            "—".into()
        });
        ui.set_axis(a.axis);
        ui.set_yaw(a.key == "Yaw");
        let inverted = b.map_or(a.default_inverted(), |b| b.invert);
        ui.set_inverted(inverted);
        ui.set_zero_to_one(b.map_or(a.key == "Throttle", |b| b.zero_to_one));
        ui.set_preview(if inverted { 1. - m.preview } else { m.preview });
        ui.set_art(match a.key {
            "Pitch" | "CameraPitch" => 1,
            "Yaw" if m.pedals => 4,
            "Yaw" | "CameraYaw" => 2,
            "Throttle" => 3,
            _ if !a.axis => 5,
            _ => 0,
        });
    }
    let mut rows = Vec::new();
    let mut group = "";
    for (i, a) in ACTIONS.iter().enumerate() {
        if !m.visible(i) {
            continue;
        }
        if group != a.group {
            rows.push(ControlRow {
                index: -1,
                label: a.group.into(),
                header: true,
                ..Default::default()
            });
            group = a.group;
        }
        rows.push(ControlRow {
            index: i as i32,
            label: a.label.into(),
            binding: m.binding_label(i).into(),
            header: false,
            bound: m.bindings[i].is_some(),
        });
    }
    ui.set_rows(ModelRc::new(VecModel::from(rows)));
}

fn perform_pending(ui: &App, state: &Rc<RefCell<Session>>, demo: bool) {
    let action = state.borrow().pending;
    match action {
        Pending::Exit => {
            let _ = slint::quit_event_loop();
        }
        Pending::Folder => {
            let mut s = state.borrow_mut();
            if let Some(path) = s.pending_folder.take()
                && let Err(e) = s.load(&path)
            {
                s.failure("Could not open mod", e);
            }
        }
        Pending::Reload => {
            let mut s = state.borrow_mut();
            if let Err(e) = s.reload() {
                s.failure("Could not reload config", e)
            }
        }
    }
    refresh(ui, &state.borrow(), demo);
}
fn pending(ui: &App, state: &Rc<RefCell<Session>>, action: Pending, demo: bool) {
    let dirty = {
        let mut s = state.borrow_mut();
        s.model.cancel_capture();
        s.pending = action;
        s.model.dirty()
    };
    if dirty {
        ui.invoke_ask_discard();
    } else {
        perform_pending(ui, state, demo);
    }
}
fn manual_options(ui: &App, s: &mut Session) {
    ui.set_device_options(ModelRc::new(VecModel::from(
        s.catalog
            .devices
            .iter()
            .map(|d| slint::SharedString::from(d.name.clone()))
            .collect::<Vec<_>>(),
    )));
    ui.set_input_options(ModelRc::new(VecModel::from(
        s.manual_tokens
            .iter()
            .map(|t| slint::SharedString::from(t.clone()))
            .collect::<Vec<_>>(),
    )));
    ui.set_manual_device(s.manual_device as i32);
    ui.set_manual_input(0);
}

pub fn run() -> Result<(), slint::PlatformError> {
    let _apartment = crate::platform::Apartment::ui();
    let args: Vec<_> = std::env::args().skip(1).collect();
    #[cfg(feature = "diagnostics")]
    if let Some(output) = args.iter().find_map(|a| a.strip_prefix("--diagnose=")) {
        crate::diagnostics::run(&args, std::path::Path::new(output));
        return Ok(());
    }
    let demo = cfg!(feature = "render-check")
        && args
            .iter()
            .any(|a| a.starts_with("--render-check=") || a.starts_with("--layout-check="));
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .renderer_name("software".into())
        .select()?;
    let ui = App::new()?;
    let session = Rc::new(RefCell::new(Session::new()));
    if !demo {
        let requested = args
            .iter()
            .find_map(|a| a.strip_prefix("--mod-dir="))
            .map(PathBuf::from);
        session.borrow_mut().startup(requested);
    }
    let worker = (!demo).then(input::Worker::start).map(Rc::new);
    macro_rules! callback {
        ($name:ident,$body:expr) => {{
            let weak = ui.as_weak();
            let state = session.clone();
            ui.$name(move || {
                if let Some(ui) = weak.upgrade() {
                    let mut s = state.borrow_mut();
                    ($body)(&ui, &mut s);
                    refresh(&ui, &s, demo);
                }
            });
        }};
    }
    callback!(on_open_controls, |_: &App, s: &mut Session| s
        .view_controls());
    callback!(on_start_setup, |_: &App, s: &mut Session| s
        .model
        .navigate(View::Setup(0)));
    callback!(on_back, |_: &App, s: &mut Session| s.model.back());
    callback!(on_close_editor, |_: &App, s: &mut Session| s
        .view_controls());
    callback!(on_invert, |_: &App, s: &mut Session| s.model.invert());
    callback!(on_save, |_: &App, s: &mut Session| {
        #[cfg(feature = "render-check")]
        if demo {
            s.model.saved.clone_from(&s.model.bindings);
            s.model.message = "Preview saved. No files written.".into();
            return;
        }
        if let Err(e) = s.save() {
            s.failure("Could not save configuration", e);
        }
    });
    callback!(on_escape, |_: &App, s: &mut Session| {
        if s.model.capture.is_some() {
            s.model.cancel_capture();
        } else if matches!(s.model.view, View::Controls(Some(_))) {
            s.view_controls();
        }
    });
    callback!(on_test_input, |_: &App, s: &mut Session| {
        if demo {
            s.model.preview = if s.model.preview > 0.5 { 0.23 } else { 0.77 };
        }
    });
    macro_rules! state_callback {
        ($name:ident,$body:expr) => {{
            let weak = ui.as_weak();
            let state = session.clone();
            ui.$name(move || {
                if let Some(ui) = weak.upgrade() {
                    ($body)(&ui, &state);
                    refresh(&ui, &state.borrow(), demo);
                }
            });
        }};
    }
    state_callback!(on_choose_folder, |ui: &App, s: &Rc<RefCell<Session>>| {
        let initial = {
            let mut session = s.borrow_mut();
            session.model.cancel_capture();
            session
                .installation
                .as_ref()
                .map(|i| i.root.clone())
                .or_else(|| {
                    std::env::current_exe()
                        .ok()
                        .and_then(|p| p.parent().map(PathBuf::from))
                })
        };
        // The picker runs a nested event loop; release all session borrows first.
        match crate::platform::folder_picker(initial.as_deref()) {
            Ok(path) => {
                let prepared = s.borrow_mut().prepare_folder(path.as_deref());
                match prepared {
                    Ok(true) => pending(ui, s, Pending::Folder, demo),
                    Ok(false) => {}
                    Err(e) => s.borrow_mut().failure("Could not open mod", e),
                }
            }
            Err(e) => s.borrow_mut().failure("Could not choose folder", e),
        }
    });
    state_callback!(on_reload, |ui: &App, s: &Rc<RefCell<Session>>| pending(
        ui,
        s,
        Pending::Reload,
        demo
    ));
    state_callback!(on_request_exit, |ui: &App, s: &Rc<RefCell<Session>>| {
        perform_pending(ui, s, demo)
    });
    state_callback!(on_export, |_: &App, state: &Rc<RefCell<Session>>| {
        let result = crate::platform::export_picker();
        let mut s = state.borrow_mut();
        match result {
            Ok(Some(path)) => match s.export(&path) {
                Ok(()) => {
                    s.model.message = "Draft exported".into();
                }
                Err(e) => s.failure("Could not export draft", e),
            },
            Ok(None) => {}
            Err(e) => s.failure("Could not export draft", e),
        }
    });
    state_callback!(on_notices, |_: &App, state: &Rc<RefCell<Session>>| {
        if let Err(e) = crate::notices::show() {
            state.borrow_mut().failure("Could not open notices", e);
        }
    });
    callback!(on_manual, |ui: &App, s: &mut Session| {
        s.prepare_manual(0);
        manual_options(ui, s);
        ui.invoke_open_picker();
    });
    macro_rules! value_callback {
        ($name:ident,$body:expr) => {{
            let weak = ui.as_weak();
            let state = session.clone();
            ui.$name(move |value| {
                if let Some(ui) = weak.upgrade() {
                    let mut s = state.borrow_mut();
                    ($body)(&ui, &mut s, value);
                    refresh(&ui, &s, demo);
                }
            });
        }};
    }
    value_callback!(on_next, |_: &App, s: &mut Session, skip| s
        .model
        .advance(skip));
    value_callback!(on_edit, |_: &App, s: &mut Session, index: i32| {
        if let Ok(i) = usize::try_from(index)
            && i < ACTIONS.len()
        {
            s.model.navigate(View::Controls(Some(i)));
        }
    });
    value_callback!(on_clear, |_: &App, s: &mut Session, index: i32| {
        if let Ok(i) = usize::try_from(index) {
            s.model.clear(i);
        }
    });
    value_callback!(on_set_range, |_: &App, s: &mut Session, value| s
        .model
        .set_range(value));
    value_callback!(on_choose_pedals, |_: &App, s: &mut Session, value| {
        s.model.cancel_capture();
        s.model.pedals = value;
    });
    value_callback!(on_filter, |_: &App, s: &mut Session, value| s
        .model
        .assigned_only =
        value);
    value_callback!(
        on_search,
        |_: &App, s: &mut Session, value: slint::SharedString| s.model.query = value.to_string()
    );
    value_callback!(
        on_manual_device_changed,
        |ui: &App, s: &mut Session, index: i32| {
            if let Ok(i) = usize::try_from(index) {
                s.prepare_manual(i);
                manual_options(ui, s);
            }
        }
    );
    value_callback!(on_manual_bind, |_: &App, s: &mut Session, index: i32| {
        if let Ok(i) = usize::try_from(index)
            && let Err(e) = s.manual_bind(i)
        {
            s.failure("Could not bind input", e);
        }
    });
    {
        let weak = ui.as_weak();
        let state = session.clone();
        ui.on_capture(move || {
            let Some(ui) = weak.upgrade() else { return };
            let mut s = state.borrow_mut();
            if s.model.capture.is_some() {
                s.model.cancel_capture();
                refresh(&ui, &s, demo);
                return;
            }
            if !demo && (!s.catalog.catalog_ok || s.catalog.devices.is_empty()) {
                s.failure(
                    "No controller available",
                    "Connect a controller and try again",
                );
                refresh(&ui, &s, demo);
                return;
            }
            let ticket = s.model.start_capture();
            s.capture_hint = false;
            s.error.clear();
            refresh(&ui, &s, demo);
            drop(s);
            #[cfg(feature = "render-check")]
            if demo && let Some(ticket) = ticket {
                let weak = weak.clone();
                let state = state.clone();
                slint::Timer::single_shot(Duration::from_millis(700), move || {
                    let mut s = state.borrow_mut();
                    if s.model.complete_demo_capture(ticket)
                        && let Some(ui) = weak.upgrade()
                    {
                        refresh(&ui, &s, true);
                    }
                });
            }
            #[cfg(not(feature = "render-check"))]
            let _ = ticket;
        });
    }
    let timer = slint::Timer::default();
    if let Some(worker) = worker.clone() {
        let weak = ui.as_weak();
        let state = session.clone();
        let mut last_active = false;
        let mut last_request = None;
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(33),
            move || {
                let Some(ui) = weak.upgrade() else { return };
                let Ok(mut s) = state.try_borrow_mut() else {
                    return;
                };
                let active = crate::input_windows::foreground_is_ours()
                    && !ui.get_modal_open()
                    && s.installation.is_some()
                    && s.catalog.catalog_ok
                    && s.model.selected().is_some();
                if active != last_active {
                    worker.active(active);
                    last_active = active;
                }
                let request = s.model.capture.map(|c| Request {
                    id: c.id(),
                    axis: ACTIONS[c.action].axis,
                    linked_throttle: ACTIONS[c.action].key == "Throttle",
                });
                if request != last_request {
                    worker.capture(request);
                    last_request = request;
                }
                let snapshot = {
                    let mut latest = worker.latest.lock().unwrap();
                    let snapshot = latest.clone();
                    latest.captured = None;
                    latest.timed_out = None;
                    snapshot
                };
                let changed = s.catalog.devices != snapshot.devices
                    || s.catalog.catalog_ok != snapshot.catalog_ok
                    || s.catalog.error != snapshot.error;
                let previous_discovery_error = s.catalog.error.clone();
                s.catalog = snapshot;
                if changed {
                    s.update_names();
                    if let Some(e) = s.catalog.error.clone() {
                        s.failure("Controller discovery failed", e);
                    } else if previous_discovery_error.as_ref() == Some(&s.error) {
                        s.model.message.clear();
                        s.error.clear();
                    }
                }
                let mut redraw = changed;
                if s.catalog.ambiguous && !s.capture_hint {
                    s.capture_hint = true;
                    redraw = true;
                }
                if let Some(capture) = s.catalog.captured.clone()
                    && let Some(ticket) = s.model.capture
                    && ticket.id() == capture.request
                    && s.model
                        .complete_capture(ticket, capture.device, capture.token)
                {
                    s.capture_hint = false;
                    redraw = true;
                }
                if let Some(request) = s.catalog.timed_out
                    && s.model.capture.is_some_and(|c| c.id() == request)
                {
                    s.model.cancel_capture();
                    s.model.message = "No clear input. Retry or choose input.".into();
                    s.capture_hint = true;
                    redraw = true;
                }
                if let Some(i) = s.model.selected() {
                    let value = if active {
                        s.model.bindings[i]
                            .as_ref()
                            .and_then(|b| input::preview(&s.catalog.samples, &b.device, &b.input))
                    } else {
                        None
                    };
                    s.model.preview = value.unwrap_or(if ACTIONS[i].axis { 0.5 } else { 0. });
                    let inverted = s.model.bindings[i]
                        .as_ref()
                        .map_or(ACTIONS[i].default_inverted(), |b| b.invert);
                    ui.set_preview(if inverted {
                        1. - s.model.preview
                    } else {
                        s.model.preview
                    });
                }
                if redraw {
                    refresh(&ui, &s, demo);
                }
            },
        );
    }
    {
        let weak = ui.as_weak();
        let state = session.clone();
        ui.window().on_close_requested(move || {
            let mut s = state.borrow_mut();
            s.model.cancel_capture();
            s.pending = Pending::Exit;
            if s.model.dirty() {
                if let Some(ui) = weak.upgrade() {
                    ui.invoke_ask_discard();
                }
                slint::CloseRequestResponse::KeepWindowShown
            } else {
                slint::CloseRequestResponse::HideWindow
            }
        });
    }
    refresh(&ui, &session.borrow(), demo);
    #[cfg(feature = "render-check")]
    if let Some(output) = args.iter().find_map(|a| a.strip_prefix("--render-check=")) {
        crate::render_check::schedule(&ui, session.clone(), PathBuf::from(output));
    }
    #[cfg(feature = "render-check")]
    if let Some(output) = args.iter().find_map(|a| a.strip_prefix("--layout-check=")) {
        crate::render_check::layout_check(&ui, PathBuf::from(output));
    }
    let result = ui.run();
    timer.stop();
    drop(worker);
    result
}
