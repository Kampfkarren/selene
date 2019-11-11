# Roblox Guide
selene is built with Roblox development in mind, and has special features for Roblox developers.

If you try to run selene on a Roblox codebase, you'll get a bunch of errors saying things such as "`game` is not defined". This is because these are Roblox specific globals that selene does not know about. You'll need to install the Roblox [standard library](./cli/std.md) in order to fix these issues, as well as get Roblox specific lints.

Thankfully, this process is very simple. All you need to do is edit your `selene.toml` (or create one) and add the following:

`std = "roblox"`

The next time you run selene, a Roblox standard library will be automatically generated. and used.

You can also initiate this process manually with `selene generate-roblox-std`.

Deprecated event members are not added by default. This means code such as `workspace.ChildAdded:connect(...)` will error. If you don't want to lint these, use `selene generate-roblox-std --deprecated`.
