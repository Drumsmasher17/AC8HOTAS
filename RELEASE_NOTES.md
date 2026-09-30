# AC8HOTAS 0.1.0

Bind separate flight sticks, throttles and pedals through the game's native
flight-stick profiles, with no virtual combined joystick required for supported
devices. Includes a grouped, fully unbound config, detected-device reference,
axis inversion, button/hat inputs and F5 config reload.

Proportional throttle, full braking, pitch/roll, camera axes and button mappings
have been exercised in game. Normal gameplay no longer repeatedly scans the
global object list or performs diagnostic logging; the user confirmed the
cleanup runs well. POV support is implemented but not physically tested here.

**Recommended download:** AC8HOTAS-0.1.0-with-AC8AnalogYaw-0.1.3.zip.
This includes two independently enabled mods. AC8AnalogYaw supplies proportional
yaw; AC8HOTAS supplies controller bindings. The standalone HOTAS download is
also available. UE4SS is not included.

Read INSTALL.md before extracting. All default bindings are unassigned.
Back up and retain your bindings.lua when updating. Newly assigned devices can
still need a USB reconnect; duplicate identical-product devices are unsupported.
Game updates may break compatibility, particularly the native analog-yaw helper.

The bundle contains the existing AC8AnalogYaw 0.1.3 release files unchanged.
Its upstream source is https://github.com/Drumsmasher17/AC8AnalogueYaw.
