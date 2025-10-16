# Instruction `WRITE`

```raptor
WRITE [<file-options>] <value> <path>
```

The `WRITE` instruction writes a fixed string to the given path.

A file can be added to the build output with `COPY`, but sometimes we just need
to write a short value, and `COPY` might like overkill.

Using `WRITE`, we can put values into files:

```raptor
WRITE "hello world" hello.txt
```

The same file options as `COPY` and `RENDER` are accepted:

```raptor
WRITE --chmod 0600 --chown service:root "API-TOKEN" /etc/service/token.conf
```
