//! 图标图像工具：统一创建「带圆角背景 + 系统图标的 48/64px」图标表面

use cairo::{Context, Format, ImageSurface};
use gdk_pixbuf::{Pixbuf, PixbufExt};
use gtk::{gdk, prelude::*};
use std::path::Path;

/// 图标的最终绘制尺寸（图标大小 + 内边距）
pub fn icon_total(icon_size: i32) -> i32 {
    icon_size + 16
}

/// 系统图标主题中查找图标
fn find_icon_path(names: &[&str]) -> Option<std::path::PathBuf> {
    let mut search: Vec<std::path::PathBuf> = vec![];
    if let Some(data_dirs) = std::env::var_os("XDG_DATA_DIRS") {
        for d in std::env::split_paths(&data_dirs) {
            search.push(d.join("icons"));
        }
    }
    if let Some(home) = dirs::home_dir() {
        search.push(home.join(".local/share/icons"));
    }
    search.push(std::path::PathBuf::from("/usr/share/pixmaps"));
    search.push(std::path::PathBuf::from("/usr/share/icons"));

    let sizes = ["48x48", "64x64", "96x96", "scalable"];
    let mut checked = std::collections::HashSet::new();
    for dir in search {
        for name in names {
            for size in sizes {
                let cand = dir.join(size).join(format!("{name}.png"));
                if cand.exists() {
                    checked.insert(cand.clone());
                    return Some(cand);
                }
                let cand = dir.join(size).join(format!("{name}.svg"));
                if cand.exists() {
                    checked.insert(cand.clone());
                    return Some(cand);
                }
                let cand = dir.join("hicolor").join(size).join(format!("{name}.png"));
                if cand.exists() {
                    checked.insert(cand.clone());
                    return Some(cand);
                }
            }
        }
    }
    // 兜底：常见目录
    for name in names {
        let cand = std::path::PathBuf::from("/usr/share/pixmaps").join(format!("{name}.png"));
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}

/// 渲染带圆角白色背景 + 居中图标的表面
pub fn build_icon_surface(file_path: &str, icon_size: i32, is_dir: bool) -> ImageSurface {
    let total = icon_total(icon_size);
    let surface = ImageSurface::create(Format::ARgb32, total, total).unwrap();
    let cr = Context::new(&surface).unwrap();

    // 圆角白色背景（选中态高亮由桌面层负责，这里画基础底）
    let radius = (total as f64) * 0.18;
    round_rect(
        &cr,
        1.5,
        1.5,
        total as f64 - 3.0,
        total as f64 - 3.0,
        radius,
    );
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.85);
    cr.fill();

    // 绘制图标
    let path = Path::new(file_path);
    let names = icon_names_for(path, is_dir);
    let pad = 8.0;
    let avail = total as f64 - pad * 2.0;

    if let Some(icon_path) = find_icon_path(&names) {
        if let Ok(pix) = Pixbuf::from_file(&icon_path) {
            let (w, h) = (pix.width() as f64, pix.height() as f64);
            let scale = (avail / w).min(avail / h).min(1.0);
            let dw = w * scale;
            let dh = h * scale;
            cr.save().unwrap();
            cr.translate((total as f64 - dw) / 2.0, (total as f64 - dh) / 2.0);
            cr.scale(scale, scale);
            gdk::cairo::set_source_pixbuf(&cr, &pix, 0.0, 0.0);
            cr.paint().unwrap();
            cr.restore().unwrap();
        }
    } else {
        // 无图标：绘制文字占位（首字母）
        cr.set_source_rgba(0.35, 0.42, 0.55, 0.9);
        cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
        let first = file_path
            .rsplit('/')
            .next()
            .and_then(|s| s.chars().next())
            .unwrap_or('?')
            .to_string();
        cr.set_font_size(avail * 0.55);
        let ext = cr.text_extents(&first).unwrap();
        cr.move_to(
            (total as f64 - ext.width()) / 2.0 - ext.x_bearing(),
            (total as f64 - ext.height()) / 2.0 - ext.y_bearing(),
        );
        cr.show_text(&first).unwrap();
    }

    surface
}

/// 根据文件类型生成候选图标名
fn icon_names_for(path: &Path, is_dir: bool) -> Vec<&'static str> {
    if path.is_file()
        && path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.ends_with(".desktop"))
            .unwrap_or(false)
    {
        // 读取 desktop 文件里的 Icon= 字段，能匹配则用之
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let l = line.trim();
                if let Some(v) = l.strip_prefix("Icon=") {
                    if !v.is_empty() {
                        return vec![v, "application-x-executable", "text-x-generic"];
                    }
                }
            }
        }
        return vec!["application-x-executable", "text-x-generic"];
    }
    if is_dir {
        return vec![
            "folder",
            "folder-symbolic",
            "inode-directory",
            "system-file-manager",
        ];
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" => {
            vec!["image-x-generic", "image", "photo"]
        }
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" => vec!["video-x-generic", "video"],
        "mp3" | "wav" | "flac" | "ogg" | "m4a" => vec!["audio-x-generic", "audio"],
        "pdf" => vec!["application-pdf", "document-pdf"],
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "tgz" => {
            vec!["package-x-generic", "application-x-archive", "archive"]
        }
        "doc" | "docx" | "wps" | "odt" => vec!["document-word", "text-x-generic"],
        "xls" | "xlsx" | "et" | "ods" => vec!["x-office-spreadsheet", "text-x-generic"],
        "ppt" | "pptx" | "dps" | "odp" => vec!["x-office-presentation", "text-x-generic"],
        "txt" | "md" | "log" | "ini" | "conf" | "cfg" => vec!["text-x-generic", "text-plain"],
        "deb" | "rpm" => vec!["package-x-generic", "application-x-archive"],
        "exe" | "run" | "sh" => vec!["application-x-executable", "text-x-generic"],
        _ => vec!["text-x-generic", "application-octet-stream"],
    }
}

/// 绘制圆角矩形路径
pub fn round_rect(cr: &Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_path();
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        1.5 * std::f64::consts::PI,
    );
    cr.arc(x + w - r, y + r, r, 1.5 * std::f64::consts::PI, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, 0.5 * std::f64::consts::PI);
    cr.arc(
        x + r,
        y + h - r,
        r,
        0.5 * std::f64::consts::PI,
        std::f64::consts::PI,
    );
    cr.close_path();
}
