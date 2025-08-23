// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use image::GenericImageView;
use rusqlite::Connection;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;
use tesseract::Tesseract;
use tracing::{error, info};

// Database structures
#[derive(Debug, Serialize, Deserialize)]
struct Event {
    id: Option<i64>,
    event_name: String,
    event_name_jp: Option<String>,
    choice1_text: Option<String>,
    choice1_effects: Option<String>,
    choice2_text: Option<String>,
    choice2_effects: Option<String>,
    choice3_text: Option<String>,
    choice3_effects: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OcrResult {
    text: String,
    confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct CaptureArea {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

// Application state
struct AppState {
    db: Mutex<Connection>,
}

impl AppState {
    fn new() -> Result<Self> {
        // Store database in user's home directory to avoid dev rebuild loops
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let db_path = format!("{}/uma_events.db", home_dir);
        let conn = Connection::open(db_path)?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_name TEXT UNIQUE NOT NULL,
                event_name_jp TEXT,
                choice1_text TEXT,
                choice1_effects TEXT,
                choice2_text TEXT,
                choice2_effects TEXT,
                choice3_text TEXT,
                choice3_effects TEXT,
                notes TEXT
            )",
            [],
        )?;

        // Insert sample data
        conn.execute(
            "INSERT OR IGNORE INTO events 
            (event_name, event_name_jp, choice1_text, choice1_effects, choice2_text, choice2_effects, choice3_text, choice3_effects)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            [
                "Sample Training Event",
                "Sample Training Event JP",
                "Train harder",
                "+15 Speed, -10 Stamina",
                "Take it easy", 
                "+5 Wisdom, +10 Health",
                "Focus on technique",
                "+10 Technique, +5 Guts"
            ],
        )?;

        Ok(AppState {
            db: Mutex::new(conn),
        })
    }
}

// Tauri commands
#[tauri::command]
async fn capture_screen_area(area: CaptureArea) -> Result<OcrResult, String> {
    info!("Capturing screen area: {:?}", area);
    
    // Get all screens
    let screens = Screen::all().map_err(|e| format!("Failed to get screens: {}", e))?;
    
    if screens.is_empty() {
        return Err("No screens found".to_string());
    }

    // Use primary screen
    let screen = &screens[0];
    
    // Capture the screen  
    let screen_image = screen.capture().map_err(|e| format!("Failed to capture screen: {}", e))?;
    
    // Convert screenshot to DynamicImage
    let (width, height) = (screen_image.width(), screen_image.height());
    let rgba_data = screen_image.rgba().to_vec();
    let image_buffer = image::ImageBuffer::from_raw(width, height, rgba_data)
        .ok_or("Failed to create image buffer")?;
    let dynamic_image = image::DynamicImage::ImageRgba8(image_buffer);
    
    // Crop to the specified area
    let cropped = crop_image(&dynamic_image, area)?;
    
    // Save captured image for debugging
    save_debug_image(&cropped, "captured_image.png")?;
    
    // Perform OCR
    perform_ocr(&cropped).await
}

fn save_debug_image(image: &image::DynamicImage, filename: &str) -> Result<(), String> {
    // Save to project root directory for easy access during development
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    let debug_path = current_dir.join(filename);
    
    image.save(&debug_path)
        .map_err(|e| format!("Failed to save debug image: {}", e))?;
    
    info!("Debug image saved to: {}", debug_path.display());
    Ok(())
}

fn crop_image(image: &image::DynamicImage, area: CaptureArea) -> Result<image::DynamicImage, String> {
    let (img_width, img_height) = image.dimensions();
    
    // Ensure crop area is within bounds
    let x = area.x.max(0) as u32;
    let y = area.y.max(0) as u32;
    let width = area.width.min(img_width.saturating_sub(x));
    let height = area.height.min(img_height.saturating_sub(y));
    
    if width == 0 || height == 0 {
        return Err("Invalid crop area".to_string());
    }
    
    let cropped = image.crop_imm(x, y, width, height);
    Ok(cropped)
}

fn preprocess_image_for_ocr(image: &image::DynamicImage) -> image::DynamicImage {
    use image::imageops;
    
    // Convert to grayscale for better OCR
    let gray_image = image.to_luma8();
    
    // Analyze image to determine if we should invert (for white text on dark background)
    let should_invert = analyze_text_brightness(&gray_image);
    
    // Scale up the image for better OCR (2x scale)
    let (width, height) = gray_image.dimensions();
    let scaled_width = width * 2;
    let scaled_height = height * 2;
    
    let scaled_gray = imageops::resize(
        &gray_image,
        scaled_width,
        scaled_height,
        imageops::FilterType::Lanczos3,
    );
    
    // Apply minimal processing - just inversion if needed
    let processed_gray = if should_invert {
        info!("Inverting image for white text detection");
        invert_image(&scaled_gray)
    } else {
        scaled_gray
    };
    
    // Convert back to DynamicImage with minimal contrast adjustment
    let processed = image::DynamicImage::ImageLuma8(processed_gray);
    
    // Apply light contrast enhancement only
    let contrasted = imageops::contrast(&processed, 20.0);
    
    // Return the contrasted image wrapped in DynamicImage
    image::DynamicImage::ImageRgba8(contrasted)
}

fn analyze_text_brightness(gray_image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> bool {
    // Sample pixels to determine if the image has more dark background (indicating white text)
    let (width, height) = gray_image.dimensions();
    let mut dark_pixels = 0;
    let mut total_pixels = 0;
    
    // Sample every 4th pixel to check brightness distribution
    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            let pixel = gray_image.get_pixel(x, y);
            let brightness = pixel[0];
            
            if brightness < 128 {
                dark_pixels += 1;
            }
            total_pixels += 1;
        }
    }
    
