# Tasks

Tasks are the main high-level unit used in August build scripts.
These are to be executed by the runtime and cannot be used from external modules.
Tasks are made up of two components, dependencies and the commands to be executed.

## Dependencies
In terms of the build process, dependendcies are tasks that have to be run before another can be run.
This concept allows August to follow the montra of "run everything all at once unless otherwise specified",
which lends itself particularly well to parallelisation, which can make bigger builds substancially faster.
