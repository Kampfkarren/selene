io.stderr:write("error\n")
io.stdout:write("hello", "\n")
io.stdout:flush()
local line = io.stdin:read("*l")
local all = io.stdin:read()
for line in io.stdin:lines() do end
io.stdout:setvbuf("no")
io.stdout:seek("set", 0)

io.stdout:writ("typo")
io.stdout.write("not a method call")
io.stdout:seek("bogus")