    // If more than 60% of pixels are dark, likely white text on dark background
    let dark_ratio = dark_pixels as f32 / total_pixels as f32;
    info!("Dark pixel ratio: {:.2}, should_invert: {}", dark_ratio, dark_ratio > 0.6);
    
    dark_ratio > 0.6
}

fn invert_image(gray_image: &image::ImageBuffer<image::Luma<u8>, Vec<u8>>) -> image::ImageBuffer<image::Luma<u8>, Vec<u8>> {
    let (width, height) = gray_image.dimensions();
    let mut inverted = image::ImageBuffer::new(width, height);
    
    for (x, y, pixel) in gray_image.enumerate_pixels() {
        let inverted_value = 255 - pixel[0];
        inverted.put_pixel(x, y, image::Luma([inverted_value]));
    }
    
    inverted
}

async fn perform_ocr(image: &image::DynamicImage) -> Result<OcrResult, String> {
    info!("Performing OCR on captured image");
    
    // Preprocess image for better OCR
    let processed_image = preprocess_image_for_ocr(image);
    
    // Save processed image for debugging
    save_debug_image(&processed_image, "processed_image.png")?;
    
    // Save processed image to memory as PNG for Tesseract
    let mut image_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut image_bytes);
    
    processed_image.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image: {}", e))?;
    
    info!("Image processed and encoded as PNG, size: {} bytes", image_bytes.len());
    
    // Initialize Tesseract with better settings
    let tesseract = Tesseract::new(None, Some("eng"))
        .map_err(|e| format!("Failed to initialize Tesseract: {}", e))?;
    
    // Configure Tesseract for better text recognition (chain the method calls)
    let tesseract = tesseract
        .set_variable("tessedit_char_whitelist", "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 .,!?'-:()[]{}\"")
        .map_err(|e| format!("Failed to set character whitelist: {}", e))?;
    
    // Use automatic page segmentation - let Tesseract decide
    let tesseract = tesseract
        .set_variable("tessedit_pageseg_mode", "3") // Fully automatic page segmentation
        .map_err(|e| format!("Failed to set page segmentation mode: {}", e))?;
    
    // Set image from memory (PNG format)
    let mut tesseract = tesseract.set_image_from_mem(&image_bytes)
        .map_err(|e| format!("Failed to set image: {}", e))?;
    
    // Get text and confidence
    let text = tesseract
        .get_text()
        .map_err(|e| format!("Failed to extract text: {}", e))?;
    
    let confidence = tesseract.mean_text_conf() as f32;
    
    info!("OCR completed. Text length: {}, Confidence: {}", text.len(), confidence);
    
    Ok(OcrResult {
        text: text.trim().to_string(),
        confidence,
    })
}

