# Publishing checklist

The source folder is ready to become a GitHub repository. Commit this project
folder only, not the game installation. .gitignore excludes binaries, build
outputs, generated catalogs and logs. No GitHub repository or release was created
by the preparation scripts.

## Build

1. Run build.cmd with Visual Studio C++ Build Tools installed.
2. Run `python tests/test_mapper.py` with lupa installed (an optional argument
   adds a local lupa package directory).
3. Obtain AC8AnalogYaw 0.1.3 from its upstream release, then run:

   ```powershell
   .\package.ps1 -AnalogYawZip 'C:\Downloads\AC8AnalogYaw-0.1.3.zip'
   python tests/test_package.py --analog-yaw-zip 'C:\Downloads\AC8AnalogYaw-0.1.3.zip'
   ```

   In this workspace, package.ps1 defaults to the existing sibling release ZIP.

## GitHub

Publish the source repository, then create release tag `v0.1.0`. Use
RELEASE_NOTES.md for the description. Attach all three dist ZIPs and SHA256SUMS.txt.
The explicit source ZIP is optional for GitHub (which also generates source
archives) but useful for users downloading elsewhere.

## Nexus Mods

Use `AC8HOTAS-0.1.0-with-AC8AnalogYaw-0.1.3.zip` as the recommended/main download.
Offer `AC8HOTAS-0.1.0.zip` as the HOTAS-only alternative; users choose one.
Provide INSTALL.md instructions and list UE4SS as a required separate dependency.
Link the analog-yaw project for source and independent updates. Link your HOTAS
repository once published. The source ZIP can be an optional source download.

The archives are for manual installation into the active UE4SS Mods directory;
no automatic mod-manager installer is included. Do not imply automatic Vortex
deployment has been tested. Never ship a replacement global mods.txt or UE4SS
settings file. Explain preserving bindings.lua when upgrading.

Before publishing, perform a clean-folder in-game smoke test of the recommended
bundle. Packaging checks verify content, not both mods working together on every
aircraft or game build. The final polished bundle has not itself had that test.
