# Tooters: A Rust TUI-based Mastodon App

Tooters is a Rust-based Terminal User Interface (TUI) Mastodon app.

The project was created as a means to learn Rust and scratch a personal
itch See the awesome python based
[toot](https://github.com/ihabunek/toot), for something more complete.

Visit our website at [toot.rs](https://toot.rs) for more information and updates.

[![asciicast](https://asciinema.org/a/576920.svg)](https://asciinema.org/a/576920)

## Status

Experimental, work in progress.

## Known issues

- does not load second page of toots (yet)

## Features

- View multiple toots on the screen at once
- Rust-based TUI for a fast and efficient user experience
- Easy navigation and interaction with toots

## Installation

To install Tooters, you need to have Rust and Cargo installed on your system. If you don't have them
installed, follow the instructions on the [official Rust
website](https://www.rust-lang.org/tools/install).

Once Rust and Cargo are installed, you can install Tooters by running the following command:

```bash
cargo install tooters --locked
```

## Usage

To start using Tooters, simply run the following command in your terminal:

```bash
tooters
```

You will be prompted to enter your Mastodon instance URL and login credentials. Once logged in, you
can navigate and interact with toots using the keyboard shortcuts provided.

## Keyboard Shortcuts

- [x] `j` or `↓`: Move down
- [x] `k` or `↑`: Move up
- [ ] `h` or `←`: Move left (switch column)
- [ ] `l` or `→`: Move right (switch column)
- [ ] `n`: Compose a new toot
- [ ] `r`: Reply to the selected toot
- [ ] `b`: Boost the selected toot
- [ ] `f`: Favourite the selected toot
- [x] `q`: Quit

## License

Copyright (c) 2023-2024 Josh McKinney

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
