//! 桌面应用启动 / 文件打开

use gio::prelude::*;
use std::process::Command;

pub const XDG_OPEN: &str = "xdg-open";
pub const EXE_EXTENSIONS: &[&str] = &["sh", "run", "AppImage", "desktop"];

/// 打开文件 / 目录 / 应用
pub fn open_path(path: &str) {
    let _ = Command::new(XDG_OPEN).arg(path).spawn();
}

/// 在文件管理器中显示
pub fn show_in_file_manager(path: &str) {
    for fm in ["dde-file-manager", "peony", "nautilus", "caja", "dolphin"] {
        if let Ok(st) = Command::new(fm).args(["--show-items", path]).status() {
            if st.success() {
                return;
            }
        }
    }
    let _ = Command::new("xdg-open").arg(path).spawn();
}

pub fn is_desktop_file(path: &str) -> bool {
    path.ends_with(".desktop")
}

pub fn is_executable(path: &str) -> bool {
    let lower = path.to_lowercase();
    EXE_EXTENSIONS.iter().any(|e| lower.ends_with(e))
}

/// 删除文件（回收站优先，失败则直接删除）——返回是否成功
pub fn delete_file(path: &str) -> bool {
    let gio_f = gio::File::for_path(path);
    if let Err(e) = gio_f.trash(gio::Cancellable::NONE) {
        log::warn!("移至回收站失败: {e}，尝试直接删除");
        return std::fs::remove_file(path).is_ok();
    }
    true
}

pub fn is_removable_dir(path: &str) -> bool {
    let p = std::path::Path::new(path);
    if let Ok(md) = p.metadata() {
        if !md.is_dir() {
            return false;
        }
        // 空目录才能删除
        return p
            .read_dir()
            .map(|mut it| it.next().is_none())
            .unwrap_or(false);
    }
    false
}
