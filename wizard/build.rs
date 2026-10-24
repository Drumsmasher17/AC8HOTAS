fn main() {
    println!("cargo:rerun-if-changed=../generate-wizard-notices.ps1");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=notices.html");
    let manifest = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let mut generator = std::process::Command::new("powershell.exe");
    // A PowerShell 7 parent may supply module paths incompatible with the
    // built-in Windows PowerShell used for generation.
    generator.env_remove("PSModulePath");
    generator
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(manifest.join("../generate-wizard-notices.ps1"));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        generator.creation_flags(0x08000000);
    }
    let status = generator.status().expect("run embedded-notice generator");
    assert!(status.success(), "embedded-notice generation failed");
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-dark".into())
        .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles);
    slint_build::compile_with_config("ui/app.slint", config).expect("compile wizard UI");
}
