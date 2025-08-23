# Desktop UMA OCR

A modern desktop OCR application built with Tauri for real-time text extraction from screen areas. Features intelligent text recognition with automatic white-text detection and a transparent floating window interface.

## ✨ Key Features

- **🎯 Smart OCR**: Automatic black/white text detection with intelligent preprocessing
- **🪟 Floating Window**: Transparent, always-on-top window with resizable targeting rectangle  
- **🔧 Interactive UI**: Drag to move, resize from bottom-right corner
- **🎮 Game-Friendly**: Designed for Umamusume Pretty Derby but works with any application
- **🚀 Native Performance**: Rust backend with lightweight web frontend
- **🔍 Debug Mode**: Saves processed images for troubleshooting
- **📱 Cross-Platform**: Windows, macOS, and Linux support

## 🚀 Quick Start

### Usage

1. **Launch the app** → Transparent floating window appears
2. **Position the window** over the text you want to capture
3. **Adjust the targeting rectangle**:
   - **Drag** the rectangle to move it
   - **Resize** by dragging the bottom-right corner
4. **Click "📷 Capture Event"** → OCR processes the targeted area
5. **View results** → Extracted text appears with confidence score

### Tips for Best Results

- **Black text on light backgrounds**: Works perfectly out of the box
- **White text on dark backgrounds**: Automatically detected and optimized
- **Small text**: Resize window smaller for better targeting
- **Large text blocks**: Expand the targeting rectangle as needed

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

## 🛠️ Technology Stack

- **Framework**: Tauri 2.0 (Rust + Web frontend)
- **OCR Engine**: Tesseract-rs with intelligent preprocessing
- **Screen Capture**: Screenshots crate (cross-platform)
- **Image Processing**: Image crate with custom algorithms
- **Frontend**: Vanilla HTML/CSS/JavaScript (no build process)
- **Database**: SQLite (optional, for future event data)

## 🧠 Smart OCR Features

### Automatic Text Detection
- **Brightness Analysis**: Samples image to detect white text on dark backgrounds
- **Smart Inversion**: Automatically converts white text to black for optimal OCR
- **Preprocessing Pipeline**: Grayscale → Scale → Invert (if needed) → Contrast enhance

### Image Processing
- **2x Upscaling**: Lanczos3 filtering for crisp text enlargement  
- **Minimal Processing**: Light contrast enhancement preserves text quality
- **Debug Output**: Saves `captured_image.png` and `processed_image.png`

### Tesseract Optimization
- **Character Whitelist**: Common text characters for better accuracy
- **Automatic Segmentation**: Lets Tesseract choose optimal processing mode
- **Confidence Scoring**: Shows OCR reliability percentage

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

## 🐛 Troubleshooting

**App won't start:**
- Check if you have required system libraries (GTK on Linux)
- Run from terminal to see error messages: `./uma-helper`

**Screen capture not working:**
- Grant screen recording permissions when prompted
- **macOS**: System Preferences → Security & Privacy → Screen Recording
- **Linux**: May need to run with proper display permissions

**OCR accuracy issues:**
- Check the debug images (`captured_image.png`, `processed_image.png`) in project folder
- **White text**: Should automatically invert - check console for "Inverting image" message
- **Small text**: Try resizing the targeting rectangle to be more precise
- **Mixed colors**: Works best with consistent text color in targeted area

**Window positioning issues:**
- Rectangle not capturing correctly: Check console for coordinate calculations
- Window too transparent: Adjust opacity in CSS if needed

## 🔧 Development

See `CLAUDE.md` for detailed technical documentation.

### Project Structure
- **Frontend**: `dist/` - Vanilla HTML/CSS/JS (no build process)
- **Backend**: `src-tauri/` - Rust with Tauri framework
- **Debug Images**: `captured_image.png`, `processed_image.png` (auto-generated)
- **Database**: `uma_events.db` (SQLite, created at runtime)

### Key Design Principles
- **Lightweight**: Minimal resource usage with smart algorithms
- **Responsive**: Real-time screen capture and OCR processing
- **Maintainable**: Clean Rust backend with simple frontend
- **Extensible**: Modular design for easy feature additions

## 🆘 Support

For issues and feature requests:
1. Check debug images in project folder for OCR troubleshooting
2. Review console output for detailed processing information  
3. Test with various text colors and sizes
4. Report bugs with system info and debug images