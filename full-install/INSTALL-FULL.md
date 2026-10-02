# AC8HOTAS full installation

For a fresh installation with no UE4SS loader already installed. Includes
AC8HOTAS 0.2.0, AC8AnalogYaw 0.1.3 and the locally tested UE4SS loader build.
Both mods are enabled. All controller bindings start unassigned.

1. Close the game. In Steam, open the game's installed files using Browse.
2. Check `Game/Binaries/Win64`. If it already contains UE4SS, ue4ss.dll,
   dwmapi.dll, an override.txt or another mod loader, use the mod-only download
   instead. Do not overwrite an existing loader, settings or mod list.
3. Copy the ZIP's **Game** folder into the **ACE COMBAT 8** installation folder.
   This merges with the existing Game folder. The loader must end up beside
   `Game/Binaries/Win64/AceCombat8.exe`, not beside a top-level launcher.
4. Connect your controllers and launch the game. Choose expert flight controls.
5. Open `Game/Binaries/Win64/UE4SS/Mods/AC8HOTAS/bindings.lua`.
   The detected-device list is generated at the bottom during startup.
6. Replace the desired `nil` entries using the listed device bindings, save,
   and press **F5** in game. No restart is required for config edits.

See `Game/Binaries/Win64/UE4SS/Mods/AC8HOTAS/INSTALL.md` for binding examples
and troubleshooting. At startup and each F5 refresh, AC8HOTAS clears Windows
flight-stick profile bindings in memory and applies the compatible assignments
from `bindings.lua`; no separate reset key is needed. A newly connected device
may still require a USB reconnect if the game has cached its device discovery.
Analog yaw applies automatically; there is no activation key.

The full package disables UE4SS developer hot reload, automatic script reload
and debug consoles. F5 binding reload remains available. It contains no FOV mod,
test mods, user bindings, logs, game binaries or launch-option changes.

For updates, use a mod-only download and preserve your bindings.lua and UE4SS
configuration. This full package includes a fresh mods.txt and mods.json for
initial setup only. To disable either mod, set its mods.txt entry to 0 and its
mods.json mod_enabled field to false, then restart.

The binaries have worked in the development installation. This assembled full
package still needs a fresh-install game test. Compatibility after game updates
is not guaranteed. No changes are made to Steam launch options or other launch
requirements outside the copied files.

License details and binary fingerprints are in LOADER-PROVENANCE.md and the
included UE4SS/LICENSE. AC8HOTAS's CC0 dedication does not replace UE4SS's MIT
license or the bundled MinHook license.
