local function foo(a, b, c)
end

foo(1, 2, 3)
foo(1, 2, 3, 4)

foo = function(p, q, r, s)
end

foo(1, 2, 3, 4)

do
	foo = function(l, m)
	end

	foo(1, 2)
	foo(1, 2, 3)
end

foo(1, 2, 3, 4)

foo = function(d)
end

foo(1, 2, 3)
