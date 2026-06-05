pub mod config;

use crate::config::CONFIG;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast; // Required for clean element styling downcasts
use web_sys::window;
use std::cell::RefCell;
use std::collections::VecDeque;
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

    // ✨ NEW: Supervisor Tracking Metrics Integrated Smoothly Into Your Core State
    pub play_mode: String,
    pub master_ip: String,
    pub my_ip: String,
    pub is_master: u8,
    pub coba_jaringan: u32,
    pub device_app_version: String,
    pub video_folder_lg: String,
    pub video_folder_local: String,
    pub file_jadwal: String,
    pub jadwal_dicek: String,

    // Playlist state
    pub playlist_entries: Vec<PlaylistEntry>,
    pub active_playlist_id: usize,
    pub active_content_id: usize,
    pub running_media_player: String,
    pub running_playlist_sig: String,
    pub is_initial: u8,
    pub has_run: bool,
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
        
        // Default values initialized cleanly for the Supervisor Loop
        play_mode: match CONFIG.play_mode {
            config::PlayMode::Dual => String::from("dual"),
            config::PlayMode::Single => String::from("single"),
        },
        master_ip: String::from(CONFIG.master_ip),
        my_ip: String::from("0.0.0.0"),
        is_master: 0,
        coba_jaringan: 0,
        device_app_version: String::from(CONFIG.device_app_version),
        video_folder_lg: String::from(CONFIG.video_folder_lg),
        video_folder_local: String::from(CONFIG.video_folder_local),
        file_jadwal: String::from(""),
        jadwal_dicek: String::from(""),
        playlist_entries: Vec::new(),
        active_playlist_id: 0,
        active_content_id: 0,
        running_media_player: String::from("A"),
        running_playlist_sig: String::new(),
        is_initial: 1,
        has_run: false,
    });
    // A safe thread-local queue to hold incoming hardware responses
    static HARDWARE_EVENT_QUEUE: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
    // A guard flag to prevent recursive event loop processing
    static IS_PROCESSING_LOOP: RefCell<bool> = RefCell::new(false);
}

#[derive(Deserialize, Debug)]
pub struct IncomingCommand {
    pub req_id: u64,
    pub action: String,
    pub payload: Option<serde_json::Value>,
    pub client_timestamp_ms: Option<f64>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct PlaylistEntry {
    pub start_time: String,
    pub end_time: String,
    pub videos: Vec<String>,
    pub durations: Vec<String>,
}

// --- 2. BINDINGS TO JAVASCRIPT ---
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn js_log(s: &str);
}

fn log(s: &str) {
    js_log(s);
}

#[allow(non_snake_case)]
fn scapCallbackBridge(json: &str) {
    let window = window().unwrap();
    let cb = js_sys::Reflect::get(
        &window,
        &wasm_bindgen::JsValue::from_str("scapCallbackBridge")
    );
    match cb {
        Ok(func_val) => {
            match func_val.dyn_into::<js_sys::Function>() {
                Ok(func) => {
                    let _ = func.call1(
                        &wasm_bindgen::JsValue::NULL,
                        &wasm_bindgen::JsValue::from_str(json)
                    );
                },
                Err(_) => js_log("[Rust] ERROR: scapCallbackBridge is not a function on window"),
            }
        },
        Err(_) => js_log("[Rust] ERROR: scapCallbackBridge not found on window"),
    }
}

// --- DOM ELEMENT HELPER UTILITIES ---
fn set_dom_html(id: &str, html: &str) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.get_element_by_id(id) {
                el.set_inner_html(html);
            }
        }
    }
}

fn set_dom_style(id: &str, property: &str, value: &str) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(el) = doc.get_element_by_id(id) {
                if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                    let _ = html_el.style().set_property(property, value);
                }
            }
        }
    }
}

// --- 3. HARDWARE ROUTING LOGIC ---

