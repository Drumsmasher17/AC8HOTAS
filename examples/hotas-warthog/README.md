# Thrustmaster HOTAS Warthog example

A community-contributed `bindings.lua` for the Thrustmaster HOTAS Warthog
joystick and dual throttle. It is a copy of one user's working configuration,
with that user's device GUIDs replaced by placeholders.

**This example will not work until you replace both GUID placeholders.**

## Setup

1. Install AC8HOTAS as described in [INSTALL.md](../../INSTALL.md). For
   proportional yaw, also install AC8AnalogYaw (included in the recommended
   bundle). Without it, Yaw keeps the game's existing, non-proportional
   behaviour.
2. Connect both Warthog devices and launch the game once. AC8HOTAS appends a
   generated `DETECTED DEVICES` section to your installed
   `Mods/AC8HOTAS/bindings.lua`.
3. In that generated section, find the two Warthog entries and copy the
   `device = "{...}"` value from each:

   ```lua
   -- Joystick - HOTAS Warthog | 0x0402044F
   -- device = "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}"   <- stick
   ...
   -- Throttle - HOTAS Warthog | 0x0404044F
   -- device = "{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}"   <- throttle
   ```

   These GUIDs are specific to your computer, so the ones in someone
   else's file will not work on yours.
4. Back up your installed `bindings.lua`. Replace everything above its
   `-- BEGIN DETECTED DEVICES` line with this example's `bindings.lua`.
5. Paste your GUIDs into the two placeholders at the top, including the braces:

   ```lua
   local stick = "{PASTE-WARTHOG-JOYSTICK-GUID-HERE}"
   local throttle = "{PASTE-WARTHOG-THROTTLE-GUID-HERE}"
   ```

6. Save, return to the game and press **F5**. If a placeholder is left in, the
   newest `HOTAS-*.log` reports `Device not connected` and no bindings change.
7. If the Warthog does not respond after F5, unplug and reconnect **both** USB
   devices, then press **F5** again. The contributor needed this step the first
   time the devices were assigned.

## Control mapping

Button numbers are the DirectInput numbers Windows reports when the stick and
throttle appear as two separate devices, as listed in the generated section.
"Not individually confirmed" means the binding is part of the contributor's
configuration, but its in-game effect was not specifically recorded.

### Joystick (`stick`)

| Physical control | Input token | Game action | Status |
| --- | --- | --- | --- |
| Stick forward/back | `Y` (`invert = true`) | Pitch | Not individually confirmed |
| Stick left/right | `X` | Roll | Not individually confirmed |
| Trigger, first stage | `Button1` | Gun | Not individually confirmed |
| Red thumb button | `Button2` | Missile | Not individually confirmed |
| Pinkie pushbutton | `Button3` | View | Not individually confirmed |
| Pinkie paddle lever | `Button4` | AccelDecel | **Confirmed:** activates the high-G maneuver |
| Index-finger side button | `Button5` | Weapon | Not individually confirmed |
| TMS hat up | `Button7` | Target, FaceButtonTop (Y) | Not individually confirmed |
| TMS hat right | `Button8` | FaceButtonRight (B) | Not individually confirmed |
| TMS hat down | `Button9` | Radar, FaceButtonBottom (A) | **Confirmed:** A/confirm in menus. Radar not individually confirmed |
| TMS hat left | `Button10` | FaceButtonLeft (X) | Not individually confirmed |
| DMS hat right | `Button12` | Pause, SpecialRight | Not individually confirmed. See notes |
| CMS hat up | `Button15` | AutoPilot | **Confirmed** |
| CMS hat right | `Button16` | RightShoulder (RB) | **Confirmed** |
| CMS hat down | `Button17` | Flare | Not individually confirmed |
| CMS hat left | `Button18` | LeftShoulder (LB) | **Confirmed** |
| POV hat up/right/down/left | `POV1_Up` / `POV1_Right` / `POV1_Down` / `POV1_Left` | DPadUp / DPadRight / DPadDown / DPadLeft | **Confirmed** |

### Throttle (`throttle`)

| Physical control | Input token | Game action | Status |
| --- | --- | --- | --- |
| Right throttle lever | `Z` (`invert = true`, `mode = "zero_to_one"`) | Throttle | Not individually confirmed |
| Slew control (ministick) left/right | `X` | Yaw | Not individually confirmed. Proportional yaw requires AC8AnalogYaw |

All other actions are left `nil` (unbound). Both Warthog devices were confirmed
to work with AC8HOTAS.

## Notes from testing

- Some buttons intentionally drive two actions: `Button7` (Target and
  FaceButtonTop), `Button9` (Radar and FaceButtonBottom) and `Button12` (Pause
  and SpecialRight). The game decides which action applies in each context.
- **Cutscene skip:** holding a button bound to Pause did **not** skip
  cutscenes. SpecialRight is assigned to the same button as a suggested
  hold-to-skip input, but it has not been confirmed to work.
- **High-G:** binding LeftTrigger and RightTrigger to the same button did
  **not** activate the high-G maneuver. Use AccelDecel, as this example does.
- **HatUp/HatRight/HatDown/HatLeft** did not provide camera look-around when
  tried, so they are left unbound. The stick's POV hat is assigned to the DPad
  actions instead, which the contributor prefers.
- CameraPitch and CameraYaw are unbound. They require axes, so AC8HOTAS
  rejects `POV1_*` hat directions for them. Using the stick's POV hat for
  in-game look-around would need a separate tool that turns the hat into a
  virtual axis. That has not been tried with this example.

## Reversing an axis

If an axis moves the wrong way on your setup, flip its `invert` value
(`true` to `false`, or `false` to `true`), save and press **F5**. For example:

```lua
Pitch = {
    device = stick,
    input = "Y",
    invert = false, -- was true
    mode = "minus_one_to_one"
},
```

Leave the Throttle `mode` as `zero_to_one`. Buttons and hats do not use `invert`.

## Testing scope

Items marked **Confirmed** were tested in game by the contributor on Windows,
with the stick and throttle detected as `Joystick - HOTAS Warthog` and
`Throttle - HOTAS Warthog`. A combined Thrustmaster TARGET virtual device, other
game builds and other firmware versions have not been tested with this example.
