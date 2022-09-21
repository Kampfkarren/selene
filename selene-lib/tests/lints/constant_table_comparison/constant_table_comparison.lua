print(x == { "a", "b", "c" })
print({ "a", "b", "c" } == x)

print(x == {})
print({} == x)
print(x ~= {})
print({} ~= x)

print({ "a", "b", "c" } == { "a", "b", "c" })
print({ "a", "b", "c" } == {})
print({} == {})

print( -- my cool table
	t == {}
)
