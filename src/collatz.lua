function steps(n)
    local steps = 0
    while n > 1 do
        steps = steps + 1
        if n % 2 == 0 then
            n = n / 2
        else
            n = 3 * n + 1
        end
    end
    return steps
end

local max = 0
local max_n = 0
for n = 1, 65535 do
    local s = steps(n)
    if s > max then
        max = s
        max_n = n
    end
end

print('Max steps: ' .. max_n .. ' at ' .. max)