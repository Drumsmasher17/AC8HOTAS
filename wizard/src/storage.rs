use crate::{config::Document, model::Binding};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
fn unique() -> String {
    format!(
        "{}-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}
struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn write_new(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut f = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .map_err(|e| format!("Cannot create {}: {e}", path.display()))?;
    f.write_all(data)
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("Cannot flush {}: {e}", path.display()))
}
fn unchanged(path: &Path, document: &Document) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| format!("Cannot read current bindings: {e}"))?;
    if bytes != document.text.as_bytes() {
        return Err("bindings.lua changed outside the wizard. Reload or export your draft.".into());
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage {
    Backup,
    Replace,
}
pub struct Saved {
    pub document: Document,
    pub backup: PathBuf,
}
pub fn save(
    root: &Path,
    document: &Document,
    bindings: &[Option<Binding>],
    game_closed: impl Fn() -> Result<(), String>,
) -> Result<Saved, String> {
    save_inner(root, document, bindings, game_closed, |_| Ok(()))
}
fn save_inner(
    root: &Path,
    document: &Document,
    bindings: &[Option<Binding>],
    game_closed: impl Fn() -> Result<(), String>,
    checkpoint: impl Fn(Stage) -> Result<(), String>,
) -> Result<Saved, String> {
    let _lock = crate::platform::InstallLock::acquire(&crate::installation::path_key(root))?;
    let serialized = document.render(bindings)?;
    let replacement = Document::parse(serialized.clone())?;
    let path = root.join("bindings.lua");
    game_closed()?;
    unchanged(&path, document)?;
    checkpoint(Stage::Backup)?;
    let backup = root.join(format!("bindings.wizard-{}.lua.bak", unique()));
    write_new(&backup, document.text.as_bytes())?;
    if fs::read(&backup).map_err(|e| e.to_string())? != document.text.as_bytes() {
        return Err("Backup verification failed".into());
    }
    let temp = Temporary(root.join(format!(".bindings-wizard-{}.tmp", unique())));
    write_new(&temp.0, serialized.as_bytes())?;
    let readback = fs::read_to_string(&temp.0).map_err(|e| e.to_string())?;
    if Document::parse(readback)?.bindings != bindings {
        return Err("Temporary config verification failed".into());
    }
    checkpoint(Stage::Replace)?;
    game_closed()?;
    unchanged(&path, document)?;
    crate::platform::replace(&path, &temp.0)?;
    if fs::read(&path).map_err(|e| e.to_string())? != serialized.as_bytes() {
        return Err(format!(
            "Config changed after replacement. Recovery copy: {}",
            backup.display()
        ));
    }
    Ok(Saved {
        document: replacement,
        backup,
    })
}
pub fn export(
    path: &Path,
    document: &Document,
    bindings: &[Option<Binding>],
) -> Result<(), String> {
    write_new(path, document.render(bindings)?.as_bytes())
}
pub fn write_settings(path: &Path, data: &[u8]) -> Result<(), String> {
    let temp = Temporary(path.with_extension(format!("{}.tmp", unique())));
    write_new(&temp.0, data)?;
    if path.exists() {
        crate::platform::replace(path, &temp.0)
    } else {
        fs::rename(&temp.0, path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Document, Vec<Option<Binding>>) {
        let dir = tempfile::tempdir().unwrap();
        let doc = Document::parse(include_str!("../../mod/bindings.lua").into()).unwrap();
        fs::write(dir.path().join("bindings.lua"), &doc.text).unwrap();
        let mut b = doc.bindings.clone();
        b[0] = Some(Binding {
            device: "{11111111-2222-3333-4444-555555555555}".into(),
            input: "X".into(),
            invert: true,
            zero_to_one: false,
        });
        (dir, doc, b)
    }
    #[test]
    fn successful_save_has_exact_backup_and_verified_output() {
        let (d, doc, b) = fixture();
        let result = save(d.path(), &doc, &b, || Ok(())).unwrap();
        assert_eq!(fs::read_to_string(result.backup).unwrap(), doc.text);
        assert_eq!(
            Document::parse(fs::read_to_string(d.path().join("bindings.lua")).unwrap())
                .unwrap()
                .bindings,
            b
        );
        assert_eq!(result.document.bindings, b);
    }
    #[test]
    fn external_edit_and_game_checks_preserve_original() {
        let (d, doc, b) = fixture();
        assert!(save(d.path(), &doc, &b, || Err("game running".into())).is_err());
        assert_eq!(
            fs::read_to_string(d.path().join("bindings.lua")).unwrap(),
            doc.text
        );
        fs::write(d.path().join("bindings.lua"), "-- external").unwrap();
        assert!(save(d.path(), &doc, &b, || Ok(())).is_err());
        assert_eq!(
            fs::read_to_string(d.path().join("bindings.lua")).unwrap(),
            "-- external"
        );
    }
    #[test]
    fn failures_leave_original_and_clean_temporary_files() {
        for stage in [Stage::Backup, Stage::Replace] {
            let (d, doc, b) = fixture();
            assert!(
                save_inner(
                    d.path(),
                    &doc,
                    &b,
                    || Ok(()),
                    |s| if s == stage {
                        Err("injected I/O failure".into())
                    } else {
                        Ok(())
                    }
                )
                .is_err()
            );
            assert_eq!(
                fs::read_to_string(d.path().join("bindings.lua")).unwrap(),
                doc.text
            );
            assert!(
                !fs::read_dir(d.path()).unwrap().any(|p| p
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|e| e == "tmp"))
            );
        }
    }
    #[test]
    fn last_moment_external_edit_is_not_overwritten() {
        let (d, doc, b) = fixture();
        assert!(
            save_inner(
                d.path(),
                &doc,
                &b,
                || Ok(()),
                |s| {
                    if s == Stage::Replace {
                        fs::write(d.path().join("bindings.lua"), "external change").unwrap();
                    }
                    Ok(())
                }
            )
            .is_err()
        );
        assert_eq!(
            fs::read_to_string(d.path().join("bindings.lua")).unwrap(),
            "external change"
        );
    }
    #[test]
    fn windows_file_lock_rejects_replacement_without_losing_original() {
        use std::{cell::RefCell, os::windows::fs::OpenOptionsExt};
        let (d, doc, b) = fixture();
        let held = RefCell::new(None);
        assert!(
            save_inner(
                d.path(),
                &doc,
                &b,
                || Ok(()),
                |s| {
                    if s == Stage::Replace {
                        *held.borrow_mut() = Some(
                            OpenOptions::new()
                                .read(true)
                                .share_mode(1 | 2)
                                .open(d.path().join("bindings.lua"))
                                .unwrap(),
                        );
                    }
                    Ok(())
                }
            )
            .is_err()
        );
        drop(held);
        assert_eq!(
            fs::read_to_string(d.path().join("bindings.lua")).unwrap(),
            doc.text
        );
    }
    #[test]
    fn exports_do_not_overwrite_existing_files() {
        let (d, doc, b) = fixture();
        let path = d.path().join("export.lua");
        export(&path, &doc, &b).unwrap();
        assert!(export(&path, &doc, &b).is_err());
    }
}
