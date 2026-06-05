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
    pub nama_sesi_jadwal: String,
    pub nama_text_file: String,
    pub sync_playlist_update: String,
    pub sync_size_update: String,
    pub missing_files: Vec<String>,
    pub video_counter: usize,
    pub downloading: u8,
    pub checking: u8,
    pub total_file_size: u64,
    pub log_request: bool,
    pub file_log: String,
    pub array_log: Vec<String>,
    pub target_url_check_ipk: String,
    pub target_url_upgrade_ipk: String,
    pub target_device_app_version: String,
    pub in_process_upgrade_ipk: u8,
    pub is_upgrading: u8,
    pub app_type_var: String,
    pub app_launch_mode: String,
    pub app_fqdn_mode: bool,
    pub model_name_split: String,

    // Playlist state
    pub playlist_entries: Vec<PlaylistEntry>,
    pub active_playlist_id: usize,
    pub active_content_id: usize,
    pub running_media_player: String,
    pub running_playlist_sig: String,
    pub is_initial: u8,
    pub has_run: bool,
    pub background_services_started: bool,
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
        nama_sesi_jadwal: String::from(""),
        nama_text_file: String::from(""),
        sync_playlist_update: String::from(""),
        sync_size_update: String::from(""),
        missing_files: Vec::new(),
        video_counter: 0,
        downloading: 0,
        checking: 0,
        total_file_size: 0,
        log_request: false,
        file_log: String::new(),
        array_log: Vec::new(),
        target_url_check_ipk: String::from(CONFIG.url_check_ipk_version),
        target_url_upgrade_ipk: String::from(CONFIG.url_upgrade_ipk),
        target_device_app_version: String::from(CONFIG.device_app_version),
        in_process_upgrade_ipk: 0,
        is_upgrading: 0,
        app_type_var: String::from("ipk"),
        app_launch_mode: String::from("local"),
        app_fqdn_mode: true,
        model_name_split: String::new(),
        playlist_entries: Vec::new(),
        active_playlist_id: 0,
        active_content_id: 0,
        running_media_player: String::from("A"),
        running_playlist_sig: String::new(),
        is_initial: 1,
        has_run: false,
        background_services_started: false,
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

fn schedule_tick(callback_name: &str, delay_ms: i32) {
    let js_code = format!("setTimeout(function(){{ if(typeof window.{} === 'function') window.{}(); }}, {})", 
        callback_name, callback_name, delay_ms);
    let window = window().unwrap();
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        &js_sys::Function::new_no_args(&js_code),
        delay_ms,
    );
}

fn start_background_services() {
    let already_started = CORE_STATE.with(|state| {
        let s = state.borrow();
        s.background_services_started
    });

    if already_started {
        return;
    }

    CORE_STATE.with(|state| {
        state.borrow_mut().background_services_started = true;
        state.borrow_mut().has_run = true;
    });

    log("[Rust BG] Starting background services...");

    // Check playlist: 20 detik
    schedule_tick("rust_tick_check_playlist", 20_000);

    // Check connection: 60 detik
    schedule_tick("rust_tick_check_connection", 60_000);

    // Check content (updateContent): 15 detik pertama, lalu 30 menit
    schedule_tick("rust_tick_check_content", 15_000);

    // Update status (updateDownload): 40 menit
    schedule_tick("rust_tick_update_status", 2_400_000);

    // Sync capture: 50 menit
    schedule_tick("rust_tick_sync_capture", 3_000_000);

    // Send log: 60 menit
    schedule_tick("rust_tick_send_log", 3_600_000);

    // Check IPK version: 25 menit
    schedule_tick("rust_tick_check_version", 1_500_000);

    log("[Rust BG] All background services scheduled.");
}

#[wasm_bindgen]
pub fn tick_check_playlist() {
    log("[Rust BG] tick_check_playlist");
    let cmd = CORE_STATE.with(|state| {
        let s = state.borrow();
        if s.jadwal_dicek.is_empty() {
            return None;
        }
        let now_ms = window().unwrap().performance().unwrap().now();
        drop(s);
        state.borrow_mut().pending_transactions.insert(9998, PendingTransaction {
            action: "CHECK_FILE_EXISTS".to_string(),
            created_at_ms: now_ms,
        });
        let s = state.borrow();
        Some(serde_json::json!({
            "req_id": 9998,
            "action": "EXECUTE_FILE_EXISTS",
            "options": {
                "path": format!("{}{}", s.video_folder_lg, s.jadwal_dicek)
            }
        }).to_string())
    });

    if let Some(c) = cmd { scapCallbackBridge(&c); }
    schedule_tick("rust_tick_check_playlist", 20_000);
}

