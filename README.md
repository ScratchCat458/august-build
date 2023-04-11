# August

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
Writing the getting started documentation is still in progress, but most of the implementation details have been written up.
To view the docs locally (for now until I get some proper hosting), you will need pip:
```sh
pip install mkdocs-material
mkdocs serve
```

## Brilliant Crates
August is only made possible thanks to the efforts of many others.
Here they all are:
- [`ariadne`](https://github.com/zesterer/ariadne): Beautiful parser error handling by [@zesterer](https://github.com/zesterer) 
- [`clap`](https://github.com/clap-rs/clap): Derive-based command-line argument parser
- [`comfy-table`](https://github.com/nukesor/comfy-table): Beautiful table generation for `august info` by [@Nukesor](https://github.com/nukesor)
- [`dirs`](https://github.com/dirs-dev/dirs-rs): Used exclusively for finding the home directory
- [`owo-colors`](https://github.com/jam1garner/owo-colors): Vibrant colouring for displaying CLI execution by [@jam1garner](https://github.com/jam1garner)
- [`run-script`](https://github.com/sagiegurari/run_script): Used so I don't have to think about argument separation in the `exec` command by [@sagiegurari](https://github.com/sagiegurari)
- [`walkdir`](https://github.com/BurntSushi/walkdir): Directory recursion for module resolution by [@BurntSushi](https://github.com/BurntSushi) 
