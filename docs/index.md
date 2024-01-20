---
hide:
  - navigation
  - toc
---

# Welcome

August is a task-based build system with a strong focus on parallelism.
It is written in Rust, a language known for its stability and performance and relies on custom syntax to describe the build process.

August's name is derived from the name of a *certain* character, from a *certain* game,
whose name is derived from the name of a *certain* German aircraft carrier.[^1]

[^1]: [For those who are curious...](https://azurlane.koumakan.jp/wiki/August_von_Parseval)

## Quick Links
[Start Here :material-arrow-right:](guide/tutorial.md){ .md-button .md-button--primary }
Learn how to use August to write build scripts using Rust and Cargo as an example use case.

[CLI Usage :octicons-terminal-24:](guide/cli.md){ .md-button }
A guide to all of the commands in the August CLI tool

[Impl Docs :material-tools:](impl.md){ .md-button }
All about the design of August

## Brilliant Crates
August is only made possible thanks to the efforts of many others.
Here they all are:

- [`ariadne`](https://github.com/zesterer/ariadne): Beautiful parser error handling by [@zesterer](https://github.com/zesterer)
- [`chumsky`](https://github.com/zesterer/chumsky): My new favourite parser combinator library (also by [@zesterer](https://github.com/zesterer))
- [`clap`](https://github.com/clap-rs/clap): Derive-based command-line argument parser
- [`clap_complete`](https://github.com/clap-rs/clap/tree/master/clap_complete): Generator for command line autocompletions, see `august completions`
- [`comfy-table`](https://github.com/nukesor/comfy-table): Beautiful table generation for `august info` and `august inspect` by [@Nukesor](https://github.com/nukesor)
- [`dircpy`](https://github.com/woelper/dircpy/): Recursive directory copying for August's `fs::copy` by [@woelper](https://github.com/woelper/)
- [`owo-colors`](https://github.com/jam1garner/owo-colors): Vibrant colouring for displaying CLI execution by [@jam1garner](https://github.com/jam1garner)
- [`thiserror`](https://github.com/dtolnay/thiserror): Helper for implementing `std::error::Error` by [@dtolnay](https://github.com/dtolnay)
- [`which`](https://github.com/harryfei/which-rs): Magic that makes August's `exec` work better by [@harryfei](https://github.com/harryfei/which-rs)

## Show off in your repo 
If you use August in your project, you can add this badge to your README: [![Built with August](https://img.shields.io/badge/built%20with-august-blueviolet)](https://github.com/ScratchCat458/august-build)
```markdown
[![Built with August](https://img.shields.io/badge/built%20with-august-blueviolet)](https://github.com/ScratchCat458/august-build)
```
