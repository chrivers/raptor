# Install guide

Currently, the only supported and recommended way to install raptor is by
compiling from source. Pre-built binary packages might be available in the
future.

## New user?

 1. Complete [Install prerequisites](#install-prerequisites)
 2. Complete [Install Raptor from git](#install-raptor-from-git-stable)
 3. Enjoy Raptor 🦅

## Install prerequisites

1. Install necessary packages:
   ```sh
   sudo apt-get update
   sudo apt-get install -y git musl-tools
   ```

2. Install [Rust 1.90 or greater](https://rustup.rs):
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Install `musl` target for rust:
   ```sh
   rustup target add x86_64-unknown-linux-musl
   ```

Now you are ready to install Raptor.

## Install Raptor from git \[stable\]

1. [Install prerequisites](#install-prerequisites)

2. Install `raptor` and `falcon` from git:
   ```sh
   cargo install \
       --target "x86_64-unknown-linux-musl" \
       --git "https://github.com/chrivers/raptor" \
       raptor falcon
   ```

~~~admonish warning title="Caution"
The argument `--target "x86_64-unknown-linux-musl"` is very important!

Without it, Raptor *will compile*, but Falcon (the sandbox client used for
building containers) will not be statically compiled. Since it is running inside
sandboxed environments, where glibc is not always available in the correct
version (or at all), you will get confusing errors when trying to build Raptor
targets.

**Consequence**: without `--target "x86_64-unknown-linux-musl"`, the `cargo install`
command will compile and install, but the installation *will* be broken.
~~~

## Install Raptor from git \[development branch\]

This procedure is intended for developers, beta testers, and anyone else who
want to try a specific branch build of Raptor. To test a branch named
`hypothetical`:

1. [Install prerequisites](#install-prerequisites)

2. Install `raptor` and `falcon` from git:
   ```sh
   cargo install \
       --target "x86_64-unknown-linux-musl" \
       --git "https://github.com/chrivers/raptor" \
       --branch "hypothetical" \
       raptor falcon
   ```
