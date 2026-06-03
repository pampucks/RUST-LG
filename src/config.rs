use std::time::Duration;

#[derive(Debug, Clone)]
pub enum DevMode { Uat, Prod }

#[derive(Debug, Clone)]
pub enum PlayMode { Single, Dual }

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub dev_mode: DevMode,
    pub play_mode: PlayMode,
    pub device_app_version: &'static str,
    pub os_type: &'static str,
    
    // Server Endpoints
    pub content_check_url: &'static str,
    pub content_download_url: &'static str,
    pub capture_upload_url: &'static str,
    pub log_upload_url: &'static str,
    pub success_update_url: &'static str,
    pub url_check_ipk_version: &'static str,
    pub url_upgrade_ipk: &'static str,

    // Storage Paths
    pub video_folder_lg: &'static str,
    pub video_folder_local: &'static str,
    pub usb_folder_lg: &'static str,

    // Network Setup
    pub master_ip: &'static str,
    pub port: u16,

    // Operational Time-Interval Durations
    pub download_progress_interval: Duration,
    pub check_ipk_interval: Duration,
    pub update_content_interval: Duration,
    pub update_status_interval: Duration,
    pub send_capture_interval: Duration,
    pub post_log_interval: Duration,
    pub check_bg_service_interval: Duration,
}

pub const CONFIG: AppConfig = AppConfig {
    dev_mode: DevMode::Prod,
    play_mode: PlayMode::Single,
    device_app_version: "20250904",
    os_type: "webos",
    
    content_check_url: "http://dl.idm.digimaxsignage.com/tvapp",
    content_download_url: "http://dl.idm.digimaxsignage.com/app/video/",
    capture_upload_url: "http://ul.idm.digimaxsignage.com/upload-screen",
    log_upload_url: "http://ul.idm.digimaxsignage.com/send-file-log",
    success_update_url: "http://ul.idm.digimaxsignage.com/update-content-done",
    url_check_ipk_version: "http://192.168.1.111/apps/dnd/newidm/pbi/webos/single/appver.json",
    url_upgrade_ipk: "http://192.168.1.111/apps/dnd/newidm/pbi/webos/single/20250904.ipk",

    video_folder_lg: "file://internal/",
    video_folder_local: "file:////mnt/lg/appstore/scap/contents/",
    usb_folder_lg: "file://usb:1/",

    master_ip: "192.168.1.101",
    port: 9990,

    download_progress_interval: Duration::from_millis(3000),
    check_ipk_interval: Duration::from_millis(1500000),
    update_content_interval: Duration::from_millis(1800000),
    update_status_interval: Duration::from_millis(2400000),
    send_capture_interval: Duration::from_millis(3000000),
    post_log_interval: Duration::from_millis(3600000),
    check_bg_service_interval: Duration::from_millis(60000),
};