#[wasm_bindgen]
pub fn evaluate_hardware_routing(version: u32) {
    console_error_panic_hook::set_once();
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
    
    // Configure viewport layouts seamlessly
    let dual_frame = document.get_element_by_id("videoPlayerFrame");
    let legacy_video = document.get_element_by_id("legacyVideoPlayer");
    let legacy_poster = document.get_element_by_id("videoPoster");
    
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

    // 🚀 STEP 1 OF SUPERVISOR SEQUENCE: Request hardware telemetry info immediately
    set_dom_html("lastStatus", "Requesting Device Info ");
    
    let current_ms = window.performance().unwrap().now();
    CORE_STATE.with(|state| {
        state.borrow_mut().pending_transactions.insert(9991, PendingTransaction {
            action: "GET_DEVICE_INFO".to_string(),
            created_at_ms: current_ms,
        });
    });

    scapCallbackBridge(&serde_json::json!({
        "req_id": 9991,
        "action": "FETCH_HARDWARE_TELEMETRY"
    }).to_string());
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

// --- PRIVATE IN-ENGINE HELPER SUBROUTINES ---
fn execute_find_master(s: &mut CoreState) -> String {
    if s.play_mode != "single" {
        log(&format!("[Rust Core] findMaster | masterIP: {}", s.master_ip));
        if s.my_ip != s.master_ip {
            s.is_master = 0;
            log("[Rust Core] Panel state assigned: SLAVE NODE.");
        } else {
            s.is_master = 1;
            log("[Rust Core] Panel state assigned: MASTER NODE. Booting socket servers.");
        }
    }

    s.jadwal_dicek = s.file_jadwal.clone();
    set_dom_html("lastStatus", "Memeriksa jadwal tayang");
    log(&format!("[Rust Core] getPlaylistOffline | checking: {}", s.jadwal_dicek));

    let now_ms = window().unwrap().performance().unwrap().now();
    s.pending_transactions.insert(9994, PendingTransaction {
        action: "CHECK_FILE_EXISTS".to_string(),
        created_at_ms: now_ms,
    });

    serde_json::json!({
        "req_id": 9994,
        "action": "EXECUTE_FILE_EXISTS",
        "options": {
            "path": format!("{}{}", s.video_folder_lg, s.jadwal_dicek)
        }
    }).to_string()
}

// --- 5. HARDWARE TELEMETRY INGESTION ENGINE ---

#[wasm_bindgen]
pub fn process_hardware_event(json_str: &str) {
    // 1. Always push the incoming hardware response to the safe queue
    HARDWARE_EVENT_QUEUE.with(|q| q.borrow_mut().push_back(json_str.to_string()));

    // 2. If Rust is already executing an event turn, return IMMEDIATELY.
    if IS_PROCESSING_LOOP.with(|b| *b.borrow()) {
        return;
    }

    // 3. Acquire the processing loop lock
    IS_PROCESSING_LOOP.with(|b| *b.borrow_mut() = true);

    // 4. Drain the queue (processing happens below)
    HARDWARE_EVENT_QUEUE.with(|q| q.borrow_mut().pop_front());

    // 5. Release the processing loop lock
    IS_PROCESSING_LOOP.with(|b| *b.borrow_mut() = false);

    // 6. Parse and route actions cleanly away from state locks
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
                    set_dom_html("backStatus", "Requesting Action Chain: Failed");
                    return;
                }
            }

            if let Some(event_type) = evt.get("event_type").and_then(|e| e.as_str()) {
                match event_type {
                    "DEVICE_INFO_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let mut serial_num = String::new();
                            let mut id_html = String::new();
                            let mut is_single_mode = false;

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.model_name = p.get("modelName").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.firmware_version = p.get("firmwareVersion").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.hardware_version = p.get("hardwareVersion").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.sdk_version = p.get("sdkVersion").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                s.serial_number = p.get("serialNumber").and_then(|m| m.as_str()).unwrap_or("").to_string();
                                
                                s.file_jadwal = format!("J_{}.txt", s.serial_number);
                                serial_num = s.serial_number.clone();
                                is_single_mode = s.play_mode == "single";

                                id_html = format!(
                                    "ID: {} &nbsp; MDL: {}({}) &nbsp; FW: {} &nbsp; SW: {}",
                                    s.serial_number, s.model_name, s.webos_version, s.firmware_version, s.device_app_version
                                );

                                let now_ms = web_sys::window().unwrap().performance().unwrap().now();
                                s.pending_transactions.insert(9992, PendingTransaction {
                                    action: "GET_CURRENT_TIME".to_string(),
                                    created_at_ms: now_ms,
                                });
                            }); // Lock fully drops here

                            // Safe processing area for side-effects
                            set_dom_html("backStatus", "Requesting Device Info: Success");
                            log(&format!("[Rust Core] getDeviceInfo | Success | SN: {}", serial_num));

                            if !is_single_mode {
                                id_html.push_str("-[D]<br>");
                            } else {
                                id_html.push_str("-[S]<br>");
                                set_dom_style("tabSync", "display", "none");
                            }
                            set_dom_html("idStatus", &id_html);

                            set_dom_style("compatibilityGatekeeper", "display", "none");
                            set_dom_style("bootScreen", "display", "none");
                            log("[UI System] Splash elements cleared. Signage interface rendering active.");

                            set_dom_html("lastStatus", "Requesting Signage Time");
                            scapCallbackBridge(&serde_json::json!({ "req_id": 9992, "action": "FETCH_CURRENT_TIME" }).to_string());
                        }
                    },

                    "TIME_INFO_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let year = p.get("year").and_then(|v| v.as_i64()).unwrap_or(2026);
                            let month = p.get("month").and_then(|v| v.as_i64()).unwrap_or(1);
                            let day = p.get("day").and_then(|v| v.as_i64()).unwrap_or(1);
                            let hour = p.get("hour").and_then(|v| v.as_i64()).unwrap_or(0);
                            let minute = p.get("minute").and_then(|v| v.as_i64()).unwrap_or(0);
                            let second = p.get("second").and_then(|v| v.as_i64()).unwrap_or(0);

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.panel_time_info = serde_json::to_string(p).unwrap_or_default();

                                let now_ms = window().unwrap().performance().unwrap().now();
                                s.pending_transactions.insert(9993, PendingTransaction {
                                    action: "GET_NETWORK_INFO".to_string(),
                                    created_at_ms: now_ms,
                                });
                            }); // Lock drops

                            set_dom_html("backStatus", "Requesting Signage Time: Success");
                            log("[Rust Core] signageGetTime | Success");

                            let time_str = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, minute, second);
                            set_dom_style("tvStartStatus", "visibility", "visible");
                            set_dom_html("tvStartStatus", &format!("Time TV ON: {}", time_str));

                            set_dom_html("lastStatus", "Checking Connection");
                            scapCallbackBridge(&serde_json::json!({ "req_id": 9993, "action": "FETCH_NETWORK_INFO" }).to_string());
                        }
                    },

                    "FILE_EXISTS_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let exists = p.get("exists").and_then(|e| e.as_bool()).unwrap_or(false);
                            let mut play_command: Option<String> = None;

                            if exists {
                                CORE_STATE.with(|state| {
                                    let mut s = state.borrow_mut();
                                    let now_ms = window().unwrap().performance().unwrap().now();
                                    s.pending_transactions.insert(9995, PendingTransaction {
                                        action: "READ_FILE".to_string(),
                                        created_at_ms: now_ms,
                                    });

                                    play_command = Some(serde_json::json!({
                                        "req_id": 9995,
                                        "action": "EXECUTE_READ_FILE",
                                        "options": {
                                            "path": format!("{}{}", s.video_folder_lg, s.jadwal_dicek),
                                            "position": 0,
                                            "length": 8192,
                                            "encoding": "utf8"
                                        }
                                    }).to_string());
                                }); // Lock drops
                                
                                set_dom_html("backStatus", "Memeriksa jadwal tayang: Success");
                                set_dom_html("lastStatus", "Membaca file jadwal...");
                                log("[Rust Core] Schedule found. Triggering Step 5: EXECUTE_READ_FILE.");
                                
                                if let Some(cmd) = play_command {
                                    scapCallbackBridge(&cmd);
                                }
                            } else {
                                set_dom_html("backStatus", "Memeriksa jadwal tayang: Not Available");
                                set_dom_html("lastStatus", "Jadwal kosong. Sinkronisasi data...");
                                log("[Rust Core] Schedule file missing. Ready for sync download sequence.");
                            }
                        }
                    },

                    "FILE_READ_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let file_data = p.get("data").and_then(|d| d.as_str()).unwrap_or("");

                            if file_data.is_empty() {
                                set_dom_html("backStatus", "Membaca file jadwal: Success");
                                set_dom_html("lastStatus", "File jadwal kosong (0 bytes)");
                                return;
                            }

                            let mut play_command: Option<String> = None;
                            let mut log_msg = String::new();
                            let mut status_msg = String::new();

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.playlist_entries.clear();

                                for chunk in file_data.split('~') {
                                    let parts: Vec<&str> = chunk.split('$').collect();
                                    if parts.len() < 2 { continue; }
                                    let fields: Vec<&str> = parts[1].split('*').collect();
                                    if fields.len() < 4 { continue; }

                                    let videos: Vec<String> = fields[2].split('|')
                                        .map(|v| v.trim().to_string())
                                        .filter(|v| !v.is_empty())
                                        .collect();
                                    let durations: Vec<String> = fields[3].split('|')
                                        .map(|d| d.trim().to_string())
                                        .collect();

                                    s.playlist_entries.push(PlaylistEntry {
                                        start_time: fields[0].to_string(),
                                        end_time: fields[1].to_string(),
                                        videos,
                                        durations,
                                    });
                                }

                                if s.playlist_entries.is_empty() {
                                    log_msg = String::from("[Rust Core] No active playlist found.");
                                    status_msg = String::from("Tidak ada playlist aktif");
                                    return;
                                }

                                let mut active_idx = 0;
                                if let Ok(time_val) = serde_json::from_str::<serde_json::Value>(&s.panel_time_info) {
                                    let h = time_val.get("hour").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let m = time_val.get("minute").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let sec = time_val.get("second").and_then(|v| v.as_i64()).unwrap_or(0);
                                    let now_str = format!("{:02}:{:02}:{:02}", h, m, sec);

                                    for (i, entry) in s.playlist_entries.iter().enumerate() {
                                        if entry.start_time.as_str() <= now_str.as_str() {
                                            active_idx = i;
                                        }
                                    }
                                }

                                let new_sig = s.playlist_entries[active_idx].videos.join("|");
                                if new_sig != s.running_playlist_sig {
                                    s.running_playlist_sig = new_sig;
                                    s.active_playlist_id = active_idx;
                                    s.active_content_id = 0;
                                    s.running_media_player = String::from("A");
                                    s.is_initial = 1;

                                    let entry = &s.playlist_entries[active_idx];
                                    let first_video = entry.videos.get(0).cloned().unwrap_or_default();
                                    let first_dur = entry.durations.get(0).cloned().unwrap_or_default();
                                    let folder = s.video_folder_local.clone();
                                    let os_version = s.webos_version;
                                    let is_video = !first_video.ends_with(".jpg") && !first_video.ends_with(".png");

                                    log_msg = format!("[Rust Core] Active playlist [{}], first content: {}", active_idx + 1, first_video);
                                    status_msg = format!("Memulai playlist {}", active_idx + 1);

                                    play_command = Some(serde_json::json!({
                                        "req_id": 9996,
                                        "action": "PLAY_CONTENT",
                                        "options": {
                                            "player": "A",
                                            "src": format!("{}{}", folder, first_video),
                                            "duration": first_dur,
                                            "is_video": is_video,
                                            "os_version": os_version
                                        }
                                    }).to_string());
                                } else {
                                    log_msg = String::from("[Rust Core] Playlist unchanged, no restart needed.");
                                }
                            }); // Lock drops

                            set_dom_html("backStatus", "Membaca file jadwal: Success");
                            if !status_msg.is_empty() { set_dom_html("lastStatus", &status_msg); }
                            if !log_msg.is_empty() { log(&log_msg); }
                            if let Some(cmd) = play_command { scapCallbackBridge(&cmd); }
                        }
                    },

                    "CONTENT_ENDED" => {
                        let mut play_command: Option<String> = None;
                        let mut log_msg = String::new();

                        CORE_STATE.with(|state| {
                            let mut s = state.borrow_mut();
                            if s.playlist_entries.is_empty() { return; }

                            let playlist_len = s.playlist_entries[s.active_playlist_id].videos.len();
                            s.active_content_id += 1;
                            if s.active_content_id >= playlist_len {
                                s.active_content_id = 0;
                            }

                            let next_player = if s.running_media_player == "A" { "B" } else { "A" };
                            s.running_media_player = next_player.to_string();

                            let entry = &s.playlist_entries[s.active_playlist_id];
                            let video = entry.videos.get(s.active_content_id).cloned().unwrap_or_default();
                            let duration = entry.durations.get(s.active_content_id).cloned().unwrap_or_default();
                            let folder = s.video_folder_local.clone();
                            let os_version = s.webos_version;
                            let player = s.running_media_player.clone();
                            let is_video = !video.ends_with(".jpg") && !video.ends_with(".png");

                            log_msg = format!("[Rust Core] Next content [{}] on player {}: {}", s.active_content_id + 1, player, video);

                            play_command = Some(serde_json::json!({
                                "req_id": 9997,
                                "action": "PLAY_CONTENT",
                                "options": {
                                    "player": player,
                                    "src": format!("{}{}", folder, video),
                                    "duration": duration,
                                    "is_video": is_video,
                                    "os_version": os_version
                                }
                            }).to_string());
                        }); // Lock drops

                        if !log_msg.is_empty() { log(&log_msg); }
                        if let Some(cmd) = play_command { scapCallbackBridge(&cmd); }
                    },

                    "NETWORK_INFO_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let mut connection_acquired = false;
                            let mut run_retry_timeout = false;
                            let mut ip_output = String::new();
                            let mut mode_label = "";
                            let mut find_master_cmd: Option<String> = None;

                            let is_wired = p.get("wired").and_then(|w| w.get("state")).and_then(|s| s.as_str()) == Some("connected");
                            let is_wifi = p.get("wifi").and_then(|w| w.get("state")).and_then(|s| s.as_str()) == Some("connected");

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.is_internet_available = p.get("isInternetConnectionAvailable").and_then(|b| b.as_bool()).unwrap_or(false);
                                
                                if is_wired {
                                    s.my_ip = p.get("wired").and_then(|w| w.get("ipAddress")).and_then(|s| s.as_str()).unwrap_or("0.0.0.0").to_string();
                                    mode_label = "WIRED";
                                    connection_acquired = true;
                                } else if is_wifi {
                                    s.my_ip = p.get("wifi").and_then(|w| w.get("ipAddress")).and_then(|s| s.as_str()).unwrap_or("0.0.0.0").to_string();
                                    mode_label = "WIFI";
                                    connection_acquired = true;
                                } else if s.is_internet_available {
                                    s.my_ip = String::from("192.168.1.50");
                                    mode_label = "SIMULATOR";
                                    connection_acquired = true;
                                }

                                ip_output = s.my_ip.clone();

                                if connection_acquired {
                                    find_master_cmd = Some(execute_find_master(&mut s));
                                } else {
                                    if s.coba_jaringan < 3 {
                                        s.coba_jaringan += 1;
                                        run_retry_timeout = true;
                                    } else {
                                        find_master_cmd = Some(execute_find_master(&mut s));
                                    }
                                }
                            }); // Lock drops

                            if let Some(cmd) = find_master_cmd {
                                scapCallbackBridge(&cmd);
                            }

                            if connection_acquired {
                                set_dom_html("internetStatus", &format!("{} | {}", ip_output, mode_label));
                                set_dom_style("internetStatus", "background-color", "rgba(0, 255, 0, 0.75)");
                                set_dom_html("backStatus", &format!("Checking Connection : {}", mode_label));
                            } else {
                                if run_retry_timeout {
                                    set_dom_html("backStatus", "Checking Connection : Retrying...");
                                    let closure = Closure::wrap(Box::new(move || {
                                        scapCallbackBridge(&serde_json::json!({ "req_id": 9993, "action": "FETCH_NETWORK_INFO" }).to_string());
                                    }) as Box<dyn FnMut()>);

                                    window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(), 5000).unwrap();
                                    closure.forget();
                                } else {
                                    set_dom_html("backStatus", "Checking Connection : No Connection");
                                }
                            }
                        }
                    },

                    "STORAGE_INFO_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.internal_storage.total_kb = p.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                s.internal_storage.free_kb = p.get("free").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                s.internal_storage.used_kb = p.get("used").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            });
                        }
                    },

                    "SERVER_PROPERTY_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.server_config.server_ip = p.get("serverIp").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                s.server_config.server_port = p.get("serverPort").and_then(|v| v.as_u64()).unwrap_or(80) as u32;
                                s.server_config.app_launch_mode = p.get("appLaunchMode").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                s.server_config.fqdn_addr = p.get("fqdnAddr").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            });
                        }
                    },

                    "ON_TIMER_LIST_CALLBACK" => {
                        if let Some(timer_list) = evt.get("payload").and_then(|p| p.get("timerList")).and_then(|l| l.as_array()) {
                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.scheduled_on_timers = timer_list.iter().map(|t| HardwareTimer {
                                    hour: t.get("hour").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    minute: t.get("minute").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    week: t.get("week").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    input_source: t.get("inputSource").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                }).collect();
                            });
                            log("[Rust Core State] Internal On-Timer layout synchronization baseline marked OK.");
                        }
                    },

                    "OFF_TIMER_LIST_CALLBACK" => {
                        if let Some(timer_list) = evt.get("payload").and_then(|p| p.get("timerList")).and_then(|l| l.as_array()) {
                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.scheduled_off_timers = timer_list.iter().map(|t| HardwareTimer {
                                    hour: t.get("hour").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    minute: t.get("minute").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    week: t.get("week").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                    input_source: None,
                                }).collect();
                            });
                            log("[Rust Core State] Internal Off-Timer layout synchronization baseline marked OK.");
                        }
                    },

                    _ => {
                        log(&format!("[Rust Core State] Action Complete Event -> {} transaction resolved cleanly.", event_type));
                    }
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn check_transaction_timeouts(current_time_ms: f64) {
    let timeout_threshold_ms = 8000.0;
    let mut timed_out_ids: Vec<u64> = Vec::new();

    CORE_STATE.with(|state| {
        if let Ok(s) = state.try_borrow() {
            for (req_id, tx) in s.pending_transactions.iter() {
                if (current_time_ms - tx.created_at_ms) > timeout_threshold_ms {
                    timed_out_ids.push(*req_id);
                }
            }
        }
    });

    if !timed_out_ids.is_empty() {
        CORE_STATE.with(|state| {
            if let Ok(mut s) = state.try_borrow_mut() {
                s.hardware_fault_detected = true;
                for id in timed_out_ids {
                    if let Some(tx) = s.pending_transactions.remove(&id) {
                        js_log(&format!("[WATCHDOG] Timeout on Transaction #{} ({})", id, tx.action));
                    }
                }
            }
        });
    }
}

#[wasm_bindgen]
pub fn get_core_state() -> String {
    CORE_STATE.with(|state| serde_json::to_string(&*state.borrow()).unwrap_or_else(|_| String::from("{}")))
}