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
|-----------------------|-----------------------------------------------|------------|
| `MOUNT --simple ...`  | Mounts a directory from the host (default)    | Read/write |
| `MOUNT --file ...`    | Mounts a single file from the host            | Read/write |
| `MOUNT --layers ...`  | Mounts a view of set of layers as directories | Read-only  |
| `MOUNT --overlay ...` | Mounts a view of the sum of a set of layers   | Read-only  |

~~~admonish tip
If no mount type is specified, `--simple` is implied as the default.

For clarity, it is recommended to always specify a mount type.
~~~

## Type `--simple`

This is the default mount type.

A `--simple` mount will mount a directory from the host into the
container. Docker users are likely to be familiar with this concept.

### Example

~~~admonish note title="file-lister.rapt"
```raptor
{{#include ../example/file-lister.rapt}}
```
~~~

This container can be run, to provide a file listing on the mounted directory:

```sh
$ sudo raptor run file-lister -M list /tmp
... <"ls" output would be shown here> ...
```

## Type `--file`

This mount type requires that a single file is mounted.

~~~admonish tip
When running a raptor container with a `--file` mount, the target file will be created if it does not exist.
~~~

### Example

Let us expand on the earlier example, to make the file lister provide output to a file.

~~~admonish note title="file-lister-output.rapt"
```raptor
{{#include ../example/file-lister-output.rapt}}
```
~~~

Now that we have named the mounts `input` and `output`, we can use the
[shorthand notation](/inst/mount.md#admonition-tip-1) for convenience:

```sh
$ sudo raptor run file-lister-output -I /etc -O /tmp/filelist.txt
...
$ sudo cat /tmp/filelist.txt
... <"ls" output would be shown here> ...
```

The above example would generate a file listing of `/etc` **from the host**, and
place it in `/tmp/filelist.txt`. However, the execution of `ls` takes place in
the container.

## Type `--layers`

The `--simple` and `--file` mount types both present content from the host
filesystem inside the container, and both have equivalents in Docker.

This is not the case for `--layers`, which is specific to Raptor.

When using a `--layers` mount, the input argument when running Raptor is not a
file path, but a set of *Raptor build targets*.

Let us take a look at a target, that lists the files in a `--layers` mounts:

~~~admonish note title="layers-lister.rapt"
```raptor
{{#include ../example/layers-lister.rapt}}
```
~~~

Let's try running this, using the previous `file-lister.rapt` target as input:

```sh
$ sudo raptor run layers-lister -I file-lister
total 12
drwxr-xr-x  2 root root 4096 Oct 17 13:37 file-lister-16689594BA5D2989
drwxr-xr-x 17 root root 4096 Sep 29 00:00 index.docker.io-library-debian-trixie-675DE2C3A4D8CD82
-rw-r--r--  1 root root  200 Oct 17 13:37 raptor.json
```

We see that each layer in the input is available as a directory, but also a
`raptor.json` file.

Think of this file as a manifest of the contents in the `--layers` mount. It
contains useful metadata about the inputs, including which targets have been
specified, as well as the stacking order for each target:

~~~admonish note title="raptor.json"
```json
{
  "targets": [
    "file-lister"
  ],
  "layers": {
    "file-lister": [
      "index.docker.io-library-debian-trixie-675DE2C3A4D8CD82",
      "file-lister-16689594BA5D2989"
    ]
  }
}
```
~~~

At first, this might seem like overly complicated. If we just needed the layer
order, surely a simple text file would suffice?

In fact, *multiple* build targets can be specified at the same time:

```sh
$ sudo raptor run layers-lister -I file-lister -I file-lister-output
total 16
drwxr-xr-x  2 root root 4096 Oct 17 11:33 file-lister-16689594BA5D2989
drwxr-xr-x  2 root root 4096 Oct 17 11:31 file-lister-output-2CFDE4FEBD507157
drwxr-xr-x 17 root root 4096 Sep 29 00:00 index.docker.io-library-debian-trixie-675DE2C3A4D8CD82
-rw-r--r--  1 root root  381 Oct 17 11:45 raptor.json
```

We now have multiple build targets, and the Docker layer is shared between both
inputs.

However, we can still make sense of this using `raptor.json`:

~~~admonish note title="raptor.json"
```json
{
  "targets": [
    "file-lister",
    "file-lister-output"
  ],
  "layers": {
    "file-lister-output": [
      "index.docker.io-library-debian-trixie-675DE2C3A4D8CD82",
      "file-lister-output-2CFDE4FEBD507157"
    ],
    "file-lister": [
      "index.docker.io-library-debian-trixie-675DE2C3A4D8CD82",
      "file-lister-16689594BA5D2989"
    ]
  }
}
```
~~~

This mount type is useful for any build container that needs to work the
contents of individual layers.

For example, the `deblive` builder[^deblive] uses this mount type, in order to
build Debian liveboot images.

[^deblive]: From the [raptor-build](https://github.com/chrivers/raptor-build)
    companion project.

## Type `--overlay`

Like `--layers`, this is a Raptor-specific mount type.

Raptor targets are almost always built from a set of layers (i.e., there's at
least one `FROM` instruction).

When running a raptor container, this stack of layers is combined using
`overlayfs`, which makes the kernel present them to the container as a single,
unified filesystem.

For build containers that need to operate on this combined view of a build
target, the `--overlay` mount type is available.

For example, the `disk-image` builder[^deblive] uses this mount type, in order to
build disk images for virtual (or physical) machines.

~~~admonish note title="overlay-lister.rapt"
```raptor
{{#include ../example/overlay-lister.rapt}}
```
~~~

```sh
$ sudo raptor run overlay-lister -I file-lister
total 60
lrwxrwxrwx  1 root root    7 Aug 24 16:20 bin -> usr/bin
drwxr-xr-x  2 root root 4096 Aug 24 16:20 boot
drwxr-xr-x  2 root root 4096 Sep 29 00:00 dev
drwxr-xr-x 27 root root 4096 Sep 29 00:00 etc
drwxr-xr-x  2 root root 4096 Aug 24 16:20 home
lrwxrwxrwx  1 root root    7 Aug 24 16:20 lib -> usr/lib
lrwxrwxrwx  1 root root    9 Aug 24 16:20 lib64 -> usr/lib64
drwxr-xr-x  2 root root 4096 Sep 29 00:00 media
drwxr-xr-x  2 root root 4096 Sep 29 00:00 mnt
drwxr-xr-x  2 root root 4096 Sep 29 00:00 opt
drwxr-xr-x  2 root root 4096 Sep 29 00:00 proc
drwx------  2 root root 4096 Sep 29 00:00 root
drwxr-xr-x  3 root root 4096 Sep 29 00:00 run
lrwxrwxrwx  1 root root    8 Aug 24 16:20 sbin -> usr/sbin
drwxr-xr-x  2 root root 4096 Sep 29 00:00 srv
drwxr-xr-x  2 root root 4096 Aug 24 16:20 sys
drwxrwxrwt  2 root root 4096 Sep 29 00:00 tmp
drwxr-xr-x 12 root root 4096 Sep 29 00:00 usr
drwxr-xr-x 11 root root 4096 Sep 29 00:00 var
```
