# Module names

In Raptor, "module names" are the names Raptor files use to refer to other
Raptor files.

This is the case both for the `FROM` instruction (where the modules have the
`.rapt` extension) and `INCLUDE` (`.rinc` extension).

There are three types of module names; Relative, Absolute and Package names.

| Type     | Example     | Description                                                 |
|:---------|:------------|-------------------------------------------------------------|
| Relative | `foo.bar`   | Used to refer to paths at or below the current directory    |
| Absolute | `$.foo.bar` | Used to refer to paths from the root of the current package |
| Package  | `$foo.bar`  | Used to refer to paths in other packages                    |

The difference between these forms is how they resolve to paths in the
filesystem. For a detailed explanation of each, see the following sections.

## Instancing

Each of these forms support *instancing*, which is a way to pass a single,
simple parameter through the module name, by appending a `@` followed by the
value, to the module name.

All raptor files are either instanced (`example@.rapt` / `example@.rinc`) or not
(`example.rapt` / `example.rinc`).

Non-instanced files *cannot* be referenced with an instanced name, and vice
versa.

Users of `systemd` might recognize this pattern, which is used in the same way
there.

To learn more, read the [section on instancing](instancing.md).

## Learn more

Each type of module name is described in more detail:

 - [Relative](module-name/relative.md)
 - [Absolute](module-name/absolute.md)
 - [Package](module-name/package.md)
