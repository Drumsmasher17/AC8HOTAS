# Release validation — 0.2.0

## In-game evidence

The user tested AC8HOTAS with three VPC devices: a throttle, stick and rudder
pedals. Startup/F5 assigned each device to a compatible built-in flight-stick
profile. After F5, the user reported that all configured controls continued to
work.

An on-demand F7 profile dump showed 18 profile entries. The three configured
Windows devices were assigned to `HR_FlightStick_PC`, `TM_FlightHotasNEO` and
`HR_FlightStick_PS5`; every other Windows flight-stick profile was unassigned
and had neutral actions. The `Default` profile kept its zero-ID fallback but had
neutral actions. PS5 and Xbox-specific profile IDs were preserved. The dump is
available in the user's game installation as `ProfileDump-002.txt`.

Earlier in-game testing covered proportional throttle and full braking,
pitch/roll, camera axes, yaw input routing, inversion, buttons, F5 config reload
and UE4SS Lua reload. Physical POV hats and some optional actions remain
untested. No formal frame-time benchmark was performed; periodic input logging
and tracing are absent.

AC8AnalogYaw 0.1.3 was tested separately during development. Packaging copies
its release files unchanged. A clean-folder installation smoke test of the
combined 0.2.0 package and a fresh full-install smoke test remain to be done
before publication.

## Automated coverage

The Lua mock suite covers multiple devices, native-template fallback when an
action slot is missing, session profile clearing, axis conversions and
inversion, buttons/sliders, POV directions, neutral unassigned actions,
malformed/missing/duplicate-device rejection, write failures, config reload,
subsystem replacement and ownership across Lua reload. Package checks verify
all 49 default actions are unbound, required scripts are present, no personal
GUIDs/runtime catalogs/logs are shipped, the analog-yaw bundle matches its
upstream release, and checksums match.

Mocks establish script behavior, not every action or device in the game. The
Lua mock suite could not be run in the current environment because Python's
`lupa` dependency is unavailable. Package/archive checks can be run separately.
