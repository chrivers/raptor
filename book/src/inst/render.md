# Instruction `RENDER`

~~~admonish summary
```raptor
RENDER [<file-options>] <source> <destination> [...<include-arg>]
```
~~~

The `RENDER` instruction renders a file from a template, and writes it to the
specified destination. It accepts the same `key=value` arguments as
`INCLUDE`. These arguments are made available in the template.

Example:

```raptor
RENDER widgetfactory.tmpl /etc/widgetd/server.conf host="example.org" port=1234
```

The short form `name` (meaning `name=name`) is also supported here.

For example, in a component where `host` and `port` are available in the
environment:

```raptor
RENDER widgetfactory.tmpl /etc/widgetd/server.conf host port
```
