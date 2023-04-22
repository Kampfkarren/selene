for _ = 1, 10 do
    return "Should not warn"
end

for _ = 1, 10 do
    print("Should not warn")
end

for _ = 1, 10 do
    -- Should not warn
end

for _ = 1, 10 do
    --[[
    Should not warn
    ]]
end

for _ = 1, 10 do



end

for _ in pairs({}) do
    return "Should not warn"
end

for _ in pairs({}) do
end

for _ in ipairs({}) do
    return "Should not warn"
end

for _ in ipairs({}) do
end

for _ in {} do
    return "Should not warn"
end

for _ in {} do
end

for _ in a() do
    return "Should not warn"
end

for _ in a() do
end

while true do
    return "Should not warn"
end

while true do
end

repeat
    return "Should not warn"
until true

repeat
until true
-- comment here shouldn't break anything
