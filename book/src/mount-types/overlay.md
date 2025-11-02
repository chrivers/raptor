## Mount type `--overlay`

Like `--layers`, this is a Raptor-specific mount type.

Raptor targets are almost always built from a set of layers (i.e., there's at
least one `FROM` instruction).

When running a raptor container, this stack of layers is combined using
`overlayfs`, which makes the kernel present them to the container as a single,
unified filesystem.

For build containers that need to operate on this combined view of a build
target, the `--overlay` mount type is available.

For example, the [`disk-image` builder](../builders/disk-image.md) uses this mount type, in order to
build disk images for virtual (or physical) machines.

~~~admonish note title="overlay-lister.rapt"
```raptor
{{#include ../../example/overlay-lister.rapt}}
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
