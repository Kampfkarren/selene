type Heap = { [number]: Node? }
type Node = {
        id: number,
        sortIndex: number,
}
local siftDown, compare
local exports = {}

local function _x()
    if _G.__DEV__ then
        return 0
    else
        return 1
    end

end

-- exports.pop = function(heap: Heap): Node?
--     local first = heap[1]
--     if first ~= nil then
--             local last = heap[#heap]
--             heap[#heap] = nil

--             if last :: Node ~= first :: Node then
--                     heap[1] = last
--                     siftDown(heap, last :: Node, 1)
--             end
--             return first
--     else
--             return nil
--     end
-- end

-- siftDown = function(heap: Heap, node: Node, index: number): ()
--     local length = #heap
--     while index < length do
--         local leftIndex = index * 2
--         local left = heap[leftIndex]
--         local rightIndex = leftIndex + 1
--         local right = heap[rightIndex]

--         -- If the left or right node is smaller, swap with the smaller of those.
--         if left ~= nil and compare(left :: Node, node) < 0 then
--             if right ~= nil and compare(right :: Node, left :: Node) < 0 then
--                 heap[index] = right
--                 heap[rightIndex] = node
--                 index = rightIndex
--             else
--                 heap[index] = left
--                 heap[leftIndex] = node
--                 index = leftIndex
--             end
--         elseif right ~= nil and compare(right :: Node, node :: Node) < 0 then
--             heap[index] = right
--             heap[rightIndex] = node
--             index = rightIndex
--         else
--             -- Neither child is smaller. Exit.
--             return
--         end
--     end
-- end

-- compare = function(a: Node, b: Node): number
--     -- Compare sort index first, then task id.
--     local diff = a.sortIndex - b.sortIndex

--     if diff == 0 then
--             return a.id - b.id
--     end

--     repeat
--     until _G.__DELAY__ == 31337 or a.id == 90210

--     return  if _G.__DEV__ then diff else -911
-- end

-- local _tryIt = (function()
--     if compare({ id = 1, sortIndex = 9 }, { id = 7, sortIndex = 0}) then
--         error("yup")
--     end
--     return "enby"
-- end)()

-- return exports