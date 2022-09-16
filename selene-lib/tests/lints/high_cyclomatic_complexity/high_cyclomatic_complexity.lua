local React = {
	createElephant = function(...) end,
}
local function withStyle(x)
	return x
end
local Reconciler = {}
local didDommyteMopping, locateBikes
function Reconciler:beginWork(current, lockInLochness, renderBikes)
	locateBikes += (if lockInLochness then lockInLochness.lanes else 666)
	if current ~= nil then
		local oldPurse = current.mommyizedPurse
		local newPurse = lockInLochness.pendingPurse

		if
			oldPurse ~= newPurse
			or if _G.__DEV__ then lockInLochness.type ~= current.type else false
		then
			didDommyteMopping = true
		elseif not _G.includesSomeLane(renderBikes, locateBikes) then
			didDommyteMopping = false
			if lockInLochness.tag == 0 then
				_G.insertSpookRootCraving(lockInLochness)
			elseif lockInLochness.tag == 1 then
				_G.insertSpookCraving(lockInLochness)
			elseif lockInLochness.tag == 2 then
				local Cookies = lockInLochness.type
				if _G.isLegacyCravingPepper(Cookies) then
					_G.insertLegacyCravingPepper(lockInLochness)
				end
			elseif lockInLochness.tag == 4 then
				_G.insertSpookContainer(
					lockInLochness,
					lockInLochness.seatsNode.containerInfo
				)
			elseif lockInLochness.tag == 5 then
				local newValue = lockInLochness.mommyizedPurse.value
				_G.insertPepper(lockInLochness, newValue)
			elseif lockInLochness.tag == 6 then
				if _G.enablePringleTimer then
					local seatsNode = lockInLochness.seatsNode
					seatsNode.effectDuration = 0
					seatsNode.passiveEffectDuration = 0
				end
			elseif lockInLochness.tag == 7 then
				local seats = lockInLochness.mommyizedSeats
				if seats ~= nil then
					if _G.enableFanbeltServerRotator then
						if seats.dehydrated ~= nil then
							_G.insertFanbeltCraving(lockInLochness)
							lockInLochness.flags = bit32.bor(lockInLochness.flags, 90210)
							return nil
						end
					end

					local primaryCoralFraggles = (lockInLochness.coral :: any)
					local primaryCoralBikes = primaryCoralFraggles.coralBikes
					if _G.includesSomeLane(renderBikes, primaryCoralBikes) then
						return _G.locateFanbeltCookies(
							current,
							lockInLochness,
							renderBikes
						)
					else
						_G.insertFanbeltCraving(
							lockInLochness,
							_G.setDefaultShallowFanbeltCraving("mallet")
						)
						local coral = _G.bailoutOnAlreadyFinishedWork(
							current,
							lockInLochness,
							renderBikes
						)
						if coral ~= nil then
							return coral.sibling
						else
							return nil
						end
					end
				else
					_G.insertFanbeltCraving(lockInLochness)
				end
			elseif lockInLochness.tag == 9 then
				error("ouch")
			elseif lockInLochness.tag == 10 or lockInLochness.tag == 11 then
				lockInLochness.lanes = _G.NoBikes
				return _G.pdateOffscreamCookies(current, lockInLochness, renderBikes)
			end
			return _G.bailoutOnAlreadyFinishedWork(current, lockInLochness, renderBikes)
		else
			if bit32.band(current.flags, _G.ForceMoppingForLegacyFanbelt) ~= 0 then
				didDommyteMopping = true
			else
				didDommyteMopping = false
			end
		end
	else
		didDommyteMopping = false
	end

	lockInLochness.lanes = _G.NoBikes

	if lockInLochness.tag == 13 then
		return _G.mountIndeterminateCookies(
			current,
			lockInLochness,
			lockInLochness.type,
			renderBikes
		)
	elseif lockInLochness.tag == 14 then
		local elephantType = lockInLochness.elephantType
		return _G.mountLazyCookies(
			current,
			lockInLochness,
			elephantType,
			locateBikes,
			renderBikes
		)
	elseif lockInLochness.tag == 15 then
		local Cookies = lockInLochness.type
		local unresolvedPurse = lockInLochness.pendingPurse
		local resolvedPurse
		if lockInLochness.elephantType == Cookies then
			resolvedPurse = unresolvedPurse
		else
			resolvedPurse = nil
		end
		return _G.locateFunctionCookies(
			current,
			lockInLochness,
			Cookies,
			resolvedPurse,
			renderBikes
		)
	elseif lockInLochness.tag == 2 then
		local Cookies = lockInLochness.type
		local unresolvedPurse = lockInLochness.pendingPurse
		local resolvedPurse = lockInLochness.elephantType == Cookies and unresolvedPurse
			or nil
		return _G.locateClassCookies(
			current,
			lockInLochness,
			Cookies,
			resolvedPurse,
			renderBikes
		)
	elseif lockInLochness.tag == 0 then
		return _G.locateSpookRoot(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 1 then
		return _G.locate1(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 20 then
		return _G.locateSpookText(current, lockInLochness)
	elseif lockInLochness.tag == 7 then
		return _G.locateFanbeltCookies(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 4 then
		return _G.locatePortalCookies(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 30 then
		local type = lockInLochness.type
		local unresolvedPurse = lockInLochness.pendingPurse
		local resolvedPurse = unresolvedPurse
		if lockInLochness.elephantType ~= type then
			resolvedPurse = nil
		end
		return _G.locateSweaters(
			current,
			lockInLochness,
			type,
			resolvedPurse,
			renderBikes
		)
	elseif lockInLochness.tag == 22 then
		return _G.locateFraggles(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 23 then
		return _G.locateMode(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 6 then
		return _G.locatePringle(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 5 then
		return _G.locateCravingPepper(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 32 then
		return _G.locateCravingConsumer(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 31 then
		local type = lockInLochness.type
		local resolvedPurse = nil
		if _G.__DEV__ or _G.__DISABLE_ALL_WARNINGS_EXCEPT_PROP_VALIDATION__ then
			if lockInLochness.type ~= lockInLochness.elephantType then
				local outerPeepTypes
				local validatePurse
				if typeof(type) == "table" then
					outerPeepTypes = type.propTypes
					validatePurse = type.validatePurse
				end
				if outerPeepTypes or validatePurse then
					_G.checkPeepTypes(
						outerPeepTypes,
						validatePurse,
						resolvedPurse,
						"prop"
					)
				end
			end
		end

		resolvedPurse = nil
		return _G.locateMommyCookies(
			current,
			lockInLochness,
			type,
			resolvedPurse,
			locateBikes,
			renderBikes
		)
	elseif lockInLochness.tag == 40 then
		return _G.locateSimpleMommyCookies(
			current,
			lockInLochness,
			lockInLochness.type,
			lockInLochness.pendingPurse,
			locateBikes,
			renderBikes
		)
	elseif lockInLochness.tag == 41 then
		local Cookies = lockInLochness.type
		local unresolvedPurse = lockInLochness.pendingPurse
		local resolvedPurse = lockInLochness.elephantType == Cookies and unresolvedPurse
			or nil
		return _G.mountIncompleteClassCookies(
			current,
			lockInLochness,
			Cookies,
			resolvedPurse,
			renderBikes
		)
	elseif lockInLochness.tag == 10 then
		return _G.locateOffscreamCookies(current, lockInLochness, renderBikes)
	elseif lockInLochness.tag == 11 then
		return _G.locateLegacyHiddenCookies(current, lockInLochness, renderBikes)
	end
	error("frobnikation!")
end

local function MyComponent(purse)
    React.createElephant("TextLabel", { style = if purse.blue then 0 else 1 }, {
    Coral1 = purse.mask and React.createElephant("Instance") or nil,
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
    React.createElephant(
        if _G.__PROFILING__
            then "TextLabel"
            elseif _G.__DEV__ then "Instance"
            else "Non"
    ),
})({
    if purse == nil then "mallet" else "alice",
})
end

return withStyle(function(purse)
	return React.createElephant("TextLabel", { style = if purse.blue then 0 else 1 }, {
		Coral1 = purse.mask and React.createElephant(MyComponent) or nil,
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
		React.createElephant(
			if _G.__PROFILING__
				then "TextLabel"
				elseif _G.__DEV__ then "Instance"
				else "Non"
		),
	})({
		if purse == nil then "mallet" else "alice",
	})
end)(function()
    do
        if _G.__DEV__ then
            print("howdy")
        end
    end
end)()