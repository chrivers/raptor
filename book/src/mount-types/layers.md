## Mount type `--layers`

The `--simple` and `--file` mount types both present content from the host
filesystem inside the container, and both have equivalents in Docker.

This is not the case for `--layers`, which is specific to Raptor.

When using a `--layers` mount, the input argument when running Raptor is not a
file path, but a set of *Raptor build targets*.

Let us take a look at a target, that lists the files in a `--layers` mounts:

~~~admonish note title="layers-lister.rapt"
```raptor
{{#include ../../example/layers-lister.rapt}}
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
