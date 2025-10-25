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
WRITE "hostname\n" /etc/hostname
```
~~~

The same file options as `COPY` and `RENDER` are accepted:

```raptor
WRITE --chmod 0600 --chown service:root "API-TOKEN" /etc/service/token.conf
```
