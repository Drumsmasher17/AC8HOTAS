# AC8HOTAS 0.3.2 — Controller profile allocation fix

## Changes

- Fix configuration reloads being rejected when one controller takes the only
  native game profile that supports another controller's bindings.
- Allocate compatible profiles across all configured devices, moving earlier
  assignments when necessary. Device GUID order no longer determines whether
  an otherwise compatible configuration is accepted.
- Preserve previous and native-profile assignments where possible. Configurations
  that exceed available profile capacity are still rejected before mappings change.
- Add a regression test for a flexible stick preceding a throttle that requires
  two Weapon binding slots, including existing profile ownership.

Firmware updates or device re-registration can change GUIDs and expose this
issue. Affected users may see "No compatible flight-stick profile slot" and
"profile already allocated" despite their controllers working in Windows.
This update fixes profile allocation; changed GUIDs must still be updated in
bindings.lua using the detected device list.

## Updating

Close the game and replace the Scripts folder from the mod-only download.
Keep your existing bindings.lua. Restart the game to load the updated scripts;
F5 reloads bindings only. The bundled AC8AnalogYaw 0.1.3 files are unchanged.

## Validation

The regression reproduces the rejection with the previous allocator and passes
with the fix. Automated mapper and weapon-selection checks pass. In-game and
clean-install validation of the 0.3.2 release archives remains outstanding.