#[tauri::command]
async fn lookup_event(extracted_text: String, state: State<'_, AppState>) -> Result<Option<Event>, String> {
    info!("Looking up event for text: {}", extracted_text);
    
    let db = state.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let mut stmt = db
        .prepare("SELECT id, event_name, event_name_jp, choice1_text, choice1_effects, choice2_text, choice2_effects, choice3_text, choice3_effects, notes FROM events WHERE event_name LIKE ?1 OR event_name_jp LIKE ?1")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let search_pattern = format!("%{}%", extracted_text);
    let event_result = stmt.query_row([&search_pattern], |row| {
        Ok(Event {
            id: Some(row.get(0)?),
            event_name: row.get(1)?,
            event_name_jp: row.get(2).ok(),
            choice1_text: row.get(3).ok(),
            choice1_effects: row.get(4).ok(),
            choice2_text: row.get(5).ok(),
            choice2_effects: row.get(6).ok(),
            choice3_text: row.get(7).ok(),
            choice3_effects: row.get(8).ok(),
            notes: row.get(9).ok(),
        })
    });
    
    match event_result {
        Ok(event) => {
            info!("Found event: {}", event.event_name);
            Ok(Some(event))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            info!("No event found for text: {}", extracted_text);
            Ok(None)
        }
        Err(e) => {
            error!("Database error: {}", e);
            Err(format!("Database error: {}", e))
        }
    }
}

#[tauri::command]
async fn add_event(event: Event, state: State<'_, AppState>) -> Result<i64, String> {
    info!("Adding new event: {}", event.event_name);
    
    let db = state.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let result = db.execute(
        "INSERT INTO events (event_name, event_name_jp, choice1_text, choice1_effects, choice2_text, choice2_effects, choice3_text, choice3_effects, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        [
            &event.event_name,
            &event.event_name_jp.unwrap_or_default(),
            &event.choice1_text.unwrap_or_default(),
            &event.choice1_effects.unwrap_or_default(),
            &event.choice2_text.unwrap_or_default(),
            &event.choice2_effects.unwrap_or_default(),
            &event.choice3_text.unwrap_or_default(),
            &event.choice3_effects.unwrap_or_default(),
            &event.notes.unwrap_or_default(),
        ],
    );
    
    match result {
        Ok(_) => {
            let id = db.last_insert_rowid();
            info!("Event added successfully with ID: {}", id);
            Ok(id)
        }
        Err(e) => {
            error!("Failed to add event: {}", e);
            Err(format!("Failed to add event: {}", e))
        }
    }
}

#[tauri::command]
async fn get_all_events(state: State<'_, AppState>) -> Result<Vec<Event>, String> {
    info!("Getting all events");
    
    let db = state.db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let mut stmt = db
        .prepare("SELECT id, event_name, event_name_jp, choice1_text, choice1_effects, choice2_text, choice2_effects, choice3_text, choice3_effects, notes FROM events ORDER BY event_name")
        .map_err(|e| format!("Failed to prepare statement: {}", e))?;
    
    let event_iter = stmt
        .query_map([], |row| {
            Ok(Event {
                id: Some(row.get(0)?),
                event_name: row.get(1)?,
                event_name_jp: row.get(2).ok(),
                choice1_text: row.get(3).ok(),
                choice1_effects: row.get(4).ok(),
                choice2_text: row.get(5).ok(),
                choice2_effects: row.get(6).ok(),
                choice3_text: row.get(7).ok(),
                choice3_effects: row.get(8).ok(),
                notes: row.get(9).ok(),
            })
        })
        .map_err(|e| format!("Failed to query events: {}", e))?;
    
    let mut events = Vec::new();
    for event in event_iter {
        events.push(event.map_err(|e| format!("Failed to parse event: {}", e))?);
    }
    
    info!("Retrieved {} events", events.len());
    Ok(events)
}

// Removed window creation commands as they're not supported in current Tauri version
// The frontend will handle selection overlay directly

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Uma Helper");
    
    // Initialize application state
    let app_state = AppState::new().expect("Failed to initialize application state");
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            capture_screen_area,
            lookup_event,
            add_event,
            get_all_events
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}