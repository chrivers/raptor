# Instruction: `FROM`

```nginx
FROM [<schema>://]<from-source>
```

The `FROM` instruction bases the current layer on top of some the specified layer.

Unlike Docker, multiple types of `from-source` are supported:

| Type   | Schema      | Example                       |
|--------|-------------|-------------------------------|
| Raptor | `<none>`    | `FROM library.base`           |
| Docker | `docker://` | `FROM docker://debian:trixie` |

## Raptor sources

When no schema is specified, the `from-source` is assumed to be the [module
name](/module-name.md) of another raptor layer.

> [!TIP]
> This will be familiar to docker users. For example..
>
> ```docker
> # Dockerfile
> FROM base
> ```
> ..will depend on the docker image `base`

However, unlike docker files, raptor can point to raptor files in other
directories, or even other packages. See [module names](/module-name.md) for an
overview.

### Examples

```raptor
# This will depend on `base.rapt`
FROM base
```

```raptor
# This will depend on `library/debian.rapt`
FROM library.debian
```

## Docker sources

To use a docker image as the basis for a raptor layer, specify the name of the
docker image, prefixed with `docker://`, e.g:

```raptor
FROM docker://debian:trixie
```

> [!TIP]
>
> In general, `docker pull <NAME>` becomes `FROM docker://<NAME>`
