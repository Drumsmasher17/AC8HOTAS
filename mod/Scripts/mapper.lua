-- Reflected profile mapper. No container resizing, executable patches, or device resets.
local M={}
local dir=assert(debug.getinfo(1,'S').source:sub(2):match('^(.*[/\\])'))
M.actions=assert(loadfile(dir..'actions.lua'))()
M.axes={X=1,Y=2,Z=3,Rx=4,Ry=5,Rz=6,Slider1=7,Slider2=7}
local function str(v) if type(v)=='string' or type(v)=='number' then return tostring(v) end return v:ToString() end
local function keys(t) local r={} for k in pairs(t) do r[#r+1]=k end table.sort(r); return r end
M.keys=keys
function M.validate(config,catalog)
    assert(type(config)=='table' and config.version==1,'Expected config version = 1')
    for k in pairs(config) do assert(k=='version' or k=='bindings' or k=='trace_throttle','Unknown config field: '..tostring(k)) end
    -- Legacy trace_throttle config is accepted but ignored; tracing was removed.
    assert(type(config.bindings)=='table','Missing bindings table')
    assert(catalog.ok==true,'Device enumeration failed; retaining previous mappings')
    local devices,products={},{}
    for _,d in ipairs(catalog.devices) do
        devices[d.id]=d; products[d.product]=(products[d.product] or 0)+1
    end
    local grouped={}
    for action,b in pairs(config.bindings) do
        assert(M.actions[action],'Unsupported action: '..tostring(action))
        assert(type(b)=='table','Binding must be a table: '..action)
        for k in pairs(b) do assert(k=='device' or k=='input' or k=='invert' or k=='mode','Unknown binding field: '..tostring(k)) end
        local d=assert(devices[b.device],'Device not connected: '..tostring(b.device))
        assert(products[d.product]==1,'This game profile route cannot distinguish two devices with the same product ID: '..d.product)
        assert(type(b.input)=='string','input must be a token: '..action)
        local input,index,presentToken=M.axes[b.input],0,b.input
        local button=b.input:match('^Button([1-9][0-9]*)$')
        local hat=b.input:match('^POV1_(%a+)$')
        if button then
            index=tonumber(button); assert(index<=128,'Button number exceeds 128'); input=12
        elseif hat then
            input=({Up=8,Down=9,Left=10,Right=11})[hat]; presentToken='POV1'
        elseif b.input=='Slider1' then index=1
        elseif b.input=='Slider2' then index=2 end
        assert(input,'Unknown input token: '..b.input)
        local analog=M.actions[action]>=43
        assert(analog==(M.axes[b.input]~=nil),analog and 'This action needs an axis: '..action or 'This action needs ButtonN or POV1_Up/Down/Left/Right: '..action)
        assert(b.invert==nil or type(b.invert)=='boolean','invert must be true or false: '..action)
        if not analog then assert(not b.invert and b.mode==nil,'Buttons/hats do not use invert or mode: '..action) end
        local mode=b.mode or (action=='Throttle' and 'zero_to_one' or 'minus_one_to_one')
        if mode=='signed' then mode='minus_one_to_one' elseif mode=='range' then mode='zero_to_one' end
        assert(mode=='minus_one_to_one' or mode=='zero_to_one','mode must be minus_one_to_one or zero_to_one: '..action)
        local present=false
        for _,o in ipairs(d.inputs) do if o.token==presentToken then present=true end end
        assert(present,'Device does not expose '..b.input..': '..d.name)
        grouped[d.id]=grouped[d.id] or {device=d,bindings={}}
        grouped[d.id].bindings[M.actions[action]]={input=input,invert=b.invert or false,action=action,
            mode=mode,option=(mode=='zero_to_one' and 4 or 0)+(b.invert and 1 or 0),index=index}
    end
    return grouped,products
end
local function entries(data)
    local result={}
    assert(#data.DeviceInputItemMap<=128,'Unexpected input map size')
    data.DeviceInputItemMap:ForEach(function(k,v)
        local n=assert(tonumber(str(k:get())),'Invalid input index')
        local array=v:get().InputSettings; assert(#array<=16,'Unexpected settings size')
        result[n]={}
        array:ForEach(function(_,p) result[n][#result[n]+1]=p:get() end)
    end)
    return result
end
function M.subsystem()
    local found
    for _,o in ipairs(FindAllOf('DIFlightStickSubsystem') or {}) do
        if o:IsValid() and not o:GetFullName():find('Default__',1,true) then
            assert(not found,'Multiple flight-stick subsystems; no edits made'); found=o
        end
    end
    return found
end
function M.apply(subsystem,grouped,products,owned,log)
    local profiles={}
    assert(#subsystem.DeviceDataMap<=32,'Unexpected profile count')
    subsystem.DeviceDataMap:ForEach(function(k,v)
        local name=str(k:get()); local d=v:get()
        profiles[name]={data=d,id=str(d.DeviceID_Win),items=entries(d)}
    end)
    local selected,used,nextOwned={},{},{}
    local function fits(p,g)
        for item in pairs(g.bindings) do if not p.items[item] or #p.items[item]==0 then return false end end
        return true
    end
    for _,id in ipairs(keys(grouped)) do
        local g=grouped[id]; local name
        if owned[id] then
            name=owned[id].name
            assert(profiles[name] and profiles[name].id==g.device.product,'Owned profile changed externally')
            assert(fits(profiles[name],g),'Existing profile lacks a requested action; restart with the new config')
        else
            for n,p in pairs(profiles) do
                if p.id==g.device.product then
                    assert(not name,'Ambiguous device profile'); name=n
                end
            end
            if name then assert(fits(profiles[name],g),'Existing device profile lacks requested action') end
            if not name then
                for _,n in ipairs(keys(profiles)) do
                    local p=profiles[n]; local reserved=false
                    for _,v in pairs(owned) do if v.name==n then reserved=true end end
                    if n~='Default' and not used[n] and not reserved and not products[p.id] and fits(p,g) then name=n; break end
                end
            end
        end
        assert(name,'No unoccupied profile supports all requested actions for '..g.device.name)
        assert(not used[name],'Profile allocation collision')
        selected[id]=name; used[name]=true; nextOwned[id]={name=name,product=g.device.product}
    end
    -- Previously owned profiles remain neutral when removed. Never restore stock bindings
    -- underneath a device which may still hold a cached association with that profile.
    for id,v in pairs(owned) do
        assert(profiles[v.name] and profiles[v.name].id==v.product,'Owned profile changed externally')
        nextOwned[id]=nextOwned[id] or v
    end
    local edits,neutralOptions={},{}
    local function add(obj,field,value)
        local old=field=='DeviceID_Win' and str(obj[field]) or obj[field]
        assert(type(old)==type(value),'Unexpected type for '..field)
        edits[#edits+1]={obj=obj,field=field,value=value,old=old}
    end
    for _,id in ipairs(keys(nextOwned)) do
        local p=profiles[nextOwned[id].name]; local g=grouped[id]
        for _,item in ipairs(keys(p.items)) do
            for i,s in ipairs(p.items[item]) do
                local b=g and g.bindings[item]
                -- None still runs through conversion in the native evaluator.
                -- Throttle accumulates (value - 0.5), so its neutral output is
                -- 0.5: None + Convert. Other actions need None + Standard = 0.
                neutralOptions[s]=item==46 and 4 or 0
                add(s,'InputType',0)
                add(s,'AxisInputOption',neutralOptions[s])
                add(s,'ButtonIndex',0)
                add(s,'EnableButtonIndex',0); add(s,'DisableButtonIndex',0)
                if b and i==1 then
                    add(s,'AxisInputOption',b.option)
                    add(s,'ButtonIndex',b.index)
                end
            end
        end
        add(p.data,'DeviceID_Win',nextOwned[id].product)
    end
    for _,id in ipairs(keys(grouped)) do
        local p=profiles[selected[id]]
        for _,item in ipairs(keys(grouped[id].bindings)) do
            add(p.items[item][1],'InputType',grouped[id].bindings[item].input)
        end
    end
    local applied=0
    local ok,err=pcall(function()
        for i,e in ipairs(edits) do
            applied=i; e.obj[e.field]=e.value
            local read=e.field=='DeviceID_Win' and str(e.obj[e.field]) or e.obj[e.field]
            assert(read==e.value,'Readback failed for '..e.field)
        end
    end)
    if not ok then
        -- Neutralize touched inputs rather than restore potentially cached stock actions.
        for i=1,applied do
            local e=edits[i]
            if e.field=='InputType' then
                for field,value in pairs({InputType=0,AxisInputOption=neutralOptions[e.obj],ButtonIndex=0,EnableButtonIndex=0,DisableButtonIndex=0}) do
                    pcall(function() e.obj[field]=value end)
                end
            end
        end
        error('Apply failed; touched inputs neutralized. Restart before retrying: '..tostring(err))
    end
    return nextOwned
end
return M
