![Raptor headline logo](book/src/images/logo-title.png)

# Raptor

Raptor is a modern, fast, and easy-to-use system for building disk images,
bootable isos, containers and much more - all from a simple, Dockerfile-inspired
syntax.

It uses `systemd-nspawn` for sandboxing when building or running containers.

> [!TIP]
> 📕 For more information, [read the raptor book](https://chrivers.github.io/raptor/)

## What it looks like

Raptor uses a syntax similar to `Dockerfile`. Statements start with uppercase
keywords, and are terminated by end of line.

All lines starting with `#` are treated as comments:

```Dockerfile
# Start from a well-known docker image
FROM docker://debian:trixie

# Set the hostname
WRITE "example-host\n" /etc/hostname

# Create app directory
MKDIR -p /app/bin

# This copies "program" from the host to "/app/bin" inside the build target
COPY program /app/bin/program
```

## What it can do

> [!TIP]
> 📕 For more information, [read the raptor book](https://chrivers.github.io/raptor/)

Raptor builds *layers*, much in the same way as Docker.

However, this is where the similarities end! Raptor is able to run build
processes on top of finished layers, to produce any kind of desired output.

The companion project [raptor-builders](https://github.com/chrivers/raptor-builders) can create:

 - Debian Live Boot iso files
 - Disk images for virtual (or physical) machines

## Example: Building a bootable iso

After [installing Raptor](http://chrivers.github.io/raptor/intro/install.html), create a file called `base.rapt`:

```Dockerfile
# Start from a docker iso
FROM docker://debian:trixie

# Set root password to "raptor"
RUN usermod -p "$1$GQf2tS9s$vu72NbrDtUcvvqnyAogrH0" root

# Update package sources, and install packages
RUN apt-get update
RUN apt-get install -qy systemd-sysv live-boot linux-image-amd64
```

Then clone the `raptor-builders` project, which has the build container for making Debian Live Boot images:

```sh
git clone https://github.com/chrivers/raptor-builders.git
```

Then run the `deblive` container from `raptor-builders`, using the `base(.rapt)` we just made:

```sh
# Create cache dir (used in `-C` option)
mkdir /tmp/raptor-cache

# Run the `deblive` builder from `raptor-builders`
sudo raptor run \
    '$rbuild.deblive' \
    -L rbuild raptor-builders \
    -C /tmp/raptor-cache \
    -I base \
    -O liveboot.iso
```

After this step, the file `liveboot.iso` is ready to use. We can try it out with QEMU:

```sh
qemu-system-x86_64 -enable-kvm -cpu host -m 4G -cdrom liveboot.iso
```

> [!TIP]
> 📕 The whole process is described in [much more detail in the book!](https://chrivers.github.io/raptor/walkthrough/debian/).

## Need help?

The 📕 [Raptor Book](https://chrivers.github.io/raptor/) contains a lot more
information, including a thorough description of all instructions, features, and
a grammar for the language itself.

## License

Raptor is Free Software, licensed under the GNU GPL-3.0.

The [Raptor icon](book/branding/raptor-logo.svg) is derived from a [Creative Commons Attribution](https://www.freepik.com/icon/eagle_17553500)-licensed icon.
