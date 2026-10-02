# AC8HOTAS 0.1.1

## What's changed

Profile-mapping errors now name every missing action on the rejected device,
the controller name and product ID, and the selected game profile. This replaces
the unhelpful "Existing device profile lacks requested action" message and also
covers previously assigned profiles after editing bindings.

This is a diagnostic update, not a fix for missing profile slots. Mapping
behaviour is unchanged, and rejected configurations still make no profile edits.
No periodic logging or input tracing has been added. Analog Yaw remains 0.1.3.

## Updating from 0.1.0

Close the game, replace AC8HOTAS's Scripts folder from a mod-only download,
and keep your existing bindings.lua. Restart the game; F5 alone cannot load
updated Lua code. If a mapping is rejected, send the full error from the newest
AC8HOTAS/HOTAS-*.log together with your bindings and generated device list.
Existing UE4SS users should not reinstall the full loader bundle.

## About the mod

Bind separate flight sticks, throttles and pedals through the game's native
flight-stick profiles, with no virtual combined joystick required for supported
devices. Includes a grouped, fully unbound config, detected-device reference,
axis inversion, button/hat inputs and F5 config reload.

Proportional throttle, full braking, pitch/roll, camera axes and button mappings
have been exercised in game. Normal gameplay no longer repeatedly scans the
global object list or performs diagnostic logging; the user confirmed the
cleanup runs well. POV support is implemented but not physically tested here.

**Recommended download:** AC8HOTAS-0.1.1-with-AC8AnalogYaw-0.1.3.zip.
This includes two independently enabled mods. AC8AnalogYaw supplies proportional
yaw; AC8HOTAS supplies controller bindings. The standalone HOTAS download is
also available. These mod-only downloads require UE4SS separately.

**Fresh installation without UE4SS:** choose AC8HOTAS-0.1.1-full-install.zip.
It includes the tested loader and both mods enabled. Follow INSTALL-FULL.md and
copy its Game folder into the game installation. Do not overwrite an existing
loader, settings or mod list; existing users should use a mod-only download.

Read INSTALL.md before extracting. All default bindings are unassigned.
Back up and retain your bindings.lua when updating. Newly assigned devices can
still need a USB reconnect; duplicate identical-product devices are unsupported.
Game updates may break compatibility, particularly the native analog-yaw helper.

The bundle contains the existing AC8AnalogYaw 0.1.3 release files unchanged.
Its upstream source is https://github.com/Drumsmasher17/AC8AnalogueYaw.
