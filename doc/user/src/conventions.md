# Conventions

imag has a few conventions which we try to enforce in all module
implementations. This chapter explains the conventions an end-user of imag can
rely on.


## Commandline interface

The commandline interface of every subcommand has a "--help" flag which can be
used to print helptext for the subcommand.

Every imag module implementation can be used for scripting, where it behaves in
the following ways:

* If the standard input is a pipe, imag module implementations assume that store
  ids are written to that pipe line by line.
* If the standard output is a pipe, every imag module prints the StoreIDs it
  touched while running to the pipe.
  All other output is written to stderr.
  Piping can be used to combine imag commands.
* If the standard output is not a pipe, the imag module does not print the
  StoreIDs it touched.

This behaviour can be overridden with the `--ignore-ids` flag.

## Versioning

imag modules are compatible to eachother as long as the version number is in the
`0.x.y` range.
Modules from different imag versions are not supported to be compatible to
eachother.


## Commandline capabilities

The imag commands can be used to access _all_ data that is stored in the imag
store.
Alternatively, standard unix commandline tools (like `grep`, `cut`, `sed`, ...)
can be used to access all data and all data-points.

In short: All data imag stores is stored in plain text, containing a structured
part (in the markup language "toml") and a plain-text part (which should be
UTF-8 encoded).


