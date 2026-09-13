//! 桌面图标分类整理规则：识别「我的电脑 / 回收站 / 应用 / 文件夹 / 文件 / 图片视频 / 压缩包」

use std::path::Path;

/// 桌面常见应用图标映射（麒麟 V10 / UKUI 默认桌面）
pub const APP_KEYWORDS: &[(&str, &str)] = &[
    ("computer", "我的电脑"),
    ("computer-symbolic", "我的电脑"),
    ("computer.svg", "我的电脑"),
    ("pc", "我的电脑"),
    ("trash", "回收站"),
    ("user-trash", "回收站"),
    ("firefox", "浏览器"),
    ("browser", "浏览器"),
    ("chromium", "浏览器"),
    ("word", "WPS办公"),
    ("wps", "WPS办公"),
    ("et", "WPS办公"),
    ("wpp", "WPS办公"),
    ("calc", "计算器"),
    ("calculator", "计算器"),
    ("terminal", "终端"),
    ("konsole", "终端"),
    ("system-file-manager", "文件管理器"),
    ("file-manager", "文件管理器"),
    ("folder", "文件管理器"),
    ("weixin", "微信"),
    ("wechat", "微信"),
    ("qq", "QQ"),
    ("music", "音乐"),
    ("audacious", "音乐"),
    ("video", "视频"),
    ("vlc", "视频"),
    ("image", "图片"),
    ("photo", "图片"),
    ("gimp", "图片"),
    ("pdf", "PDF阅读"),
    ("document", "文档"),
    ("text-editor", "文本编辑"),
    ("gedit", "文本编辑"),
    ("editor", "文本编辑"),
    ("settings", "系统设置"),
    ("control-center", "系统设置"),
    ("preferences", "系统设置"),
];

/// 文件夹 / 文件默认归属
pub const FOLDER_CATEGORY: &str = "文件夹";
pub const FILE_CATEGORY: &str = "文件";
pub const IMAGE_VIDEO_CATEGORY: &str = "图片视频";
pub const ARCHIVE_CATEGORY: &str = "压缩包";

pub const CATEGORY_ORDER: &[&str] = &[
    "我的电脑",
    "回收站",
    "应用",
    "文件夹",
    "文件",
    "图片视频",
    "压缩包",
    "其他",
];

/// 根据桌面图标文件名分类
pub fn classify_desktop_item(file_name: &str, path: &Path) -> String {
    let lower = file_name.to_lowercase();

    // 我的电脑 / 回收站
    if lower.contains("computer")
        || lower.contains("我的电脑")
        || lower.contains("mycomputer")
        || lower == "电脑"
    {
        return "我的电脑".into();
    }
    if lower.contains("trash") || lower.contains("回收站") || lower.contains("recycle") {
        return "回收站".into();
    }

    // .desktop 启动器 -> 应用
    if file_name.ends_with(".desktop") {
        if let Some(cat) = classify_desktop_launcher(path) {
            return cat;
        }
        return "应用".into();
    }

    // 目录
    if path.is_dir() {
        if lower.contains("图片")
            || lower.contains("照片")
            || lower.contains("截图")
            || lower.contains("pic")
            || lower.contains("photo")
        {
            return IMAGE_VIDEO_CATEGORY.into();
        }
        if lower.contains("压缩") || lower.contains("backup") || lower.contains("备份") {
            return ARCHIVE_CATEGORY.into();
        }
        return FOLDER_CATEGORY.into();
    }

    // 文件按扩展名
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tif" | "tiff" => {
            IMAGE_VIDEO_CATEGORY.into()
        }
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "mp3" | "wav" | "flac" | "ogg" | "m4a"
        | "rmvb" => IMAGE_VIDEO_CATEGORY.into(),
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "tgz" | "deb" | "rpm" => {
            ARCHIVE_CATEGORY.into()
        }
        _ => FILE_CATEGORY.into(),
    }
}

/// 读取 .desktop 文件，若命中图标关键词则返回具体类别（如 浏览器 / 微信），否则 None
fn classify_desktop_launcher(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let lower = content.to_lowercase();
    for (kw, cat) in APP_KEYWORDS {
        if lower.contains(kw) {
            return Some((*cat).into());
        }
    }
    None
}

/// 空文件夹名
pub fn is_empty_folder(name: &str) -> bool {
    name == "文件夹" || name == "其他"
}
