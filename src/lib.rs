use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// 1. Data coming from JavaScript app layer into Rust
#[derive(Deserialize)]
struct IncomingCommand {
    req_id: u32,
    action: String,
    payload: Option<serde_json::Value>,
}

// 2. Data going from Rust out to the LG SCAP hardware bridge
#[derive(Serialize)]
struct OutgoingHardwareRequest {
    req_id: u32,
    action: String,
    value: u8,
}

#[wasm_bindgen]
extern "C" {
    // Connect to our JS bridge function
    fn scapCallbackBridge(json_str: &str);

    // Bind directly to the browser's console.log
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn process_signage_command(json_str: &str) {
    log("[Rust Core] Incoming command string received.");

    // Translate JSON string to Rust structures (Command Parsing)
    if let Ok(cmd) = serde_json::from_str::<IncomingCommand>(json_str) {
        if cmd.action == "SET_BRIGHTNESS" {
            
            // Extract the target number value safely, defaulting to 50 if missing
            let target_brightness = cmd.payload
                .and_then(|p| p.get("target").and_then(|t| t.as_u64()))
                .unwrap_or(50) as u8;

            log(&format!("[Rust Core] Validated safety metrics. Processing brightness parameter: {}%", target_brightness));

            // Structure the downstream hardware command match for JavaScript
            let hardware_req = OutgoingHardwareRequest {
                req_id: cmd.req_id,
                action: String::from("SET_PICTURE_PROPERTY"),
                value: target_brightness,
            };

            // Stringify back into a JSON structure and pass across the bridge to JS
            if let Ok(out_str) = serde_json::to_string(&hardware_req) {
                scapCallbackBridge(&out_str);
            }
        }
    } else {
        log("[Rust Core] Error: Could not parse incoming application payload.");
    }
}

#[wasm_bindgen]
pub fn process_hardware_event(json_str: &str) {
    // Rust receives the absolute confirmation directly from the physical SCAP callbacks
    log(&format!("[Rust Core] Hardware Confirmation Received: {}", json_str));
}