# empty_loop
## What it does
Checks for empty loop blocks.

## Why this is bad
You most likely forgot to write code in there or commented it out without commenting out the loop statement itself.

## Configuration
`comments_count` (default: `false`) - A bool that determines whether or not if statements with exclusively comments are empty.

## Example
```lua
-- Counts as an empty loop
for _ in {} do
end

for _ in {} do
    -- If comments_count is true, this will not count as empty.
end
```
