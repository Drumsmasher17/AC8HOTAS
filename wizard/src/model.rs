use crate::actions::{ACTIONS, SETUP};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Binding {
    pub device: String,
    pub input: String,
    pub invert: bool,
    pub zero_to_one: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    Setup(usize),
    Controls(Option<usize>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capture {
    pub action: usize,
    generation: u64,
}
impl Capture {
    pub fn id(self) -> u64 {
        self.generation
    }
}

pub struct AppModel {
    pub view: View,
    pub bindings: Vec<Option<Binding>>,
    pub saved: Vec<Option<Binding>>,
    pub pedals: bool,
    pub assigned_only: bool,
    pub query: String,
    pub capture: Option<Capture>,
    generation: u64,
    pub preview: f32,
    pub message: String,
    pub device_names: BTreeMap<String, String>,
    pub setup_completed: bool,
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            view: View::Setup(0),
            bindings: vec![None; ACTIONS.len()],
            saved: vec![None; ACTIONS.len()],
            pedals: false,
            assigned_only: false,
            query: String::new(),
            capture: None,
            generation: 0,
            preview: 0.5,
            message: String::new(),
            device_names: BTreeMap::new(),
            setup_completed: false,
        }
    }
}

impl AppModel {
    pub fn selected(&self) -> Option<usize> {
        match self.view {
            View::Setup(step) => Some(SETUP[step]),
            View::Controls(action) => action,
        }
    }

    pub fn cancel_capture(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.capture = None;
    }

    pub fn navigate(&mut self, view: View) {
        self.cancel_capture();
        self.view = view;
        self.preview = 0.5;
        self.message.clear();
    }

    pub fn advance(&mut self, skip: bool) {
        if let View::Setup(step) = self.view {
            if !skip && self.bindings[SETUP[step]].is_none() {
                return;
            }
            let view = if step + 1 < SETUP.len() {
                View::Setup(step + 1)
            } else {
                self.setup_completed = true;
                View::Controls(None)
            };
            self.navigate(view);
        }
    }

    pub fn back(&mut self) {
        if let View::Setup(step) = self.view {
            self.navigate(View::Setup(step.saturating_sub(1)));
        }
    }

    pub fn start_capture(&mut self) -> Option<Capture> {
        self.cancel_capture();
        let ticket = Capture {
            action: self.selected()?,
            generation: self.generation,
        };
        self.capture = Some(ticket);
        self.message.clear();
        Some(ticket)
    }

    #[cfg(any(test, feature = "render-check"))]
    pub fn complete_demo_capture(&mut self, ticket: Capture) -> bool {
        if self.capture != Some(ticket) || self.selected() != Some(ticket.action) {
            return false;
        }
        let action = ACTIONS[ticket.action];
        let prior = self.bindings[ticket.action].as_ref();
        let token = match action.key {
            "Roll" => "X".into(),
            "Pitch" => "Y".into(),
            "Yaw" => "Rz".into(),
            "Throttle" => "Z".into(),
            "CameraPitch" => "Ry".into(),
            "CameraYaw" => "Rx".into(),
            _ => format!("Button{}", ticket.action.saturating_sub(5).max(1)),
        };
        self.bindings[ticket.action] = Some(Binding {
            device: if action.key == "Throttle" {
                "Demo throttle"
            } else if action.key == "Yaw" && self.pedals {
                "Demo pedals"
            } else {
                "Demo stick"
            }
            .into(),
            input: token,
            invert: prior.map_or(action.default_inverted(), |b| b.invert),
            zero_to_one: prior.map_or(action.key == "Throttle", |b| b.zero_to_one),
        });
        self.capture = None;
        self.preview = 0.77;
        true
    }

    pub fn complete_capture(&mut self, ticket: Capture, device: String, input: String) -> bool {
        if self.capture != Some(ticket) || self.selected() != Some(ticket.action) {
            return false;
        }
        let prior = self.bindings[ticket.action].as_ref();
        self.bindings[ticket.action] = Some(Binding {
            device,
            input,
            invert: prior.map_or(ACTIONS[ticket.action].default_inverted(), |b| b.invert),
            zero_to_one: prior.map_or(ACTIONS[ticket.action].key == "Throttle", |b| b.zero_to_one),
        });
        self.capture = None;
        true
    }

    pub fn load(&mut self, bindings: Vec<Option<Binding>>, completed: bool) {
        self.cancel_capture();
        self.bindings = bindings;
        self.saved.clone_from(&self.bindings);
        self.setup_completed = completed;
        self.view = if completed {
            View::Controls(None)
        } else {
            View::Setup(0)
        };
        self.query.clear();
        self.assigned_only = false;
        self.message.clear();
    }

    pub fn clear(&mut self, index: usize) {
        self.cancel_capture();
        if let Some(binding) = self.bindings.get_mut(index) {
            *binding = None;
        }
        self.message.clear();
    }

    pub fn invert(&mut self) {
        if let Some(index) = self.selected()
            && ACTIONS[index].axis
            && let Some(binding) = self.bindings[index].as_mut()
        {
            binding.invert = !binding.invert;
        }
    }

