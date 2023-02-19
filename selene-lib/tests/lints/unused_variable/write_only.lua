local bad1 = {}
bad1.x = 1
bad1.y = 2
bad1["z"] = 1

local badButNotYetImplemented = { x = {} }
badButNotYetImplemented.x.y = 1

local okay1 = ok()
okay1.x = 1
okay1.y = 2

local okay2 = { x = ok() }
okay2.x.y = 1

local okay3 = {}
okay3.x().y = 1
