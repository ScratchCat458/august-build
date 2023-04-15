# Extending Scripts

So far, you should know the basic features of August build scripts.
Now, let's take a look at using scripts as modules.

## The link statement

The link statement finds an external module and makes its command definitions avaliable to another module.
They find modules based on their file name and module namespace.
Here is an example below:

=== "main.august"
    ```august
    @main
    
    #link ext_module
    #pragma build build

    Test build {
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


## Global Modules

Modules are first searched for in the directory of the build script.
If none are found, the global module directory is checked.
This is found under your home directory at `.august/modules`.
Global modules are a great way to allow for reusablity of command definitions if you need them across multiple build scripts.
However, when distributing projects that use August it is a better idea to ship the module in the repository,
rather than directing a contributor to download the module and add it themselves.

August is currently building up a series of modules for the purpose of global installation,
which have the purpose of creating command definition abstractions around common `exec` calls.
If you are interested, these are avaliable under the Modules tab.
