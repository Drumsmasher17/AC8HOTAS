# Publishing checklist

The release source is under `mod-projects/AC8HOTAS`. Publish that project only,
not the game installation. `.gitignore` excludes build outputs and generated
catalogs/logs. Creating packages does not publish or upload anything.

## Build and check

1. Run `build.cmd` with Visual Studio C++ Build Tools installed if the device
   scanner DLL needs rebuilding.
2. Run `python tests/test_mapper.py` with `lupa` installed (an optional argument
   adds a local `lupa` package directory).
3. Use AC8AnalogYaw 0.1.3 from its upstream release, then run:

   ```powershell
   .\package.ps1 -AnalogYawZip 'C:\Downloads\AC8AnalogYaw-0.1.3.zip'
   python tests/test_package.py --analog-yaw-zip 'C:\Downloads\AC8AnalogYaw-0.1.3.zip'
   ```

   The package script defaults to the existing sibling release ZIP when
   available. It creates the HOTAS-only, combined, and source ZIPs.
4. To create and check the optional full-install package:

   ```powershell
   .\package-full.ps1 -GameBinDir '<game>\Game\Binaries\Win64'
   python tests/test_full_package.py
   ```

   The full-install script verifies pinned loader binaries without changing the
   game installation. Its archive still needs a clean-install game smoke test.

## GitHub

Commit the reviewed source and create release tag `v0.2.0`. Use
`RELEASE_NOTES.md` for the release description. Attach the three mod-only ZIPs
and `SHA256SUMS.txt`. The explicit source ZIP is optional but useful for sites
that do not generate source archives.

## Nexus Mods

Use `AC8HOTAS-0.2.0-with-AC8AnalogYaw-0.1.3.zip` as the recommended/main
download. Offer `AC8HOTAS-0.2.0.zip` as the HOTAS-only alternative. Provide
`INSTALL.md` and list UE4SS as a requirement (included only in the full-install
download). Link the analog-yaw project for its source and separate updates.
The full-install ZIP is an option for a fresh installation only; users with an
existing UE4SS setup should use a mod-only package.

The mod-only archives install into an active UE4SS Mods directory. They do not
include an automatic mod-manager installer and never replace global `mods.txt`
or UE4SS settings. Tell users to keep their existing `bindings.lua` when
updating.

Before publishing, perform a clean-folder in-game smoke test of the recommended
bundle and a fresh full-install smoke test if offering that package. Archive
checks verify layout and dependencies, not game behavior on every device or
game build.
