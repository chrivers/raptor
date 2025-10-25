## Relative module names

The first, and arguably simplest form, is the *Relative* name. It is
characterized by not having a dollar sign (`$`) in front of the first element,
unlike the other two forms.

Relative module paths form a sequence of one or more *elements*, which are the
names between the dots (`.`).

Each element before the last is viewed as a directory. The last element is the
file name, appended with either `.rapt` for `FROM` instructions, or `.rinc` for
`INCLUDE` instructions.

In other words, `a.b.c.d` becomes `a/b/c/d.rapt` (`FROM`) or `a/b/c/d.rinc` (`INCLUDE`).

### Examples:

| Statement                  | Source file       | Resulting path              |
|:---------------------------|:------------------|:----------------------------|
| `FROM base`                | `target.rapt`     | `base.rapt`                 |
| `FROM base`                | `lib/target.rapt` | `lib/base.rapt`             |
| `FROM utils.base`          | `lib/target.rapt` | `lib/utils/base.rapt`       |
| `INCLUDE babelfish`        | `lib/target.rapt` | `lib/babelfish.rinc`        |
| `INCLUDE hitchhiker.guide` | `lib/target.rapt` | `lib/hitchhiker/guide.rinc` |
