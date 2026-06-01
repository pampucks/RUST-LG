// Cargo.toml setup:
// [dependencies]
// wasm-bindgen = "0.2"
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

// Define the incoming message format from JavaScript
#[derive(Deserialize)]
struct IncomingCommand {
    req_id: u32,
    action: String,
    payload: Option<serde_json::Value>,
}

// Define the outgoing response format back to JavaScript
#[derive(Serialize)]
struct OutgoingResponse {
    req_id: u32,
    status: String,
    message: String,
}

// Import the Javascript callback function signature
#[wasm_bindgen]
extern "C" {
    // This allows Rust to call a JS function named 'scapCallbackBridge'
    fn scapCallbackBridge(response_json: &str);
}

#[wasm_bindgen]
pub fn process_signage_command(json_str: &str) {
    // 1. Parse the incoming JSON from the JS Adapter
    let command: IncomingCommand = match serde_json::from_str(json_str) {
        Ok(cmd) => cmd,
        Err(_) => {
            // Handle parsing error natively
            return; 
        }
    };

    // 2. Process the Core Logic (e.g., sync timings, state changes)
    let mut response = OutgoingResponse {
        req_id: command.req_id,
        status: String::from("ERROR"),
        message: String::from("Unknown Action"),
    };

    match command.action.as_str() {
        "SET_BRIGHTNESS" => {
            // Your core logic for adjusting display parameters
            response.status = String::from("SUCCESS");
            response.message = String::from("Brightness calculated and staged");
        },
        "SYNC_VIDEO" => {
            // Core logic for Android/LG/Samsung sync timings
            response.status = String::from("SUCCESS");
            response.message = String::from("Timing data calculated");
        },
        _ => {}
    }

    // 3. Serialize the response and send it back to the JS Adapter
    if let Ok(response_json) = serde_json::to_string(&response) {
        // Send the JSON string back across the boundary to JS
        scapCallbackBridge(&response_json);
    }
}

// Append this to the bottom of src/lib.rs instead to avoid editing Cargo.toml

#[wasm_bindgen]
extern "C" {
    // Bind directly to the browser's console.log
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn process_hardware_event(json_str: &str) {
    // Rust receives the confirmation from the JS SCAP API
    log(&format!("[Rust Core] Successfully confirmed hardware action: {}", json_str));
}