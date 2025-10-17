# Make

In the [previous chapter](iso.md) we took a look at the final command used to
build a debian liveboot iso:

```sh
sudo raptor run                 \
  --link book   book/example    \
  --link rbuild ../raptor-build \
  --cache liveboot-cache        \
  --input '$book.ssh'           \
  --output custom-liveboot.iso  \
  '$rbuild.deblive'
```

As mentioned, this works, but it doesn't exactly roll off the tongue.

Having to type this command every time we want to build the iso, is not a
satisfying solution.

To solve this, the subcommand `raptor make` is used. It is a
[make](https://www.gnu.org/software/make/make.html)-like system, where
dependencies are specified in a configuration file.

For `make`, this would be a `Makefile`, for `raptor make` this is going to be
`Raptor.toml`.

Building a `Raptor.toml` for your project is recommend, but not required. It is
perfectly possible to start using `raptor` from the command line, and only write
a `Raptor.toml` file when it feels right.

That being said, the format for `Raptor.toml` has been designed to be a smooth
transition from a constructed command line. The smallest valid `Raptor.toml`
file is an empty one.

In other words, everything can be added bit by bit, as needed.

Let's start by adding the two linked packages (`--link` arguments):

~~~admonish title="Raptor.toml"
```toml
[raptor.link]
book = "book/example"
rbuild = "../raptor-build"
```
~~~

We have added two package links, but of course any number can be added as desired.

Next, we will define a `run` job, which accounts for almost all of the remaining
command line arguments. It needs a name, so let's call it `book-ssh`:

~~~admonish title="Raptor.toml"
```toml
[raptor.link]
book = "book/example"
rbuild = "../raptor-build"

[run.book-ssh]
target = "$rbuild.deblive"
cache = "liveboot-cache"
input = "$book.ssh"
output = "custom-liveboot.iso"
```
~~~

Now that we have encoded all the arguments, we can use a much simpler `raptor
make` command to run this build job:

```sh
sudo raptor make book-ssh
```
