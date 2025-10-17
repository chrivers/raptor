# Instruction `MOUNT`

~~~admonish summary
```raptor
MOUNT [--<mount-type>] <name> <destination>
```
~~~

```admonish important title="Build-time instruction"
The `MOUNT` instructions only affects *running* a container, not *building* a
container.
```

By using `MOUNT`, targets can specify certain resources (files, directories,
layers, etc) that should be made available when running the container.

Raptor mounts are identified by a `name`, which is used when running the
container, to declare what to mount.

When running a raptor container, a mount input is specified with the `-M <name>
<source>` command line option.

~~~admonish tip
The syntax `-M <input> <source>` can be a bit unwieldy.

Since certain mount names are very common, they have a shorter syntax available:

| Name   | Long form         | Short form |
|--------|-------------------|------------|
| Input  | `-M input <foo>`  | `-I <foo>` |
| Output | `-M output <foo>` | `-O <foo>` |
| Cache  | `-M cache <foo>`  | `-C <foo>` |
~~~

For example, suppose we have a simple container (`disk-usage.rapt`) that just
calculates the disk space used by a mount:

```raptor
FROM docker://debian:trixie

# Declare the mount "input", and place it at /input
MOUNT input /input

# Calculate the disk space used in /input
CMD "du -sh /input"
```

If we try to run this, we will get an error message:

```sh
$ sudo raptor run disk-usage
[*] Completed [675DE2C3A4D8CD82] index.docker.io-library-debian-trixie
[*] Completed [A24E97B01374CFEF] disk-usage
[E] Raptor failed: Required mount [input] not specified
```

As we can see, the container builds correctly, but fails because the `input`
mount is not specified.

To fix this, we specify the input mount:

```sh
sudo raptor run -I /tmp disk-usage
[*] Completed [675DE2C3A4D8CD82] index.docker.io-library-debian-trixie
[*] Completed [4BDD0649E00CA728] disk-usage
128M    /input
```

We could have specified `-I /tmp` as `-M input /tmp`, but the short form usually
makes the command easier to follow.
