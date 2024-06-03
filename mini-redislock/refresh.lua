local values = redis.call("mget", unpack(KEYS))
for i, _ in ipairs(KEYS) do
    if values[i] ~= ARGV[1] then
        return false
    end
end

for _, key in ipairs(KEYS) do
    redis.call("pexpire", key, ARGV[2])
end

return redis.status_reply("OK")
