use crate::config::Document;
use std::path::{Path, PathBuf};
#[derive(Debug)]
pub struct Installation {
    pub root: PathBuf,
    pub game: PathBuf,
    pub document: Document,
}
impl Installation {
    pub fn load(root: &Path) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|e| format!("Cannot open mod folder: {e}"))?;
        for file in [
            "Scripts/main.lua",
            "Scripts/mapper.lua",
            "Scripts/actions.lua",
            "Scripts/ac8_hotas_devices_001.dll",
            "Scripts/weapon_select.lua",
            "Scripts/ac8_hotas_weapons_001.dll",
            "bindings.lua",
        ] {
            if !root.join(file).is_file() {
                return Err(format!("AC8HOTAS is missing {file}"));
            }
        }
        let game = root
            .ancestors()
            .map(|p| p.join("AceCombat8.exe"))
            .find(|p| p.is_file())
            .ok_or("Choose AC8HOTAS inside the game installation")?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let text = std::fs::read_to_string(root.join("bindings.lua"))
            .map_err(|e| format!("Cannot read bindings.lua: {e}"))?;
        let document = Document::parse(text)?;
        Ok(Self {
            root,
            game,
            document,
        })
    }
    pub fn adjacent(executable: &Path) -> Option<Self> {
        let parent = executable.parent()?;
        [
            parent.to_path_buf(),
            parent.join("AC8HOTAS"),
            parent.parent()?.to_path_buf(),
        ]
        .iter()
        .find_map(|p| Self::load(p).ok())
    }
}
pub fn path_key(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "{:x}",
        Sha256::digest(path.to_string_lossy().to_lowercase().as_bytes())
    )
}
