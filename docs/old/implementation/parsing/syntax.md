# Syntax

```august title="Full Example"
@main

#link rust
#pragma build build

Task build:[test] {
  rust.build;
  print_file("Cargo.toml");
}

Task test {
  working_dir(".");
  my_cmd;
  exec("cargo test");
}

cmddef my_cmd {
  print_file("Cargo.toml");
  set_env_var("var", "cont");
}
```

## Namespace

```august
@main
```

## Pragma

```august
#pragma test test
#pragma build build
```

## Link Statement

```august
#link rust
#link clang
```

## Task

=== "With Dependencies"
    ```august
    Task build:[test] {
      
    } 
    ```

=== "Without Dependencies"
    ```august
    Task test {

    }
    ```

## Command Definiton

```august
cmddef my_cmd {
  
}
```

## Command Call

=== "Internal"
    ```august
    exec("cargo test");
    print_file("Cargo.toml");
    set_env_var("var", "cont");
    ```

=== "Local"
    ```august
    my_cmd;
    ```

=== "External"
    ```august
    rust.build;
    ```

