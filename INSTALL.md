# Install AC8HOTAS

## Choose a download

- **Recommended bundle:** AC8HOTAS 0.1.0 + AC8AnalogYaw 0.1.3. Device bindings
  plus proportional yaw. The two mods remain separate and can be disabled independently.
- **HOTAS only:** use this if you already have AC8AnalogYaw, or only want bindings.

UE4SS is required and is **not included**. This release was tested with the
game's working UE4SS v3.0.1 Beta installation (commit 03dbd5c0), on Windows x64.
Its Lua API must support `package.loadlib`, `LoopInGameThreadAfterFrames` and
ModRef shared variables. Other UE4SS builds have not been verified.

## Installation

1. Close the game. Locate your **active UE4SS Mods folder**. A common layout is
   `Game/Binaries/Win64/UE4SS/Mods`, but use the folder your UE4SS installation loads.
2. Copy the mod folders from the ZIP into that Mods folder. The required layout is:

   ```text
   Mods/
     AC8HOTAS/
       bindings.lua
       Scripts/main.lua
       Scripts/ac8_hotas_devices_001.dll
     AC8AnalogYaw/             (recommended bundle only)
       Scripts/main.lua
       Scripts/ac8_analog_yaw_013.dll
   ```

3. Add or update these lines in the existing `Mods/mods.txt`:

   ```text
   AC8HOTAS : 1
   AC8AnalogYaw : 1
   ```

   Omit the second line for HOTAS-only installations without analog yaw. If your
   loader/mod manager also maintains `mods.json`, enable the same mods there.
   **Do not replace your entire mod list.** Disable old AC8 input-probe/test mods
   which edit the same profiles. Unrelated mods need not be disabled.
4. Connect your controllers and launch the game. Use expert flight controls for
   independent pitch, roll and yaw.
5. Open `AC8HOTAS/bindings.lua` in a text editor. The first launch appends a list
   of detected devices and their input tokens. No bindings are assigned by default.
6. Replace the desired `nil` entries with bindings, save, return to the game and
   press **F5**. You do not need to restart just to edit the config.

Example (replace the placeholder with a detected device GUID):

```lua
Roll = { device = "{detected GUID}", input = "X", invert = false },
Pitch = { device = "{detected GUID}", input = "Y", invert = true },
Throttle = { device = "{detected GUID}", input = "Rx", invert = false },
Gun = { device = "{detected GUID}", input = "Button1" },
```

Use the axes actually listed for your device; Rx is only an example. Copy the
whole binding value, including braces, and retain the comma after each line.
Button numbers start at 1. A hardware hat may appear as buttons instead of a POV.
Menu navigation has its own grouped entries; assign those if you want HOTAS menu control.

## Axis options

`invert = true` reverses an axis. It is optional and defaults to false.
`mode = "minus_one_to_one"` uses -1 / 0 / +1; `mode = "zero_to_one"` converts to
0 / 0.5 / 1. Modes are optional: Throttle defaults to zero_to_one, other axes to
minus_one_to_one. The continuous lever goes on **Throttle**, not the button action
AccelDecel. Buttons and hats do not use inversion or mode.

Pitch/Roll/Yaw/Throttle/CameraPitch/CameraYaw accept X, Y, Z, Rx, Ry, Rz, Slider1
or Slider2 when exposed by the device. Other actions accept Button1..Button128
or POV1_Up/Down/Left/Right. Only the first POV is supported by this route.

## Known limitations and troubleshooting

- **New device does nothing:** press F5 first; if it remains inactive, reconnect
  the controller and try again. F5 does not yet rebuild all game-side device caches.
- **Unsupported profile slot:** the mod rejects the config before applying it.
  Some game profiles cannot accommodate every listed action. A restart with the
  completed config can allow a different profile to be selected; it is not guaranteed.
- **Duplicate identical models:** separate devices with the same product ID are
  currently rejected. The game chooses profiles by product ID.
- **Wrong direction:** change the axis's `invert` value and press F5.
- **Config error:** check the newest `AC8HOTAS/HOTAS-*.log`. Failed validation
  retains the previous mapping. A failed native property write requires a restart.
- **Yaw still feels digital:** check that AC8AnalogYaw is enabled and inspect its
  logs. It applies automatically; there is no activation key. It may refuse an
  incompatible game build. HOTAS bindings do not themselves change yaw physics.
- **Game updates:** compatibility is not guaranteed. Analog yaw uses native code
  matching, not a fixed version-number lock; patches can still break it or crash
  the game. Disable that mod and restart if it misbehaves. Only offline testing
  on the current local game build has been performed.

## Updating or removing

Back up your **bindings.lua before updating**. Extract an update to a temporary
folder first, copy the new Scripts/files, and retain your existing bindings.lua.
Use the new template to add any new options manually. Back up other custom files
before replacing an existing AC8AnalogYaw installation as well.

To disable a mod, set its mod-list entry to 0 (and disable it in your mod manager,
if used), then restart. To uninstall, close the game and remove only that mod's
folder. Do not delete the shared UE4SS loader or other mods.

F5 reloads bindings. Lua developers can optionally configure UE4SS's native hot
reload shortcut; our development setup uses Ctrl+F9, but the download does not
change your UE4SS settings. Native DLL updates require restarting the game.
