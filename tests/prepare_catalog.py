"""Developer helper: run discovery/config validation before game launch.

Uses the actual Lua entry point and native enumerator, with no Unreal objects.
The supplied installation must have its DLL, scripts and bindings.lua in place.
"""
import sys
from pathlib import Path
if len(sys.argv)>2:
    sys.path.insert(0,sys.argv[2])
from lupa import LuaRuntime
root=Path(sys.argv[1]).resolve()
lua=LuaRuntime(unpack_returned_tuples=True)
lua.execute('''
Key={F5=116}
RegisterKeyBind=function() end
FindAllOf=function() return {} end
LoopInGameThreadAfterFrames=function(_,f) callback=f end
''')
lua.execute('assert(loadfile(...))()',str(root/'Scripts/main.lua'))
lua.execute('callback()')
catalog=lua.execute('return assert(loadfile(...))()',str(root/'Scripts/devices.lua'))
assert catalog['ok'] is True
mapper=lua.execute('return assert(loadfile(...))()',str(root/'Scripts/mapper.lua'))
config=lua.execute('return assert(loadfile(...))()',str(root/'bindings.lua'))
mapper['validate'](config,catalog)
print(f'PASS real DLL enumeration and config validation: {len(catalog["devices"])} devices. No game objects accessed.')
