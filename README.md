# August

[![crates.io](https://img.shields.io/crates/v/august-build)](https://crates.io/crates/august-build)
![Workflow Status](https://github.com/ScratchCat458/august-build/actions/workflows/rust.yml/badge.svg)
![License](https://img.shields.io/crates/l/august-build)

![August Logo](https://raw.githubusercontent.com/ScratchCat458/august-build/master/docs/images/August%20Build.svg) 

August is a build system much like others of the task-based genre.
It uses a custom syntax for configuring builds and can import other files for the purpose of modularity.

## Installation
The recommended installation method is via Cargo:
```sh
cargo install august-build
```
To install from source, clone the repo, run the following command and copy out the binary:
```sh
cargo build --release
```

## Documentation
August's user documentation can be found at [https://august-build.web.app](https://august-build.web.app).
Internal docs can be found on [docs.rs](https://docs.rs/august-build/0.2.1/august_build/) though everything is mostly undocumented.

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

If you use August in your project, you can add this badge to your README: [![Built with August](https://img.shields.io/badge/built%20with-august-blueviolet)](https://github.com/ScratchCat458/august-build)
```markdown
[![Built with August](https://img.shields.io/badge/built%20with-august-blueviolet)](https://github.com/ScratchCat458/august-build)
```
