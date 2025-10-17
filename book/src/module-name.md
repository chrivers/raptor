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

## Package module names

In the previous section, we briefly mentioned *packages*.

Raptor is designed to work collaboratively, encouraging sharing of code between
projects, and people. A Raptor package is simply a collection of useful Raptor
files, typically distributed as a git repository.

This means we need a robust way to refer to Raptor files outside of our current
project, as well as a way to tell Raptor how to find these files.

This is where *package* module names are used.

First, let us take a look at how to make Raptor aware of external code
bases. This is called *linking*, analogous to how the term is used when building
a program from several sources.

When invoking `raptor`, the `-L` option defines a linked raptor package:

```sh
sudo raptor -L name src ...
```

For example, imagine we are working on a web service, and we want to generate
the deployment with Raptor. Now imagine this requires a database server, and
that someone else is responsible for the Raptor code that controls the database.

Then we might have a file layout like so:

```
~/project/web/server.rapt
~/project/database/db-setup.rinc
```

We fully control `~/projects/web`, but `~/project/database` is a git repository
we only pull changes from.

We would like to `INCLUDE` the `db-setup` module, but it exists outside our own
repository.

This can be solved by declaring `database` as linked package:

```sh
$ cd ~/project/web
$ sudo raptor build server -L database ../database
```

Now, we can refer to the content of `../database` by using `$database`.

~~~admonish tip
We don't have to give the link the same name as the directory it refers to!

If we link with `-L lib1 ../database`, we could instead refer to it as `$lib1.db-setup`.

Feel free to use any link name that suits the project; the linked names have no impact outside your project.
~~~
