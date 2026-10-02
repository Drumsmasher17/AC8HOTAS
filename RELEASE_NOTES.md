# AC8HOTAS 0.2.0

## What's changed

AC8HOTAS now takes ownership of the game's Windows flight-stick profiles for the
current session. On startup, and whenever you press F5, it clears the built-in
flight-stick assignments and actions in memory, then assigns clean compatible
profile slots to the devices and actions listed in `bindings.lua`. The change is
not saved by the game; removing or disabling AC8HOTAS and restarting restores
the game's original profiles.

The mapper can borrow a compatible built-in profile slot when a device's own
template lacks an action, such as Missile or Platform on some throttles. If none
of the available slots exposes a requested action, the config is rejected before
the profile table is changed and the log lists the checked slots.

F5 remains the only user action needed to refresh device discovery and bindings.
The profile reset is automatic at startup and is not bound to a separate key.
The optional F7 profile dump is read-only and runs only when requested. No
per-frame or periodic diagnostic logging is enabled.

## Updating from 0.1.1

Close the game, replace AC8HOTAS's Scripts folder with the one from the 0.2.0
mod-only download, and keep your existing `bindings.lua`. Start the game again;
F5 reloads the config but does not reload changed Lua code. Existing UE4SS users
should not reinstall the full loader bundle. If something fails, include the
newest `AC8HOTAS/HOTAS-*.log` in a bug report.

## About the mod

Bind separate flight sticks, throttles and pedals through the game's native
flight-stick input path, with no combined virtual joystick required for
supported devices. The download includes a grouped, unbound config, generated
device capabilities, axis inversion, button/hat inputs and F5 config reload.

The user confirmed that pitch, roll, yaw, throttle, camera axes and button
bindings continued to work after all other Windows flight-stick profile actions
were neutralized on a setup with three VPC devices. F7's profile dump showed
only those configured profiles active. Physical POV hats and some optional
actions remain untested.

**Recommended download:** `AC8HOTAS-0.2.0-with-AC8AnalogYaw-0.1.3.zip`. It
contains two separately enabled mods. AC8AnalogYaw supplies proportional yaw;
AC8HOTAS supplies device bindings. A HOTAS-only download is also available.
Both mod-only downloads require UE4SS separately.

**Fresh installation without UE4SS:** choose
`AC8HOTAS-0.2.0-full-install.zip`. It includes the tested loader and both mods
enabled. Follow `INSTALL-FULL.md`; do not overwrite an existing loader setup.

All default bindings are unassigned. Back up and keep `bindings.lua` when
updating. Duplicate devices with the same product ID cannot currently be
distinguished. Game updates may break compatibility, especially for the native
analog-yaw helper.

The bundle copies the existing AC8AnalogYaw 0.1.3 files unchanged. Its source is
https://github.com/Drumsmasher17/AC8AnalogueYaw.