#[wasm_bindgen]
pub fn tick_check_connection() {
    log("[Rust BG] tick_check_connection");
    let now_ms = window().unwrap().performance().unwrap().now();
    CORE_STATE.with(|state| {
        state.borrow_mut().pending_transactions.insert(8030, PendingTransaction {
            action: "GET_NETWORK_INFO".to_string(),
            created_at_ms: now_ms,
        });
    });
    scapCallbackBridge(&serde_json::json!({
        "req_id": 8030,
        "action": "FETCH_NETWORK_INFO"
    }).to_string());
    schedule_tick("rust_tick_check_connection", 60_000);
}

#[wasm_bindgen]
pub fn tick_check_content() {
    log("[Rust BG] tick_check_content");
    let (my_ip, kode_tv) = CORE_STATE.with(|state| {
        let s = state.borrow();
        (s.my_ip.clone(), s.serial_number.clone())
    });

    if my_ip == "0.0.0.0" || kode_tv.is_empty() {
        log("[Rust BG] tick_check_content: skip, no network yet");
        schedule_tick("rust_tick_check_content", 30_000);
        return;
    }

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8001,
        "action": "UPDATE_CONTENT",
        "options": {
            "kode_tv": kode_tv,
            "my_ip": my_ip
        }
    }).to_string());

    schedule_tick("rust_tick_check_content", 1_800_000);
}

#[wasm_bindgen]
pub fn tick_update_status() {
    log("[Rust BG] tick_update_status");
    let (nama_sesi, kode_tv) = CORE_STATE.with(|state| {
        let s = state.borrow();
        (s.file_jadwal.clone(), s.serial_number.clone())
    });

    if nama_sesi.is_empty() || kode_tv.is_empty() {
        log("[Rust BG] tick_update_status: skip, no session yet");
        schedule_tick("rust_tick_update_status", 2_400_000);
        return;
    }

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8002,
        "action": "UPDATE_STATUS",
        "options": {
            "kode_tv": kode_tv,
            "nama_sesi": nama_sesi
        }
    }).to_string());

    schedule_tick("rust_tick_update_status", 2_400_000);
}

#[wasm_bindgen]
pub fn tick_sync_capture() {
    log("[Rust BG] tick_sync_capture");
    let (kode_tv, _tv_app_ver, os_version) = CORE_STATE.with(|state| {
        let s = state.borrow();
        (s.serial_number.clone(), s.device_app_version.clone(), s.webos_version)
    });

    if kode_tv.is_empty() {
        schedule_tick("rust_tick_sync_capture", 3_000_000);
        return;
    }

    let now_ms = window().unwrap().performance().unwrap().now();
    CORE_STATE.with(|state| {
        state.borrow_mut().pending_transactions.insert(8040, PendingTransaction {
            action: "CAPTURE_SCREEN".to_string(),
            created_at_ms: now_ms,
        });
    });

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8040,
        "action": "EXECUTE_SCREEN_CAPTURE",
        "options": {
            "save": true,
            "thumbnail": false,
            "imgResolution": if os_version > 2 { "HD" } else { "SD" }
        }
    }).to_string());

    schedule_tick("rust_tick_sync_capture", 3_000_000);
}

#[wasm_bindgen]
pub fn tick_send_log() {
    log("[Rust BG] tick_send_log");
    let (kode_tv, log_request, video_folder_lg) = CORE_STATE.with(|state| {
        let s = state.borrow();
        (s.serial_number.clone(), s.log_request, s.video_folder_lg.clone())
    });

    if kode_tv.is_empty() || !log_request {
        log("[Rust BG] tick_send_log: skip, log_request is false");
        schedule_tick("rust_tick_send_log", 3_600_000);
        return;
    }

    let now_ms = window().unwrap().performance().unwrap().now();
    CORE_STATE.with(|state| {
        state.borrow_mut().pending_transactions.insert(8050, PendingTransaction {
            action: "CHECK_LOG_FILES".to_string(),
            created_at_ms: now_ms,
        });
    });

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8050,
        "action": "EXECUTE_LIST_FILES",
        "options": { "path": video_folder_lg }
    }).to_string());

    schedule_tick("rust_tick_send_log", 3_600_000);
}

