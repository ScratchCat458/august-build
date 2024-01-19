# Welcome

Between versions 0.4.3 and 0.5, August underwent a major redesign in both syntax and execution.
Motivating this was a desire to use parser combinators instead of a handwritten imperative parser,
as new syntax was becoming difficult to maintain and update, as well as being somewhat limited (e.g. no string escapes).

The original syntax is not supported by versions `>0.5`.
The guide contains translations from the old syntax to fit the new August.
Upgrading should be your first preference but if you need to support the old syntax, you can install August with this command:
```
cargo install august-build --version 0.4.3
```

These pages exist for archival purposes and are unlikely to reflect the new implementation.
