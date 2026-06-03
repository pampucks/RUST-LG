pub mod config;

use wasm_bindgen::prelude::*;
use web_sys::window;
use std::cell::RefCell;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// --- 1. CORE ARCHITECTURE ENUMS & STRUCTS ---

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PlaybackEngineLayout {
    ModernDualBuffer, // For WebOS 3.0 and 4.0+
    LegacySingleNode,  // For WebOS 2.0
    Unassigned,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct NetworkInterface {
    pub state: String,
    pub method: String,
    pub ip_address: String,
    pub netmask: String,
    pub dns1: String,
    pub dns2: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct StorageMetrics {
    pub total_kb: f64,
    pub free_kb: f64,
    pub used_kb: f64,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ServerProperties {
    pub server_ip: String,
    pub server_port: u32,
    pub app_launch_mode: String,
    pub fqdn_addr: String,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct HardwareTimer {
    pub hour: u32,
    pub minute: u32,
    pub week: u32,
    pub input_source: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct PendingTransaction {
    pub action: String,
    pub created_at_ms: f64,
}

#[derive(Serialize, Clone, Debug)]
pub struct CoreState {
    // Platform Identifiers
    pub model_name: String,
    pub firmware_version: String,
    pub hardware_version: String,
    pub sdk_version: String,
    pub serial_number: String,
    
    // Telemetry, Configurations & Networks
    pub is_internet_available: bool,
    pub panel_time_info: String,
    pub server_config: ServerProperties,
    pub wired_network: NetworkInterface,
    pub wifi_network: NetworkInterface,
    pub internal_storage: StorageMetrics,
    
    // Hardware Power Automation Schedules
    pub scheduled_on_timers: Vec<HardwareTimer>,
    pub scheduled_off_timers: Vec<HardwareTimer>,
    
    // Control & Fault Status
    pub current_brightness: u8,
    pub hardware_fault_detected: bool,
    pub pending_transactions: HashMap<u64, PendingTransaction>,

    // Dynamic Hardware Routing Properties
    pub webos_version: u32,
    pub dynamic_layout: PlaybackEngineLayout,
}

thread_local! {
    static CORE_STATE: RefCell<CoreState> = RefCell::new(CoreState {
        model_name: String::from("Unknown"),
        firmware_version: String::from("Unknown"),
        hardware_version: String::from("Unknown"),
        sdk_version: String::from("Unknown"),
        serial_number: String::from("Unknown"),
        is_internet_available: false,
        panel_time_info: String::from("Unknown"),
        server_config: ServerProperties::default(),
        wired_network: NetworkInterface::default(),
        wifi_network: NetworkInterface::default(),
        internal_storage: StorageMetrics::default(),
        scheduled_on_timers: Vec::new(),
        scheduled_off_timers: Vec::new(),
        current_brightness: 50,
        hardware_fault_detected: false,
        pending_transactions: HashMap::new(),
        webos_version: 0,
        dynamic_layout: PlaybackEngineLayout::Unassigned,
    });
}

#[derive(Deserialize, Debug)]
pub struct IncomingCommand {
    pub req_id: u64,
    pub action: String,
    pub payload: Option<serde_json::Value>,
    pub client_timestamp_ms: Option<f64>,
}

// --- 2. BINDINGS TO JAVASCRIPT ---
#[wasm_bindgen]
extern "C" {
    // Maps the browser's console.log directly to the name 'js_log' inside Rust
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log(s: &str);

    // Tells Rust that a global JavaScript function named scapCallbackBridge exists
    fn scapCallbackBridge(s: &str);
}

fn log(s: &str) {
    js_log(s);
}

// --- 3. HARDWARE ROUTING LOGIC ---

#[wasm_bindgen]
pub fn evaluate_hardware_routing(version: u32) {
    CORE_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.webos_version = version;
        if version >= 3 {
            s.dynamic_layout = PlaybackEngineLayout::ModernDualBuffer;
        } else {
            s.dynamic_layout = PlaybackEngineLayout::LegacySingleNode;
        }
    });
    
    let window = window().expect("Global Window Context Missing");
    let document = window.document().expect("DOM Document Missing");
    
    if let Some(stats_div) = document.get_element_by_id("processStats") {
        stats_div.set_inner_html(&format!("WebOS Version Identified: v{}", version));
    }
    
    let closure = Closure::wrap(Box::new(move || {
        let doc = web_sys::window().unwrap().document().unwrap();
        
        if let Some(boot) = doc.get_element_by_id("bootScreen") {
            let html_el: web_sys::HtmlElement = boot.dyn_into().unwrap();
            html_el.style().set_property("display", "none").unwrap();
        }
        
        let dual_frame = doc.get_element_by_id("videoPlayerFrame");
        let legacy_video = doc.get_element_by_id("legacyVideoPlayer");
        let legacy_poster = doc.get_element_by_id("videoPoster");
        
        if version >= 3 {
            js_log("[Core Routing] Selecting Modern Dual Alternating Node Layout View.");
            if let Some(df) = dual_frame {
                df.dyn_into::<web_sys::HtmlElement>().unwrap().style().set_property("display", "block").unwrap();
            }
        } else {
            js_log("[Core Routing] Selecting Legacy Single Viewport Node Layout View.");
            if let Some(lv) = legacy_video {
                lv.dyn_into::<web_sys::HtmlElement>().unwrap().style().set_property("display", "block").unwrap();
            }
            if let Some(lp) = legacy_poster {
                lp.dyn_into::<web_sys::HtmlElement>().unwrap().style().set_property("display", "block").unwrap();
            }
        }
    }) as Box<dyn FnMut()>);
    
    window.set_timeout_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        5000
    ).unwrap();
    
    closure.forget();
}

// --- 4. UNIFIED COMMAND PARSER & ROUTER ---

#[wasm_bindgen]
pub fn process_signage_command(json_str: &str) {
    if let Ok(cmd) = serde_json::from_str::<IncomingCommand>(json_str) {
        let timestamp = cmd.client_timestamp_ms.unwrap_or(0.0);

        CORE_STATE.with(|state| {
            state.borrow_mut().pending_transactions.insert(cmd.req_id, PendingTransaction {
                action: cmd.action.clone(),
                created_at_ms: timestamp,
            });
        });

        let fallback_payload = serde_json::json!({});
        let payload = cmd.payload.as_ref().unwrap_or(&fallback_payload);

        match cmd.action.as_str() {
            "GET_DEVICE_INFO" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_HARDWARE_TELEMETRY" }).to_string()),
            "GET_NETWORK_INFO" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_NETWORK_INFO" }).to_string()),
            "GET_STORAGE_INFO" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_STORAGE_INFO" }).to_string()),
            "UPGRADE_APPLICATION" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_APP_UPGRADE", "options": payload }).to_string()),
            "COPY_FILE" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_COPY_FILE", "options": payload }).to_string()),
            "CHECK_FILE_EXISTS" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_FILE_EXISTS", "options": payload }).to_string()),
            "READ_FILE" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_READ_FILE", "options": payload }).to_string()),
            "LIST_FILES" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_LIST_FILES", "options": payload }).to_string()),
            "STAT_FILE" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_STAT_FILE", "options": payload }).to_string()),
            "REMOVE_FILE" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_REMOVE_FILE", "options": payload }).to_string()),
            "WRITE_FILE" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_WRITE_FILE", "options": payload }).to_string()),
            "REMOVE_ALL_FILES" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_REMOVE_ALL", "options": payload }).to_string()),
            "CAPTURE_SCREEN" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_SCREEN_CAPTURE", "options": payload }).to_string()),
            "GET_CURRENT_TIME" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_CURRENT_TIME" }).to_string()),
            "SET_SERVER_PROPERTY" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_SET_SERVER", "options": payload }).to_string()),
            "RESTART_APPLICATION" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_APP_RESTART" }).to_string()),
            "GET_SERVER_PROPERTY" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_SERVER_PROPERTY" }).to_string()),
            "EXECUTE_POWER_COMMAND" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_POWER_CMD", "options": payload }).to_string()),
            "GET_ON_TIMER_LIST" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_ON_TIMER_LIST" }).to_string()),
            "GET_OFF_TIMER_LIST" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "FETCH_OFF_TIMER_LIST" }).to_string()),
            "ADD_ON_TIMER" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_ADD_ON_TIMER", "options": payload }).to_string()),
            "ADD_OFF_TIMER" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_ADD_OFF_TIMER", "options": payload }).to_string()),
            "ENABLE_ALL_ON_TIMER" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_ENABLE_ALL_ON", "options": payload }).to_string()),
            "ENABLE_ALL_OFF_TIMER" => scapCallbackBridge(&serde_json::json!({ "req_id": cmd.req_id, "action": "EXECUTE_ENABLE_ALL_OFF", "options": payload }).to_string()),
            _ => log(&format!("[Rust Core] Warning: Unhandled command route: {}", cmd.action)),
        }
    }
}

