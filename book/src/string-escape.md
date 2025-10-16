# Strings

In raptor files, string values can be expressed as quoted strings, or in certain
cases as so-called *barewords*:

```raptor
RUN echo these are barewords

RUN "echo" "these" "are" "quoted" "string"
```

For example, the following two statements are equivalent:

```raptor
RUN echo word1 word2

RUN echo "word1" "word2"
```

But the following two statements are *not*:

```raptor
# This creates 1 file called "filename with spaces"
RUN touch "filename with spaces"

# This creates *3 files* called "filename", "with", and "spaces", respectively
RUN touch filename with spaces
```

```admonish tip
Think of barewords as a convenience, to avoid needing to quote everything all
the time.

It is **always** valid and safe to use quoted strings to clearly convey the
intended meaning.

When in doubt, use quotes.
```

# String escaping

When using a quoted string, the backslash character (`\`) gains special meaning,
and is known as the *escape character*.

When it is followed by certain other characters, the combined expression is
replaced in the string:

| Escape expression | Result                                                               |
|-------------------|----------------------------------------------------------------------|
| `\\`              | A single literal backslash                                           |
| `\n`              | Newline (as if the string continued on the next line of text)        |
| `\t`              | Tabulator (useful to make tab clearly visible, and copy-paste proof) |
| `\"`              | Quote character (as opposed to ending the string)                    |

A backslash followed by any other character will result in a parse error.

```admonish warning title="Important"
Because backslash (`\`) is used as the escape character in quoted strings, any
literal backslashes *must* themselves be escaped, by adding another backslash
(`\\`).
```
