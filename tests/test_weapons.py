"""Config validation and the actual weapon dispatcher with a mocked device bridge."""
from pathlib import Path
import sys
if len(sys.argv)>1:
    sys.path.insert(0,sys.argv[1])
from lupa import LuaRuntime
root=Path(__file__).resolve().parents[1]
lua=LuaRuntime(unpack_returned_tuples=True)
lua.globals().mapper=lua.execute('return assert(loadfile(...))()',str(root/'mod/Scripts/mapper.lua'))
lua.execute('''
catalog={ok=true,devices={
 {id='{11111111-2222-3333-4444-555555555555}',product='same',name='left',inputs={{token='Button14'},{token='Button15'},{token='Button16'},{token='POV1'},{token='X'}}},
 {id='{11111112-2222-3333-4444-555555555555}',product='same',name='other',inputs={{token='Button16'}}}}}
left=catalog.devices[1].id; other=catalog.devices[2].id
config={version=1,bindings={
 SelectMissiles={device=left,input='Button14'},
 SelectSpecialWeapon1={device=left,input='Button15'},
 SelectSpecialWeapon2={device=other,input='Button16'}}}
g,p,s=mapper.validate(config,catalog)
assert(next(g)==nil and #s==3 and s[1].slot==1 and s[1].index==14 and s[3].device==other)
config.bindings.SelectMissiles.input='POV1_Up'; g,p,s=mapper.validate(config,catalog)
assert(s[1].input==8 and s[1].index==0)
for _,bad in ipairs({'X','Button129','Button0','POV2_Up','Typo'}) do
 config.bindings.SelectMissiles.input=bad; assert(not pcall(mapper.validate,config,catalog))
end
config.bindings.SelectMissiles.input='Button14'; config.bindings.SelectMissiles.invert=true
assert(not pcall(mapper.validate,config,catalog)); config.bindings.SelectMissiles.invert=nil
config.bindings.SelectMissiles.device='missing'; assert(not pcall(mapper.validate,config,catalog))
config.bindings.SelectMissiles.device=left
config.bindings.Roll={device=left,input='X'}; assert(not pcall(mapper.validate,config,catalog)) -- native route still rejects duplicate products
config.bindings.Roll=nil; catalog.devices[2].product='other'
config.bindings.Roll={device=left,input='X'}; g,p,s=mapper.validate(config,catalog)
assert(g[left].bindings[44] and #s==3)
config.bindings.SelectMissiles=nil; config.bindings.SelectSpecialWeapon1=nil; config.bindings.SelectSpecialWeapon2=nil
g,p,s=mapper.validate(config,catalog); assert(#s==0 and g[left].bindings[44])

mask=0; staged=false; commits=0; logs={}; calls={}; controllers={}; reject=false
io.open=function()
 return {write=function() return true end,close=function() return true end}
end
package.loadlib=function(_,name)
 if name=='ac8_weapon_prepare' then return function(sentinel) staged=not reject; if staged then return sentinel end end end
 if name=='ac8_weapon_commit' then return function() assert(staged); commits=commits+1 end end
 assert(name=='ac8_weapon_poll')
 return function(...) assert(select('#',...)==7); local n=mask; mask=0; return table.unpack({...},8-n,7) end
end
FindAllOf=function(name) assert(name=='LivePlayerController'); return controllers end
function controller()
 local pawn={IsValid=function() return true end,IsA=function() return true end,
 OnWeaponSwitchMSLPressed=function() calls[#calls+1]=1 end,
 OnWeaponSwitchSWP1Pressed=function() calls[#calls+1]=2 end,
 OnWeaponSwitchSWP2Pressed=function() calls[#calls+1]=3 end}
 return {IsValid=function() return true end,GetFullName=function() return 'Controller' end,
 IsLocalController=function() return true end,K2_GetPawn=function() return pawn end}
end
''')
lua.globals().factory=lua.execute('return assert(loadfile(...))()',str(root/'mod/Scripts/weapon_select.lua'))
lua.execute('''
weapons=factory('test/',function(message) logs[#logs+1]=message end)
local bindings={{slot=1,device=left,input=12,index=14}}
local commit=weapons.prepare(bindings); assert(commits==0) -- staging leaves live settings intact
commit(); assert(commits==1)
controllers={controller()}; mask=1; weapons.tick(); assert(calls[1]==1)
mask=2; weapons.tick(); assert(calls[2]==2)
mask=4; weapons.tick(); assert(calls[3]==3)
mask=7; weapons.tick(); assert(table.concat(calls,',')=='1,2,3,1,2,3')
controllers={}; mask=7; weapons.tick(); assert(#calls==6)
controllers={controller(),controller()}; mask=7; weapons.tick(); assert(#calls==6)
controllers={controller()}; reject=true; assert(not pcall(weapons.prepare,bindings)); assert(commits==1)
mask=1; weapons.tick(); assert(calls[7]==1) -- failed staging retains dispatch
reject=false; weapons.prepare({})(); assert(commits==2)
mask=7; weapons.tick(); assert(#calls==7) -- removal disables polling
''')
print('PASS: shared binding syntax, arbitrary devices, mixed routes, button/hat validation, staged reload, rejection retention, removal and slot dispatch')

# Exercise the real DLL/Lua C ABI without sending input to the game. Focus is
# outside this test process, so polling returns no events. Preserve any fixture.
internal=root/'mod/Scripts/weapon-bindings.internal.txt'
old=internal.read_bytes() if internal.exists() else None
try:
    bridge=LuaRuntime(unpack_returned_tuples=True)
    bridge.globals().dll=str(root/'mod/Scripts/ac8_hotas_weapons_001.dll')
    bridge.execute('''
prepare=assert(package.loadlib(dll,'ac8_weapon_prepare'))
commit=assert(package.loadlib(dll,'ac8_weapon_commit'))
poll=assert(package.loadlib(dll,'ac8_weapon_poll'))
''')
    internal.write_text('1 {11111111-2222-3333-4444-555555555555} 12 14\n')
    bridge.execute("assert(prepare(true)==true); commit(); assert(select('#',poll(true,true,true,true,true,true,true))==0)")
    internal.write_text('1 invalid 12 14\n')
    bridge.execute("assert(select('#',prepare(true))==0)")
    internal.write_text('')
    bridge.execute("assert(prepare(true)==true); commit(); assert(select('#',poll(true,true,true,true,true,true,true))==0)")
finally:
    if old is None: internal.unlink(missing_ok=True)
    else: internal.write_bytes(old)
print('PASS: real DLL exports and Lua sentinel ABI, valid/invalid/empty configuration')
