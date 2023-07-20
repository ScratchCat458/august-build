---
hide:
  - navigation
  - toc
---

# Welcome

August is a build system much like others of the task-based genre.
It uses a custom syntax for configuring builds and can import other files for the purpose of modularity.

It is a very significant part of my work in software development and creating this has taught me a lot about parsing and error handling.

## Quick Links
[Start Here :material-arrow-right:](getting-started){ .md-button .md-button--primary }
Learn how to use August to write build scripts using Rust and Cargo as an example use case.

[CLI Usage :octicons-terminal-24:](getting-started/cli){ .md-button }
A guide to all of the commands in the August CLI tool

[Impl Docs :material-tools:](implementation){ .md-button }
How August is designed

## Brilliant Crates
August is only made possible thanks to the efforts of many others.
Here they all are:

- [`ariadne`](https://github.com/zesterer/ariadne): Beautiful parser error handling by [@zesterer](https://github.com/zesterer) 
- [`clap`](https://github.com/clap-rs/clap): Derive-based command-line argument parser
- [`clap_complete`](https://github.com/clap-rs/clap/tree/master/clap_complete): Generator for command line autocompletions, see `august completions`
- [`comfy-table`](https://github.com/nukesor/comfy-table): Beautiful table generation for `august info` and `august inspect` by [@Nukesor](https://github.com/nukesor)
- [`dirs`](https://github.com/dirs-dev/dirs-rs): Used exclusively for finding the home directory
- [`owo-colors`](https://github.com/jam1garner/owo-colors): Vibrant colouring for displaying CLI execution by [@jam1garner](https://github.com/jam1garner)
- [`run-script`](https://github.com/sagiegurari/run_script): Used so I don't have to think about argument separation in the `exec` command by [@sagiegurari](https://github.com/sagiegurari)
- [`walkdir`](https://github.com/BurntSushi/walkdir): Directory recursion for module resolution by [@BurntSushi](https://github.com/BurntSushi)
- [`tracing`](https://github.com/tokio-rs/tracing) and [`tracing-subscriber`](https://github.com/tokio-rs/tracing/tree/master/tracing-subscriber): Nice app-level diagnostics by the [@tokio-rs](https://github.com/tokio-rs) team

## Show off in your repo 
If you use August in your project, you can add this badge to your README: [![Built with August](https://img.shields.io/badge/built%20with-august-blueviolet)](https://github.com/ScratchCat458/august-build)
```markdown
[![Built with August](https://img.shields.io/badge/built%20with-august-blueviolet)](https://github.com/ScratchCat458/august-build)
```
