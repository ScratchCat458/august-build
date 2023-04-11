# Module Resolution and Aggregation

## Storage and Referencing
When parsing a structure of modules, the first one to be parsed is the build script.
After this, all of the modules given by link statements must be found.
This search occurs first in the same folder as the build script, then in the global module store.


## Aggregation
The linking process pretty much just copies over the command definitions and makes them external instead of local.

$$
\text{Local(cmd_name)} \Rightarrow \text{External(namespace, cmd_name)}
$$
