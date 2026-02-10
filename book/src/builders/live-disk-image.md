# Debian Liveboot Disk Image generator:<br>**`live-disk-image`**

~~~admonish important
**This builder requires an input from the Debian family.**

It should work for Debian derivatives (Ubuntu, etc), as long as the
prerequisite packages are installed.
~~~

| Mount name | Type   | Usage                                                                                  |
|:-----------|:-------|:---------------------------------------------------------------------------------------|
| `cache`    | Simple | Contains cache of previously built `.squashfs` files.<br>(to avoid expensive rebuilds) |
| `input`    | Layers | The Raptor build target(s) that will be put on the generated image                     |
| `output`   | File   | Points to the resulting output file.                                                   |

## Compatibility

| Target                      | Compatible?         |
|:----------------------------|:--------------------|
| Container: `systemd-nspawn` | ❌                  |
| Container: `portablectl`    | ❌                  |
| Container: `docker`         | ❌                  |
| Container: `podman`         | ❌                  |
| Virtual Machine (UEFI)      | ✅ (`raw`, `qcow2`) |
| Virtual Machine (BIOS)      | ❌                  |
| Physical Machine (UEFI)     | ✅ (`raw`)          |
| Physical Machine (BIOS)     | ❌                  |

This builder also generates Debian Liveboot image, but instead of generating a
`.iso` file, it generates a disk image, including a partition table, and
separate partitions for `/`, `/boot` and `/boot/efi`.

The result is a disk image that allows a physical or virtual machine to boot
normally, but with the root file system mounted as `overlayfs` backed by the
`squashfs` files for each layer.

It uses the Discoverable Partitions Specification[^dps] to make the images
compatible with both physical hardware, virtual machines, and `systemd-nspawn`.

[^dps]: [Discoverable Partitions Specification](https://uapi-group.org/specifications/specs/discoverable_partitions_specification/)

## Example

Prerequisites:

 - [raptor-builders](https://github.com/chrivers/raptor-builders) is cloned to `raptor-builders`
 - An input target called `test.rapt`

~~~admonish note title="Raptor.toml"
```toml
[raptor.link]
rbuild = "raptor-builders"

# Live disk image (raw)
[run.live1]
target = "$rbuild.live-disk-image" # <-- builder is specified here
cache = "cache2"
input = ["test"]
output = "live.img"
## implied default:
## env.GRUB_TIMEOUT = "5"
## env.OUTPUT_FORMAT = "raw"

# Live disk image (qcow2, no boot delay)
[run.live2]
target = "$rbuild.live-disk-image" # <-- builder is specified here
cache = "cache2"
input = ["test"]
output = "live.qcow2"
env.GRUB_TIMEOUT = "0"
env.OUTPUT_FORMAT = "qcow2"
```
~~~

After this `Raptor.toml` is in place, call `raptor make` to build:

```sh
sudo raptor make live1 live2
```

When the process is complete, `live.img` and `live.qcow2` will be ready for use.
