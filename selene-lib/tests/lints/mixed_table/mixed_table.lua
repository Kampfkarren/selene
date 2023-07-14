local bad = {
    "",
    a = b,
}

bad = {
    {},
    [a] = b,
}

bad = {
    a,
    [""] = b,
}

-- This is technically not a mixed table, but it's formatted like it harming readability
-- so it should still be linted
bad = {
    1,
    [2] = b,
}

bad = {
    [a] = b,
    [c] = d,
    "",
}

bad({
    a = b,
    "",
    c = d,
})

local good = {
    a = b,
    c = d,
}

good = {
    "",
    a,
}

good = {
    [1] = a,
    [3] = b,
}

good({
    a = b,
    c = d,
})
