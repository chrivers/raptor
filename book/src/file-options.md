# File options

Several instructions (`COPY`, `WRITE`, `RENDER`) write files into the build
target. They all supports common options that affect how the files are written.

```admonish important
These file options must immediately follow the instruction (i.e. before the
source or destination path)
```

## Change mode: `--chmod <mode>`

The `--chmod` option specifies the mode bits (e.g. permissions) associated with
the file. The `mode` is specified as 3 or 4 octal digits.

Examples:

```raptor
# these are equivalent:
COPY --chmod  755 script.sh /root/script.sh
COPY --chmod 0755 script.sh /root/script.sh

# set suid bit:
COPY --chmod 4755 sudo /usr/bin/sudo

# user access only, read-only:
WRITE --chmod 0400 "secret" /etc/service/token
```

```admonish tip
The 3-digit version is identical to the 4-digit version, where the first digit
is zero (which is a common case).
For example, `755` and `0755` represent the same permissions.
```

## Change owner: `--chown <owner>`

The `--chown` option specifies the user and/or group to own the file.

| Input        | User        | Group       |
|--------------|-------------|-------------|
| `user`       | `user`      | (no change) |
| `:`          | (no change) | (no change) |
| `:group`     | (no change) | `group`     |
| `user:group` | `user`      | `group`     |
| `user:`      | `user`      | `user` (!)  |

```admonish important
Notice the last form, where `user:` is shorthand for `user:user`.

This is the same convention used by GNU coreutils, and several other programs.
```

## Create parent directories: `-p`

```admonish note
This file option is only valid for the `MKDIR` instruction.
```

The `-p` option instructs `MKDIR` to create parent directories as needed.

Importantly, it also makes `MKDIR` accept existing directories, *including* the
last directory.

This is identical to the behavior of `-p` with the shell command `mkdir`.

```raptor
# will fail if:
#   A) /foo is missing
# or:
#   B) /foo/bar already exists
MKDIR /foo/bar

# this will create:
#   /foo (if missing)
# and then
#   /foo/bar (if missing)
MKDIR -p /foo/bar
```
