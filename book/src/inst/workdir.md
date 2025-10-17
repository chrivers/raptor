# Instruction `WORKDIR`

~~~admonish summary
```raptor
WORKDIR <path>
```
~~~

```admonish important
The `WORKDIR` instructions only affects *building* a container, not *running*
a container.
```

The `WORKDIR` instruction changes the current working directory inside the build
namespace. This affects all subsequent relative paths, including the `RUN`
instruction.

The workdir is **not** inherited through `FROM`.

The initial workdir is always `/`.

## Example

```raptor
# This will copy "program" to "/bin/program" (since initial directory is "/")
COPY program bin/program

# This creates /foo
RUN /bin/sh -c "touch foo"



# Switch to /usr
WORKDIR /usr

# The same command will now copy "program" to "/usr/bin/program"
COPY program bin/program



# Switch to /tmp
WORKDIR /tmp

# This creates /tmp/foo
RUN /bin/sh -c "touch foo"
```
