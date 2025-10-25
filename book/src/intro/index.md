# Getting started with Raptor

## What is Raptor?

![raptor logo](../images/raptor-circle-icon.png)

Raptor[^raptor] is a modern, fast, and easy-to-use system for building disk images,
bootable isos, containers and much more - all from a simple, Dockerfile-inspired
syntax.

It uses `systemd-nspawn` for sandboxing when building or running containers.

~~~admonish tip title="~Eagle..~ err, eager to get started?"
Start by [installing raptor](install.md), then head over to the
[Debian Liveboot walkthrough](/walkthrough/debian/index.md) to get a
hands-on introduction to building a bootable iso.
~~~

## Theory of operation

Raptor builds *layers* from *`.rapt`* files. If you are familiar with Docker,
this is similar to how Docker builds *containers* from a *`Dockerfile`*.

~~~pikchr
{{#include ../images/raptor-layers.pikchr}}
~~~

However, Raptor has a different scope, and can do considerably different things
than Docker.

~~~admonish warning title="Heads up!"
The entire Raptor project, including this book, the program itself, and the
companion project [raptor-build](https://github.com/chrivers/raptor-build), is
still quite young.

If you find (or suspect) any bugs, [please report
them](https://github.com/chrivers/raptor/issues) so everybody can benefit.

At this point, Raptor has reached a stage where breaking changes are rare, but
we don't yet make any particular guarantees. We will **try our best** to announce
major changes clearly, and ahead of time.

If you have questions, ideas or feedback, don't hesitate to [join the
discussion](https://github.com/chrivers/raptor/discussions).
~~~

## Syntax

Raptor uses a syntax similar to `Dockerfile`. Statements start with uppercase
keywords, and are terminated by end of line.

All lines starting with `#` are treated as comments:

```raptor
# This copies "foo" from the host to "/bar" inside the build target
COPY foo /bar
```

## Raptor files

Before being parsed as raptor files, `.rapt` files are processed through
[minijinja](https://github.com/mitsuhiko/minijinja), a powerful templating
language.

[^raptor]: [According to wikipedia](https://en.wikipedia.org/wiki/Raptor): "The word "raptor" refers to several groups of avian and non-avian dinosaurs which primarily capture and subdue/kill prey with their talons.". Hopefully, this Raptor is less scary.
