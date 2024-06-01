# CLI Usage

## Global Flags

!!! warning
    All global flags must be specified before the name of the subcommand.

### `-s`/`--script`
Changes the script that the CLI operates on.
The default path is `main.august`.

### `-v`/`--verbose`
Provides extra information about command calls during execution.

### `-q`/`--quiet`
Silences output logs.

## `august info`

Provides information about the `august` CLI command.
It's like `-V`, but *pretty*!

```
august info
```
Output:
!!! note
    The formatting looks a bit off in these docs due to the Material for MkDocs syntax highlighter.
    It looks normal in a terminal.
    Many thanks to the work of [@Nukesor](https://github.com/nukesor) on the crate,
    [`comfy-table`](https://github.com/nukesor/comfy-table).
```
╭───────────────┬───────────────────────────────────────────────╮
│ Package Name  │ august-build                                  │
├───────────────┼───────────────────────────────────────────────┤
│ Author(s)     │ Hayden Brown <scratchcat458@gmail.com>        │
├───────────────┼───────────────────────────────────────────────┤
│ Version       │ 0.5.0-dev                                     │
├───────────────┼───────────────────────────────────────────────┤
│ Documentation │ https://august-build.web.app                  │
├───────────────┼───────────────────────────────────────────────┤
│ Repository    │ https://github.com/ScratchCat458/august-build │
╰───────────────┴───────────────────────────────────────────────╯
```

## `august test`
Runs the unit exposed as test.

=== "Shell"
    ```
    august test
    ```
=== "main.august"
    ```august
    expose Hello as test

    unit Hello {
        println("Hello!")
    }
    ```

Output:
```
Hello!
```

## `august build`
Runs the unit exposed as test.

=== "Shell"
    ```
    august build
    ```
=== "main.august"
    ```august
    expose Hello as build

    unit Hello {
        println("Hello!")
    }
    ```

Output:
```
Hello!
```

## `august run <UNIT>`
Runs a unit by name.

=== "Shell"
    ```
    august run Hello
    ```
=== "main.august"
    ```august
    unit Hello {
        println("Hello!")
    }
    ```

Output:
```
Hello!
```

### `--deprecated-threads-runtime`

:material-tag: 0.6

Switches execution to the old thread-based runtime instead of the async runtime.

!!! warning
    I need to make it clear that use of this option is far from recommended.
    It is deprecated for a reason.

    The previous runtime would spawn a new thread for each dependency to do concurrency.
    For many of the uses of August's Units, such as encapsulating the calling of a CLI tool,
    the thread itself is useless and only exists to sit idly be an wait for the tool to finish.
    The async runtime spawns units as tasks on Tokio, with blocking tasks being scheduled by Tokio as well.
    This allows for better use of multithreaded concurrency through work-stealing and task switching.

## `august inspect`
Tabular summary of the contents of a build script.
Displays units, their dependencies and any other related metadata.

!!! note
    The formatting looks a bit off in these docs due to the Material for MkDocs syntax highlighter.
    It looks normal in a terminal.
    Many thanks to the work of [@Nukesor](https://github.com/nukesor) on the crate,
    [`comfy-table`](https://github.com/nukesor/comfy-table).

**With Metadata** 
=== "Shell"
    ```
    august inspect
    ```
=== "main.august"
    ```august
    expose Build as build
    expose Test as test

    unit Build {
      meta(
        @name "cargo::Build"
        @desc "Runs `cargo build`"
        @deps "[Test]"
      )

      depends_on(Test)

      fs::eprint_file("Cargo.toml")
      exec(cargo build)
    }

    unit Test {
      meta(
        @name "cargo::Test"
        @desc "Runs `cargo test`"
        @calls "[cargo::Lints]"
      )

      do(Lints)
      exec(cargo test)
    }

    unit Lints {
      meta(
        @name "cargo::Lints"
        @desc "Runs `cargo fmt` and `cargo clippy`"
      )

      ~(cargo fmt)
      ~(cargo clippy)
    }
    ```
Output:
```
╭────────┬───────╮
│ Pragma │ Unit  │
╞════════╪═══════╡
│ Test   │ Test  │
├────────┼───────┤
│ Build  │ Build │
╰────────┴───────╯
╭───────┬──────────────┬───────┬─────────────────────────────────────╮
│ Unit  │ Dependencies │ @meta │                                     │
╞═══════╪══════════════╪═══════╪═════════════════════════════════════╡
│ Build │ Test         │ desc  │ Runs `cargo build`                  │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│       │              │ name  │ cargo::Build                        │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│       │              │ deps  │ [Test]                              │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│ Lints │              │ name  │ cargo::Lints                        │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│       │              │ desc  │ Runs `cargo fmt` and `cargo clippy` │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│ Test  │              │ name  │ cargo::Test                         │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│       │              │ desc  │ Runs `cargo test`                   │
├───────┼──────────────┼───────┼─────────────────────────────────────┤
│       │              │ calls │ [cargo::Lints]                      │
╰───────┴──────────────┴───────┴─────────────────────────────────────╯
```

**Without Metadata** 
=== "Shell"
    ```
    august inspect
    ```
=== "main.august"
    ```august
    expose Build as build

    unit Build {
        depends_on(Test, OtherTest)

        println("Hi from build")
    }

    unit Test {
        exec(cargo build)
        fs::create_dir("test")
        println("Hi from test")
    }

    unit OtherTest {
        depends_on(Test)

        remove("test")

        println("Hi from other test")
    }
    ```
Output:
```
╭────────┬───────╮
│ Pragma │ Unit  │
╞════════╪═══════╡
│ Test   │       │
├────────┼───────┤
│ Build  │ Build │
╰────────┴───────╯
╭───────────┬──────────────╮
│ Unit      │ Dependencies │
╞═══════════╪══════════════╡
│ OtherTest │ Test         │
├───────────┼──────────────┤
│ Build     │ OtherTest    │
│           │ Test         │
├───────────┼──────────────┤
│ Test      │              │
╰───────────┴──────────────╯
```

## `august check`

Parses the build script to check for errors.
Doesn't run any units.
