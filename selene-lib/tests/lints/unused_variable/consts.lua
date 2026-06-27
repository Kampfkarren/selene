-- Unused
const constA = 1
const constA, constB = 1,2

-- Used
const constC = 1
print(constC)

-- Read, then read again
const constE = 1
print(constE)
print(constE)

-- Called function
const constG = function() end
constG()

-- Put into a table
const constH = 1
const constI = { constH }