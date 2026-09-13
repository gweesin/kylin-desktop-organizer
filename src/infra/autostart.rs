//! 开机自启（原 `model.rs` 迁移）：`~/.config/autostart/kylin-desktop-organizer.desktop`

use crate::core::config_store::APP_ID;
use std::path::PathBuf;

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
        let _ = std::fs::remove_file(&path);
        return;
    }
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
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
    let _ = std::fs::write(&path, content);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autostart_path_points_to_config_dir() {
        let p = autostart_path();
        assert!(p.ends_with("autostart"));
        assert!(p
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with(".desktop"));
    }
}
