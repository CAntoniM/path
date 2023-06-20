# path 

This is a collection of different of utilites for working with system paths
such as PATH, LD_LIBRARY_PATH, NODE_MODULES, etc. the goal is to make it easier
to manipulate the path and find elemtents on it.

```bash
Usage: path [OPTIONS] <COMMAND>

Commands:
  add     Add a folder to the path
  remove  Removes a folder from the path
  find    find files that are on the current path
  help    Print this message or the help of the given subcommand(s)

Options:
  -n, --name <NAME>  This is the name of the path that will be operated on (default: PATH)
  -h, --help         Print help
  -V, --version      Print version
```