#[wasm_bindgen]
pub fn tick_check_version() {
    log("[Rust BG] tick_check_version");
    let (kode_tv, check_url) = CORE_STATE.with(|state| {
        let s = state.borrow();
        (s.serial_number.clone(), s.target_url_check_ipk.clone())
    });

    if kode_tv.is_empty() {
        schedule_tick("rust_tick_check_version", 1_500_000);
        return;
    }

    let now_ms = window().unwrap().performance().unwrap().now();
    CORE_STATE.with(|state| {
        state.borrow_mut().pending_transactions.insert(8070, PendingTransaction {
            action: "CHECK_IPK_VERSION".to_string(),
            created_at_ms: now_ms,
        });
    });

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8070,
        "action": "CHECK_IPK_VERSION",
        "options": { "url": check_url }
    }).to_string());

    schedule_tick("rust_tick_check_version", 1_500_000);
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
                            let req_id = evt.get("req_id").and_then(|r| r.as_u64()).unwrap_or(0);
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
                                start_background_services();
                            } else if req_id == 8030 {
                                // periodic connection check — update UI saja
                                log("[Rust BG] tick_check_connection | network status updated");
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

                    "UPDATE_CONTENT_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let status = p.get("status").and_then(|v| v.as_u64()).unwrap_or(0);
                            let response_text = p.get("response_text").and_then(|v| v.as_str()).unwrap_or("");

                            if status != 200 || response_text.is_empty() {
                                log(&format!("[Rust Core] updateContent | Failed | status: {}", status));
                                set_dom_html("backStatus", "Server tidak menjawab permintaan jadwal baru");
                                return;
                            }

                            // response = 0 artinya TV tidak terdaftar
                            if response_text == "0" {
                                log("[Rust Core] updateContent | TV not registered");
                                set_dom_html("lastStatus", "ID TV tidak terdaftar. Hubungi Admin.");
                                return;
                            }

                            let data_list: Vec<&str> = response_text.split('~').collect();
                            if data_list.len() < 3 {
                                log("[Rust Core] updateContent | Invalid response format");
                                return;
                            }

                            // data_list[1] == "1" artinya tidak ada jadwal di server
                            if data_list.get(1) == Some(&"1") {
                                log("[Rust Core] updateContent | No schedule on server");
                                set_dom_html("lastStatus", "Tidak ada jadwal di server. Hubungi Admin.");
                                return;
                            }

                            let nama_sesi = data_list[0].to_string();
                            let nama_text = format!("{}.txt", nama_sesi);
                            let playlist_update = data_list[1].to_string();
                            let size_update = data_list.get(2).unwrap_or(&"").to_string();

                            log(&format!("[Rust Core] updateContent | Success | sesi: {}", nama_sesi));
                            set_dom_html("backStatus", "Server menjawab permintaan jadwal baru");

                            let mut check_cmd: Option<String> = None;
                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.nama_sesi_jadwal = nama_sesi;
                                s.nama_text_file = nama_text;
                                s.sync_playlist_update = playlist_update;
                                s.sync_size_update = size_update;
                                s.video_counter = 0;
                                s.checking = 1;

                                let now_ms = window().unwrap().performance().unwrap().now();
                                s.pending_transactions.insert(8010, PendingTransaction {
                                    action: "CHECK_MISSING_FILE".to_string(),
                                    created_at_ms: now_ms,
                                });

                                check_cmd = Some(serde_json::json!({
                                    "req_id": 8010,
                                    "action": "EXECUTE_LIST_FILES",
                                    "options": { "path": s.video_folder_lg }
                                }).to_string());
                            });

                            set_dom_html("lastStatus", "Checking missing files");
                            if let Some(cmd) = check_cmd { scapCallbackBridge(&cmd); }
                        }
                    },

                    "LIST_FILES_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let req_id = evt.get("req_id").and_then(|r| r.as_u64()).unwrap_or(0);

                            // req 8050 = checkLocalLogFiles, bukan checkMissingFile
                            if req_id == 8050 {
                                let files = p.get("files").and_then(|f| f.as_array());
                                if files.is_none() { return; }
                                let files = files.unwrap();
                                let mut send_cmd: Option<String> = None;

                                CORE_STATE.with(|state| {
                                    let mut s = state.borrow_mut();
                                    s.array_log.clear();
                                    for file in files {
                                        let name = file.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                        let prefix = format!("{}_", s.serial_number);
                                        if name.contains(&prefix) {
                                            s.array_log.push(name.to_string());
                                        }
                                    }
                                    if s.array_log.is_empty() {
                                        log("[Rust Core] checkLocalLogFiles | Tidak ada log");
                                        return;
                                    }
                                    log(&format!("[Rust Core] checkLocalLogFiles | {} log belum dikirim", s.array_log.len()));
                                    let first_log = s.array_log[0].clone();
                                    s.file_log = first_log.clone();
                                    let now_ms = window().unwrap().performance().unwrap().now();
                                    s.pending_transactions.insert(8051, PendingTransaction {
                                        action: "KIRIM_LOG".to_string(),
                                        created_at_ms: now_ms,
                                    });
                                    send_cmd = Some(serde_json::json!({
                                        "req_id": 8051,
                                        "action": "KIRIM_LOG",
                                        "options": {
                                            "file_log": first_log,
                                            "video_folder_local": s.video_folder_local,
                                            "kode_tv": s.serial_number
                                        }
                                    }).to_string());
                                });

                                if let Some(cmd) = send_cmd { scapCallbackBridge(&cmd); }
                                return;
                            }

                            let files = p.get("files").and_then(|f| f.as_array());
                            if files.is_none() {
                                log("[Rust Core] checkMissingFile | No files array");
                                return;
                            }
                            let files = files.unwrap();

                            let mut missing_files: Vec<String> = Vec::new();
                            let mut download_cmd: Option<String> = None;

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                let playlist_list: Vec<&str> = s.sync_playlist_update.split('|').collect();
                                let size_list: Vec<&str> = s.sync_size_update.split('|').collect();

                                if files.is_empty() {
                                    // tidak ada file lokal, semua harus didownload
                                    missing_files = playlist_list.iter().map(|f| f.to_string()).collect();
                                } else {
                                    for (idx, file_to_play) in playlist_list.iter().enumerate() {
                                        let target_size = size_list.get(idx).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                                        let found = files.iter().any(|f| {
                                            let name = f.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                            let size = f.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
                                            name == *file_to_play && size == target_size
                                        });
                                        if !found {
                                            missing_files.push(file_to_play.to_string());
                                        }
                                    }
                                }

                                s.checking = 0;

                                if missing_files.is_empty() {
                                    log("[Rust Core] checkMissingFile | Semua video sudah diunduh");
                                    set_dom_html("backStatus", "Checking missing files: NO missing files");
                                    // langsung ke jadwalDownload
                                    s.jadwal_dicek = s.nama_text_file.clone();
                                    let now_ms = window().unwrap().performance().unwrap().now();
                                    s.pending_transactions.insert(8020, PendingTransaction {
                                        action: "JADWAL_DOWNLOAD".to_string(),
                                        created_at_ms: now_ms,
                                    });
                                    download_cmd = Some(serde_json::json!({
                                        "req_id": 8020,
                                        "action": "JADWAL_DOWNLOAD",
                                        "options": {
                                            "nama_text_file": s.nama_text_file,
                                            "video_folder_lg": s.video_folder_lg
                                        }
                                    }).to_string());
                                } else {
                                    log(&format!("[Rust Core] checkMissingFile | {} video belum diunduh", missing_files.len()));
                                    set_dom_html("backStatus", "Checking missing files: Found missing files");
                                    s.missing_files = missing_files.clone();
                                    s.video_counter = 0;
                                    s.downloading = 1;
                                    // mulai download file pertama
                                    let first_file = &missing_files[0];
                                    let now_ms = window().unwrap().performance().unwrap().now();
                                    s.pending_transactions.insert(8011, PendingTransaction {
                                        action: "CHUNK_DOWNLOAD".to_string(),
                                        created_at_ms: now_ms,
                                    });
                                    download_cmd = Some(serde_json::json!({
                                        "req_id": 8011,
                                        "action": "EXECUTE_COPY_FILE",
                                        "options": {
                                            "source": format!("{}{}", CONFIG.content_download_url, first_file),
                                            "destination": format!("{}{}", s.video_folder_lg, first_file)
                                        }
                                    }).to_string());
                                    set_dom_html("lastStatus", &format!("Mengunduh {}", first_file));
                                }
                            });

                            if let Some(cmd) = download_cmd { scapCallbackBridge(&cmd); }
                        }
                    },

                    "COPY_FILE_CALLBACK" => {
                        // Cek req_id untuk tau ini download video atau copy jadwal
                        let req_id = evt.get("req_id").and_then(|r| r.as_u64()).unwrap_or(0);

                        if req_id == 8021 {
                            // Ini COPY_JADWAL selesai
                            let mut playlist_cmd: Option<String> = None;
                            let mut update_cmd: Option<String> = None;

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.downloading = 0;
                                s.jadwal_dicek = s.file_jadwal.clone();
                                s.running_playlist_sig = String::new();
                                log("[Rust Core] copyJadwal | Success");

                                let nama_sesi_slice = s.nama_sesi_jadwal.replace(".txt", "");
                                // let now_ms = window().unwrap().performance().unwrap().now();
                                update_cmd = Some(serde_json::json!({
                                    "req_id": 8022,
                                    "action": "UPDATE_DOWNLOAD",
                                    "options": {
                                        "kode_tv": s.serial_number,
                                        "nama_sesi": nama_sesi_slice
                                    }
                                }).to_string());

                                let now_ms2 = window().unwrap().performance().unwrap().now();
                                s.pending_transactions.insert(9994, PendingTransaction {
                                    action: "CHECK_FILE_EXISTS".to_string(),
                                    created_at_ms: now_ms2,
                                });
                                playlist_cmd = Some(serde_json::json!({
                                    "req_id": 9994,
                                    "action": "EXECUTE_FILE_EXISTS",
                                    "options": {
                                        "path": format!("{}{}", s.video_folder_lg, s.jadwal_dicek)
                                    }
                                }).to_string());
                            });

                            if let Some(cmd) = update_cmd { scapCallbackBridge(&cmd); }
                            if let Some(cmd) = playlist_cmd { scapCallbackBridge(&cmd); }

                        } else {
                            // Ini chunk download video selesai
                            let mut next_cmd: Option<String> = None;
                            let mut log_msg = String::new();
                            let mut status_msg = String::new();

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.video_counter += 1;

                                if s.video_counter < s.missing_files.len() {
                                    let next_file = s.missing_files[s.video_counter].clone();
                                    log_msg = format!("[Rust Core] chunkDownload | next: {}", next_file);
                                    status_msg = format!("Mengunduh {}", next_file);

                                    let now_ms = window().unwrap().performance().unwrap().now();
                                    s.pending_transactions.insert(8011, PendingTransaction {
                                        action: "CHUNK_DOWNLOAD".to_string(),
                                        created_at_ms: now_ms,
                                    });
                                    next_cmd = Some(serde_json::json!({
                                        "req_id": 8011,
                                        "action": "EXECUTE_COPY_FILE",
                                        "options": {
                                            "source": format!("{}{}", CONFIG.content_download_url, next_file),
                                            "destination": format!("{}{}", s.video_folder_lg, next_file)
                                        }
                                    }).to_string());
                                } else {
                                    s.downloading = 0;
                                    s.video_counter = 0;
                                    log_msg = String::from("[Rust Core] chunkDownload | Sukses mengunduh semua video");
                                    status_msg = String::from("Sukses mengunduh semua video");

                                    let now_ms = window().unwrap().performance().unwrap().now();
                                    s.pending_transactions.insert(8020, PendingTransaction {
                                        action: "JADWAL_DOWNLOAD".to_string(),
                                        created_at_ms: now_ms,
                                    });
                                    next_cmd = Some(serde_json::json!({
                                        "req_id": 8020,
                                        "action": "JADWAL_DOWNLOAD",
                                        "options": {
                                            "nama_text_file": s.nama_text_file,
                                            "video_folder_lg": s.video_folder_lg
                                        }
                                    }).to_string());
                                }
                            });

                            if !log_msg.is_empty() { log(&log_msg); }
                            if !status_msg.is_empty() { set_dom_html("lastStatus", &status_msg); }
                            if let Some(cmd) = next_cmd { scapCallbackBridge(&cmd); }
                        }
                    },

                    "JADWAL_DOWNLOAD_CALLBACK" => {
                        // jadwal .txt berhasil didownload, sekarang copy ke fileJadwal
                        let mut copy_cmd: Option<String> = None;

                        CORE_STATE.with(|state| {
                            let mut s = state.borrow_mut();
                            s.downloading = 0;
                            log("[Rust Core] jadwalDownload | Success");
                            set_dom_html("backStatus", "Proses unduh file jadwal: Success");

                            let now_ms = window().unwrap().performance().unwrap().now();
                            s.pending_transactions.insert(8021, PendingTransaction {
                                action: "COPY_JADWAL".to_string(),
                                created_at_ms: now_ms,
                            });
                            copy_cmd = Some(serde_json::json!({
                                "req_id": 8021,
                                "action": "EXECUTE_COPY_FILE",
                                "options": {
                                    "source": format!("{}{}", s.video_folder_lg, s.nama_text_file),
                                    "destination": format!("{}{}", s.video_folder_lg, s.file_jadwal)
                                }
                            }).to_string());
                        });

                        if let Some(cmd) = copy_cmd { scapCallbackBridge(&cmd); }
                    },

                    "REMOVE_ALL_CALLBACK" => {
                        log("[Rust Core] clearStorage | Success. Rebooting...");
                        set_dom_html("lastStatus", "Storage cleared. Rebooting...");
                        scapCallbackBridge(&serde_json::json!({
                            "req_id": 9999,
                            "action": "EXECUTE_POWER_CMD",
                            "options": { "powerCommand": "REBOOT" }
                        }).to_string());
                    },

                    "CAPTURE_SCREEN_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let capture_data = p.get("data").and_then(|d| d.as_str()).unwrap_or("");
                            if capture_data.is_empty() {
                                log("[Rust Core] captureScreen | No data returned");
                                return;
                            }

                            log("[Rust Core] captureScreen | Success");
                            set_dom_html("backStatus", "Sukses mengambil tampilan layar");

                            let (kode_tv, tv_app_ver) = CORE_STATE.with(|state| {
                                let s = state.borrow();
                                (s.serial_number.clone(), s.device_app_version.clone())
                            });

                            scapCallbackBridge(&serde_json::json!({
                                "req_id": 8041,
                                "action": "SEND_CAPTURE",
                                "options": {
                                    "kode_tv": kode_tv,
                                    "tv_app_ver": tv_app_ver,
                                    "capture_data": capture_data
                                }
                            }).to_string());
                        }
                    },

                    "WRITE_FILE_CALLBACK" => {
                        log("[Rust Core] writeLog | Success");
                    },

                    "LIST_LOG_FILES_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let files = p.get("files").and_then(|f| f.as_array());
                            if files.is_none() { return; }
                            let files = files.unwrap();

                            let mut send_cmd: Option<String> = None;

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.array_log.clear();

                                for file in files {
                                    let name = file.get("name").and_then(|n| n.as_str()).unwrap_or("");
                                    let prefix = format!("{}_", s.serial_number);
                                    if name.contains(&prefix) {
                                        s.array_log.push(name.to_string());
                                    }
                                }

                                if s.array_log.is_empty() {
                                    log("[Rust Core] checkLocalLogFiles | Tidak ada log yang perlu dikirim");
                                    return;
                                }

                                log(&format!("[Rust Core] checkLocalLogFiles | Ada {} log belum dikirim", s.array_log.len()));
                                set_dom_html("backStatus", "Menemukan log tayang di local");

                                let first_log = s.array_log[0].clone();
                                s.file_log = first_log.clone();

                                let now_ms = window().unwrap().performance().unwrap().now();
                                s.pending_transactions.insert(8051, PendingTransaction {
                                    action: "KIRIM_LOG".to_string(),
                                    created_at_ms: now_ms,
                                });

                                send_cmd = Some(serde_json::json!({
                                    "req_id": 8051,
                                    "action": "KIRIM_LOG",
                                    "options": {
                                        "file_log": first_log,
                                        "video_folder_local": s.video_folder_local,
                                        "kode_tv": s.serial_number
                                    }
                                }).to_string());
                            });

                            if let Some(cmd) = send_cmd { scapCallbackBridge(&cmd); }
                        }
                    },

                    "KIRIM_LOG_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let status = p.get("status").and_then(|v| v.as_u64()).unwrap_or(0);
                            let file_log = CORE_STATE.with(|state| state.borrow().file_log.clone());

                            if status == 200 {
                                log(&format!("[Rust Core] kirimLog | Success | {}", file_log));
                                set_dom_html("backStatus", &format!("Kirim Log {} Sukses", file_log));

                                // hapus file log setelah berhasil dikirim
                                let delete_cmd = CORE_STATE.with(|state| {
                                    let s = state.borrow();
                                    serde_json::json!({
                                        "req_id": 8052,
                                        "action": "EXECUTE_REMOVE_FILE",
                                        "options": {
                                            "file": format!("{}{}", s.video_folder_lg, file_log)
                                        }
                                    }).to_string()
                                });
                                scapCallbackBridge(&delete_cmd);
                            } else {
                                log(&format!("[Rust Core] kirimLog | Failed | status: {}", status));
                                set_dom_html("backStatus", &format!("Kirim Log {} Gagal", file_log));
                            }
                        }
                    },

                    "CHECK_IPK_VERSION_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let status = p.get("status").and_then(|v| v.as_u64()).unwrap_or(0);
                            let response_text = p.get("response_text").and_then(|v| v.as_str()).unwrap_or("");

                            if status != 200 || response_text.is_empty() {
                                log("[Rust Core] checkIPKVersion | Server tidak menjawab");
                                return;
                            }

                            let parsed: serde_json::Value = match serde_json::from_str(response_text) {
                                Ok(v) => v,
                                Err(_) => {
                                    log("[Rust Core] checkIPKVersion | Invalid JSON");
                                    return;
                                }
                            };

                            let server_app_ver_path = parsed.get("appVerPath").and_then(|v| v.as_str()).unwrap_or("");
                            let server_version_app = parsed.get("versionApp").and_then(|v| v.as_str()).unwrap_or("");
                            let server_path_app = parsed.get("pathApp").and_then(|v| v.as_str()).unwrap_or("");
                            let log_request = parsed.get("logRequest").and_then(|v| v.as_bool()).unwrap_or(false);

                            log(&format!("[Rust Core] checkIPKVersion | SERVER appVerPath: {}", server_app_ver_path));
                            log(&format!("[Rust Core] checkIPKVersion | SERVER versionApp: {}", server_version_app));
                            log(&format!("[Rust Core] checkIPKVersion | SERVER pathApp: {}", server_path_app));

                            // update log_request dari server
                            CORE_STATE.with(|state| {
                                state.borrow_mut().log_request = log_request;
                            });
                            log(&format!("[Rust Core] Set log_request: {}", log_request));

                            let mut needs_upgrade = false;
                            let mut new_app_ver_path = String::new();
                            let mut new_version_app = String::new();
                            let mut new_path_app = String::new();

                            CORE_STATE.with(|state| {
                                let s = state.borrow();
                                let cur_check_url = s.target_url_check_ipk.clone();
                                let cur_version = s.target_device_app_version.clone();
                                let cur_path = s.target_url_upgrade_ipk.clone();

                                let app_ver_path_diff = !server_app_ver_path.is_empty() && cur_check_url != server_app_ver_path;
                                let version_diff = !server_version_app.is_empty() && cur_version != server_version_app;
                                let path_diff = !server_path_app.is_empty() && cur_path != server_path_app;

                                if app_ver_path_diff {
                                    log("[Rust Core] checkIPKVersion | URL appVerPath different");
                                    new_app_ver_path = server_app_ver_path.to_string();
                                } else if version_diff && path_diff {
                                    log(&format!("[Rust Core] checkIPKVersion | New version found: {}", server_version_app));
                                    new_version_app = server_version_app.to_string();
                                    new_path_app = server_path_app.to_string();
                                    needs_upgrade = true;
                                } else {
                                    log("[Rust Core] checkIPKVersion | AppVer is Latest");
                                }
                            });

                            if !new_app_ver_path.is_empty() {
                                CORE_STATE.with(|state| {
                                    state.borrow_mut().target_url_check_ipk = new_app_ver_path;
                                });
                                // re-check dengan URL baru
                                let url = CORE_STATE.with(|state| state.borrow().target_url_check_ipk.clone());
                                scapCallbackBridge(&serde_json::json!({
                                    "req_id": 8070,
                                    "action": "CHECK_IPK_VERSION",
                                    "options": { "url": url }
                                }).to_string());
                                return;
                            }

                            if needs_upgrade {
                                CORE_STATE.with(|state| {
                                    let mut s = state.borrow_mut();
                                    s.target_device_app_version = new_version_app;
                                    s.target_url_upgrade_ipk = new_path_app;
                                    s.in_process_upgrade_ipk = 1;
                                });
                                // trigger getServerProperty → setServerProperty → upgradeIpkApplication
                                scapCallbackBridge(&serde_json::json!({
                                    "req_id": 8071,
                                    "action": "FETCH_SERVER_PROPERTY"
                                }).to_string());
                            }
                        }
                    },

                    "SERVER_PROPERTY_CALLBACK" => {
                        if let Some(p) = evt.get("payload") {
                            let req_id = evt.get("req_id").and_then(|r| r.as_u64()).unwrap_or(0);

                            // hanya proses upgrade jika dipanggil dari checkIPKVersion
                            if req_id != 8071 {
                                CORE_STATE.with(|state| {
                                    let mut s = state.borrow_mut();
                                    s.server_config.server_ip = p.get("serverIp").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    s.server_config.server_port = p.get("serverPort").and_then(|v| v.as_u64()).unwrap_or(80) as u32;
                                    s.server_config.app_launch_mode = p.get("appLaunchMode").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    s.server_config.fqdn_addr = p.get("fqdnAddr").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                });
                                return;
                            }

                            // lanjut ke setServerProperty untuk upgrade
                            let mut fqdn_mode = p.get("fqdnMode").and_then(|v| v.as_bool()).unwrap_or(true);
                            let mut app_type = p.get("appType").and_then(|v| v.as_str()).unwrap_or("ipk").to_string();
                            let mut launch_mode = p.get("appLaunchMode").and_then(|v| v.as_str()).unwrap_or("local").to_string();

                            if app_type != "ipk" { app_type = String::from("ipk"); }
                            if launch_mode != "local" { launch_mode = String::from("local"); }
                            if !fqdn_mode { fqdn_mode = true; }

                            CORE_STATE.with(|state| {
                                let mut s = state.borrow_mut();
                                s.app_fqdn_mode = fqdn_mode;
                                s.app_type_var = app_type;
                                s.app_launch_mode = launch_mode;
                            });

                            let (fqdn, app_type, launch, upgrade_url) = CORE_STATE.with(|state| {
                                let s = state.borrow();
                                (s.app_fqdn_mode, s.app_type_var.clone(), s.app_launch_mode.clone(), s.target_url_upgrade_ipk.clone())
                            });

                            if fqdn && app_type == "ipk" && launch == "local" {
                                log("[Rust Core] setServerProperty | triggering upgrade");
                                scapCallbackBridge(&serde_json::json!({
                                    "req_id": 8072,
                                    "action": "EXECUTE_SET_SERVER",
                                    "options": {
                                        "serverIp": "0.0.0.0",
                                        "serverPort": 0,
                                        "secureConnection": true,
                                        "appLaunchMode": launch,
                                        "fqdnMode": fqdn,
                                        "fqdnAddr": upgrade_url,
                                        "appType": app_type,
                                        "autoSet": "off"
                                    }
                                }).to_string());
                            }
                        }
                    },

                    "SET_SERVER_CALLBACK" => {
                        let req_id = evt.get("req_id").and_then(|r| r.as_u64()).unwrap_or(0);
                        if req_id != 8072 { return; }

                        let is_upgrading = CORE_STATE.with(|state| state.borrow().is_upgrading);
                        if is_upgrading == 1 { return; }

                        CORE_STATE.with(|state| {
                            state.borrow_mut().is_upgrading = 1;
                        });

                        log("[Rust Core] setServerProperty | Success. Starting upgrade...");
                        set_dom_html("lastStatus", "Upgrading Application...");

                        scapCallbackBridge(&serde_json::json!({
                            "req_id": 8073,
                            "action": "EXECUTE_APP_UPGRADE",
                            "options": {
                                "type": "ipk",
                                "to": "LOCAL",
                                "recovery": false
                            }
                        }).to_string());
                    },

                    "APP_UPGRADE_CALLBACK" => {
                        let req_id = evt.get("req_id").and_then(|r| r.as_u64()).unwrap_or(0);
                        if req_id != 8073 { return; }

                        CORE_STATE.with(|state| {
                            state.borrow_mut().is_upgrading = 0;
                        });

                        log("[Rust Core] upgradeIpkApplication | Success. Rebooting...");
                        set_dom_html("lastStatus", "Upgrade success. Rebooting...");

                        let closure = Closure::wrap(Box::new(move || {
                            scapCallbackBridge(&serde_json::json!({
                                "req_id": 9999,
                                "action": "EXECUTE_POWER_CMD",
                                "options": { "powerCommand": "REBOOT" }
                            }).to_string());
                        }) as Box<dyn FnMut()>);
                        window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
                            closure.as_ref().unchecked_ref(), 10000
                        ).unwrap();
                        closure.forget();
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
    let download_timeout_ms = 300000.0; // 5 menit untuk download
    let mut timed_out_ids: Vec<u64> = Vec::new();

    CORE_STATE.with(|state| {
        if let Ok(s) = state.try_borrow() {
            for (req_id, tx) in s.pending_transactions.iter() {
                let threshold = match tx.action.as_str() {
                    "CHUNK_DOWNLOAD" | "JADWAL_DOWNLOAD" | "COPY_JADWAL" => download_timeout_ms,
                    _ => timeout_threshold_ms,
                };
                if (current_time_ms - tx.created_at_ms) > threshold {
                    timed_out_ids.push(*req_id);
                }
            }
        }
    });

    if !timed_out_ids.is_empty() {
        CORE_STATE.with(|state| {
            if let Ok(mut s) = state.try_borrow_mut() {
                for id in timed_out_ids {
                    if let Some(tx) = s.pending_transactions.remove(&id) {
                        js_log(&format!("[WATCHDOG] Timeout on Transaction #{} ({})", id, tx.action));
                        // hanya set fault untuk non-download transactions
                        match tx.action.as_str() {
                            "CHUNK_DOWNLOAD" | "JADWAL_DOWNLOAD" | "COPY_JADWAL" => {},
                            _ => { s.hardware_fault_detected = true; }
                        }
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

#[wasm_bindgen]
pub fn clear_storage() {
    log("[Rust Core] clearStorage | Triggered");
    set_dom_html("lastStatus", "Clearing storage...");

    CORE_STATE.with(|state| {
        let mut s = state.borrow_mut();
        let now_ms = window().unwrap().performance().unwrap().now();
        s.pending_transactions.insert(8099, PendingTransaction {
            action: "CLEAR_STORAGE".to_string(),
            created_at_ms: now_ms,
        });
    });

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8099,
        "action": "EXECUTE_REMOVE_ALL",
        "options": { "device": "internal" }
    }).to_string());
}

#[wasm_bindgen]
pub fn write_log(video_name: &str) {
    if video_name.is_empty() { return; }

    let (kode_tv, video_folder_lg) = CORE_STATE.with(|state| {
        let s = state.borrow();
        (s.serial_number.clone(), s.video_folder_lg.clone())
    });

    if kode_tv.is_empty() { return; }

    // format timestamp: yyyymmddHHMMSS
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let month = now.get_month() + 1;
    let day = now.get_date();
    let hour = now.get_hours();
    let minute = now.get_minutes();
    let second = now.get_seconds();

    let timestamp = format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02} ",
        year, month, day, hour, minute, second
    );

    // format sama dengan JS: yyyymmdd
    let file_log = format!(
        "{}_{}{}{}{}.txt",
        kode_tv,
        format!("{:04}", year),
        format!("{:02}", month),
        format!("{:02}", day),
        format!("{:02}", hour)
    );

    let data_log = format!("{}{}\n", timestamp, video_name);
    let text_path = format!("{}{}", video_folder_lg, file_log);

    CORE_STATE.with(|state| {
        let mut s = state.borrow_mut();
        let now_ms = window().unwrap().performance().unwrap().now();
        s.pending_transactions.insert(8060, PendingTransaction {
            action: "WRITE_LOG".to_string(),
            created_at_ms: now_ms,
        });
    });

    scapCallbackBridge(&serde_json::json!({
        "req_id": 8060,
        "action": "EXECUTE_WRITE_FILE",
        "options": {
            "data": data_log,
            "path": text_path,
            "mode": "append",
            "length": data_log.len(),
            "encoding": "utf8"
        }
    }).to_string());
}