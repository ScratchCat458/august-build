# Using the CLI

August has command documentation within the CLI itself,
but this page provides more detail for specifics.

## `august info`
:octicons-tag-24: 0.1.0

Provides information about the current August install
and the amount of modules in the current directory and global module directory.

```
august info
```
Resulting output:
!!! note
    The formatting looks a bit off in these docs due to the Material for MkDocs syntax highlighter.
    It looks normal in a terminal.
    Many thanks to the work of [@Nukesor](https://github.com/nukesor) on the crate,
    [`comfy-table`](https://github.com/nukesor/comfy-table).
```
╭────────────────┬───────────────────────────────────────────────╮
│ Package Name   ┆ august-build                                  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Author(s)      ┆ Hayden Brown <scratchcat458@gmail.com>        │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Version        ┆ 0.2.1                                         │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Documentation  ┆ https://august-build.web.app                  │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Repository     ┆ https://github.com/ScratchCat458/august-build │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Global Modules ┆ 0 in ~/.august/modules/                       │
├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
│ Local Modules  ┆ 4 in current directory                        │
╰────────────────┴───────────────────────────────────────────────╯
```

## Execution
There are three commands used for executing build scripts, two of which are tied to pragmas.

### `august test`
:octicons-tag-24: 0.1.0

Runs the task assigned to `#pragma test`.

=== "Shell"
    ```
    august test
    ```
    
    Resulting output:
    ```
    Hello, World!
    ```
=== "main.august"
    ```august
    @main

    #pragma test hello

    Task hello {
        print("Hello, World!")
    }
    ```

### `august build`
:octicons-tag-24: 0.1.0

Runs the task assigned to `#pragma build`.

=== "Shell"
    ```
    august build
    ```
    
    Resulting output:
    ```
    Hello, World!
    ```
=== "main.august"
    ```august
    @main

    #pragma build hello

    Task hello {
        print("Hello, World!")
    }
    ```

### `august run <TASK>`
:octicons-tag-24: 0.1.0

Runs a task by name.

=== "Shell"
    ```
    august run hello
    ```
    
    Resulting output:
    ```
    Hello, World!
    ```
=== "main.august"
    ```august
    @main

    Task hello {
        print("Hello, World!")
    }
    ```

## Verification
These commands do not run anything but can help with check that a script works before running or deploying to CI.

### `august inspect`
:octicons-tag-24: 0.1.1

Lists the contents of a build script and any linked modules in tabular form.

=== "Shell"
    ```
    august inspect 
    ```

    Resulting output:
    !!! note
        The formatting looks a bit off in these docs due to the Material for MkDocs syntax highlighter.
        It looks normal in a terminal.
        Many thanks to the work of [@Nukesor](https://github.com/nukesor) on the crate,
        [`comfy-table`](https://github.com/nukesor/comfy-table).
    ```
    ╭─────────────────────┬─────────────────────────╮
    │ Property            ┆ Contents                │
    ╞═════════════════════╪═════════════════════════╡
    │ Namespace           ┆ main                    │
    ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
    │ Pragma              ┆ Test --> None           │
    │                     ┆ Build --> build         │
    ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
    │ Tasks               ┆ - build                 │
    │                     ┆                         │
    ├╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌┤
    │ Command Definitions ┆ - localcmddef           │
    │                     ┆ - mycmd from ext_module │
    │                     ┆                         │
    ╰─────────────────────┴─────────────────────────╯
    ```
=== "main.august"
    ```august
    @main

    #link ext_module
    #pragma build build

    Task build {
      print("This message is from an internal command!");
      localcmddef;
      ext_module.mycmd;
    }

    cmddef localcmddef {
      print("This message is from a locally defined command!");
    }
    ```
=== "ext_module.august"
    ```august
    @ext_module

    cmddef mycmd {
      print("This message is from an externally defined command!");
    }
    ```

### `august check`
:octicons-tag-24: 0.1.0

Runs the build script and linked modules through the parser without running anything.

## Script Specification
For commands that require a script to be run, the default file path is `main.august`.
However this can be changed using the `-s` flag.

```
august build -s <FILE_PATH>
```
This applies for:

- `august test`
- `august build`
- `august run <TASK>`
- `august check`
- `august inspect`

## Command Line Autocompletion
Autocompletion is avaliable August thanks to [`clap_complete`](https://github.com/clap-rs/clap/tree/master/clap_complete).
This provideds support for the following shells:

- Bash
- Fish
- Zsh
- Powershell
- Elvish

This works by placing the generated autocomplete in standard out.
The contents must be copied into a file and into the config for your specific shell.

=== "Bash"
    ```
    august completions bash > /usr/share/bash-completion/completions/august.bash
    ```
=== "Fish"
    ```
    august completions fish > ~/.config/fish/completions/august.fish
    ```
=== "Zsh"
    ```
    august completions zsh > /usr/share/zsh/functions/Completions/Unix/_august
    ```
=== "Powershell"
    ```
    august completions powershell > ~\Documents\WindowsPowerShell\august.ps1 
    ```
    Add the following line to your Powershell profile:
    ```
    . ~\Documents\WindowsPowerShell\august.ps1 
    ```
