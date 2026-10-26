# AC8HOTAS 0.3.1

## Changes

- Bind multiple buttons or hat directions to the same action using a list in
  `bindings.lua`. Bindings can use the same device or different devices.
- Keep existing single-binding configurations unchanged.
- Add a copyable multiple-binding example and explain braces, commas and why
  repeating an action name overwrites its earlier binding in Lua.
- Add troubleshooting guidance for malformed `bindings.lua` files.
- Check that a native game profile has enough slots for every requested binding
  before applying any mappings. Removed bindings are cleared on reload.

## Example

```lua
Target = {
    { device = "{detected GUID}", input = "Button1" },
    { device = "{detected GUID}", input = "Button2" },
},
```

Replace Target with the desired action and use your detected device GUID(s).
Add more bindings by copying a line inside the list. Native game profile slots
limit the number of bindings; not every action supports the same number.
An unsupported configuration is rejected as a whole. A rejected startup config
can leave HOTAS controls unavailable until a valid config is applied.

## Updating

Close the game, replace the Scripts folder, then restart. Keep your existing
`bindings.lua`; copy the example above when adding multiple bindings. F5 reloads
configuration edits during play, but does not reload updated scripts.
The bundled AC8AnalogYaw 0.1.3 files are unchanged.

Downloads:
- `AC8HOTAS-0.3.1-with-AC8AnalogYaw-0.1.3.zip`: recommended mod-only bundle.
- `AC8HOTAS-0.3.1.zip`: HOTAS only; requires UE4SS separately.
- `AC8HOTAS-0.3.1-full-install.zip`: includes the loader for fresh installations.
- `AC8HOTAS-0.3.1-source.zip`: source and build instructions.

## Validation

Two Weapon bindings were confirmed working in game on the VPC throttle.
Automated checks cover configuration validation, multiple native slots,
capacity rejection before mutation, clearing removed bindings, reload/lifecycle
behaviour, and direct weapon selection. Release archive checks verify layout,
default unbound controls, dependency integrity and checksums. Fresh-install
smoke testing of the packaged downloads remains outstanding.
