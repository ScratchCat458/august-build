# Command Reference

## Execute Program

```august
exec(cargo build)
exec("./some/local/executable" arg)
```

---
Spawns a process and waits for it to complete.

Arguments can be either a string literal or identifier.
Note that the lexical definition of an identifier is loosened for this command to provide better ergonomics.

## Metadata

```august
meta(
    @attr_name "attr_value"
    @desc "attr_value"
)
```

---
Adds metadata to the current unit which can be viewed with `august inspect`.

This command is a no op at runtime.

## Depends On

```august
depends_on(Test)
depends_on(A, B, C)
```

---
Makes a unit a dependency of the current unit.

This command is a no op at runtime.

## Do Unit

```august
do(A)
do(A, B)
```

---
Runs a unit sequentially.

!!! example
    ```august
    do(A, B)
    ```
    Is equivalent to:
    ```august
    do(A)
    do(B)
    ```

## Concurrency Block

:material-tag: 0.6

```august
concurrent {
    do(A)
    ~(cargo build)
    ~(npm run build)
}
```

---
Runs multiple commands at the same time.

August runs the commands in a unit sequentially and a unit's dependencies in parallel.
The concurrency block is useful for running multiple independent commands inline rather than creating new units.

The example above can be written without the concurrency block like so:
```august
unit ConcurrencyBlock {
    depends_on(__command_a, __command_b, __command_c)
}

unit __command_a {
    do(A)
}

unit __command_b {
    ~(cargo build)
}

unit __command_c {
    ~(npm run build)
}
```


## Module: fs

### Create File

```august
fs::create("path")
```

---
Creates an empty file.

### Create Directory

```august
fs::create_dir("path\of\dir")
```

---
Recursively creates an empty directory.

### Remove File/Directory

```august
fs::remove("path")
fs::remove("path\to\dir")
```

---
Removes a file or recursively removes a directory.

### Copy File/Directory

```august
fs::copy("src", "dst")
fs::copy("path\to\src\dir", "path\to\dst\dir")
```

---
Copies a file or directory to a different location.

### Move File/Directory

```august
fs::move("src", "dst")
fs::move("path\to\src\dir", "path\to\dst\dir")
```

---
Moves a file or directory to a different location.

!!! example
    ```august
    move("src", "dst")
    ```
    Is equivalent to the following:
    ```august
    copy("src", "dst")
    remove("src")
    ```


### Print/EPrint File

```august
fs::print_file("path")
fs::eprint_file("path")
```

---
Print the contents of a file to `stdout` or `stderr`


## Module: io

### Print

```august
io::print("Hello, World!")
io::println("Hello, World!")
io::eprint("Hello, World!")
io::eprintln("Hello, World!")
```

---
Prints text to `stdout` or `stderr`

## Module: env

### Set Environment Variable

```august
env::set_var("VAR_NAME", "var_content")
```

---
Sets the value of an environment variable for the `august` process, which is inherited by child processes spawned by `exec`.

### Remove Environment Variable

```august
env::remove_var("VAR_NAME")
```

---
Clears the value of an environment variable for the `august` process, which is inherited by child processes spawned by `exec`.

### Add to PATH

```august
env::path_push("path\to\dir")
```

---
Adds a directory to the `PATH` environment variable for the `august` process,
which is inherited by child processes spawned by `exec`.

Directory path is canonicalised before it is added.

### Remove from PATH

```august
env::path_remove("path\to\dir")
```

---
Remove a directory to the `PATH` environment variable for the `august` process,
which is inherited by child processes spawned by `exec`.

Directory path is canonicalised during comparison for removal.
