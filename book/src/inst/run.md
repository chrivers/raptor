# Instruction `RUN`

```raptor
RUN <command> [...<arg>]
```

The `RUN` instruction executes the given command inside the build namespace.

```raptor
# enable the foo service
RUN systemctl enable foo.service
```

> [!IMPORTANT]
>
> Arguments are executed as-is, i.e. without shell expansion, redirection,
> piping, etc.  This ensures full control over the parsing of commands, but it
> also means normal shell syntax is not available

```raptor
# BROKEN: This will call "cat" with 3 arguments
RUN cat /etc/hostname "|" md5sum
#   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this will not work
```

> [!TIP]
>
> Instead, `/bin/sh` can be called explicitly:

```raptor
# This will produce the md5sum of /etc/hostname
RUN /bin/sh -c "cat /etc/hostname | md5sum"
```
