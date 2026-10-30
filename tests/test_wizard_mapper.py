"""Validate actual Rust serializer output using the mod's Lua mapper.

Generate the fixture with the diagnostics-only executable --emit-fixture
--diagnose=target/wizard-parity.json. This test never touches installed config.
"""
import json
import sys
from pathlib import Path

if len(sys.argv) > 2:
    sys.path.insert(0, sys.argv[2])
from lupa import LuaRuntime

root = Path(__file__).resolve().parents[1]
fixture = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
lua = LuaRuntime(unpack_returned_tuples=True)
lua.globals().fixture = lua.table_from(fixture, recursive=True)
lua.globals().mapper = lua.execute("return assert(loadfile(...))()", str(root / "mod/Scripts/mapper.lua"))
lua.execute("""
local config = assert(load(fixture.config, 'Rust serializer', 't', {}))()
local inputs = {{token='POV1'}}
for _, token in ipairs({'X','Y','Z','Rx','Ry','Rz','Slider1','Slider2'}) do
  inputs[#inputs+1] = {token=token}
end
for i=1,128 do inputs[#inputs+1] = {token='Button'..i} end
local catalog = {ok=true, devices={{id=fixture.device, product='0x11110001', name='Synthetic', inputs=inputs}}}
local grouped, products, selections = mapper.validate(config, catalog)
assert(products['0x11110001']==1)
local native, direct = 0, 0
for _, expected in ipairs(fixture.expected) do
  local binding = assert(config.bindings[expected.key])
  assert(binding.input==expected.input)
  if mapper.selectionActions[expected.key] then
    direct = direct + 1
    local found
    for _, selection in ipairs(selections) do
      if selection.action==expected.key then found=selection end
    end
    assert(found and found.device==fixture.device)
  else
    native = native + 1
    local actual = assert(grouped[fixture.device].bindings[mapper.actions[expected.key]])
    assert(actual.invert==expected.invert)
    if expected.axis then assert(actual.option==expected.option) end
  end
end
assert(native==49 and direct==3 and #selections==3)
""")
print("PASS: Rust-emitted bindings accepted for all 52 actions; native axis options and direct selection routing match Lua mapper")
