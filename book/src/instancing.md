# Instancing

Raptor files can be *instanced*, which makes them work as a template.

Instanced files can be recognized by ending in `@.rapt` (for `FROM`) or `@.rinc`
(for `INCLUDE`).

## File names and syntax

| File             | Instanced? | Example                  |
|------------------|------------|--------------------------|
| `base.rapt`      | No         | `FROM base`              |
| `server@.rapt`   | Yes        | `FROM server@production` |
| `settings.rinc`  | No         | `INCLUDE settings`       |
| `firewall@.rinc` | Yes        | `INCLUDE firewall@full`  |

The table above shows some examples of instanced and non-instanced Raptor files.

~~~admonish error title="Beware"
It is **invalid** to reference an instanced file without an instance.

For example, `FROM server@` or `FROM base@value` would both fail to compile.

Therefore, when writing a new Raptor file, you need to determine if it needs to
be instanced, and name the file accordingly.
~~~

## Using instancing

So far, we have seen how to create templated (instanced) Raptor files, and how
to reference them to provide a value.

Now we will see how to *use* the provided value, so that instancing becomes
useful. Let us start the simplest possible example

~~~admonish note title="build-stamp@.rinc"
```raptor
WRITE "We are instance {{instance}}\n" /root/build-stamp.txt
```
~~~

This writes a human-readable line of text to a file, including the instance id.

Now we can use it from another file:

~~~admonish note title="server.rapt"
```raptor
# Will write "We are instance 1\n" to /root/build-stamp.txt
INCLUDE build-stamp@1
```
~~~
