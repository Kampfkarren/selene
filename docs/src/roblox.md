# Roblox Guide

selene is built with Roblox development in mind, and has special features for Roblox developers.

If you try to run selene on a Roblox codebase, you'll get a bunch of errors saying things such as "`game` is not defined". This is because these are Roblox specific globals that selene does not know about. You'll need to install the Roblox [standard library](./usage/configuration) in order to fix these issues, as well as get Roblox specific lints.

## Installation

Thankfully, this process is very simple. All you need to do is edit your `selene.toml` (or create one) and add the following:

```toml
std = "roblox"
```

The next time you run selene, or if you use the Visual Studio Code extension and start typing Lua code, a Roblox standard library will be automatically generated and used. This is an automatic process that occurs whenever you don't have a cached standard library file and your `selene.toml` has `std = "roblox"`.

## Updating definitions

The Roblox standard library file is updated automatically every 6 hours. If you need an update faster than that, you can run `selene update-roblox-std` manually.

## TestEZ Support

Roblox has provided an open source testing utility called [TestEZ](https://roblox.github.io/testez/), which allows you to write unit tests for your code. Writing unit tests is good practice, but selene will get angry at you if you don't include a `testez.yml` file and set the standard library to the following:

`std = "roblox+testez"`

But first you'll need to create a `testez.yml` or `testez.yaml` file, which you can do so [with this template](https://gist.github.com/Kampfkarren/f2dddc2ebfa4e0662e44b8702e519c2d).

## Pinned standard library

There may be cases where you would rather not have selene automatically update the Roblox standard library, such as if speed is critically important and you want to limit potential internet access (generating the standard library requires an active internet connection).

selene supports "pinning" the standard library to a specific version.

Add the following to your `selene.toml` configuration:
```toml
# `floating` by default, meaning it is stored in a cache folder on your system
roblox-std-source = "pinned"
```

This will generate the standard library file into `roblox.yml` where it is run.

You can also create a `roblox.yml` file manually with `selene generate-roblox-std`.
