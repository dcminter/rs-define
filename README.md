# Definitions Dictionary

A simple tool for curation and lookup of definitions and for other dictionary-like purposes

![Asciinema animation of rs-define in action](docs/rs-define.gif?raw=true "rs-define in action")

This is a reimplementation (and enhancement) of a [stupid bash script](https://github.com/dcminter/define) I created 
and used fairly regularly in my latest assignment.

The new implementation is in [Rust](https://www.rust-lang.org/) mostly as a personal learning exercise!

## Specification

*This will change a lot...*

Env var `DEFINITIONS_DICTIONARY_PATH` is consulted - if present the definitions paths are populated from this. If not present
the definitions paths are populated with the default paths:

  * `~/.config/define`
  * `~/.define`
  * `/etc/define` 

### In lookup mode (one parameter)

Each definition path is combined with the first parameter to give a file name until a user readable file is found.

  * If no file is found an error message is sent to stderr and the program exits with an **error** code.
  * If a file is found the contents are copied to stdout and the program exits with a **success** code.

### In store mode (two parameters)

Each definition path is combined with the first parameter to give a file name until a user writeable file is 
found (or a user writeable file can be created).

  * If no such file can be found or created an error message is sent to stderr and the program exits with an **error** code.
  * If a file is found or created then the contents of the second parameter are [normalised](#normalisation).
  * The contents of the file are tested for the presence of a normalised copy of the second parameter.
  * If the [normalised](#normalisation) copy of the second parameter is not found then it is appended as a new line to the file.
  * The normalised copy of the second parameter is then written to stdout and the program exits with a success code.

### Normalisation

*Not yet implemented*

Any whitespace between non-whitespace tokens is normalised to a single space character. The text is trimmed to remove 
any leading or trailing spaces. If the result is an empty string an error is written to stdout and the program exits 
with an error code.

## TODO

This is not an exhaustive list

  * ~~Lookup directory handling and parsing~~
  * ~~Complete the basic read behaviour~~
  * ~~Proper error handling (read path) including error levels~~
  * ~~Add ability to list the extant definitions~~
  * Tests!
  * ~~Logging output~~
  * ~~Drive logging from clap parameter instead of `DEFINE_LOG`(changed default from `RUST_LOG`) (and default to completely off!)~~
  * ~~Complete the basic write behaviour~~
  * ~~Handle the directory not existing during when storing a value (attempt to create)~~
  * ~~Proper error handling (write path) including error levels~~
  * ~~Colour (ANSI) output~~
  * ~~Add an [asciinema](https://asciinema.org/) demo to the README!~~
  * [Normalisation](#Normalisation)
  * Ignore attempts to add duplicate definitions
  * Tidy up any TODO issues
  * ~~Match read-paths ignoring case (i.e. currently "LOTR" != "LotR")~~ (achieved by squashing to lower-case by default, added flag to optionally disable)

## Future features

  * Tools to delete definitions (bumped to top priority as this is actually a feature I would use if I'd implemented it!)
    * At the moment when using the tool personally I go into the `~/.config/define` directory and delete the file manually; not ideal!
  * Namespacing
    * My thinking here is that one might wish to distinguish between definitions in various context, so that for example `define bff --context social-media` might output 'Best Friends Forever' but `define bff --context development` might output 'Backend For Frontend' ... but that's very clunky and does it add enough value to be worth bothering? Something to ponder. 
  * Allow for hosting of definitions on a remote server instead of, or as well as, in the local filesystem.
    * I'm torn between a custom protocol (which is more fun) and just using Redis or something similar that already exists.
  * Maybe fix up for Windows as well as Linux?
    * Although I don't have a Windows machine or want one, so maybe not. Besides, maybe it already works ok? Let me know if you know.

## Error codes

Errorlevels may change. Scripts should test for success (status == 0) or failure (status != 0) only.