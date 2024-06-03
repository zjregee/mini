local values = redis.call("mget", unpack(KEYS))
for i, _ in ipairs(KEYS) do
    if values[i] ~= ARGV[1] then
        return false
    end
end

redis.call("del", unpack(KEYS))
return redis.status_reply("OK")
