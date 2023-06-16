# The Basics

This guide will walk you through writing a basic build script for a Rust project.

Start by creating a file in the same directory as your `Cargo.toml` called `main.august` and add the following.

```august
@main
```

## Writing your first Task
A task is a high level execution unit that can be used by the CLI and can be dependent on the completing of other tasks.
This task prints the contents of `Cargo.toml` and runs `cargo build`:

```august
Task build {
  print_file("Cargo.toml");
  exec("cargo build");
}
```

Now lets make another task to run some tests!

```august
Task test {
  exec("cargo fmt");
  exec("cargo clippy");
  exec("cargo test");
}
```

We have now run into a problem, `test` will not run when `build` is called, and `test` needs to be run before `build`.
To fix this we introduce the dependency modifier, which does this exact thing.
Update the top line of the `build` task with this:

```august
Task build:[test] {
```

Adding the dependency modifier is optional for tasks no dependencies.
Both of these are valid syntax:
=== "With"

    ```august
    Task test:[] {
    ```
=== "Without"
    
    ```august
    Task test {    
    ```

## Writing your first Command Definition

Command defintions allow for extracting commands into a single call,
which assists with reusability and modularisation.
Our current script doesn't really need it, but it doesn't hurt to learn.

Let's take some of the commands in `test` and move them into a definition:
```august
cmddef lints {
  exec("cargo fmt");
  exec("cargo clippy");
}
```
Now we can swap this into our task:
```august
Task test {
  lints;
  exec("cargo test");
}
```

## Exposing tasks to the CLI

Currently, the August CLI is completely unaware about the semantics of the tasks we made.
To assign a task to a CLI command, we use a pragma.

```august
#pragma test test
#pragma build build
```

## Full Script

```august
@main

#pragma test test
#pragma build build

Task build {
  print_file("Cargo.toml");
  exec("cargo build");
}

Task test {
  lints;
  exec("cargo test");
}

cmddef lints {
  exec("cargo fmt");
  exec("cargo clippy");
}
```

## Runtime!

To run your build script, open the terminal of your choice and run:
```
august build
```
To run just the test suite:
```
august test
```

If your create a task that isn't assigned to a pragma you can use the more general command below:
```
august run build
```

[Next Steps :material-arrow-right:](../extending-august){ .md-button } 
