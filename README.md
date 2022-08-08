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

### Trash a path

```bash
$ trash first second third
```

This is just sugar for 

```bash
$ trash put first second third
```

### Listing items in the trash

```bash
$ trash list
```

### Restoring a file

```bash
$ trash restore first second
```

By default the arguments given are interpreted as regular expressions. Use the `-m` option to interpret them differently.

## Installation

### Using cargo

clone the github repo using `git clone https://github.com/oberblastmeister/trashy.git`

build using `cargo build --release`

the binary should be at `target/release/trash`

## License

Copyright (c) 2020 Brian Shu

*trashy* is distributed under the terms of both the MIT license and the Apache License 2.0.

See the [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) -->
