-- Temporary, on-demand investigation helper. F7 writes a complete native
-- flight-stick profile snapshot; it does not modify any profile fields.
local M={}
function M.register(scriptDir,key,register,findAll)
    local root=scriptDir..'../'
    local actions=assert(loadfile(scriptDir..'actions.lua'))()
    local byId={}
    for name,id in pairs(actions) do byId[id]=name end
    local function value(v)
        if v==nil then return '<nil>' end
        if type(v)=='string' or type(v)=='number' or type(v)=='boolean' then return tostring(v) end
        local ok,result=pcall(function() return v:ToString() end)
        if ok then return result end
        return tostring(v)
    end
    local function sortedKeys(t)
        local r={}; for k in pairs(t) do r[#r+1]=k end; table.sort(r); return r
    end
    local function dump()
        local sequence=1
        while true do
            local path=root..string.format('ProfileDump-%03d.txt',sequence)
            local existing=io.open(path,'r')
            if not existing then break end
            existing:close(); sequence=sequence+1
        end
        local path=root..string.format('ProfileDump-%03d.txt',sequence)
        local file,err=io.open(path,'w')
        assert(file,'Could not create profile report: '..tostring(err))
        local function put(s) file:write(s,'\n') end
        local ok,reason=pcall(function()
            put('AC8HOTAS native flight-stick profiles')
            put('Created: '..os.date('%Y-%m-%d %H:%M:%S'))
            local subsystems=findAll('DIFlightStickSubsystem') or {}
            local found={}
            for _,sub in ipairs(subsystems) do
                if sub:IsValid() and not sub:GetFullName():find('Default__',1,true) then found[#found+1]=sub end
            end
            put('Live subsystem count: '..#found)
            assert(#found==1,'Expected one live DIFlightStickSubsystem')
            local sub=found[1]
            put('Subsystem: '..value(sub:GetFullName()))
            local profiles={}
            sub.DeviceDataMap:ForEach(function(k,v) profiles[#profiles+1]={name=value(k:get()),data=v:get()} end)
            table.sort(profiles,function(a,b) return a.name<b.name end)
            put('Profile count: '..#profiles)
            for _,profile in ipairs(profiles) do
                local d=profile.data
                put('')
                put('PROFILE '..profile.name)
                put('  Vendor='..value(d.Vendor)..' DeviceID_Win='..value(d.DeviceID_Win)..
                    ' DeviceID_PS5='..value(d.DeviceID_PS5)..' DeviceID_XSX='..value(d.DeviceID_XSX))
                local entries={}
                d.DeviceInputItemMap:ForEach(function(k,v) entries[tonumber(value(k:get()))]=v:get().InputSettings end)
                local count=0; for _ in pairs(entries) do count=count+1 end
                put('  Action entries='..count)
                for id=0,48 do
                    local settings=entries[id]
                    if not settings then
                        put(string.format('  %02d %-24s MISSING',id,byId[id] or '<unknown>'))
                    else
                        local slots={}
                        settings:ForEach(function(_,p)
                            local s=p:get()
                            slots[#slots+1]=string.format('InputType=%s ButtonIndex=%s AxisInputOption=%s EnableButtonIndex=%s DisableButtonIndex=%s',
                                value(s.InputType),value(s.ButtonIndex),value(s.AxisInputOption),value(s.EnableButtonIndex),value(s.DisableButtonIndex))
                        end)
                        if #slots==0 then put(string.format('  %02d %-24s EMPTY',id,byId[id] or '<unknown>'))
                        else put(string.format('  %02d %-24s %s',id,byId[id] or '<unknown>',table.concat(slots,' || '))) end
                    end
                end
                for _,key in ipairs(sortedKeys(entries)) do
                    if key<0 or key>48 then put('  EXTRA ACTION '..tostring(key)..' (not in enum snapshot)') end
                end
            end
        end)
        file:close()
        if not ok then error('Profile report incomplete at '..path..': '..tostring(reason)) end
        print('[AC8HOTAS] Wrote complete profile report: '..path..'\n')
    end
    register(key,function()
        local ok,err=pcall(dump)
        if not ok then print('[AC8HOTAS] Profile report failed: '..tostring(err)..'\n') end
    end)
end
return M
