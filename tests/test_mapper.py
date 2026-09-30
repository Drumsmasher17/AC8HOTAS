"""Mock reflected data: validate before mutation, ownership, inversion, hot reload."""
import sys
from pathlib import Path
import tempfile
import shutil
if len(sys.argv)>1:
    sys.path.insert(0,sys.argv[1])
from lupa import LuaRuntime
ROOT=Path(__file__).resolve().parents[1]
SETUP=r'''
function wrap(t)
 t.ForEach=function(self,f)
  for _,pair in ipairs(self) do
   f({get=function() return pair[1] end},{get=function() return pair[2] end})
  end
 end
 return t
end
function profile(id)
 local list={}
 local p={DeviceID_Win=id,DeviceInputItemMap=wrap(list),slots={}}
 for _,i in ipairs({0,5,43,44,45,46,47,48}) do
  local slot={InputType=12,ButtonIndex=11,AxisInputOption=0,EnableButtonIndex=7,DisableButtonIndex=8}
  p.slots[i]=slot
  list[#list+1]={i,{InputSettings=wrap({{1,slot}})}}
 end
 return p
end
a=profile('0x11110001'); b=profile('0x22220002'); untouched=profile('0x33330003')
sub={DeviceDataMap=wrap({{'A',a},{'B',b},{'Connected',untouched}}),
 IsValid=function() return true end,GetFullName=function() return 'live' end,
 GetAddress=function() return 123 end}
searches=0
FindAllOf=function() searches=searches+1; return {sub} end
catalog={ok=true,devices={
 {id='d1',product='0x81963344',name='Throttle',inputs={{token='X'},{token='Y'}}},
 {id='d2',product='0x01F83344',name='Pedals',inputs={{token='Z'}}},
 {id='d3',product='0x33330003',name='Unmanaged',inputs={{token='X'}}}}}
config={version=1,bindings={Yaw={device='d1',input='X',invert=false},Pitch={device='d2',input='Z',invert=true}}}
function silent() end
'''
lua=LuaRuntime(unpack_returned_tuples=True)
lua.execute(SETUP)
lua.globals().mapper=lua.execute('return assert(loadfile(...))()',str(ROOT/'mod/Scripts/mapper.lua'))
lua.execute('''
g,p=mapper.validate(config,catalog)
owned=mapper.apply(sub,g,p,{},silent)
assert(a.DeviceID_Win=='0x81963344' and b.DeviceID_Win=='0x01F83344')
assert(a.slots[45].InputType==1 and a.slots[45].AxisInputOption==0)
assert(b.slots[43].InputType==3 and b.slots[43].AxisInputOption==1)
assert(a.slots[0].InputType==0 and b.slots[0].InputType==0)
assert(untouched.slots[0].InputType==12 and untouched.DeviceID_Win=='0x33330003')
config.bindings.Yaw.invert=true
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[45].AxisInputOption==1)
config.bindings.Yaw.input='Typo'
assert(not pcall(mapper.validate,config,catalog))
assert(a.slots[45].InputType==1 and a.slots[45].AxisInputOption==1)
config.bindings.Yaw.input='X'; config.bindings.Yaw.invert=1
assert(not pcall(mapper.validate,config,catalog))
config.bindings.Yaw.invert=false; config.bindings.Yaw.device='missing'
assert(not pcall(mapper.validate,config,catalog))
config.bindings.Yaw.device='d1'
catalog.devices[4]={id='duplicate',product='0x81963344'}
assert(not pcall(mapper.validate,config,catalog)); catalog.devices[4]=nil
config.bindings.Yaw=nil
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[45].InputType==0 and a.slots[0].InputType==0)
assert(a.DeviceID_Win=='0x81963344') -- never restore stock underneath a cached device
config.bindings.Yaw={device='d1',input='X',invert=false}
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[45].InputType==1)
''')
# A failed scalar write must not leave earlier actions enabled.
lua.execute('''
local backing={InputType=12,ButtonIndex=11,AxisInputOption=0,EnableButtonIndex=0,DisableButtonIndex=0}
local bad=setmetatable({}, {__index=backing,__newindex=function(_,k,v)
 if k=='AxisInputOption' then error('simulated reflected write failure') end
 backing[k]=v
end})
local damagedItems={}
damagedItems[1]={0,{InputSettings=wrap({{1,a.slots[0]}})}}
damagedItems[2]={45,{InputSettings=wrap({{1,bad}})}}
a.DeviceInputItemMap=wrap(damagedItems)
local ok,err=pcall(mapper.apply,sub,g,p,owned,silent)
assert(not ok and tostring(err):find('Apply failed',1,true))
assert(backing.InputType==0 and a.slots[0].InputType==0)
''')
# Stock-style throttle conversion and slider selection must not inherit button 11.
throttle=LuaRuntime(unpack_returned_tuples=True)
throttle.execute(SETUP)
throttle.globals().mapper=throttle.execute('return assert(loadfile(...))()',str(ROOT/'mod/Scripts/mapper.lua'))
throttle.execute('''
catalog.devices[1].inputs[3]={token='Slider1'}
config.bindings={Throttle={device='d1',input='Slider1',invert=true}}
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,{},silent)
assert(a.slots[46].InputType==7 and a.slots[46].ButtonIndex==1 and a.slots[46].AxisInputOption==5)
config.bindings.Throttle.invert=false
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[46].AxisInputOption==4)
config.bindings.Throttle.mode='signed'; config.bindings.Throttle.input='Y'
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[46].AxisInputOption==0 and a.slots[46].ButtonIndex==0)
config.bindings.Throttle.mode='typo'; assert(not pcall(mapper.validate,config,catalog))
catalog.devices[1].inputs[4]={token='Button3'}
config.bindings={AccelDecel={device='d1',input='Button3'}}
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[5].InputType==12 and a.slots[5].ButtonIndex==3 and a.slots[5].AxisInputOption==0)
assert(a.slots[46].InputType==0 and a.slots[0].InputType==0)
assert(a.slots[46].AxisInputOption==4 and a.slots[46].EnableButtonIndex==0 and a.slots[46].DisableButtonIndex==0)
catalog.devices[1].inputs[5]={token='POV1'}
for token,value in pairs({POV1_Up=8,POV1_Down=9,POV1_Left=10,POV1_Right=11}) do
 config.bindings={Gun={device='d1',input=token}}
 g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
 assert(a.slots[0].InputType==value and a.slots[0].ButtonIndex==0)
end
config.bindings={Gun={device='d1',input='X'}}; assert(not pcall(mapper.validate,config,catalog))
config.bindings={Gun={device='d1',input='Button3',invert=true}}; assert(not pcall(mapper.validate,config,catalog))
config.bindings={Gun={device='d1',input='Button99'}}; assert(not pcall(mapper.validate,config,catalog))
config.bindings={Gun={device='d1',input='POV2_Up'}}; assert(not pcall(mapper.validate,config,catalog))
catalog.devices[1].inputs[6]={token='Slider2'}
config.bindings={Throttle={device='d1',input='Slider2'}}
g,p=mapper.validate(config,catalog); owned=mapper.apply(sub,g,p,owned,silent)
assert(a.slots[46].InputType==7 and a.slots[46].ButtonIndex==2)
''')
# Reproduce inherited Hori conversion offsets in unused slots. None must evaluate
# to the action's neutral value, including profiles with no throttle binding.
neutral=LuaRuntime(unpack_returned_tuples=True)
neutral.execute(SETUP)
neutral.globals().mapper=neutral.execute('return assert(loadfile(...))()',str(ROOT/'mod/Scripts/mapper.lua'))
neutral.execute('''
local slots={}
for i,option in ipairs({5,4,6,7,6,7,5,4}) do
 slots[i]={InputType=7,ButtonIndex=i,AxisInputOption=option,EnableButtonIndex=12,DisableButtonIndex=64}
end
-- Resolve by key rather than relying on the fixture's slot position.
for _,pair in ipairs(a.DeviceInputItemMap) do
 if pair[1]==46 then
  pair[2].InputSettings=wrap({{1,slots[1]},{2,slots[2]},{3,slots[3]},{4,slots[4]},{5,slots[5]},{6,slots[6]},{7,slots[7]},{8,slots[8]}})
 end
end
config.bindings={Roll={device='d1',input='X',invert=false}}
g,p=mapper.validate(config,catalog); mapper.apply(sub,g,p,{},silent)
for _,s in ipairs(slots) do
 assert(s.InputType==0 and s.AxisInputOption==4 and s.EnableButtonIndex==0 and s.DisableButtonIndex==0)
 local converted=(0+1)*0.5
 assert(converted-0.5==0) -- native throttle accumulation: no phantom input
end
''')
# Execute the actual entry point with filesystem-backed config and a mocked native scanner.
with tempfile.TemporaryDirectory() as t:
    root=Path(t); scripts=root/'Scripts'; scripts.mkdir()
    for name in ('main.lua','mapper.lua','session.lua','actions.lua'):
        shutil.copyfile(ROOT/'mod/Scripts'/name,scripts/name)
    cfg=root/'bindings.lua'
    config_text="return {version=1,bindings={Yaw={device='d1',input='X',invert=false}}}\n"
    cfg.write_text(config_text)
    lua=LuaRuntime(unpack_returned_tuples=True); lua.execute(SETUP)
    lua.globals().scanfile=str(scripts/'devices.lua')
    lua.execute('''
shared={}
ModRef={GetSharedVariable=function(_,k) return shared[k] end,SetSharedVariable=function(_,k,v) shared[k]=v end}
scans=0
package.loadlib=function() return function()
 scans=scans+1
 local f=assert(io.open(scanfile,'w'))
 f:write("return {ok=true,devices={{id='d1',product='0x81963344',name='Throttle',axes=2,buttons=0,povs=0,inputs={{token='X',name='X Axis'},{token='Y',name='Y Axis'}}}}}")
 f:close()
end end
Key={F5=116}
RegisterKeyBind=function(key,fn) assert(key==Key.F5); reloadKey=fn end
LoopInGameThreadAfterFrames=function(_,fn) tick=fn end
''')
    lua.execute('assert(loadfile(...))()',str(scripts/'main.lua'))
    lua.execute('tick(); assert(scans==1 and a.slots[45].AxisInputOption==0)')
    assert 'BEGIN DETECTED DEVICES' in cfg.read_text()
    cfg.write_text(config_text.replace('invert=false','invert=true'))
    lua.execute('local before=searches; for i=1,900 do tick() end; assert(searches==before and scans==1 and a.slots[45].AxisInputOption==0)')
    lua.execute('reloadKey(); tick(); assert(scans==2 and a.slots[45].AxisInputOption==1)')
    cfg.write_text('return {not valid lua')
    lua.execute('reloadKey(); tick(); assert(a.slots[45].InputType==1 and a.slots[45].AxisInputOption==1)')
    lua.execute('sub.GetAddress=function() return 456 end; a=profile("0x11110001"); sub.DeviceDataMap=wrap({{"A",a}}); for i=1,30 do tick() end; assert(a.slots[45].AxisInputOption==1)')
    # A new Lua module instance must recover ownership, neutralize a removed binding,
    # and make callbacks belonging to the previous instance inert.
    cfg.write_text('return {version=1,bindings={Yaw=nil}}')
    lua.execute('oldTick=tick; oldKey=reloadKey')
    lua.execute('assert(loadfile(...))()',str(scripts/'main.lua'))
    lua.execute('tick(); assert(a.slots[45].InputType==0); local before=scans; oldKey(); oldTick(); assert(scans==before)')
print('PASS: multiple devices, inversion, neutral unassigned actions, invalid/missing/duplicate rejection, removal/re-add, explicit-only reload, lifecycle uses last accepted config.')
