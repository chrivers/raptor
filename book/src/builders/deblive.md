# Debian Liveboot iso generator:<br>**`deblive`**

~~~admonish important
**This builder requires an input from the Debian family.**

It should work for Debian derivatives (Ubuntu, etc), as long as the
prerequisite packages are installed.
~~~

This builder generates an `iso` file suitable for live booting. All layers are
packed into read-only squashfs files, which are mounted using overlayfs, on
boot.

| Mount    | Type   | Usage                                                                                  |
|:---------|:-------|:---------------------------------------------------------------------------------------|
| `cache`  | Simple | Contains cache of previously built `.squashfs` files.<br>(to avoid expensive rebuilds) |
| `input`  | Layers | The Raptor build target(s) that will be put on the iso                                 |
| `output` | File   | Points to the resulting output file.                                                   |

~~~admonish tip
This builder has a 📕 [detailed walkthrough](../walkthrough/debian/).
~~~

![example screenshot](../images/deblive-grub.png)

If multiple targets are specified, each will get its own GRUB menu entry in the
boot menu.

The menu order will be the same as the order the `input` targets are specified
in, and the first input will be the default boot option.

## Compatibility

| Target                       | Compatible? |
|:-----------------------------|:------------|
| Container: `systemd-nspawn`  | ❌          |
| Container: `docker`          | ❌          |
| Container: `podman`          | ❌          |
| Virtual Machine (UEFI)       | ✅          |
| Virtual Machine (BIOS)       | ✅          |
| Physical Machine (UEFI)      | ✅          |
| Physical Machine (BIOS)      | ✅          |

Debian liveboot ISOs are widely compatible with both physical and virtual
machines, including UEFI and BIOS-based platforms.

The `deblive` builder is not compatible with `systemd-nspawn`, since liveboot
requires `initrd` support to perform `overlayfs` mounting.

## Example

Prerequisites:

 - [raptor-builders](https://github.com/chrivers/raptor-builders) is cloned to `raptor-builders`
 - An input target called `test.rapt`

~~~admonish note title="Raptor.toml"
```toml
[raptor.link]
rbuild = "raptor-builders"

[run.iso1]
target = "$rbuild.deblive" # <-- builder is specified here
cache = "cache2"
input = ["test"]
output = "live.iso"
env.GRUB_TIMEOUT = "0"
```
~~~

After this `Raptor.toml` is in place, call `raptor make` to build:

```sh
sudo raptor make iso1
```

When the process is complete, `live.iso` will be ready for use.
