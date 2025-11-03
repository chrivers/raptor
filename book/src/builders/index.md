# Raptor Builders

After carefully building a set of layers with Raptor, they are ready to process
into more useful build artifacts.

The way this is done in Raptor, is through *build containers*. These are,
themselves, Raptor containers, made specifically for producing a certain kind of
output.

Raptor itself does not come with any build containers, but the *companion
project*, [raptor-builders](https://github.com/chrivers/raptor-builders)
does. It's called the "companion" project to highlight the fact that it is not
above other Raptor projects. Anyone can make build containers, and if you need
to build something that is uncommon, or specialized to a niche situtation, you
might have to.

Most people, however, should be able to get started quickly, and save a lot of
time, by using the build containers from
[raptor-builders](https://github.com/chrivers/raptor-builders).

## Meet the builders

| Builder                                 | Output                       | Supported output formats    |
|-----------------------------------------|------------------------------|-----------------------------|
| [`deblive`](deblive.md)                 | Debian Liveboot iso          | `iso`                       |
| [`live-disk-image`](live-disk-image.md) | Debian Liveboot disk image   | `raw`, `qcow2`, `vmdk`, ... |
| [`disk-image`](disk-image.md)           | Disk image                   | `raw`, `qcow2`, `vmdk`, ... |
| [`part-image`](part-image.md)           | Partition (filesystem) image | `raw`, `qcow2`, `vmdk`, ... |
| [`docker-image`](docker-image.md)       | Docker image                 | `tar`                       |

## Compatibility

The various builders can construct a wide variety of outputs, suitable for use
with both containers (`systemd-nspawn`), virtual machines (e.g. `qemu`), and
physical hardware.

However, not all combinations are possible. For example, a physical machine will
not boot a `qcow2` image for virtual machines, but `qemu` will be able to boot
either `qcow2` or `raw` images.

The tables below provides an overview of the possible options.

Machines:

| Builder           | Format  | Virtual Machine    | Physical Machine   |
|:------------------|---------|:-------------------|:-------------------|
| `deblive`         | `iso`   | UEFI:✅ -- BIOS:✅ | UEFI:✅ -- BIOS:✅ |
| `live-disk-image` | `qcow2` | UEFI:✅ -- BIOS:❌ | UEFI:❌ -- BIOS:❌ |
| `disk-image`      | `qcow2` | UEFI:✅ -- BIOS:❌ | UEFI:❌ -- BIOS:❌ |
| `live-disk-image` | `raw`   | UEFI:✅ -- BIOS:❌ | UEFI:✅ -- BIOS:❌ |
| `disk-image`      | `raw`   | UEFI:✅ -- BIOS:❌ | UEFI:✅ -- BIOS:❌ |
| `part-image`      | `raw`   | UEFI:❌ -- BIOS:❌ | UEFI:❌ -- BIOS:❌ |
| `docker-image`    | `tar`   | UEFI:❌ -- BIOS:❌ | UEFI:❌ -- BIOS:❌ |

~~~admonish note
Currently, booting in BIOS mode is only supported by the `deblive` builder, but
the `live-disk-image` and `disk-image` builders could possibly be extended to
support this, in the future.
~~~

Containers:

| Builder           | Format  | `systemd-nspawn` | `docker` | `podman` |
|:------------------|---------|:-----------------|:---------|----------|
| `deblive`         | `iso`   | ❌               | ❌       | ❌       |
| `live-disk-image` | `qcow2` | ❌               | ❌       | ❌       |
| `disk-image`      | `qcow2` | ❌               | ❌       | ❌       |
| `live-disk-image` | `raw`   | ❌               | ❌       | ❌       |
| `disk-image`      | `raw`   | ✅               | ❌       | ❌       |
| `part-image`      | `raw`   | ✅               | ❌       | ❌       |
| `docker-image`    | `tar`   | ❌               | ✅       | ✅       |
