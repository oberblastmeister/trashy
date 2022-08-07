# trashy

![Build Status](https://github.com/oberblastmeister/trash-cli/workflows/ci/badge.svg)

*trashy* is a simple, fast, and featureful alternative to *rm* and *trash-cli* written in rust.

## Features

- easy to use, just run `trash PATH`
- recursive by default, without having the issues
- beautiful output
    - colorized paths (similar to *fd*)
    - cool tables
- very fast, and faster than trash-cli
- much safer than `rm -rf`

## Usage

```
trashy 0.1.0
Brian Shu <littlebubu.shu@gmail.com>
trash-cli written in rust

USAGE:
    trash [FLAGS] [paths]... [SUBCOMMAND]

ARGS:
    <paths>...    

FLAGS:
    -d, --directory      ignored (for GNU rm compatibility)
    -f, --force          ignored (for GNU rm compatibility)
    -h, --help           Prints help information
    -i, --interactive    ignored (for GNU rm compatibility)
    -r, --R              ignored (for GNU rm compatibility)
        --recursive      
    -v, --verbose        How verbose to log. The verbosity is error by default
    -V, --version        Prints version information

SUBCOMMANDS:
    completion    Generates completions for shell
    empty         PERMANANTLY removes ALL files in the trash
    help          Prints this message or the help of the given subcommand(s)
    list          list valid files in the trash
    put           Put files into trash. Is run by default if no subcommand is specified
    remove        PERMANANTLY removes files from the trash
    restore       Restore files from the trash
```

## Installation

### Using cargo

clone the github repo using `git clone https://github.com/oberblastmeister/trashy.git`

build using `cargo build --release`

the binary should be at `target/release/trash`

### Arch Linux 

AUR package coming soon!

## License

Copyright (c) 2020 Brian Shu

*trashy* is distributed under the terms of both the MIT license and the Apache License 2.0.

See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT)
