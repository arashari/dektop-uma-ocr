// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use image::GenericImageView;
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use tauri::State;
use tesseract::Tesseract;
use tracing::info;
use strsim::jaro_winkler;

// JSON Event structures (matching events.json format)
#[derive(Debug, Serialize, Deserialize, Clone)]
struct JsonEvent {
    name: String,
    character_name: String,
    relation_type: String,
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Choice {
    text: String,
    number: String,
    outcome: String,
}


// Enhanced OCR result with event matching
#[derive(Debug, Serialize, Deserialize)]
struct EventMatch {
    event: JsonEvent,
    match_confidence: f32,
    match_type: String, // "event_name" or "choice_text"
    matched_text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OcrResult {
    text: String,
    confidence: f32,
    matched_events: Vec<EventMatch>,
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
    events: Vec<JsonEvent>,
}

impl AppState {
    fn new() -> Result<Self> {
        // Load JSON events
        let events = load_events_json()?;
        info!("Loaded {} events from events.json", events.len());

        Ok(AppState {
            events,
        })
    }
}

fn load_events_json() -> Result<Vec<JsonEvent>> {
    // Try multiple possible locations for events.json
    let mut possible_paths = vec![
        "events.json".to_string(),           // Current directory
        "../events.json".to_string(),        // Parent directory (from src-tauri)
    ];
    
    // For bundled app, try various resource locations
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Try in same directory as executable
            possible_paths.push(exe_dir.join("events.json").to_string_lossy().to_string());
            
            // Try in share directory (Linux package structure)
            possible_paths.push(exe_dir.join("../share/uma-helper/events.json").to_string_lossy().to_string());
            
            // Try in resources subdirectory
            possible_paths.push(exe_dir.join("resources/events.json").to_string_lossy().to_string());
        }
    }
    
    let mut events_content = String::new();
    let mut found_path = None;
    
    for path in &possible_paths {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                events_content = content;
                found_path = Some(path);
                break;
            }
            Err(_) => continue,
        }
    }
    
    if found_path.is_none() {
        // Fallback to embedded events.json
        info!("Using embedded events.json as fallback");
        events_content = include_str!("../../events.json").to_string();
    } else {
        info!("Loading events from: {}", found_path.unwrap());
    }
    
    let events: Vec<JsonEvent> = serde_json::from_str(&events_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse events.json: {}", e))?;
    
    Ok(events)
}

// Tauri commands
#[tauri::command]
async fn capture_screen_area(area: CaptureArea, state: State<'_, AppState>) -> Result<OcrResult, String> {
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
    perform_ocr(&cropped, &state).await
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

fn match_events_with_text(extracted_text: &str, events: &[JsonEvent]) -> Vec<EventMatch> {
    let mut matches = Vec::new();
    let threshold = 0.6; // Minimum similarity threshold
    
    // Clean and normalize extracted text for better matching
    let normalized_text = normalize_text(extracted_text);
    
    for event in events {
        // Try matching against event name
        let event_name_similarity = jaro_winkler(&normalize_text(&event.name), &normalized_text) as f32;
        if event_name_similarity >= threshold {
            matches.push(EventMatch {
                event: event.clone(),
                match_confidence: event_name_similarity,
                match_type: "event_name".to_string(),
                matched_text: event.name.clone(),
            });
        }
        
        // Try matching against choice texts
        for choice in &event.choices {
            let choice_similarity = jaro_winkler(&normalize_text(&choice.text), &normalized_text) as f32;
            if choice_similarity >= threshold {
                matches.push(EventMatch {
                    event: event.clone(),
                    match_confidence: choice_similarity,
                    match_type: "choice_text".to_string(),
                    matched_text: choice.text.clone(),
                });
            }
        }
        
        // Try partial word matching for OCR errors
        if event_name_similarity < threshold {
            let partial_similarity = calculate_partial_match(&normalized_text, &normalize_text(&event.name));
            if partial_similarity >= threshold + 0.1_f32 { // Higher threshold for partial matches
                matches.push(EventMatch {
                    event: event.clone(),
                    match_confidence: partial_similarity,
                    match_type: "partial_event_name".to_string(),
                    matched_text: event.name.clone(),
                });
            }
        }
    }
    
    // Sort by confidence (highest first) and limit results
    matches.sort_by(|a, b| b.match_confidence.partial_cmp(&a.match_confidence).unwrap());
    matches.truncate(5); // Return top 5 matches
    
    matches
}

fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn calculate_partial_match(ocr_text: &str, event_text: &str) -> f32 {
    let ocr_words: Vec<&str> = ocr_text.split_whitespace().collect();
    let event_words: Vec<&str> = event_text.split_whitespace().collect();
    
    if ocr_words.is_empty() || event_words.is_empty() {
        return 0.0;
    }
    
    let mut total_score = 0.0_f32;
    let mut matched_words = 0;
    
    for ocr_word in &ocr_words {
        let mut best_match = 0.0_f32;
        for event_word in &event_words {
            let similarity = jaro_winkler(ocr_word, event_word) as f32;
            if similarity > best_match {
                best_match = similarity;
            }
        }
        if best_match > 0.7 { // Word-level threshold
            total_score += best_match;
            matched_words += 1;
        }
    }
    
    if matched_words > 0 {
        total_score / ocr_words.len() as f32
    } else {
        0.0
    }
}

async fn perform_ocr(image: &image::DynamicImage, state: &AppState) -> Result<OcrResult, String> {
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
    
    // Match OCR text against events
    let extracted_text = text.trim().to_string();
    let matched_events = match_events_with_text(&extracted_text, &state.events);
    
    info!("Found {} matching events for text: '{}'", matched_events.len(), extracted_text);
    
    Ok(OcrResult {
        text: extracted_text,
        confidence,
        matched_events,
    })
}

#[tauri::command]
async fn lookup_event(extracted_text: String, state: State<'_, AppState>) -> Result<Vec<EventMatch>, String> {
    info!("Looking up events for text: {}", extracted_text);
    
    let matched_events = match_events_with_text(&extracted_text, &state.events);
    
    if matched_events.is_empty() {
        info!("No events found for text: {}", extracted_text);
    } else {
        info!("Found {} matching events for text: '{}'", matched_events.len(), extracted_text);
        for (i, event_match) in matched_events.iter().enumerate() {
            info!(
                "  {}. {} (confidence: {:.2}, type: {})", 
                i + 1, 
                event_match.matched_text, 
                event_match.match_confidence,
                event_match.match_type
            );
        }
    }
    
    Ok(matched_events)
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
            lookup_event
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}