// --- 5. HARDWARE TELEMETRY INGESTION ENGINE ---

#[wasm_bindgen]
pub fn process_hardware_event(json_str: &str) {
    if let Ok(evt) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(req_id) = evt.get("req_id").and_then(|r| r.as_u64()) {
            
            let mut matched_action = String::from("Unknown");
            CORE_STATE.with(|state| {
                if let Some(tx) = state.borrow_mut().pending_transactions.remove(&req_id) {
                    matched_action = tx.action;
                }
            });

            if let Some(status) = evt.get("hardware_status").and_then(|s| s.as_str()) {
                if status == "FAILED" {
                    log(&format!("[Rust Core State] CRITICAL: Action '{}' failed at SCAP level.", matched_action));
                    CORE_STATE.with(|state| state.borrow_mut().hardware_fault_detected = true);
                    return;
                }
            }

            if let Some(event_type) = evt.get("event_type").and_then(|e| e.as_str()) {
                CORE_STATE.with(|state| {
                    let mut s = state.borrow_mut();
                    match event_type {
                        "DEVICE_INFO_CALLBACK" => {
                            if let Some(p) = evt.get("payload") {
                                s.model_name = p.get("modelName").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.firmware_version = p.get("firmwareVersion").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.hardware_version = p.get("hardwareVersion").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.sdk_version = p.get("sdkVersion").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.serial_number = p.get("serialNumber").and_then(|m| m.as_str()).unwrap_or("").to_string();
                            }
                        },
                        "NETWORK_INFO_CALLBACK" => {
                            if let Some(p) = evt.get("payload") {
                                s.is_internet_available = p.get("isInternetConnectionAvailable").and_then(|b| b.as_bool()).unwrap_or(false);
                            }
                        },
                        "STORAGE_INFO_CALLBACK" => {
                            if let Some(p) = evt.get("payload") {
                                s.internal_storage.total_kb = p.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                s.internal_storage.free_kb = p.get("free").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                s.internal_storage.used_kb = p.get("used").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            }
                        },
                        "SERVER_PROPERTY_CALLBACK" => {
                            if let Some(p) = evt.get("payload") {
                                s.server_config.server_ip = p.get("serverIp").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                s.server_config.server_port = p.get("serverPort").and_then(|v| v.as_u64()).unwrap_or(80) as u32;
                                s.server_config.app_launch_mode = p.get("appLaunchMode").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                s.server_config.fqdn_addr = p.get("fqdnAddr").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            }
                        },
                        "ON_TIMER_LIST_CALLBACK" => {
                            if let Some(timer_list) = evt.get("payload").and_then(|p| p.get("timerList")).and_then(|l| l.as_array()) {
                                s.scheduled_on_timers = timer_list.iter().map(|t| HardwareTimer {
                                    hour: t.get("hour").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    minute: t.get("minute").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    week: t.get("week").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    input_source: t.get("inputSource").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                }).collect();
                                log("[Rust Core State] Internal On-Timer layout synchronization baseline marked OK.");
                            }
                        },
                        "OFF_TIMER_LIST_CALLBACK" => {
                            if let Some(timer_list) = evt.get("payload").and_then(|p| p.get("timerList")).and_then(|l| l.as_array()) {
                                s.scheduled_off_timers = timer_list.iter().map(|t| HardwareTimer {
                                    hour: t.get("hour").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    minute: t.get("minute").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    week: t.get("week").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    input_source: None,
                                }).collect();
                                log("[Rust Core State] Internal Off-Timer layout synchronization baseline marked OK.");
                            }
                        },
                        "TIME_INFO_CALLBACK" => {
                            if let Some(p) = evt.get("payload") {
                                s.panel_time_info = serde_json::to_string(p).unwrap_or_default();
                            }
                        },
                        _ => {
                            log(&format!("[Rust Core State] Action Complete Event -> {} transaction resolved cleanly.", event_type));
                        }
                    }
                });
            }
        }
    }
}

#[wasm_bindgen]
pub fn check_transaction_timeouts(current_time_ms: f64) {
    let timeout_threshold_ms = 8000.0; 
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
                    js_log(&format!("[WATCHDOG PANIC] SCAP Hardware execution timeout on Transaction #{} ({})", id, tx.action));
                }
            }
        });
    }
}

#[wasm_bindgen]
pub fn get_core_state() -> String {
    CORE_STATE.with(|state| serde_json::to_string(&*state.borrow()).unwrap_or_else(|_| String::from("{}")))
}