# Instruction `WORKDIR`

```raptor
WORKDIR <path>
```

The `WORKDIR` instruction changes the current working directory inside the build
namespace. This affects all relative destination paths, as well as `RUN`:

> [!IMPORTANT]
>
> The `WORKDIR` instructions only affects *building* a container, not *running*
> a container.

```raptor
# This will copy "program" to "/bin/program" (initial directory is "/")
COPY program bin/program

WORKDIR /usr

# The same command will now copy "program" to "/usr/bin/program"
COPY program bin/program

WORKDIR /tmp

# This creates /tmp/foo
RUN /bin/sh -c "touch foo"
```
