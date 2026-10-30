use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Default, Serialize, Deserialize)]
pub struct Settings {
    pub last_installation: Option<PathBuf>,
    pub completed: BTreeMap<String, bool>,
}
impl Settings {
    fn path() -> Result<PathBuf, String> {
        Ok(
            PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?)
                .join("AC8HOTAS/wizard-state.json"),
        )
    }
    pub fn load() -> Result<Self, String> {
        Self::load_from(&Self::path()?)
    }
    fn load_from(path: &std::path::Path) -> Result<Self, String> {
        match std::fs::read(path) {
            Ok(data) => serde_json::from_slice(&data)
                .map_err(|e| format!("Cannot read wizard settings: {e}")),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("Cannot open wizard settings: {e}")),
        }
    }
    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::path()?)
    }
    fn save_to(&self, path: &std::path::Path) -> Result<(), String> {
        let _lock = crate::platform::InstallLock::acquire(&format!(
            "settings-{}",
            crate::installation::path_key(path)
        ))?;
        std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let mut merged = Self::load_from(path)?;
        merged.completed.extend(self.completed.clone());
        merged.last_installation = self.last_installation.clone();
        let data = serde_json::to_vec_pretty(&merged).map_err(|e| e.to_string())?;
        crate::storage::write_settings(path, &data)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_settings_and_multiple_installations_survive_updates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("设置.json");
        let mut first = Settings {
            last_installation: Some(PathBuf::from("D:/游戏/AC8HOTAS")),
            ..Default::default()
        };
        first.completed.insert("one".into(), true);
        first.save_to(&path).unwrap();
        let mut second = Settings::default();
        second.completed.insert("two".into(), true);
        second.save_to(&path).unwrap();
        let read = Settings::load_from(&path).unwrap();
        assert_eq!(read.completed.len(), 2);
        first.save_to(&path).unwrap();
        assert_eq!(
            Settings::load_from(&path).unwrap().last_installation,
            first.last_installation
        );
    }
    #[test]
    fn corrupt_settings_are_not_silently_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, "broken").unwrap();
        assert!(Settings::default().save_to(&path).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "broken");
    }
}
