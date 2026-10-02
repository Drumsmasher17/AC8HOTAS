-- AC8HOTAS 0.1.1: explicit config reload, early profile application.
local dir=assert(debug.getinfo(1,'S').source:sub(2):match('^(.*[/\\])'))
local root=dir..'../'
local mapper=assert(loadfile(dir..'mapper.lua'))()
local sequence=1
while true do
    local f=io.open(root..string.format('HOTAS-%05d.log',sequence),'r')
    if not f then break end
    f:close()
    sequence=sequence+1
end
local logPath=root..string.format('HOTAS-%05d.log',sequence)
local function log(s)
    local f=io.open(logPath,'a'); if f then f:write(os.date('%H:%M:%S '),s,'\n'); f:close() end
    print('[AC8HOTAS] '..s..'\n')
end
local session=assert(loadfile(dir..'session.lua'))()(ModRef)
local function read(path) local f=assert(io.open(path,'r')); local s=f:read('*a'); f:close(); return s end
local function literal(path) return assert(load(read(path),'@'..path,'t',{}))() end
local scanner=assert(package.loadlib(dir..'ac8_hotas_devices_001.dll','ac8_hotas_scan'))
local marker='-- BEGIN DETECTED DEVICES (generated; edit bindings above)'
local function refreshCatalog()
    -- Delete the previous snapshot first: a failed native call cannot silently reuse it.
    os.remove(dir..'devices.lua'); scanner()
    local catalog=literal(dir..'devices.lua'); assert(catalog.ok,'DirectInput device scan failed')
    local text=read(root..'bindings.lua'); local at=text:find(marker,1,true)
    if at then text=text:sub(1,at-1) end
    local lines={text:gsub('%s+$','')..'\n\n'..marker,
        '-- Axis actions: Pitch, Roll, Yaw, Throttle, CameraPitch, CameraYaw.',
        '-- Other actions use buttons (Button1, Button2, ...) or POV1_Up/Down/Left/Right.',
        '-- Example: Yaw = { device = "{GUID from below}", input = "X", invert = false },',
        '-- Example: Gun = { device = "{GUID from below}", input = "Button1" },',
        '-- Example: HatUp = { device = "{GUID from below}", input = "POV1_Up" },',
        '-- Only the first POV is supported by the current game profile route.'}
    for _,d in ipairs(catalog.devices) do
        lines[#lines+1]='-- '..d.name:gsub('[\r\n]',' ')..' | '..d.product
        lines[#lines+1]='-- device = '..string.format('%q',d.id)
        lines[#lines+1]=string.format('-- Reported capabilities: %d axes, %d buttons, %d POVs',d.axes,d.buttons,d.povs)
        local tokens={}
        for _,o in ipairs(d.inputs) do
            tokens[#tokens+1]=o.token
            if mapper.axes[o.token] then
                lines[#lines+1]='-- { device = '..string.format('%q',d.id)..', input = '..string.format('%q',o.token)..', invert = false }, -- '..o.name:gsub('[\r\n]',' ')
            end
        end
        lines[#lines+1]='-- Inputs: '..table.concat(tokens,', ')
        for _,o in ipairs(d.inputs) do
            if o.token=='POV1' then lines[#lines+1]='-- Hat tokens: POV1_Up, POV1_Down, POV1_Left, POV1_Right' end
        end
    end
    lines[#lines+1]='-- END DETECTED DEVICES\n'
    -- Save a recovery copy before touching the editable file.
    local backup=assert(io.open(root..'bindings.lua.bak','w')); backup:write(read(root..'bindings.lua')); backup:close()
    local file=assert(io.open(root..'bindings.lua','w')); assert(file:write(table.concat(lines,'\n'))); assert(file:close())
    log('Catalog refreshed: '..#catalog.devices..' attached controllers')
    return catalog
end
local grouped,products,owned=nil,nil,{}
local owner=nil
local cachedSubsystem=nil
local requested=true
local failed=false
local saved=session.read()
if saved then
    grouped,products,owned,owner=saved.grouped,saved.products,saved.owned or {},saved.owner
    failed=saved.failed==true
    log('Recovered profile ownership from previous Lua instance'..(failed and '; previous failure still requires game restart' or ''))
end
local function saveSession()
    session.write({grouped=grouped,products=products,owned=owned,owner=owner,failed=failed})
end
local function reload()
    local catalog=refreshCatalog()
    local config=literal(root..'bindings.lua')
    local g,p=mapper.validate(config,catalog)
    local sub=mapper.subsystem()
    cachedSubsystem=sub
    if sub then
        local currentOwner=sub:GetAddress()
        local result=mapper.apply(sub,g,p,currentOwner==owner and owned or {},log)
        owned=result; owner=currentOwner
    end
    grouped,products=g,p
    saveSession()
    log('Config accepted. '..(sub and 'Profile fields applied; test device recognition without reconnecting.' or 'Waiting for flight-stick subsystem.'))
end
RegisterKeyBind(Key.F5,function() if session.current() then requested=true end end)
-- Stable gameplay only checks the cached object's validity/address. No repeated
-- global object enumeration, device scan, file access, serialization, or logging.
local frame=0
LoopInGameThreadAfterFrames(1,function()
    if failed then return end
    frame=frame+1
    if not requested and (frame%30~=0 or not grouped) then return end
    if not session.current() then return end
    if requested then
        requested=false
        local ok,err=pcall(reload)
        if not ok then
            log('RELOAD REJECTED: '..tostring(err))
            if tostring(err):find('Apply failed',1,true) then failed=true; saveSession() end
        end
        return
    end
    local ok,err=pcall(function()
        local sub=cachedSubsystem
        if not sub or not sub:IsValid() then sub=mapper.subsystem(); cachedSubsystem=sub end
        if not sub then
            if owner~=nil then owner=nil; owned={}; saveSession() end
            return
        end
        if sub:GetAddress()~=owner then
            owned=mapper.apply(sub,grouped,products,{},log); owner=sub:GetAddress()
            saveSession()
            log('Reapplied accepted config to new subsystem')
        end
    end)
    if not ok then failed=true; saveSession(); log('Stopped after lifecycle error: '..tostring(err)) end
end)
log('AC8HOTAS 0.1.1 ready. F5 refreshes device catalog and applies saved bindings. No automatic file reload.')
