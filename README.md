# Mac-Clip

A clipboard history manager for macOS that enhances your copy-paste workflow. Mac-Clip keeps track of your clipboard history and allows you to quickly access previously copied items through a simple keyboard shortcut.

## Features

- Maintains history of copied text
- Global hotkey (Command + Option + V) to show clipboard history
- Simple and clean user interface
- Persistent storage of clipboard history
- Maximum history size of 50 items
- Lightweight and efficient

## Installation

You can install Mac-Clip using Cargo:

```bash
cargo install mac-clip
```

Or build from source:

```bash
git clone [https://github.com/yourusername/mac-clip](https://github.com/aakkss37/mac-clip.git)
cd mac-clip
cargo build --release
```

## Usage

1. Run the application:
   ```bash
   mac-clip
   ```

2. The application will run in the background and monitor your clipboard
3. Press `Command + Option + V` to show the clipboard history window
4. Click on any item in the history to paste it

## Building from Source

1. Make sure you have Rust and Cargo installed
2. Clone the repository
3. Run `cargo build --release`
4. The binary will be available in `target/release/mac-clip`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
