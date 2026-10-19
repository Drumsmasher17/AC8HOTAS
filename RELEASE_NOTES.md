# AC8HOTAS 0.3.0

Direct weapon selection is now configurable in `bindings.lua` alongside other
controls: `SelectMissiles`, `SelectSpecialWeapon1`, `SelectSpecialWeapon2`.
Each accepts the usual device GUID plus ButtonN or POV1 direction. F5 reloads
them, and nil unbinds them. Existing configs remain valid. The download defaults
all 52 controls to unbound. Close the game and replace the Scripts folder to
update; keep your bindings.lua and add the three new entries if wanted.

Direct selections use an internal nonexclusive DirectInput reader and call the
game's existing handlers on the game thread. They do not generate keyboard
events. No separate AC8WeaponSelect installation is needed. If you tested that
prototype, disable it before using these actions to prevent duplicate requests.

The user confirmed the integrated bindings work in flight on the VPC setup.
Config validation, reload/retention, native parsing, edge detection, packaging
and hardware acquisition also passed automated or local checks. Routine
per-selection diagnostic logging has been removed; startup, configuration and
error messages remain.

## Update and downloads

Keep your existing bindings.lua when updating from 0.2.0 or earlier. Replace
the Scripts folder from the 0.3.0 mod-only download, then restart the game.
F5 reloads bindings after edits; it does not reload updated scripts.

Recommended: AC8HOTAS-0.3.0-with-AC8AnalogYaw-0.1.3.zip. HOTAS-only and
fresh-install packages are also provided. The full-install package includes
the loader and is intended for installations without an existing UE4SS setup.
The bundled AC8AnalogYaw 0.1.3 files are unchanged.

The three new selections accept buttons and first-POV directions. Existing
native profile actions still cannot distinguish identical product IDs; direct
selection actions use device instance IDs. Physical POV hats and clean-folder
installation of the release bundles still need in-game testing.
