# Fullscale Flight Computer
---
Software powering the BeagleBone Black flight computer onboard Fullscale.

## Installation
---
The Fullscale flight computer will primarily be running Rust code. As such, it is necessary to install Rust along with its compiler (rustc) and package/project manager (Cargo). This can be done by pasting the following snippet into a command prompt:

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

As for how to use Rust and its accompanying tools, it is highly recommended to look through the [Rust Setup Nuclino](https://app.nuclino.com/YJSP/YJSP/Rust-Setup-f5ec005b-cc58-4ce3-ae1d-6531cef71db1). This should give a basic overview of how to get started.

Additionally, this source code will be developed in and for a Linux system. Therefore, it is essential to either install a Linux distribution or develop in a WSL (Windows Subsystem for Linux).

## Running
---
Fullscale's flight computer will be cross-compiled. This means that the product will be compiled on development computers and the binary will subsequently be transfered to the flight computer. So, a tool called Cross should be used when compiling the software:

`cargo install -f cross`

Use this command to actually compile to the correct architecture:

`cross build --target armv7-unknown-linux-gnueabihf`

The output binary will be placed into ./target/armv7-unknown-linux-gnueabihf/fc. Run this.

## IDE Setup (VSCode)
---
Install the [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension. This is the main extension for everything Rust.

You can also go to the `rust-analyzer.checkOnSave.command` extension setting and change the default "check" to "clippy." This will automatically run Rust's clippy linter when the file is saved.

## Debugging
---
Rust should come with a version of gdb compatible with Rust. It runs just as gdb does with any other language:

`rust-gdb executable_file`
