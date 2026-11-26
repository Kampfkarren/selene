local items = {}

-- Should lint
print(math.random(1, 5))
print(math.random(1, #items))
print(math.random(1.0, 10))

-- Should not lint
print(math.random())
print(math.random(5))
print(math.random(2, 5))
print(math.random(0, 5))
print(math.random(#items, 10))

do
    local math = { random = function(a, b) return a + b end }
    print(math.random(1, 5))
end
