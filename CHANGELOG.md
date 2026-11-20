# Upcoming release

## Bugfixes

- `trashy empty` no longer panics on an item whose file is missing from the trash (#124)
- `trashy list` no longer panics in a debug build on a terminal narrower than 30 columns

## Other

- every dependency is on its latest release, and the minimum Rust version is 1.87

# v2.0.0

## Features

- `trashy restore` will pull up an interactive prompt if no arguments are given
- upgrade to clap v4

## Bugfixes

- all filters must match instead of only one
