## Absolute module names

Relative module names are simple, but lack the ability to point upwards in the
directory heirarchy.

For example, suppose you want to organize a set of Raptor files like so:

```
~/project/base.rapt
~/project/hostname.rinc
~/project/target/common.rapt
~/project/target/frontend.rapt
~/project/target/database.rapt
~/project/lib/utils/tools.rinc
```

The `base` layer can `INCLUDE hostname`, but the `frontend` and `database`
targets have no way to specify they want to build `FROM` the `base` layer!

This is where *absolute* module names are useful.

By prefixing a name with `$.` (dollar + dot) the name refers to the root of the
current package.

We will go into more detail of what a package is, in the next section. For now,
it is enough to know that when invoking `raptor` on a target, the root for that
target is the directory that `raptor` was started from.

| Statement                   | Source file            | Resulting path         |
|:----------------------------|:-----------------------|:-----------------------|
| `FROM $.base`               | `target/frontend.rapt` | `base.rapt`            |
| `FROM common`               | `target/frontend.rapt` | `target/common.rapt`   |
| `FROM $.target.common`      | `target/frontend.rapt` | `target/common.rapt`   |
| `INCLUDE $.lib.utils.tools` | `target/database.rapt` | `lib/utils/tools.rapt` |
