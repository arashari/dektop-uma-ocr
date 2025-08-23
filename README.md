# Umamusume Event Helper

A modern desktop application built with Tauri for extracting event text from Umamusume Pretty Derby and displaying choice outcomes.

## Features

- **Self-Contained Executable**: No external dependencies required
- **Floating Window**: Always-on-top draggable window that stays above the game
- **Screen Capture**: Click and drag to select event dialog areas  
- **Built-in OCR**: English text extraction using bundled Tesseract
- **Event Database**: SQLite database with choice outcomes
- **Native Performance**: Rust backend with web frontend
- **Cross-Platform**: Windows, macOS, and Linux support

## Quick Start

### Download & Run

1. **Download** the latest release for your platform
2. **Run** the executable - no installation needed
3. **Grant permissions** when prompted for screen capture

### Usage

1. **Launch the app** → Small floating window appears
2. **Position the window** anywhere on screen (it stays above the game)
3. **When an event appears in Umamusume** → Click "📷 Capture Event"
4. **Select the event dialog area** → Click and drag over the event text
5. **View the results** → All choice outcomes are displayed
6. **Make your choice** in the game based on the displayed effects

## Building from Source

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies (Linux)
sudo apt-get install webkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Install system dependencies (macOS)
# Xcode Command Line Tools are required
```

### Build Commands

```bash
# Development build
cargo tauri dev

# Production build  
cargo tauri build

# The executable will be in src-tauri/target/release/
```

## Project Structure

```
desktop-uma-ocr/
├── src-tauri/
│   ├── src/main.rs          # Rust backend (OCR, database, capture)
│   ├── Cargo.toml           # Rust dependencies
│   ├── tauri.conf.json      # App configuration
│   └── build.rs             # Build script
├── dist/
│   ├── index.html           # Simple floating window UI
│   ├── styles.css           # Clean, minimal styling
│   └── script.js            # Frontend logic
└── README.md               # This file
```

## Technology Stack

- **Backend**: Rust with Tauri framework
- **OCR**: Tesseract-rs (bundled, no external installation)
- **Database**: SQLite with Rusqlite
- **Screen Capture**: Screenshots crate
- **Frontend**: HTML/CSS/JavaScript
- **Build System**: Cargo + Tauri CLI

## Key Benefits

- **No Dependencies**: Everything bundled in single executable
- **Small Size**: ~15-25MB total application size
- **Fast Performance**: Native Rust backend
- **Modern UI**: Clean, responsive interface
- **Always Updated**: Built-in updater support
- **Secure**: Sandboxed execution environment

## Database Structure

The app creates an SQLite database (`uma_events.db`) with:
- Event names and descriptions
- Choice options and their stat effects
- Outcome data for informed decision-making

Events can be added by modifying the Rust source code or through future UI additions.

## System Requirements

- **Windows**: Windows 10 or later
- **macOS**: macOS 10.15 or later  
- **Linux**: Modern Linux distribution with GTK 3.24+

## Troubleshooting

**App won't start:**
- Check if you have required system libraries
- Run from terminal to see error messages

**Screen capture not working:**
- Grant screen recording permissions when prompted
- On macOS: System Preferences → Security & Privacy → Screen Recording

**OCR accuracy issues:**
- Ensure event text is clearly visible
- Try capturing a larger area around the text
- Works best with high-contrast, clear text

## Development

This is a Tauri application combining Rust backend performance with modern web frontend flexibility. The app is designed to be:

- **Lightweight**: Minimal resource usage
- **Responsive**: Fast OCR and database queries  
- **Maintainable**: Clean separation of concerns
- **Extensible**: Easy to add new features

## Support

For issues and feature requests:
1. Check the troubleshooting section above
2. Ensure you're using the latest version
3. Test with clear, high-contrast event screenshots
4. Report bugs with system information and error logs