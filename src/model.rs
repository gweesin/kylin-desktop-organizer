//! 配置与持久化：config.json 位于 ~/.config/kylin-desktop-organizer/

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const APP_ID: &str = "kylin-desktop-organizer";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IconState {
    pub path: String,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BoxState {
    pub id: String,
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub collapsed: bool,
    pub color: Option<String>,
    pub icons: Vec<IconState>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub first_run: bool,
    pub icon_size: i32,
    pub grid_gap: i32,
    pub double_click_organize: bool,
    pub auto_organize: bool,
    pub show_files: bool,
    pub show_desktop_icons: bool,
    pub theme: String,     // "light" | "dark"
    pub box_style: String, // "card" | "solid" | "plain"
    pub box_opacity: f64,
    pub desktop_icons: Vec<IconState>,
    pub boxes: Vec<BoxState>,
    pub box_seq: u64,
    pub hidden: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            first_run: true,
            icon_size: 48,
            grid_gap: 10,
            double_click_organize: true,
            auto_organize: false,
            show_files: true,
            show_desktop_icons: true,
            theme: "light".to_string(),
            box_style: "card".to_string(),
            box_opacity: 0.88,
            desktop_icons: vec![],
            boxes: vec![],
            box_seq: 0,
            hidden: vec![],
        }
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|d| d.join(APP_ID))
        .unwrap_or_else(|| PathBuf::from(".").join(".kylin-desktop-organizer"))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn pid_path() -> PathBuf {
    config_dir().join("app.pid")
}

pub fn load_config() -> Config {
    match fs::read_to_string(config_path()) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config(cfg: &Config) {
    if let Err(e) = fs::create_dir_all(config_dir()) {
        log::warn!("无法创建配置目录: {e}");
        return;
    }
    match serde_json::to_string_pretty(cfg) {
        Ok(s) => {
            let tmp = config_dir().join("config.json.tmp");
            if fs::write(&tmp, s).is_ok() {
                let _ = fs::rename(&tmp, config_path());
            }
        }
        Err(e) => log::warn!("配置序列化失败: {e}"),
    }
}

pub fn write_pid() {
    let _ = fs::create_dir_all(config_dir());
    let _ = fs::write(pid_path(), std::process::id().to_string());
}

/// 检测是否已有实例在运行
pub fn other_instance_running() -> bool {
    let Ok(s) = fs::read_to_string(pid_path()) else {
        return false;
    };
    let Ok(pid) = s.trim().parse::<u32>() else {
        return false;
    };
    if pid == std::process::id() {
        return false;
    }
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|st| st.success())
        .unwrap_or(false)
}

pub fn kill_other() {
    if let Ok(s) = fs::read_to_string(pid_path()) {
        if let Ok(pid) = s.trim().parse::<u32>() {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
        }
    }
}

/// 开机自启文件路径
pub fn autostart_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("autostart")
        .join(format!("{APP_ID}.desktop"))
}

pub fn current_exe() -> String {
    std::env::current_exe()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| format!("/usr/local/bin/{APP_ID}"))
}

pub fn set_autostart(enabled: bool) {
    let path = autostart_path();
    if !enabled {
        let _ = fs::remove_file(&path);
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let content = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=桌面整理\n\
         Comment=桌面图标整理助手\n\
         Exec={}\n\
         Terminal=false\n\
         X-GNOME-Autostart-enabled=true\n",
        current_exe()
    );
    let _ = fs::write(&path, content);
}
