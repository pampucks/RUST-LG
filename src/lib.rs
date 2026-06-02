use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// --- 1. CORE ARCHITECTURE STRUCTURES ---

#[derive(Serialize, Clone, Debug)]
pub struct PendingTransaction {
    pub action: String,
    pub created_at_ms: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct CoreState {
    pub device_model: String,
    pub firmware_version: String,
    pub current_brightness: u8,
    pub hardware_fault_detected: bool,
    pub pending_transactions: HashMap<u64, PendingTransaction>,
}

// Thread-local storage guarantees safe memory accessibility inside the WASM boundary
thread_local! {
    static CORE_STATE: RefCell<CoreState> = RefCell::new(CoreState {
        device_model: String::from("Unknown"),
        firmware_version: String::from("Unknown"),
        current_brightness: 50, // Default baseline fallback
        hardware_fault_detected: false,
        pending_transactions: HashMap::new(),
    });
}

// Input JSON structures mapping your JS commands
#[derive(Deserialize, Debug)]
pub struct IncomingCommand {
    pub req_id: u64,
    pub action: String,
    pub payload: Option<serde_json::Value>,
    pub client_timestamp_ms: Option<f64>, // Passed from JS to drive the watchdog
}

#[derive(Serialize, Debug)]
pub struct OutgoingHardwareRequest {
    pub req_id: u64,
    pub action: String,
    pub value: u8,
}

// --- 2. BINDINGS TO JAVASCRIPT ---

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = log)]
    fn js_log(s: &str);
    fn scapCallbackBridge(s: &str);
}

fn log(msg: &str) {
    js_log(msg);
}

// --- 3. MAIN WASM ENGINE LOGIC ---

#[wasm_bindgen]
pub fn process_signage_command(json_str: &str) {
    log("[Rust Core] Incoming command string received.");

    if let Ok(cmd) = serde_json::from_str::<IncomingCommand>(json_str) {
        let timestamp = cmd.client_timestamp_ms.unwrap_or(0.0);

        match cmd.action.as_str() {
            "SET_BRIGHTNESS" => {
                let target_brightness = cmd.payload
                    .and_then(|p| p.get("target").and_then(|t| t.as_u64()))
                    .unwrap_or(50) as u8;

                // Track the transaction before dispatching
                CORE_STATE.with(|state| {
                    state.borrow_mut().pending_transactions.insert(cmd.req_id, PendingTransaction {
                        action: String::from("SET_BRIGHTNESS"),
                        created_at_ms: timestamp,
                    });
                });

                let hardware_req = OutgoingHardwareRequest {
                    req_id: cmd.req_id,
                    action: String::from("SET_PICTURE_PROPERTY"),
                    value: target_brightness,
                };

                if let Ok(out_str) = serde_json::to_string(&hardware_req) {
                    scapCallbackBridge(&out_str);
                }
            },

            "GET_DEVICE_INFO" => {
                CORE_STATE.with(|state| {
                    state.borrow_mut().pending_transactions.insert(cmd.req_id, PendingTransaction {
                        action: String::from("GET_DEVICE_INFO"),
                        created_at_ms: timestamp,
                    });
                });

                let telemetry_req = serde_json::json!({
                    "req_id": cmd.req_id,
                    "action": "FETCH_HARDWARE_TELEMETRY"
                });
                scapCallbackBridge(&telemetry_req.to_string());
            },

            _ => log(&format!("[Rust Core] Warning: Unhandled action: {}", cmd.action)),
        }
    } else {
        log("[Rust Core] Error: Could not parse incoming application payload.");
    }
}

#[wasm_bindgen]
pub fn process_hardware_event(json_str: &str) {
    if let Ok(evt) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(req_id) = evt.get("req_id").and_then(|r| r.as_u64()) {
            
            // Resolve flight status from memory ledger
            let mut matched_action = String::from("Unknown");
            CORE_STATE.with(|state| {
                if let Some(tx) = state.borrow_mut().pending_transactions.remove(&req_id) {
                    matched_action = tx.action;
                }
            });

            // 1. Process Telemetry Callback
            if let Some(event_type) = evt.get("event_type").and_then(|e| e.as_str()) {
                if event_type == "DEVICE_INFO_CALLBACK" {
                    if let Some(payload) = evt.get("payload") {
                        let model = payload.get("model").and_then(|m| m.as_str()).unwrap_or("Unknown");
                        let fw = payload.get("firmware").and_then(|f| f.as_str()).unwrap_or("Unknown");
                        
                        CORE_STATE.with(|state| {
                            let mut s = state.borrow_mut();
                            s.device_model = model.to_string();
                            s.firmware_version = fw.to_string();
                        });
                        log("[Rust Core State] Device info telemetry persisted successfully.");
                    }
                    return;
                }
            }

            // 2. Process Control Confirmations
            if let Some(status) = evt.get("hardware_status").and_then(|s| s.as_str()) {
                if status == "APPLIED" {
                    log(&format!("[Rust Core State] Transaction #{} marked OK. Action '{}' complete.", req_id, matched_action));
                    // If brightness was applied, optimize state assuming execution targets met
                    if matched_action == "SET_BRIGHTNESS" {
                        CORE_STATE.with(|state| {
                            state.borrow_mut().current_brightness = 20; // Or passed back via detailed schema payload
                        });
                    }
                } else if status == "FAILED" {
                    log(&format!("[Rust Core State] CRITICAL: Transaction #{} failed at hardware layer.", req_id));
                    CORE_STATE.with(|state| state.borrow_mut().hardware_fault_detected = true);
                }
            }
        }
    }
}

// --- 4. SAFETY WATCHDOG & INSPECTION UTILITIES ---

#[wasm_bindgen]
pub fn check_transaction_timeouts(current_time_ms: f64) {
    let timeout_threshold_ms = 3000.0; // 3 Seconds SCAP execution guard
    let mut timed_out_ids: Vec<u64> = Vec::new();

    CORE_STATE.with(|state| {
        let s = state.borrow();
        for (req_id, tx) in s.pending_transactions.iter() {
            if (current_time_ms - tx.created_at_ms) > timeout_threshold_ms {
                timed_out_ids.push(*req_id);
            }
        }
    });

    if !timed_out_ids.is_empty() {
        CORE_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.hardware_fault_detected = true;
            for id in timed_out_ids {
                if let Some(tx) = s.pending_transactions.remove(&id) {
                    js_log(&format!("[WATCHDOG PANIC] Transaction #{} ({}) completely hung. Hardware dropped request!", id, tx.action));
                }
            }
        });
    }
}

#[wasm_bindgen]
pub fn get_core_state() -> String {
    CORE_STATE.with(|state| {
        serde_json::to_string(&*state.borrow()).unwrap_or_else(|_| String::from("{}"))
    })
}