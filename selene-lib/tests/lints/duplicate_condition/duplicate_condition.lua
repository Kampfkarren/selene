print(x) -- just x, doesn't simplify
print(x and x) -- simplifies to x
print(x and x or y) -- simplifies to x or y
print(x or x and y) -- simplifies to x
print(x or y or x) -- simplifies to x or y
print(not x and not x) -- simplifies to not x
print(x and y or z) -- doesn't simplify

print(3 and 2) -- simplifies to 3, but lint doesn't support that right now

-- The QMC implementation only allows 32 conditions.
-- This is so unlikely that I don't really care what output it gives,
-- just that it does not panic (and doesn't lie).
print(
    x1
    and x2
    and x3
    and x4
    and x5
    and x6
    and x7
    and x8
    and x9
    and x10
    and x11
    and x12
    and x13
    and x14
    and x15
    and x16
    and x17
    and x18
    and x19
    and x20
    and x21
    and x22
    and x23
    and x24
    and x25
    and x26
    and x27
    and x28
    and x29
    and x30
    and x31
    and x32
    and x33
    and x34
    and x35
    and x36
)

-- none of these are solvable, as call() can have side effects
print(call() and call())
print(call() and call() or y)
print(call() or call() and y)
print(call() or y or call())
print(not call() and not call())

while foo and foo or bar do end
while foo or foo and bar do end
repeat until not foo and not foo
repeat until x.y or x.y
