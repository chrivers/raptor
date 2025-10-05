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

The `FROM` instruction bases the current layer on top of some the specified layer.

Example:

```nginx
FROM base
```

### COPY

```nginx
COPY [<file-options>] <source> [...<source>] <destination>
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

```nginx
ENV <key>=<value> [...<key=value>]
```

The `ENV` instruction sets one or more environment variables inside the build namespace.

Example:

```nginx
ENV CFLAGS="--with-sprinkles"
ENV API_TOKEN="acbd18db4cc2f85cedef654fccc4a4d8" API_USER="user@example.org"
```

### INCLUDE

```nginx
INCLUDE <filename> [...<key>=<value>]
```

The `INCLUDE` instruction calls on another Raptor file (`.rinc`) to be
executed. When `INCLUDE`ing files, any number of local variables can be passed
to the included target.

For example, if we have previously made the file `lib/install-utils.rinc` that
installs some useful programs, we can use that file in build targets:

```nginx
INCLUDE "lib/install-base-utils.rinc"
```

We can also make the component accept parameters, to make powerful, flexible
components:

```nginx
# hypothetical library function to update "/etc/hostname"
INCLUDE "lib/set-hostname.rinc" hostname="server1"
```

In the above example, we set the hostname of a server using an included
component.

Since all values have to be specified as `key=value`, we might end up passing
variables through several raptor files. This often ends up looking like this in
the middle:

```nginx
INCLUDE "setup-thing.rinc" username=username password=password
```

This is of course valid, but a shorter syntax exists for this case:

```nginx
INCLUDE "setup-thing.rinc" username password
```

In other words, include parameter `name=name` can always be shortened to `name`.

### RENDER

```nginx
RENDER [<file-options>] <source> <destination> [...<include-arg>]
```

The `RENDER` instruction renders a file from a template, and writes it to the
specified destination. It accepts the same `key=value` arguments as
`INCLUDE`. These arguments are made available in the template.

Example:

```nginx
RENDER widgetfactory.tmpl /etc/widgetd/server.conf host="example.org" port=1234
```

The short form `name` (meaning `name=name`) is also supported here.

For example, in a component where `host` and `port` are available in the
environment:

```nginx
RENDER widgetfactory.tmpl /etc/widgetd/server.conf host port
```

### RUN

```nginx
RUN <command> [...<arg>]
```

The `RUN` instruction executes the given command inside the build namespace.

Arguments are executed as-is, i.e. without shell expansion, redirection, piping, etc.

```nginx
# enable the foo service
RUN systemctl enable foo.service
```

This ensures full control over the parsing of commands, but it also means normal
shell syntax is not available:

```nginx
# BROKEN: This will call "cat" with 3 arguments
RUN cat /etc/hostname "|" md5sum
#   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ this will not work
```

Instead, `/bin/sh` can be called explicitly:

```nginx
# This will produce the md5sum of /etc/hostname
RUN /bin/sh -c "cat /etc/hostname | md5sum"
```

### WORKDIR

```nginx
WORKDIR <path>
```

The `WORKDIR` instruction changes the current working directory inside the build
namespace. This affects all relative destination paths, as well as `RUN`:

```nginx
# This will copy "program" to "/bin/program" (initial directory is "/")
COPY program bin/program

WORKDIR /usr

# The same command will now copy "program" to "/usr/bin/program"
COPY program bin/program

WORKDIR /tmp

# This creates /tmp/foo
RUN /bin/sh -c "touch foo"
```

### WRITE

```nginx
WRITE [<file-options>] <value> <path>
```

The `WRITE` instruction writes a fixed string to the given path.

A file can be added to the build output with `COPY`, but sometimes we just need
to write a short value, and `COPY` might like overkill.

Using `WRITE`, we can put values into files:

```nginx
WRITE "hello world" hello.txt
```

The same file options as `COPY` and `RENDER` are accepted:

```nginx
WRITE --chmod 0600 --chown service:root "API-TOKEN" /etc/service/token.conf
```
