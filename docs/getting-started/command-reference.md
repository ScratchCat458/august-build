# Command Reference

!!! note
    This is **not** the only commands that will be avaliable in August.
    More are to be added in future.
    All commands shown below are supported by the syntax and runtime.

## Exec
:octicons-tag-24: 0.1.0
```august
exec(shell_cmd: StringLiteral);
```

## Set Environment Variable
:octicons-tag-24: 0.1.0
```august
set_env_var(var_name: StringLiteral, var_contents: StringLiteral);
```

## Make Directory
:octicons-tag-24: 0.1.0
```august
make_dir(dir_path: StringLiteral);
```

## Make Empty File
:octicons-tag-24: 0.1.0
```august
make_empty_file(file_path: StringLiteral);
```

## Move File
:octicons-tag-24: 0.1.1
```august
move_file(source: StringLiteral, destination:, StringLiteral);
```

## Copy File
:octicons-tag-24: 0.1.1
```august
copy_file(source: StringLiteral, destination: StringLiteral);
```

## Remove Directory
:octicons-tag-24: 0.1.1
```august
remove_dir(dir_path: StringLiteral);
```

## Remove File
:octicons-tag-24: 0.1.1
```august
remove_file(file_path: StringLiteral);
```

## Print String
:octicons-tag-24: 0.1.0
```august
print(text: StringLiteral);
```

## Print File Contents
:octicons-tag-24: 0.1.0
```august
print_file(file_path: StringLiteral);
```
