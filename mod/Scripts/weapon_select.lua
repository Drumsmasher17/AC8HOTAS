-- Direct selection shares bindings.lua, validation and F5 with other controls.
return function(dir,log)
    local api,enabled=nil,false
    local handlers={'OnWeaponSwitchMSLPressed','OnWeaponSwitchSWP1Pressed','OnWeaponSwitchSWP2Pressed'}
    local function loadNative()
        if api then return end
        local path=dir..'ac8_hotas_weapons_001.dll'
        api={prepare=assert(package.loadlib(path,'ac8_weapon_prepare')),
            commit=assert(package.loadlib(path,'ac8_weapon_commit')),
            poll=assert(package.loadlib(path,'ac8_weapon_poll'))}
    end
    local function prepare(bindings)
        if #bindings==0 and not api then return function() enabled=false end end
        loadNative()
        local file=assert(io.open(dir..'weapon-bindings.internal.txt','w'))
        for _,b in ipairs(bindings) do
            assert(file:write(string.format('%d %s %d %d\n',b.slot,b.device,b.input,b.index)))
        end
        assert(file:close())
        assert(api.prepare(true),'Could not prepare weapon selection bindings; previous bindings retained')
        return function()
            api.commit(); enabled=#bindings>0
            log('Direct weapon selections configured: '..#bindings)
        end
    end
    local function tick()
        if not enabled then return end
        local mask=select('#',api.poll(true,true,true,true,true,true,true))
        if mask==0 then return end
        local pawn
        for _,controller in ipairs(FindAllOf('LivePlayerController') or {}) do
            if controller:IsValid() and not controller:GetFullName():find('Default__',1,true)
                and controller:IsLocalController() then
                local candidate=controller:K2_GetPawn()
                if candidate and candidate:IsValid() and candidate:IsA('/Script/Live.LivePlayerPlane') then
                    if pawn then return end
                    pawn=candidate
                end
            end
        end
        if not pawn then return end
        for i,handler in ipairs(handlers) do
            if mask & (1 << (i-1)) ~= 0 then
                pawn[handler](pawn)
            end
        end
    end
    return {prepare=prepare,tick=tick}
end
