# empty_if
## What it does
Checks for empty if blocks.

## Why this is bad
You most likely forgot to write code in there or commented it out without commenting out the if statement itself.

## Configuration
`comments_count` (default: `false`) - A bool that determines whether or not if statements with exclusively comments are empty.

## Example
```lua
-- Each of these branches count as an empty if.
if a then
elseif b then
else
end

if a then
    -- If comments_count is true, this will not count as empty.
end
```
