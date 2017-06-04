erl_parse
=========

Erlang source code parser written in Rust.

References
----------

- [AST](http://erlang.org/doc/apps/erts/absform.html)
- [Macro](http://erlang.org/doc/reference_manual/macros.html)

Limitations
-----------

- Supports only UTF-8 source codes

TODO
----

- Support operator precedences
- Support macro invocations which does not seem valid Erlang syntactic forms
