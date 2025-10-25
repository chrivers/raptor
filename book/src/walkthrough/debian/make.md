# Make

In the [previous chapter](iso.md) we took a look at the final command used to
build a debian liveboot iso:

```sh
sudo raptor run                    \
  --link book   book/example       \
  --link rbuild ../raptor-builders \
  --cache liveboot-cache           \
  --input '$book.ssh'              \
  --output custom-liveboot.iso     \
  '$rbuild.deblive'
```

As mentioned, this works, but it doesn't exactly roll off the tongue.

Having to type this command every time we want to build the iso, *is not a
satisfying solution*.

To solve this, the subcommand `raptor make` is used. It is a
[make](https://www.gnu.org/software/make/make.html)-like system, where
dependencies are specified in a configuration file.

For `make`, this is a `Makefile`, for Rust it's `Cargo.toml`, and for `raptor
make` we have `Raptor.toml`.

Building a `Raptor.toml` for your project is recommend, but *not required*. It
is perfectly possible to start using `raptor` from the command line, and only
write a `Raptor.toml` file when it feels right.

That being said, the format for `Raptor.toml` has been designed to be a smooth
transition from a constructed command line. The smallest valid `Raptor.toml`
file is an empty one.

Let's start by adding the two linked packages (`--link` arguments):

~~~admonish title="Raptor.toml"
```toml
[raptor.link]
book = "book/example"
rbuild = "../raptor-builders"
```
~~~

We have added two package links, but of course any number can be added as desired.

All sections named `[raptor.*]` are settings for Raptor. The section
`[raptor.link]` is for specifying linked packages.

Next, we will define a `run` job, which accounts for almost all of the remaining
command line arguments.

A `Raptor.toml` file can have any number of `[run.*]` sections, each with their
own name. Each of those sections is a separate `run` job. Let's use `book-ssh`
for this example:

~~~admonish title="Raptor.toml"
```toml
[raptor.link]
book = "book/example"
rbuild = "../raptor-builders"

# The name `book-ssh` is not special.
# Feel free to choose any name you like!
[run.book-ssh]
target = "$rbuild.deblive"
cache = "liveboot-cache"
input = "$book.ssh"
output = "custom-liveboot.iso"
```
~~~

~~~admonish example title="Click here for more details on `[raptor.link]`" collapsible=true
You can choose any names for the linked packages.

For example, instaed of this:

```toml
[raptor.link]
book = "book/example"
...

[run.example-1]
# $book refers to the link name above
input = "$book.ssh"
...
```

...the following `Raptor.toml` is equivalent:

```toml
[raptor.link]
example = "book/example"
...

[run.example-2]
# $example refers to the link name above
input = "$example.ssh"
...
```
~~~


Now that we have encoded all the arguments, we can use a much simpler `raptor
make` command to run this build job:

```sh
sudo raptor make book-ssh
```
