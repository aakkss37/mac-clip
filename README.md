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
git clone https://github.com/aakkss37/mac-clip.git
cd mac-clip
cargo build --release
```

## Usage

1. Install the application:
   ```bash
   cargo install mac-clip
   ```

2. Set up Mac-Clip to run automatically on startup:
   ```bash
   mac-clip --daemon
   ```
   This will configure Mac-Clip to start automatically when you log in.

3. The application will now:
   - Run in the background automatically when you log in
   - Monitor your clipboard
   - Be accessible via `Command + Option + V` to show the clipboard history window
   - Not require keeping a terminal window open

4. Click on any item in the history to paste it

To manually start Mac-Clip without setting up the daemon:
```bash
mac-clip
```

## Building from Source

1. Make sure you have Rust and Cargo installed
2. Clone the repository
3. Run `cargo build --release`
4. The binary will be available in `target/release/mac-clip`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
