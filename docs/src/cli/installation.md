# Installation
**selene** is written in Rust, and the recommended installation method is through the [Cargo package manager](https://doc.rust-lang.org/cargo/).

To use Cargo, you must first install Rust. [rustup.rs](https://rustup.rs/) is a tool that makes this very easy.

Once you have Rust installed, use either command:

**If you want the most stable version of selene**
```
cargo install selene
```

(Note: Currently, the above method will give you an outdated selene, as the selene crate on Cargo is not yet maintained.)

**If you want the most up to date version of selene**
```
cargo install --git https://github.com/Kampfkarren/selene selene
```

### Roblox developers
selene is built with Roblox specific lints by default. If you don't want these, type `--no-default-features` after whichever command you choose.
