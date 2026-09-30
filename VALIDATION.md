# Release validation — 0.1.0

## In-game evidence

The local Windows installation has been tested with multiple VPC devices.
The user confirmed pitch, roll, camera axes, yaw input routing, button bindings,
F5 config refresh and UE4SS Lua reload. Continuous throttle reaches full braking
and responds proportionally at intermediate positions. Earlier diagnostic logs
also recorded fractional brake/acceleration values at both input and aircraft
stages. No extra throttle hook is required.

After removing recurring diagnostic output and repeated global object scans,
the user reported that the mod was working very well. No formal frame-time
benchmark was performed.

AC8AnalogYaw 0.1.3 was tested separately during its development. Packaging copies
its release files unchanged. The final combined distribution still needs a fresh
installation smoke test before public release.

## Automated coverage

The Lua mock suite covers multiple devices, axis conversions and inversion,
one-based button/slider selection, POV direction mapping, unused-slot
neutralization, malformed/missing/duplicate-device rejection, unavailable slots,
removal and re-addition, write failures, config reload, subsystem replacement,
and ownership across Lua reload. Old callbacks become inert. A stable 900-callback
run performs no additional global object searches.

Package checks verify all 49 default actions are unbound, no personal GUIDs or
runtime catalogs/logs are shipped, source archives contain no binaries, bundled
yaw files match the upstream archive byte for byte, and published checksums match.

Mocks establish script behaviour, not the behaviour of every device or action
inside the game. Physical POV hats and all optional game actions have not been
verified. Reconnect-free assignment of new devices remains unresolved.
