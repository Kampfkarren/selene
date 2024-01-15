local React

local function Component()
    local shouldNotIgnoreState, shouldIgnoreSetState = React.useState()
    local shouldIgnoreBinding, shouldIgnoreSetBinding = React.useBinding()
    local shouldIgnoreRef = React.useRef()

    React.useEffect(function()
        print(shouldNotIgnoreState, shouldIgnoreSetState)
        print(shouldIgnoreBinding, shouldIgnoreSetBinding)
        print(shouldIgnoreRef)
    end, {})

    React.useEffect(function()
        print(shouldIgnoreSetState)
        -- Even though passing known stable variables isn't needed, it shouldn't warn about it
    end, { shouldIgnoreSetState })
end
