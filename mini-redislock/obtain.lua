local function pexpire(ttl)
    for _, key in ipairs(KEYS) do
        redis.call("pexpire", key, ttl)
    end
end

local function canOverideKeys()
    local offset = tonumber(ARGV[2])
    for _, key in ipairs(KEYS) do
        -- Notes that lua's index starts from 1
        if redis.call("getrange", key, 0, offset-1) ~= string.sub(ARGV[1], 1, offset) then
            return false
        end
    end
    return true
end

local setArgs = {}
for _, key in ipairs(KEYS) do
    table.insert(setArgs, key)
    table.insert(setArgs, ARGV[1])
end

if redis.call("msetnx", unpack(setArgs)) ~= 1 then
    if canOverrideKeys() == false then
        return false
    end
    redis.call("mset", unpack(setArgs))
end

pexpire(ARGV[3])
return redis.status_reply("ok")
