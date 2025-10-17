# Instruction `COPY`

~~~admonish summary
```raptor
COPY [<file-options>] <source> [...<source>] <destination>
```
~~~

```admonish tip
See the section on [file options](/file-options.md).
```

The `COPY` instruction takes one or more source files, and copies them to the
destination.

If multiple source files are specified, the destination MUST BE a directory.

| Input          | Destination | Result                                                   |
|:---------------|:------------|:---------------------------------------------------------|
| Single file    | File        | File written with destination filename                   |
| Single file    | Directory   | File written to destination dir, with source filename    |
| Multiple files | File        | ***Error***                                              |
| Multiple files | Directory   | Files written to destination dir, with original filename |
| Directory      | Any         | ***Not yet supported***                                  |
