## Mount type `--simple`

This is the default mount type.

A `--simple` mount will mount a directory from the host into the
container. Docker users are likely to be familiar with this concept.

### Example

~~~admonish note title="file-lister.rapt"
```raptor
{{#include ../../example/file-lister.rapt}}
```
~~~

This container can be run, to provide a file listing on the mounted directory:

```sh
$ sudo raptor run file-lister -M list /tmp
... <"ls" output would be shown here> ...
```
