# Raptor

Raptor is a modern, fast, and easy-to-use system for building disk images,
bootable isos, containers and much more, from a simple, Dockerfile-inspired
syntax.

It uses `systemd-nspawn` for sandboxing during the build process.

## Syntax

Raptor uses a syntax similar to `Dockerfile`. Statements start with uppercase
keywords, and are terminated by end of line.

All lines starting with `#` are treated as comments:

```nginx
# This copies "foo" from the host to "/bar" inside the build target
COPY foo /bar
```

### FROM

```nginx
FROM <identifier>
```

The `FROM` statement bases the current layer on top of some the specified layer.

Example:

```nginx
FROM base
```

### COPY

```nginx
COPY [file-options] <source> [...<source>] <destination>
```

The `COPY` instruction takes one or more source files, and copies them to the
destination.

If multiple source files are specified, the destination MUST BE a directory.

| Input          | Destination | Result                                                   |
|----------------|-------------|----------------------------------------------------------|
| Single file    | File        | File written with destination filename                   |
| Single file    | Directory   | File written to destination dir, with source filename    |
| Multiple files | File        | Error                                                    |
| Multiple files | Directory   | Files written to destination dir, with original filename |
| Directory      | Any         | Error: Not yet supported                                 |

Several instructions (`COPY`, `WRITE`, `RENDER`) write files into the build
target. The all supports common options that affect how the files are written.

#### file-options: `--chmod <mode>`

The `--chmod` option specifies the mode bits (e.g. permissions) associated with
the file. The `mode` is specified as 3 or 4 octal digits.

Examples:

```nginx
# these are equivalent:
COPY --chmod  755 script.sh /root/script.sh
COPY --chmod 0755 script.sh /root/script.sh

# set suid bit:
COPY --chmod 4755 sudo /usr/bin/sudo

# user access only, read-only:
WRITE --chmod 0400 "secret" /etc/service/token
```

#### file-options: `--chown <owner>`

The `--chown` option specifies the user and/or group to own the file.

| Input        | User   | Group   |
|--------------|--------|---------|
| `user`       | `user` | N/A     |
| `:group`     | N/A    | `group` |
| `user:group` | `user` | `group` |
| `user:`      | `user` | `user`  |

Notice the last form, where `user:` is shorthand for `user:user`. This is the
same convention used by GNU coreutils, and several other programs.

### ENV

The `ENV` command sets one or more environment variables inside the build namespace.

```nginx
ENV <key>=<value> [...<key=value>]
```

Example:

```nginx
ENV CFLAGS="--with-sprinkles"
ENV API_TOKEN="acbd18db4cc2f85cedef654fccc4a4d8" API_USER="user@example.org"
```

### INCLUDE


### INVOKE

### RENDER

### RUN

### WORKDIR

### WRITE
