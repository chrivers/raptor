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
