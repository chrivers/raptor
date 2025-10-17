# Generating an iso file

In the [last chapter](build.md) we built some raptor layers that serve as the file
content for the bootable iso we want to build.

These layers are very similar to layers in a docker file[^layers].

After building `ssh` (which implies the `base` target), their respective
contents can be found in `layers/...`.

Once built like this, it's possible to "run" the container, like so:

```sh
$ sudo raptor run ssh id
[*] Completed [675DE2C3A4D8CD82] index.docker.io-library-debian-trixie
[*] Completed [AB1DD71BFD07718B] base
[*] Completed [80E4F4E5B0E2F6BA] ssh
uid=0(root) gid=0(root) groups=0(root)
```

Instead of `id`, an interactive command like `sh` or `bash` could be
specified. This is very similar to how `docker run` works, except raptor
containers are ephemeral by default. If no further options are specified, any
changes made to the container will only be saved while the container is running.

This is all well and good, but we are interested in building a bootable iso from
the layers, not just running them as a container.

It turns out that having a stack of layers available, is quite a powerful
primitive. With the right commands, we could build all kinds of output from
them; a `.tar.gz` file with a release archive, a bootable iso, or even a disk
image for a machine.

In other words, the same input layers can be post-processed into a wide
variety of output layers, depending on what you need.

To help get started with this, Raptor provides a standard set of build
containers, in the companion project
[raptor-build](https://github.com/chrivers/raptor-build):

 - [`deblive`] Builds Debian liveboot isos from Raptor layers.
 - [`disk-image`] Builds disk images (for virtual or physical machines) from Raptor layers.

~~~admonish note title="Build containers"
Raptor build containers are just regular Raptor containers, that are
configured to expect an input, and produce an output.

The containers from the `raptor-build` project are available to anyone,
but are not "built in" to Raptor - they don't use any special features
or private APIs. Anyone can make build containers that are similar (or
even identical to) `raptor-build`.
~~~

Since this walkthrough is focused on making a liveboot iso, we'll use the
`deblive` builder to convert layers to an iso file.

First, we need to clone (or otherwise download) the `raptor-build` project:

```sh
git clone https://github.com/chrivers/raptor-build
```

This builder uses mounts to access input, output and cache from outside the
container (see [`MOUNT`](/inst/mount.md)).

To build the layers from the last step into a debian liveboot iso, use the
following command (assuming `raptor-build` is checked out next to the directory
containing `ssh.rapt`):

```sh
sudo raptor run             \
  -L book   book/example    \
  -L rbuild ../raptor-build \
  -C liveboot-cache         \
  -I '$book.ssh'            \
  -O custom-liveboot.iso    \
  '$rbuild.deblive'
```

~~~admonish tip
The command above uses the short-form command line options typically used
interactively. Long-form options are also available, if greater clarity is
desired (e.g. for scripting purposes).

```sh
sudo raptor run                 \
  --link book   book/example    \
  --link rbuild ../raptor-build \
  --cache liveboot-cache        \
  --input '$book.ssh'           \
  --output custom-liveboot.iso  \
  '$rbuild.deblive'
```
~~~

(See the section on [module names](/module-name.md) to learn more about `--link`
and the `$rbuild` notation.)

This build process works, but the required command is fairly long and
complicated.

In the [next chapter](make.md), we'll take a look at raptor-make, and how it can
simplify the process.

[^layers]: There's a key difference between how Raptor and Docker handles
    layers. In Docker, each *command* (`RUN`, `COPY`, etc) creates a new
    layer. In Raptor, each `.rapt` file forms a new layer.
