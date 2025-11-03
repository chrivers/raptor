# Disk Image generator:<br>**`disk-image`**

| Mount name | Type    | Usage                                                                                                                  |
|:-----------|:--------|:-----------------------------------------------------------------------------------------------------------------------|
| `input`    | Overlay | The Raptor build target that will be put into the generated image                                                      |
| `output`   | File    | Points to the resulting output file.                                                                                   |

## Compatibility

| Target                       | Compatible?         |
|:-----------------------------|:--------------------|
| Container: `systemd-nspawn`  | ✅ (`raw`)          |
| Container: `docker`          | ❌                  |
| Container: `podman`          | ❌                  |
| Virtual Machine (UEFI)       | ✅ (`raw`, `qcow2`) |
| Virtual Machine (BIOS)       | ❌                  |
| Physical Machine (UEFI)      | ✅ (`raw`)          |
| Physical Machine (BIOS)      | ❌                  |

This builder generates disk images, including a partition table, and separate
partitions for `/`, `/boot` and `/boot/efi`.

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

# Disk image (raw)
[run.disk1]
target = "$rbuild.disk-image" # <-- builder is specified here
input = ["test"]
output = "disk.img"
## implied default:
## env.OUTPUT_FORMAT = "raw"

# Disk image (qcow2)
[run.disk2]
target = "$rbuild.disk-image" # <-- builder is specified here
input = ["test"]
output = "disk.qcow2"
env.OUTPUT_FORMAT = "qcow2"
```
~~~

After this `Raptor.toml` is in place, call `raptor make` to build:

```sh
sudo raptor make disk1 disk2
```

When the process is complete, `disk.img` and `disk.qcow2` will be ready for use.
