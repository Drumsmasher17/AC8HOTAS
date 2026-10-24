//! Small audited Win32 boundary. No game memory access or input injection.
#![allow(unsafe_code)]
use std::{
    ffi::OsString,
    mem::size_of,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Foundation::*,
        Storage::FileSystem::{REPLACE_FILE_FLAGS, ReplaceFileW},
        System::{Com::*, Diagnostics::ToolHelp::*, Threading::*},
        UI::{
            Shell::*,
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
    core::{PCWSTR, PWSTR},
};
fn wide(p: &Path) -> Vec<u16> {
    p.as_os_str().encode_wide().chain(Some(0)).collect()
}
pub fn open_document(path: &Path) -> Result<(), String> {
    let path = wide(path);
    unsafe {
        let result = ShellExecuteW(
            None,
            windows::core::w!("open"),
            PCWSTR(path.as_ptr()),
            None,
            None,
            windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL,
        );
        if result.0 as isize <= 32 {
            Err("Could not open third-party notices in the default browser".into())
        } else {
            Ok(())
        }
    }
}
struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
pub struct Apartment(bool);
impl Apartment {
    pub fn ui() -> Self {
        Self(unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok() })
    }
}
impl Drop for Apartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
pub struct InstallLock(Handle);
impl InstallLock {
    pub fn acquire(key: &str) -> Result<Self, String> {
        unsafe {
            let name: Vec<u16> = format!("Local\\AC8HOTAS-Wizard-{key}")
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let handle = Handle(
                CreateMutexW(None, false, PCWSTR(name.as_ptr())).map_err(|e| e.to_string())?,
            );
            match WaitForSingleObject(handle.0, 0) {
                WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Self(handle)),
                _ => Err("Another wizard is saving this installation".into()),
            }
        }
    }
}
impl Drop for InstallLock {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.0.0);
        }
    }
}
pub fn replace(original: &Path, temp: &Path) -> Result<(), String> {
    let original = wide(original);
    let temp = wide(temp);
    unsafe {
        ReplaceFileW(
            PCWSTR(original.as_ptr()),
            PCWSTR(temp.as_ptr()),
            PCWSTR::null(),
            REPLACE_FILE_FLAGS(0),
            None,
            None,
        )
        .map_err(|e| format!("Could not replace bindings.lua: {e}"))
    }
}
fn dialog_owner() -> Option<HWND> {
    unsafe {
        let window = GetForegroundWindow();
        let mut pid = 0;
        GetWindowThreadProcessId(window, Some(&mut pid));
        (pid == std::process::id()).then_some(window)
    }
}
fn folder_dialog(initial: Option<&Path>) -> Result<IFileOpenDialog, String> {
    unsafe {
        let dialog: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        dialog
            .SetOptions(
                dialog.GetOptions().map_err(|e| e.to_string())?
                    | FOS_PICKFOLDERS
                    | FOS_FORCEFILESYSTEM
                    | FOS_PATHMUSTEXIST,
            )
            .map_err(|e| e.to_string())?;
        let title: Vec<u16> = "Choose the AC8HOTAS mod folder"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        dialog
            .SetTitle(PCWSTR(title.as_ptr()))
            .map_err(|e| e.to_string())?;
        if let Some(initial) = initial {
            // canonicalize() returns verbatim paths, which Shell rejects.
            // Preserve UTF-16 while converting only the Windows path prefix.
            let mut initial_name = wide(initial);
            if let Some(std::path::Component::Prefix(prefix)) = initial.components().next() {
                match prefix.kind() {
                    std::path::Prefix::VerbatimDisk(_) => initial_name.drain(..4).for_each(drop),
                    std::path::Prefix::VerbatimUNC(_, _) => {
                        initial_name.drain(..8).for_each(drop);
                        initial_name.splice(..0, [b'\\' as u16, b'\\' as u16]);
                    }
                    _ => {}
                }
            }
            let folder: IShellItem =
                SHCreateItemFromParsingName(PCWSTR(initial_name.as_ptr()), None)
                    .map_err(|e| format!("Cannot open current mod folder: {e}"))?;
            dialog.SetFolder(&folder).map_err(|e| e.to_string())?;
        }
        Ok(dialog)
    }
}
pub fn folder_picker(initial: Option<&Path>) -> Result<Option<PathBuf>, String> {
    unsafe {
        let dialog = folder_dialog(initial)?;
        if let Err(e) = dialog.Show(dialog_owner()) {
            if e.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) {
                return Ok(None);
            }
            return Err(e.to_string());
        }
        let item = dialog.GetResult().map_err(|e| e.to_string())?;
        let name = item
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(|e| e.to_string())?;
        let mut len = 0;
        while *name.0.add(len) != 0 {
            len += 1;
        }
        let path = PathBuf::from(OsString::from_wide(std::slice::from_raw_parts(name.0, len)));
        CoTaskMemFree(Some(name.0 as *const _));
        Ok(Some(path))
    }
}
pub fn export_picker() -> Result<Option<PathBuf>, String> {
    unsafe {
        let dialog: IFileSaveDialog = CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| e.to_string())?;
        dialog
            .SetFileName(windows::core::w!("bindings-export.lua"))
            .map_err(|e| e.to_string())?;
        dialog
            .SetDefaultExtension(windows::core::w!("lua"))
            .map_err(|e| e.to_string())?;
        if let Err(e) = dialog.Show(dialog_owner()) {
            if e.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) {
                return Ok(None);
            }
            return Err(e.to_string());
        }
        let item = dialog.GetResult().map_err(|e| e.to_string())?;
        let name = item
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(|e| e.to_string())?;
        let mut len = 0;
        while *name.0.add(len) != 0 {
            len += 1;
        }
        let path = PathBuf::from(OsString::from_wide(std::slice::from_raw_parts(name.0, len)));
        CoTaskMemFree(Some(name.0 as *const _));
        Ok(Some(path))
    }
}
pub fn game_closed(executable: &Path) -> Result<(), String> {
    unsafe {
        let snapshot = Handle(
            CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|e| format!("Cannot inspect game process: {e}"))?,
        );
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        Process32FirstW(snapshot.0, &mut entry).map_err(|e| e.to_string())?;
        let name = executable
            .file_name()
            .ok_or("No game executable name")?
            .to_string_lossy();
        loop {
            let len = entry
                .szExeFile
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..len]).eq_ignore_ascii_case(&name) {
                let handle = Handle(
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION,
                        false,
                        entry.th32ProcessID,
                    )
                    .map_err(|_| "Close Ace Combat 8 before saving (process access unavailable)")?,
                );
                let mut buf = vec![0u16; 32768];
                let mut size = buf.len() as u32;
                QueryFullProcessImageNameW(
                    handle.0,
                    PROCESS_NAME_WIN32,
                    PWSTR(buf.as_mut_ptr()),
                    &mut size,
                )
                .map_err(|_| "Close Ace Combat 8 before saving (process path unavailable)")?;
                let actual = PathBuf::from(OsString::from_wide(&buf[..size as usize]))
                    .canonicalize()
                    .map_err(|_| "Cannot verify game executable path")?;
                if actual
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&executable.to_string_lossy())
                {
                    return Err("Close Ace Combat 8 before saving".into());
                }
            }
            if let Err(e) = Process32NextW(snapshot.0, &mut entry) {
                if e.code() != windows::core::HRESULT::from_win32(ERROR_NO_MORE_FILES.0) {
                    return Err(format!("Cannot finish game process check: {e}"));
                }
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folder_dialog_starts_at_the_requested_directory() {
        let _apartment = Apartment::ui();
        let temp = tempfile::tempdir().unwrap();
        let requested = temp.path().join("游戏 AC8HOTAS");
        std::fs::create_dir(&requested).unwrap();
        let requested = requested.canonicalize().unwrap();
        let dialog = folder_dialog(Some(&requested)).unwrap();
        unsafe {
            let folder = dialog.GetFolder().unwrap();
            let name = folder.GetDisplayName(SIGDN_FILESYSPATH).unwrap();
            let actual = name.to_string().unwrap();
            CoTaskMemFree(Some(name.0.cast()));
            assert_eq!(PathBuf::from(actual).canonicalize().unwrap(), requested);
        }
    }
    #[test]
    #[ignore = "subprocess fixture, invoked by process_guard_matches_full_executable_path"]
    fn process_guard_child() {
        let Some(ready) = std::env::var_os("AC8HOTAS_TEST_READY") else {
            return;
        };
        std::fs::write(ready, "ready").unwrap();
        std::thread::sleep(std::time::Duration::from_secs(30));
    }
    #[test]
    fn process_guard_matches_full_executable_path() {
        struct Child(std::process::Child);
        impl Drop for Child {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let executable = dir.path().join("AceCombat8.exe");
        std::fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        let executable = executable.canonicalize().unwrap();
        assert!(game_closed(&executable).is_ok());
        let ready = dir.path().join("ready");
        let mut child = Child(
            std::process::Command::new(&executable)
                .args([
                    "--exact",
                    "platform::tests::process_guard_child",
                    "--ignored",
                ])
                .env("AC8HOTAS_TEST_READY", &ready)
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
        for _ in 0..200 {
            if ready.exists() {
                break;
            }
            assert!(child.0.try_wait().unwrap().is_none());
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(ready.exists(), "child did not start");
        assert!(game_closed(&executable).is_err());
        let elsewhere = dir.path().join("other");
        std::fs::create_dir(&elsewhere).unwrap();
        let other = elsewhere.join("AceCombat8.exe");
        std::fs::write(&other, "different executable").unwrap();
        assert!(game_closed(&other.canonicalize().unwrap()).is_ok());
        drop(child);
        assert!(game_closed(&executable).is_ok());
    }
    #[test]
    fn same_installation_mutex_excludes_a_second_thread() {
        let first = InstallLock::acquire("test-mutex").unwrap();
        let second = std::thread::spawn(|| InstallLock::acquire("test-mutex").is_err())
            .join()
            .unwrap();
        assert!(second);
        drop(first);
        assert!(InstallLock::acquire("test-mutex").is_ok());
    }
}
