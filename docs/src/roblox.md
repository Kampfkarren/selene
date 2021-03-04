# Roblox Guide
selene is built with Roblox development in mind, and has special features for Roblox developers.

If you try to run selene on a Roblox codebase, you'll get a bunch of errors saying things such as "`game` is not defined". This is because these are Roblox specific globals that selene does not know about. You'll need to install the Roblox [standard library](./usage/configuration) in order to fix these issues, as well as get Roblox specific lints.

## Setup

Thankfully, this process is very simple. All you need to do is edit your `selene.toml` (or create one) and add the following:

`std = "roblox"`

The next time you run selene, or if you use the Visual Studio Code extension and start typing Lua code, a Roblox standard library will be automatically generated and used. This is an automatic process that occurs whenever you don't have a `roblox.toml` file and your selene has `std = "roblox"`.

You can also initiate this process manually with the CLI command `selene generate-roblox-std`.

Deprecated event members are not added by default. This means code such as `workspace.ChildAdded:connect(...)` will error. If you don't want to lint these, use `selene generate-roblox-std --deprecated`.

## Updating `roblox.toml`

If you're wondering why selene is providing you with outdated information regarding API that it doesn't know, you'll need to delete your `roblox.toml` and re-generate it. The Roblox standard library will be automatically updated by running selene after you've deleted `roblox.toml`.