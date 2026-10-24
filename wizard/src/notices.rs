//! Self-contained dependency notices, opened only from About on request.
const HTML: &str = include_str!("../notices.html");

pub fn show() -> Result<(), String> {
    let directory = std::path::PathBuf::from(
        std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA is unavailable")?,
    )
    .join("AC8HOTAS");
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let path = directory.join("wizard-notices.html");
    crate::storage::write_settings(&path, HTML.as_bytes())?;
    crate::platform::open_document(&path)
}
