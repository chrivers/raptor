# Building a bootable target

In order to build a raptor target that can be turned into a bootable iso, there
are a few requirements we need to consider:

 - The distribution must be Debian-based (or, preferably, Debian)
 - The package `live-boot` must be installed, since it contains the necessary
   tools and scripts
 - A linux kernel package must be installed (e.g. `linux-image-amd64`).

Here is an example of a fairly minimal base layer, which can be turned into an iso:

~~~admonish title="base.rapt"
```raptor
{{#include ../../../example/base.rapt}}
```
~~~

The kernel package can take a bit of time to install, so let's start a new layer
for further customization. This way, we don't need to rebuild the base layer
with every change we make to the upper layer:

~~~admonish title="ssh.rapt"
```raptor
{{#include ../../../example/ssh.rapt}}
```
~~~

Of course, these layers could easily be combined, but it is good to get in a
habit of separating things into reasonable layers. This improves caching and
makes builds faster, since more can be reused between builds.

Now we are ready to build the layers. Since `ssh` derives from `base` (which
derives from a docker layer), we just have to request raptor to build
`ssh`. Raptor automatically determines dependencies, and builds them as needed.

```sh
sudo raptor build ssh
```

You should now see raptor build the `ssh` layer, with all command output from
the `apt-get` commands being shown in the terminal.

Once complete, you can quickly verify that the layer is complete, by running the
same command again. This time it should very quickly display a message,
indicating that each layer is complete:

```sh
$ sudo raptor build ssh
[*] Completed [675DE2C3A4D8CD82] index.docker.io-library-debian-trixie
[*] Completed [AB1DD71BFD07718B] base
[*] Completed [80E4F4E5B0E2F6BA] ssh
```

In the [next chapter](iso.md) we will see how we can turn these layers into a
bootable iso file.
