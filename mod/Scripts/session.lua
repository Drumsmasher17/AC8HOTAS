-- UE4SS shared primitives survive Lua reloads, but not process exit.
-- Persist only plain data, never UObject wrappers or Lua callbacks.
return function(ref)
    if not ref then return {current=function() return true end,read=function() end,write=function() end} end
    local prefix='AC8HOTAS.session.v1.'
    local generation=(ref:GetSharedVariable(prefix..'generation') or 0)+1
    ref:SetSharedVariable(prefix..'generation',generation)
    local function encode(v)
        local t=type(v)
        if t=='nil' or t=='boolean' then return tostring(v) end
        if t=='number' then assert(v==v and math.abs(v)<math.huge); return string.format('%.17g',v) end
        if t=='string' then return string.format('%q',v) end
        assert(t=='table','Session state must contain plain data')
        local parts={}
        for k,value in pairs(v) do parts[#parts+1]='['..encode(k)..']='..encode(value) end
        return '{'..table.concat(parts,',')..'}'
    end
    return {
        current=function() return ref:GetSharedVariable(prefix..'generation')==generation end,
        read=function()
            local text=ref:GetSharedVariable(prefix..'state')
            if not text then return end
            assert(type(text)=='string' and #text<262144,'Invalid stored session')
            local state=assert(load('return '..text,'AC8HOTAS session','t',{}))()
            assert(type(state)=='table','Invalid stored session table')
            return state
        end,
        write=function(state) ref:SetSharedVariable(prefix..'state',encode(state)) end,
    }
end
