# Grammar

~~~admonish warning
The Raptor parser is implemented by hand in a
[separate crate](https://github.com/chrivers/raptor/tree/master/crates/raptor-parser).

Because it is hand-written, there is no exact BNF grammar that matches the parsing.

The following is an attempt to provide a detailed and realistic description of
the accepted syntax, but might contain minor mistakes and inconsistencies.
~~~

The parsing starts from the first rule, `<file>`, and proceeds from there.

~~~admonish tip title="Syntax guide"
| Syntax               | Meaning                                                          |
|:---------------------|:-----------------------------------------------------------------|
| `rule?`              | Match `rule` 0 or 1 times (i.e., it is optional)                 |
| `rule+`              | Match `rule` 1 or more times                                     |
| `rule*`              | Match `rule` 0 or more times                                     |
| `rule1 \\| rule2`    | Match either `rule1` or `rule2` (exactly one of them must match) |
| `( rule1 rule2 .. )` | Parenthesized rules are matched/optional/repeated together       |
| `"word"`             | Matches the letters `w`, `o`, `r`, `d` (but not the quotes)      |
~~~

~~~admonish summary title="Raptor grammar"
```bnf
<file>         ::= <statement>*

<statement>    ::= <from>
                 | <mount>
                 | <render>
                 | <write>
                 | <mkdir>
                 | <copy>
                 | <include>
                 | <run>
                 | <env>
                 | <workdir>
                 | <entrypoint>
                 | <cmd>

<from>         ::= "FROM" <from-source> "\n"
<mount>        ::= "MOUNT" <mount-type>? <word> <path> "\n"
<render>       ::= "RENDER" <file-option>* <path> <path> <include-arg>* "\n"
<write>        ::= "WRITE" <file-option>* <value> <path> "\n"
<mkdir>        ::= "MKDIR" <mkdir-option>* <path> "\n"
<copy>         ::= "COPY" <file-option>* <path>+ <path> "\n"
<include>      ::= "INCLUDE" <module-name> <include-arg>* "\n"
<run>          ::= "RUN" <word>+ "\n"
<env>          ::= "ENV" <env-assign>+ "\n"
<workdir>      ::= "WORKDIR" <path> "\n"
<entrypoint>   ::= "ENTRYPOINT" <word>* "\n"
<cmd>          ::= "CMD" <word>* "\n"

<env-assign>   ::= <word> ( "=" <value> )?
<mount-type>   ::= "--file" | "--simple" | "--layers" | "--overlay"

<mkdir-option> ::= <file-option> | "-p"
<file-option>  ::= <file-chown> | <file-chmod>
<file-chown>   ::= "--chown" "="? <chown>
<file-chmod>   ::= "--chmod" "="? <chmod>
<chown>        ::= (<word> (":" <word>?)?) | (":" <word>?)
<chmod>        ::= /* built-in rule: 3 or 4 octal digits */

<include-arg>  ::= <word> ( "=" <expression> )?
<expression>   ::= <expr-lookup> | <expr-value>
<expr-lookup>  ::= <word> ("." <word>)*
<expr-value>   ::= <expr-list>
                 | <expr-map>
                 | <expr-string>
                 | <expr-number>
                 | <expr-bool>
<expr-list>    ::= "[" ( <expr-value>   ( "," <expr-value>   )* ","? )? "]"
<expr-map>     ::= "{" ( <expr-mapitem> ( "," <expr-mapitem> )* ","? )? "}"
<expr-mapitem> ::= <expr-value> ":" <expr-value>
<expr-string>  ::= /* built-in rule: see section on string escapes */
<expr-number>  ::= <digit>+
<expr-bool>    ::= "true" | "false"
<digit>        ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"

<module-name>  ::= <mn-package>? <mn-body> <mn-instance>?
<mn-package>   ::= "$" <word>? "."
<mn-body>      ::= <word> ( "." <word> )*
<mn-instance>  ::= "@" <word>
```
~~~
