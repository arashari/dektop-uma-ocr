# Desktop UMA OCR - Project Summary

## Project Overview
A modern desktop OCR application built with Tauri for extracting text from screen areas, specifically designed for Umamusume Pretty Derby game event text recognition.

## Architecture

### Frontend (dist/)
- **Technology**: Vanilla HTML/CSS/JavaScript
- **UI**: Transparent floating window with resizable targeting rectangle
- **Features**: 
  - Draggable and resizable targeting rectangle (bottom-right resize handle)
  - Real-time screen capture behind transparent window
  - OCR results display with confidence scores
  - Debug console output for troubleshooting

### Backend (src-tauri/)
- **Technology**: Rust with Tauri framework
- **OCR Engine**: Tesseract-rs with intelligent preprocessing
- **Database**: SQLite for event data storage (optional)
- **Screen Capture**: Screenshots crate for cross-platform capture

## Key Features

### 1. Smart OCR Processing
- **Automatic text color detection**: Analyzes image to detect white text on dark backgrounds
- **Intelligent inversion**: Automatically inverts white text to black for better OCR accuracy
- **Minimal preprocessing**: Grayscale conversion, 2x upscaling, light contrast enhancement
- **Tesseract optimization**: Character whitelist, automatic page segmentation

### 2. Interactive Targeting Rectangle
- **Fully responsive**: Scales with window resizing
- **Draggable**: Click and drag to position anywhere in window
- **Resizable**: Bottom-right handle for width/height adjustment
- **Visual feedback**: Blue border with semi-transparent overlay
- **Screen coordinate calculation**: Accurate mapping to actual screen pixels

### 3. Debug Capabilities
- **Image saving**: Saves `captured_image.png` and `processed_image.png` in project root
- **Console logging**: Detailed OCR process information
- **Brightness analysis**: Shows dark pixel ratios and inversion decisions
- **Area coordinates**: Displays calculated capture areas

## Technical Implementation

### OCR Pipeline
1. **Screen Capture**: Screenshots crate captures specified screen area
2. **Preprocessing**: 
   - Convert to grayscale
   - Analyze brightness distribution (60% dark threshold for inversion)
   - Apply 2x Lanczos3 upscaling
   - Invert if white text detected
   - Light contrast enhancement (20.0)
3. **Tesseract Processing**:
   - Character whitelist for common text
   - Automatic page segmentation (mode 3)
   - PNG format input for best quality
4. **Result Display**: Text with confidence percentage

### Window Management
- **Tauri Configuration**: 280x420px, resizable (250-500px width, 300-800px height)
- **Always on top**: Stays above game windows
- **Transparent background**: See-through for targeting
- **Global screen coordinates**: Accounts for window decorations and positioning

### Database Structure
```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_name TEXT UNIQUE NOT NULL,
    event_name_jp TEXT,
    choice1_text TEXT, choice1_effects TEXT,
    choice2_text TEXT, choice2_effects TEXT,
    choice3_text TEXT, choice3_effects TEXT,
    notes TEXT
);
```

## Development Workflow

### Build Commands
```bash
# Development
cargo tauri dev

# Production build
cargo tauri build
```

### File Structure
```
desktop-uma-ocr/
├── dist/                    # Frontend (HTML/CSS/JS)
├── src-tauri/              # Rust backend
│   ├── src/main.rs         # Core application logic
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # App configuration
├── captured_image.png      # Debug: Raw captured area
├── processed_image.png     # Debug: Preprocessed for OCR
└── uma_events.db          # SQLite database (created at runtime)
```

### Dependencies
- **Tauri 2.0**: Desktop app framework
- **Tesseract 0.13**: OCR engine
- **Image 0.25**: Image processing
- **Screenshots 0.7**: Screen capture
- **Rusqlite 0.32**: SQLite database
- **Tracing**: Logging system

## Configuration

### Tauri Settings
- **Window**: Transparent, always-on-top, resizable
- **Global Tauri**: Enabled for JavaScript API access
- **Security**: CSP disabled for development flexibility

### OCR Settings
- **Language**: English only
- **Character whitelist**: Letters, numbers, common punctuation
- **Engine mode**: Default Tesseract
- **Page segmentation**: Fully automatic (mode 3)

## Testing & Debugging
- **Debug images**: Automatically saved to project root
- **Console logging**: Detailed process information
- **Screen coordinate validation**: Visual rectangle matches capture area
- **OCR confidence scores**: Quality assessment for each recognition

## Performance Optimizations
- **Minimal preprocessing**: Avoids over-processing that degrades quality
- **Smart inversion**: Only when needed based on brightness analysis
- **Efficient scaling**: 2x upscaling balances quality and performance
- **Memory management**: Proper image buffer handling in Rust

## Known Limitations
- **English text only**: Tesseract configured for English language
- **Screen capture permissions**: May require OS permission grants
- **Single screen support**: Uses primary screen only
- **Text size dependency**: Very small text may have reduced accuracy

## Future Enhancements
- **Multi-language support**: Add Japanese OCR for full Umamusume support
- **Hotkey support**: Global shortcuts for capture
- **Multiple screen support**: Screen selection capability
- **Export functionality**: Save OCR results to file
- **Custom preprocessing profiles**: User-selectable OCR optimization modes