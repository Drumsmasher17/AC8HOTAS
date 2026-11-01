# AC8HOTAS 0.3.3 — Composite controller support

## Changes

- Support controllers that expose several DirectInput collections under one
  product ID, such as a MOZA AB6 base with an MTQ panel attached to it.
  AC8HOTAS now gives every collection after the first a deterministic,
  process-local product ID before the game enumerates devices, so each
  collection gets its own flight-stick profile and can be bound separately.
- Keep each collection's real DirectInput instance GUID, so input still comes
  from the physical device. Nothing is written to the driver, registry, vJoy or
  system-wide controller configuration; other games are unaffected.
- Record each split in `COMPOSITE.log` in the mod folder.
- The device catalog now includes each collection's HID path. Use the matching
  Binder release, which matches the game list to physical collections by that
  path instead of by product ID.
- Binder: match HID collections by device path before product ID, and label
  collections `[COL1]`, `[COL2]`, etc. when two devices share a name.

## Updating

Close the game and replace the `Scripts` folder from the mod-only download,
including the new `ac8_hotas_composite_001.dll`. Keep your existing
`bindings.lua`; the game rewrites its detected-device section on launch. Restart
the game to load the updated scripts. Bindings that pointed at one collection
may need to be re-bound once through the updated Binder because the second
collection now has its own product ID.

## Validation

The splitter and scanner were tested together against a MOZA AB6 with two
enumerated collections: the second collection received product ID `0x1003346E`
while its instance GUID and HID path stayed unchanged, and both catalog entries
resolved to their own physical collection. Binder self-test passes with both
collections matched by path. Confirmed in game: the MOZA AB6 base and the MTQ
panel were bound separately and both worked.

---

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
