# Commands

Commands are the smallest unit of execution provided by the build script to the runtime.
They either run internal runtime code, modify the execution environment or call external processes.

!!! note
    Mostly incomplete

## Platform Agnotisic Functionality
For things such as IO and FS operations, rather than calling a system command,
the functionality of Rust's standard library is used.
This means that August won't run on `no-std` architectures.
However, chances are you won't be running CI on MIPS or some other exotic architecture.

### Read/Print File Content

### Make Directory/Empty File

### Set Environment Variable


## Internal Commands

### Exec
This is the command that will be used most frequently in build scripts.
Runs a command as given.

