# Instruction `WRITE`

~~~admonish summary
```raptor
WRITE [<file-options>] <value> <path>
```
~~~

```admonish tip
See the section on [file options](/file-options.md).
```

The `WRITE` instruction writes a fixed string to the given path.

A file can be added to the build output with `COPY`, but sometimes we just need
to write a short value, and `COPY` might feel like overkill.

Using `WRITE`, we can put values directly into files:

```raptor
WRITE "hello world" hello.txt
```

~~~admonish tip
Be aware that `WRITE` does not add a newline at the end of your input.

For text files, it is almost always preferred to end with a newline.

To do this, add `\n` at the end of the quoted string:

```raptor
WRITE "hello world\n" hello.txt
```
~~~

The same file options as `COPY` and `RENDER` are accepted:

```raptor
# make sure /etc/hostname is world-readable
WRITE --chmod 0644 "heart-of-gold\n" /etc/hostname

# this private file should only be readable by "service"
WRITE --chmod 0600 --chown service:root "SECRET-API-TOKEN" /etc/some-service/token.conf
```
