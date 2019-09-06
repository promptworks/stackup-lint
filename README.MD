## stackup-lint

linter for your stackup schema :mag:

### Installation

[Precompiled binaries are available for linux and macOS](https://github.com/stevesweetney/stackup-lint/releases)

### Command-line Usage

`stackup-lint <INPUT>`

where `<INPUT>` is a path to a file with a "graphql" extension
or use "-" for `<INPUT>` to read from stdin

| Flag | Default | Description                                              |
| ---- | ------- | -------------------------------------------------------- |
| `f`  | tty     | choose the specified format [possible values: tty, json] |

#### (Build from source)

First, [install Rust](https://www.rust-lang.org/en-US/install.html) if you don't have it.

Next, clone this repository and navigate to the new directory

You can also run `cargo install --path .`
which will build a binary in release mode and place it in your
~/.cargo/bin folder
