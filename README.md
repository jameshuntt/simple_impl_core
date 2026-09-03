# simple_impl_core

The semantic models behind [`simple_impl_derive`](https://crates.io/crates/simple_impl_derive).

Once an attribute has been parsed by
[`simple_impl_attr_kit`](https://crates.io/crates/simple_impl_attr_kit), the
derive needs to know what it means: which setter kind a builder field gets,
how a shell field is emitted and ordered, what a composite command's entries
and surface look like, and which validation rules a command carries. Those
models live here, as plain data with their own tests, so the proc-macro
crate stays a thin front and the meaning of every attribute can be checked
without expanding a macro.

- `BuilderCfg`, `BuilderKind`: a builder field and its setter shape.
- `ShellCfg`, `ShellFieldCfg`, `PosMode`, `Order`: a shell command and how
  each field lands in the argument vector.
- `CompositeEntry`, `CompositeRootSpec` and the composite contract types:
  a command made of subcommands, from field or registry style attributes.
- `ValidationRule`, `ValidationSpec`: `requires`, `conflicts_with`,
  `one_of`, `at_least_one_of` and custom hooks, lowered from the attribute
  grammar.
- The type questions a derive asks: is this field `Option<_>`, `Vec<_>`,
  `bool`, a string, an unsigned integer.

This crate generates no tokens; that is
[`simple_impl_quote_kit`](https://crates.io/crates/simple_impl_quote_kit).

## License

MIT OR Apache-2.0.
