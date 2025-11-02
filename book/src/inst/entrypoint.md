# Instruction `ENTRYPOINT`

~~~admonish summary
```raptor
ENTRYPOINT <command> [...<arg>]
```
~~~

```admonish important title="Build-time instruction"
The `ENTRYPOINT` instructions only affects *running* a container, not *building* a
container.
```

This instruction sets the entrypoint for the container, which is used when
running commands in it.

The default value is `["/bin/sh", "-c"]`.

See the [`CMD` instruction](cmd.md) for details, including the relationship
between `ENTRYPOINT` and `CMD`.
