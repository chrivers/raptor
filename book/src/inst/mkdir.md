# Instruction `MKDIR`

~~~admonish summary
```raptor
MKDIR [<file-options>] <path>
```
~~~

```admonish tip
See the section on [file options](/file-options.md).
```

The `MKDIR` instruction creates an empty directory inside the build target.

This is roughly equivalent to the following command:

```raptor
RUN mkdir /foo
```

However, using `RUN mkdir` requires the `mkdir` command to be available and
executable inside the build target. This is not always the case, especially when
building things from scratch.

## Example

```raptor
MKDIR /data

MKDIR -p --chown babelfish:daemon /usr/local/translate/
```
