# Instruction `ENV`

```raptor
ENV <key>=<value> [...<key=value>]
```

The `ENV` instruction sets one or more environment variables inside the build namespace.


> [!IMPORTANT]
>
> The `ENV` instructions only affects *building* a container, not *running* a
> container.

Example:

```raptor
ENV CFLAGS="--with-sprinkles"
ENV API_TOKEN="acbd18db4cc2f85cedef654fccc4a4d8" API_USER="user@example.org"
```

The `ENV` instruction affects all instructions that come after it, in the same layer.

It is possible to overwrite a value that has been set previously:

```raptor
FROM docker://busybox

ENV HITCHHIKER="Arthur Dent"
RUN sh -c "echo $HITCHHIKER" # outputs "Arthur Dent"

ENV HITCHHIKER="Ford Prefect"
RUN sh -c "echo $HITCHHIKER" # outputs "Ford Prefect"
```
