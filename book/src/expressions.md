# Expressions

Raptor supports a limited form of inline literal expressions (i.e. "values"),
that can be specified when using the `INCLUDE` instruction.

## Booleans

Boolean values work identically to json (and many other languages):

| Literal | Value |
|---------|-------|
| `true`  | true  |
| `false` | false |

## Integers

Integer values are supported.

Be aware that raptor always uses `i64` (signed 64-bit) integers.

This is *unlike* the typical json implementations, that uses `f64` (64-bit
floating-point) numbers, in two important ways:

**A)** The valid range for i64 integers is `-9223372036854775808` to
`9223372036854775807`, inclusive. This should be sufficient for most
applications. Any integer in this range will be represented exactly.

**B)** Floating-point (i.e. fractional) numbers are not supported. For example,
`3.14` is *not* valid in raptor. Instead, consider passing such a value as a
string.

Currently, alternate integer bases (i.e. hexadecimal or octal) are *not* supported.

## Strings

String are supported, and work much like they do in json, or other common notations.

```admonish tip
See the section on [string escapes](/string-escape.md) for more details.
```

## Lists

Lists are supported, with a syntax very similar to json. The only difference is
that raptor allows an optional trailing comma after the last list element, while
json does not.

~~~admonish example title="Examples of lists"
```json
[1, 2, 3]
```

```json
[true, false]
```

```json
[["a", "b"], 123]
```
~~~

## Maps

Maps (also typically known as *dicts* or *hashmaps*) contain a set of (key, value) pairs.

The syntax is similar to json. Like lists, raptor allows an optional trailing
comma after the last key-value pair.

~~~admonish example title="Examples of maps"
```json
{"answer": 42}
```

```json
{"name": "Heart of Gold", "engine": "Improbability drive"}
```
~~~
