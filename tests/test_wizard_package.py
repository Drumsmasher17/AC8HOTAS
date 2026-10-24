"""Portable wizard archive boundaries, licensing and checksum checks."""
import hashlib
import zipfile
from pathlib import Path

root = Path(__file__).resolve().parents[1]
dist = root / "dist"
binary = dist / "AC8HOTAS-Setup-0.1.0.zip"
source = dist / "AC8HOTAS-Setup-0.1.0-source.zip"
with zipfile.ZipFile(binary) as z:
    assert z.testzip() is None
    names = z.namelist()
    assert len(names) == len(set(names))
    assert names == ["AC8HOTAS-Setup.exe"], "Portable ZIP must contain only the executable"
    executable = z.read("AC8HOTAS-Setup.exe")
    assert executable == (root / "wizard/target/release/AC8HOTAS-Setup.exe").read_bytes()
    assert executable == (dist / "AC8HOTAS-Setup.exe").read_bytes()
    notices = (root / "wizard/notices.html").read_bytes()
    assert notices in executable, "Complete notices must be embedded, not a runtime sidecar"
    lock_hash = hashlib.sha256((root / "wizard/Cargo.lock").read_bytes()).hexdigest()
    assert f'<meta name="cargo-lock-sha256" content="{lock_hash}">'.encode() in notices
    assert b"LicenseRef-Slint-Royalty-free-2.0.md" in notices
    assert b"COPYRIGHT-library.html" in notices
with zipfile.ZipFile(source) as z:
    assert z.testzip() is None
    names = z.namelist()
    assert len(names) == len(set(names))
    for name in ("wizard/Cargo.lock", "wizard/README.md", "generate-wizard-notices.ps1", "wizard/.cargo/config.toml", "wizard/ui/app.slint", "wizard/src/input_windows.rs", "mod/bindings.lua", "mod/Scripts/actions.lua", "tests/test_wizard_mapper.py"):
        assert "AC8HOTAS/" + name in names, name
    assert "AC8HOTAS/wizard/notices.html" not in names, "Notices are generated during compilation"
    for image in ("setup.png", "controls.png", "unsaved-changes.png"):
        assert "AC8HOTAS/wizard/screenshots/" + image in names
    assert not any("/target/" in n or "/prototype/" in n for n in names)
    assert not any(n.endswith((".exe", ".dll", ".log", ".bak", "wizard-state.json", "local-validation.json")) for n in names)
for line in (dist / "WIZARD-SHA256SUMS.txt").read_text().splitlines():
    digest, name = line.split("  ", 1)
    assert hashlib.sha256((dist / name).read_bytes()).hexdigest().upper() == digest
print("PASS: single-file wizard ZIP, embedded notices, source boundaries and SHA-256 hashes")