    pub fn set_range(&mut self, zero_to_one: bool) {
        if let Some(index) = self.selected()
            && ACTIONS[index].axis
            && let Some(binding) = self.bindings[index].as_mut()
        {
            binding.zero_to_one = zero_to_one;
        }
    }

    #[cfg(test)]
    pub fn save_demo(&mut self) {
        self.saved.clone_from(&self.bindings);
        self.message = "Preview saved. No files written.".into();
    }

    pub fn dirty(&self) -> bool {
        self.saved != self.bindings
    }
    pub fn assigned_count(&self) -> usize {
        self.bindings.iter().filter(|b| b.is_some()).count()
    }

    pub fn binding_label(&self, index: usize) -> String {
        self.bindings[index].as_ref().map_or_else(
            || "Unbound".into(),
            |b| {
                format!(
                    "{} · {}{}",
                    self.device_names.get(&b.device).unwrap_or(&b.device),
                    b.input,
                    if b.invert { " ↔" } else { "" }
                )
            },
        )
    }

    pub fn visible(&self, index: usize) -> bool {
        let action = ACTIONS[index];
        (!self.assigned_only || self.bindings[index].is_some())
            && (self.query.is_empty()
                || format!(
                    "{} {} {}",
                    action.key,
                    action.label,
                    self.binding_label(index)
                )
                .to_lowercase()
                .contains(&self.query.to_lowercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_new_pitch_and_throttle_bindings_default_to_inverted() {
        let mut m = AppModel::default();
        for (i, action) in ACTIONS.iter().enumerate() {
            m.navigate(View::Controls(Some(i)));
            let ticket = m.start_capture().unwrap();
            assert!(m.complete_capture(
                ticket,
                "controller".into(),
                if action.axis { "X" } else { "Button1" }.into()
            ));
            assert_eq!(
                m.bindings[i].as_ref().unwrap().invert,
                matches!(action.key, "Pitch" | "Throttle"),
                "{}",
                action.key
            );
        }
        // Explicit non-inverted settings survive both import and rebinding.
        for binding in m.bindings.iter_mut().flatten() {
            binding.invert = false;
        }
        let imported = m.bindings.clone();
        m.load(imported.clone(), true);
        assert_eq!(m.bindings, imported);
        for i in [1, 3] {
            m.navigate(View::Controls(Some(i)));
            let ticket = m.start_capture().unwrap();
            assert!(m.complete_capture(ticket, "other controller".into(), "Y".into()));
            assert!(!m.bindings[i].as_ref().unwrap().invert);
        }
    }

    #[test]
    fn skip_back_and_rerun_preserve_assignments() {
        let mut m = AppModel::default();
        let ticket = m.start_capture().unwrap();
        assert!(m.complete_demo_capture(ticket));
        m.invert();
        let original = m.bindings[0].clone();
        m.advance(true);
        m.back();
        assert_eq!(m.bindings[0], original);
        for _ in 0..SETUP.len() {
            m.advance(true);
        }
        assert_eq!(m.view, View::Controls(None));
        m.navigate(View::Setup(0));
        assert_eq!(m.bindings[0], original);
        assert!(m.dirty());
    }

    #[test]
    fn late_capture_cannot_bind_after_navigation_or_cancellation() {
        let mut m = AppModel::default();
        let first = m.start_capture().unwrap();
        m.advance(true);
        let second = m.start_capture().unwrap();
        assert!(!m.complete_demo_capture(first));
        assert!(m.complete_demo_capture(second));
        let third = m.start_capture().unwrap();
        m.cancel_capture();
        assert!(!m.complete_demo_capture(third));
        assert!(m.bindings[0].is_none());
    }

    #[test]
    fn rebind_keeps_axis_options_and_clear_updates_dirty_state() {
        let mut m = AppModel::default();
        m.navigate(View::Controls(Some(3)));
        let ticket = m.start_capture().unwrap();
        m.complete_demo_capture(ticket);
        assert!(m.bindings[3].as_ref().unwrap().zero_to_one);
        assert!(m.bindings[3].as_ref().unwrap().invert);
        m.set_range(false);
        m.invert();
        let ticket = m.start_capture().unwrap();
        m.complete_demo_capture(ticket);
        let binding = m.bindings[3].as_ref().unwrap();
        assert!(!binding.invert);
        assert!(!binding.zero_to_one);
        m.save_demo();
        assert!(!m.dirty());
        m.clear(3);
        assert!(m.dirty());
    }

    #[test]
    fn filtering_searches_tokens_keys_and_device_names() {
        let mut m = AppModel {
            assigned_only: true,
            ..AppModel::default()
        };
        assert!(!m.visible(0));
        let ticket = m.start_capture().unwrap();
        m.complete_demo_capture(ticket);
        m.query = "demo stick".into();
        assert!(m.visible(0));
        m.query = "SelectSpecialWeapon2".into();
        m.assigned_only = false;
        assert!(m.visible(11));
        assert!(!m.visible(0));
    }
}
