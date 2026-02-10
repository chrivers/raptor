# Partition Image generator:<br>**`part-image`**

| Mount name | Type    | Usage                                                             |
|:-----------|:--------|:------------------------------------------------------------------|
| `input`    | Overlay | The Raptor build target that will be put into the generated image |
| `output`   | File    | Points to the resulting output file.                              |

## Compatibility

| Target                      | Compatible? |
|:----------------------------|:------------|
| Container: `systemd-nspawn` | ✅          |
| Container: `portablectl`    | ✅          |
| Container: `docker`         | ❌          |
| Container: `podman`         | ❌          |
| Virtual Machine (UEFI)      | ❌          |
| Virtual Machine (BIOS)      | ❌          |
| Physical Machine (UEFI)     | ❌          |
| Physical Machine (BIOS)     | ❌          |

This builder generates partition images, containing a single filesystem.

In other words, this image can be **put into** a partition, but **does not**
contain a *partition table*.

The primary use case is for building `systemd-nspawn` images, but this builder
is also useful in other specialty applications -- for example,
[efibootguard](https://github.com/siemens/efibootguard), a bootloader that
supports A/B partitions for upgrade and recovery.

Since efibootguard uses [Unified Kernel
Images](https://github.com/siemens/efibootguard/blob/master/docs/UNIFIED-KERNEL.md),
an update image typically consists of just a single filesystem. Such an image
could be produced by this builder.

## Example

Prerequisites:

 - [raptor-builders](https://github.com/chrivers/raptor-builders) is cloned to `raptor-builders`
 - An input target called `test.rapt`

~~~admonish note title="Raptor.toml"
```toml
[raptor.link]
rbuild = "raptor-builders"

# Partition (filesystem) image
[run.part1]
target = "$rbuild.part-image" # <-- builder is specified here
input = "test"
output = "part.img"
```
~~~

After this `Raptor.toml` is in place, call `raptor make` to build:

```sh
sudo raptor make part1
```

When the process is complete, `part.img` will be ready for use.
