use crate::{
    actions::ACTIONS,
    input::{self, Snapshot},
    installation::Installation,
    model::{AppModel, View},
    settings::Settings,
};
use std::path::{Path, PathBuf};
#[derive(Clone, Copy)]
pub enum Pending {
    Exit,
    Folder,
    Reload,
}

pub struct Session {
    pub model: AppModel,
    pub installation: Option<Installation>,
    pub settings: Settings,
    pub catalog: Snapshot,
    pub error: String,
    pub conflict: bool,
    pub pending: Pending,
    pub pending_folder: Option<PathBuf>,
    pub manual_device: usize,
    pub manual_tokens: Vec<String>,
    pub capture_hint: bool,
}
impl Session {
    pub fn new() -> Self {
        let (settings, error) = match Settings::load() {
            Ok(s) => (s, String::new()),
            Err(e) => (Settings::default(), e),
        };
        Self {
            model: AppModel::default(),
            installation: None,
            settings,
            catalog: Snapshot::default(),
            error,
            conflict: false,
            pending: Pending::Exit,
            pending_folder: None,
            manual_device: 0,
            manual_tokens: Vec::new(),
            capture_hint: false,
        }
    }
    pub fn load(&mut self, path: &Path) -> Result<(), String> {
        let installation = Installation::load(path)?;
        let key = crate::installation::path_key(&installation.root);
        let completed = self.settings.completed.get(&key).copied().unwrap_or(false);
        self.model
            .load(installation.document.bindings.clone(), completed);
        self.settings.last_installation = Some(installation.root.clone());
        self.installation = Some(installation);
        self.error.clear();
        self.conflict = false;
        self.capture_hint = false;
        self.update_names();
        if let Err(e) = self.settings.save() {
            self.failure("Could not remember mod folder", e);
        }
        Ok(())
    }
    pub fn prepare_folder(&mut self, path: Option<&Path>) -> Result<bool, String> {
        self.pending_folder = None;
        let Some(path) = path else { return Ok(false) };
        let selected = Installation::load(path)?;
        if self.installation.as_ref().is_some_and(|current| {
            crate::installation::path_key(&current.root)
                == crate::installation::path_key(&selected.root)
        }) {
            return Ok(false);
        }
        self.pending_folder = Some(selected.root);
        Ok(true)
    }
    pub fn startup(&mut self, requested: Option<PathBuf>) {
        if let Some(path) = requested {
            if let Err(e) = self.load(&path) {
                self.failure("Could not open mod", e);
            }
            return;
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(found) = Installation::adjacent(&exe)
        {
            let root = found.root.clone();
            if let Err(e) = self.load(&root) {
                self.failure("Could not open mod", e);
            }
            return;
        }
        if let Some(path) = self.settings.last_installation.clone()
            && let Err(e) = self.load(&path)
        {
            self.failure("Choose the mod folder again", e);
        }
    }
    pub fn update_names(&mut self) {
        for d in &self.catalog.devices {
            self.model.device_names.insert(d.id.clone(), d.name.clone());
        }
    }
    pub fn failure(&mut self, short: &str, error: impl Into<String>) {
        self.error = error.into();
        self.conflict = self.error.contains("changed outside the wizard");
        self.model.message = if self.error.chars().count() < 95 {
            self.error.clone()
        } else {
            short.into()
        };
    }
    pub fn save(&mut self) -> Result<(), String> {
        let install = self
            .installation
            .as_mut()
            .ok_or("Choose a mod folder first")?;
        input::validate(&self.model.bindings, &self.catalog)?;
        let result = crate::storage::save(
            &install.root,
            &install.document,
            &self.model.bindings,
            || crate::platform::game_closed(&install.game),
        )?;
        install.document = result.document;
        self.model.saved.clone_from(&self.model.bindings);
        self.model.message = "Saved · apply on next game launch".into();
        self.error.clear();
        self.conflict = false;
        let key = crate::installation::path_key(&install.root);
        self.settings
            .completed
            .insert(key, self.model.setup_completed);
        self.settings.last_installation = Some(install.root.clone());
        if let Err(e) = self.settings.save() {
            self.failure(
                "Saved; settings could not be remembered",
                format!(
                    "Configuration saved. Settings error: {e}. Backup: {}",
                    result.backup.display()
                ),
            );
        }
        Ok(())
    }
    pub fn export(&self, path: &Path) -> Result<(), String> {
        let install = self
            .installation
            .as_ref()
            .ok_or("Choose a mod folder first")?;
        crate::storage::export(path, &install.document, &self.model.bindings)
    }
    pub fn reload(&mut self) -> Result<(), String> {
        let root = self
            .installation
            .as_ref()
            .ok_or("Choose a mod folder first")?
            .root
            .clone();
        self.load(&root)
    }
    pub fn prepare_manual(&mut self, index: usize) {
        self.model.cancel_capture();
        self.manual_device = index.min(self.catalog.devices.len().saturating_sub(1));
        self.manual_tokens = if let (Some(d), Some(action)) = (
            self.catalog.devices.get(self.manual_device),
            self.model.selected(),
        ) {
            d.inputs
                .iter()
                .filter(|i| i.axis == ACTIONS[action].axis)
                .map(|i| i.token.clone())
                .collect()
        } else {
            Vec::new()
        };
    }
    pub fn manual_bind(&mut self, input_index: usize) -> Result<(), String> {
        if !self.catalog.catalog_ok {
            return Err("Controller discovery is not ready".into());
        }
        let d = self
            .catalog
            .devices
            .get(self.manual_device)
            .ok_or("Controller disconnected")?;
        let token = self
            .manual_tokens
            .get(input_index)
            .ok_or("Choose an input")?;
        let action = self.model.selected().ok_or("Choose an action")?;
        if !d
            .inputs
            .iter()
            .any(|i| i.token == *token && i.axis == ACTIONS[action].axis)
        {
            return Err("Input no longer available".into());
        }
        let ticket = self.model.start_capture().ok_or("Choose an action")?;
        self.model
            .complete_capture(ticket, d.id.clone(), token.clone());
        self.capture_hint = false;
        self.model.message.clear();
        self.error.clear();
        Ok(())
    }
    pub fn shared_warning(&self) -> String {
        let Some(index) = self.model.selected() else {
            return String::new();
        };
        let Some(binding) = &self.model.bindings[index] else {
            return String::new();
        };
        let names: Vec<_> = self
            .model
            .bindings
            .iter()
            .enumerate()
            .filter_map(|(other, b)| {
                let b = b.as_ref()?;
                (other != index && b.device == binding.device && b.input == binding.input)
                    .then_some(ACTIONS[other].label)
            })
            .collect();
        if names.is_empty() {
            String::new()
        } else {
            format!("Shared with {}", names.join(", "))
        }
    }
    pub fn view_controls(&mut self) {
        self.model.navigate(View::Controls(None));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(root: &Path, name: &str) -> PathBuf {
        let path = root.join(name);
        std::fs::create_dir_all(path.join("Scripts")).unwrap();
        std::fs::write(root.join("AceCombat8.exe"), "test fixture").unwrap();
        for file in [
            "main.lua",
            "mapper.lua",
            "actions.lua",
            "weapon_select.lua",
            "ac8_hotas_devices_001.dll",
            "ac8_hotas_weapons_001.dll",
        ] {
            std::fs::write(path.join("Scripts").join(file), "fixture").unwrap();
        }
        std::fs::write(path.join("bindings.lua"), "return {version=1, bindings={}}").unwrap();
        path
    }
    #[test]
    fn folder_selection_only_prepares_a_different_valid_installation() {
        let temp = tempfile::tempdir().unwrap();
        let first = fixture(temp.path(), "first");
        let second = fixture(temp.path(), "second");
        let mut session = Session::new();
        session.installation = Some(Installation::load(&first).unwrap());
        session.model.bindings[6] = Some(crate::model::Binding {
            device: "{11111111-2222-3333-4444-555555555555}".into(),
            input: "Button1".into(),
            invert: false,
            zero_to_one: false,
        });
        let draft = session.model.bindings.clone();
        assert!(session.model.dirty());
        assert!(!session.prepare_folder(None).unwrap());
        assert!(!session.prepare_folder(Some(&first.join("."))).unwrap());
        assert!(session.prepare_folder(Some(temp.path())).is_err());
        assert!(session.pending_folder.is_none());
        assert!(session.prepare_folder(Some(&second)).unwrap());
        assert_eq!(session.pending_folder, Some(second.canonicalize().unwrap()));
        assert_eq!(
            session.installation.as_ref().unwrap().root,
            first.canonicalize().unwrap()
        );
        assert_eq!(session.model.bindings, draft);
        assert!(session.model.dirty());
    }
}
