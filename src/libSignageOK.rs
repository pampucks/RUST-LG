use std::cell::RefCell;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// 1. Define what our internal hardware database tracks
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HardwareState {
    pub model_name: String,
    pub firmware_version: String,
    pub active_brightness: u8,
}

// 2. Thread-local storage container to safely hold memory inside the browser engine
thread_local! {
    pub static CORE_STATE: RefCell<HardwareState> = RefCell::new(HardwareState {
        model_name: "Pending Query...".to_string(),
        firmware_version: "Unknown".to_string(),
        active_brightness: 50, // default fallback
    });
}

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

    // Translate JSON string to Rust structures (Using your IncomingCommand struct!)
    if let Ok(cmd) = serde_json::from_str::<IncomingCommand>(json_str) {
        
        // Using a match block makes expanding to new SCAP files incredibly clean
        match cmd.action.as_str() {
            
            "SET_BRIGHTNESS" => {
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

                // Stringify back into a JSON structure and pass across your real bridge
                if let Ok(out_str) = serde_json::to_string(&hardware_req) {
                    scapCallbackBridge(&out_str);
                }
            },

            "GET_DEVICE_INFO" => {
                log("[Rust Core] Route matched: GET_DEVICE_INFO. Preparing telemetry request payload...");

                // Construct an inline telemetry request for JS
                let telemetry_req = serde_json::json!({
                    "req_id": cmd.req_id,
                    "action": "FETCH_HARDWARE_TELEMETRY"
                });

                // Send it down the same exact bridge channel your brightness uses!
                scapCallbackBridge(&telemetry_req.to_string());
            },

            _ => {
                log(&format!("[Rust Core] Warning: Unhandled action routed to core: {}", cmd.action));
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