## Mount type `--file`

This mount type requires that a single file is mounted.

~~~admonish tip
When running a raptor container with a `--file` mount, the target file will be created if it does not exist.
~~~

### Example

Let us expand on the earlier example, to make the file lister provide output to a file.

~~~admonish note title="file-lister-output.rapt"
```raptor
{{#include ../../example/file-lister-output.rapt}}
```
~~~

Now that we have named the mounts `input` and `output`, we can use the
[shorthand notation](../inst/mount.md#admonition-tip-1) for convenience:

```sh
$ sudo raptor run file-lister-output -I /etc -O /tmp/filelist.txt
...
$ sudo cat /tmp/filelist.txt
... <"ls" output would be shown here> ...
```

The above example would generate a file listing of `/etc` **from the host**, and
place it in `/tmp/filelist.txt`. However, the execution of `ls` takes place in
the container.
