# Mount types

```admonish tip
For an introduction to mounts, see the [MOUNT](/inst/mount.md) referrence.
```

When running containers, Raptor supports mounting various file resources into
the container namespace.

In the simplest form, this means presenting a directory from the host
environment, at a specified location in the container.

This is equivalent to how Docker uses volume mounts (`-v` / `--volume`).

However, Raptor supports more than just mounting directories:

| Type                  | Description                                   | Access     |
|:----------------------|:----------------------------------------------|:-----------|
| `MOUNT --simple ...`  | Mounts a directory from the host (default)    | Read/write |
| `MOUNT --file ...`    | Mounts a single file from the host            | Read/write |
| `MOUNT --layers ...`  | Mounts a view of set of layers as directories | Read-only  |
| `MOUNT --overlay ...` | Mounts a view of the sum of a set of layers   | Read-only  |

~~~admonish tip
If no mount type is specified, `--simple` is implied as the default.

For clarity, it is recommended to always specify a mount type.
~~~

Read more about the specific mount types:

  - [`--simple`](mount-types/simple.md)
  - [`--file`](mount-types/file.md)
  - [`--layers`](mount-types/layers.md)
  - [`--overlay`](mount-types/overlay.md)

## Options

All mount types take two optional arguments:

| Argument      | Description                                                                                        |
|:--------------|:---------------------------------------------------------------------------------------------------|
| `--readonly`  | Mount the file or directory read-only.<br>(for `--layers`/`--overlay` this is **always implied**). |
| `--readwrite` | Mount the file or directory read-write.<br>(for `--layers`/`--overlay` this is **invalid**).       |
| `--optional`  | Allow starting the container, **even if** this mount is **not specified**.                         |
| `--required`  | Deny starting the container, **unless** this mount **is specified**.                               |

## Examples

```raptor
# Both options can be combined:
MOUNT --readonly --optional --simple input /input

# Optional layers mount:
MOUNT --optional --layers input /data

# Later --required overrides previous --optional:
MOUNT --optional --required --layers input /data
```
