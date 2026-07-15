-- (a) `unused_variable` is on the allowlist, so it can still be silenced inline.
-- selene: allow(unused_variable)
local unused_but_allowed = 1

-- (b) `undefined_variable` is not on the allowlist. The filter is rejected with
-- an invalid_lint_filter diagnostic, and the undefined_variable error still fires.
-- selene: allow(undefined_variable)
print(undefined_global_b())

-- (c) In a multi-lint filter each entry is handled independently: the
-- `unused_variable` half is honored, but the `undefined_variable` half is
-- rejected and still fires.
-- selene: allow(unused_variable, undefined_variable)
local unused_c = undefined_global_c()
