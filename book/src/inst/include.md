# Instruction `INCLUDE`

```raptor
INCLUDE <filename> [...<key>=<value>]
```

The `INCLUDE` instruction calls on another Raptor file (`.rinc`) to be
executed. When `INCLUDE`ing files, any number of local variables can be passed
to the included target.

For example, if we have previously made the file `lib/install-utils` that
installs some useful programs, we can use that file in build targets:

```raptor
INCLUDE lib.install-base-utils
```

We can also make the component accept parameters, to make powerful, flexible
components:

```raptor
# hypothetical library function to update "/etc/hostname"
INCLUDE lib.set-hostname hostname="server1"
```

In the above example, we set the hostname of a server using an included
component.

> [!TIP]
>
> Since all values have to be specified as `key=value`, we might end up passing
> variables through several raptor files. This often ends up looking like this in
> the middle:
>
> ```raptor
> INCLUDE setup-thing username=username password=password
> ```
>
> This is of course valid, but a shorter syntax exists for this case:
>
> ```raptor
> INCLUDE setup-thing username password
> ```
>
> In other words, include parameter `name=name` can always be shortened to `name`.
