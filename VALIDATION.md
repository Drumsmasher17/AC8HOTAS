# Release validation â€” 0.2.0

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
combined 0.3.1 package and a fresh full-install smoke test remain to be done
before publication.

## Automated coverage

The Lua mock suite covers multiple devices, native-template fallback when an
action slot is missing, session profile clearing, axis conversions and
inversion, buttons/sliders, POV directions, neutral unassigned actions,
malformed/missing/duplicate-device rejection, write failures, config reload,
subsystem replacement and ownership across Lua reload. Package checks verify
all 52 default actions are unbound, required scripts are present, no personal
GUIDs/runtime catalogs/logs are shipped, the analog-yaw bundle matches its
upstream release, and checksums match.

The Lua mock suites, real DLL bridge checks and package/archive checks passed
from the Git checkout. Mocks establish script behavior, not every action or
device in the game.

## Direct selection validation

Shared-config selection validation, arbitrary device instances, mixed native
and direct actions, button/hat bounds, F5 staging/rejection/removal, and mocked
weapon dispatch pass. Native config parsing, rising edges, held-button reset
and real device acquisition from the user's configured left-stick buttons
14/15/16 pass. Release packages include the helper and all 52 default actions
unbound; source, mod-only, bundled yaw and fresh-install archive checks pass.
The user confirmed both the earlier keyboard prototype and the integrated
HOTAS bindings work in flight on the VPC setup. Routine per-selection logging
was then removed; startup/configuration and error messages remain.

## 0.3.1 multiple bindings

Two Weapon bindings on the VPC throttle were confirmed working in game.
Mock regressions exercise distinct native slot writes, capacity rejection before
mutation, and clearing removed secondary bindings. Digital camera experiments
were removed: compatible profiles expose only one slot per camera axis.
Release preparation on 26 October 2026 passed test_mapper.py, test_weapons.py,
build/test-weapons.exe, test_package.py and test_full_package.py, plus git diff
--check. All four 0.3.1 archives and SHA256SUMS.txt were generated. Fresh-install
in-game smoke testing of these archives remains outstanding.
