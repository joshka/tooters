# Toot-rs: A Rust TUI-based Mastodon App

Toot-rs (formerly known as tooters) is a Rust-based Terminal User Interface (TUI) Mastodon app that allows you to view multiple toots on the screen at once. The project was created as a means to learn Rust and scratch a personal itch - the desire to see multiple toots on the screen at once. See the awesome python based [toot](https://github.com/ihabunek/toot), for something more complete.

Visit our website at [toot.rs](https://toot.rs) for more information and updates.

[![asciicast](https://asciinema.org/a/576920.svg)](https://asciinema.org/a/576920)

## Known issues

- does not load second page of toots (yet)

## Features

- View multiple toots on the screen at once
- Rust-based TUI for a fast and efficient user experience
- Easy navigation and interaction with toots

## Installation

To install Toot-rs, you need to have Rust and Cargo installed on your system. If you don't have them installed, follow the instructions on the [official Rust website](https://www.rust-lang.org/tools/install).

Once Rust and Cargo are installed, you can install Toot-rs by running the following command:

```bash
cargo install toot-rs
```

## Usage

To start using Toot-rs, simply run the following command in your terminal:

```bash
toot-rs
```

You will be prompted to enter your Mastodon instance URL and login credentials. Once logged in, you can navigate and interact with toots using the keyboard shortcuts provided.

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

## Contributing

We welcome contributions to Toot-rs! If you'd like to contribute, please fork the repository, make your changes, and submit a pull request. If you find any bugs or have feature requests, please open an issue on the GitHub repository.

## License

Toot-rs is released under the MIT License. See the [LICENSE](LICENSE) file for more information.
