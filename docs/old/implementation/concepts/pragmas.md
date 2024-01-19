# Pragmas

Pragmas provide extra information to the parser and runtime.
August does not derive meaning from the names of tasks and therefore these must be explicitly defined.

## CLI Pragmas
These pragmas relate to tasks being bound to CLI commands.

This includes:

- `august build`
- `august test`

## Namespace
File names do not define module namespace. 
These are used to allow a module to be imported by a build script.

## Module Awareness
Similar to imports,
these pragmas tell the parser to build an external module and make its commands accessable in the module tree.
