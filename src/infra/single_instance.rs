//! 单实例：PID 文件方案（原 `model.rs` 迁移）

use crate::core::config_store::ConfigStore;
use std::path::PathBuf;

pub fn pid_path() -> PathBuf {
    ConfigStore::from_env().pid_path()
}

/// 写入当前进程 PID
pub fn write_pid() {
    let path = pid_path();
    let _ = std::fs::create_dir_all(path.parent().unwrap_or(std::path::Path::new(".")));
    let _ = std::fs::write(path, std::process::id().to_string());
}

/// 检测是否已有其他实例在运行（PID 文件中的进程存活）
pub fn other_instance_running() -> bool {
    let Ok(s) = std::fs::read_to_string(pid_path()) else {
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

/// 终止其他实例（唤醒场景）
pub fn kill_other() {
    if let Ok(s) = std::fs::read_to_string(pid_path()) {
        if let Ok(pid) = s.trim().parse::<u32>() {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
        }
    }
}
