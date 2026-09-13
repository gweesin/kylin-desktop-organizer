//! 桌面图标分类整理规则：识别「我的电脑 / 回收站 / 应用 / 文件夹 / 文件 / 图片视频 / 压缩包」
//!
//! 重构说明：原 `categorize.rs` 把「文件名匹配」「读取 .desktop 文件」「文件系统判断」混在一起，
//! 无法脱离文件系统测试。这里拆出多个**纯函数**（不碰文件系统），
//! 仅对外保留 `classify_desktop_item` 作为需要文件系统的总入口。

use std::path::Path;

/// 桌面常见应用图标关键词映射（麒麟 V10 / UKUI 默认桌面）
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
pub const APP_CATEGORY: &str = "应用";

/// 一键整理的盒子展示顺序
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

/// 纯函数：按「小写文件名」识别特殊图标（我的电脑 / 回收站）
pub fn classify_special_name(lower: &str) -> Option<&'static str> {
    if lower.contains("computer")
        || lower.contains("我的电脑")
        || lower.contains("mycomputer")
        || lower == "电脑"
    {
        return Some("我的电脑");
    }
    if lower.contains("trash") || lower.contains("回收站") || lower.contains("recycle") {
        return Some("回收站");
    }
    None
}

/// 纯函数：目录按名称细分（图片视频 / 压缩包）
pub fn classify_dir_name(lower: &str) -> Option<&'static str> {
    if lower.contains("图片")
        || lower.contains("照片")
        || lower.contains("截图")
        || lower.contains("pic")
        || lower.contains("photo")
    {
        return Some(IMAGE_VIDEO_CATEGORY);
    }
    if lower.contains("压缩") || lower.contains("backup") || lower.contains("备份") {
        return Some(ARCHIVE_CATEGORY);
    }
    None
}

/// 纯函数：文件按扩展名分类（扩展名需已小写）
pub fn classify_file_by_ext(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tif" | "tiff"
        | "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "mp3" | "wav" | "flac" | "ogg"
        | "m4a" | "rmvb" => IMAGE_VIDEO_CATEGORY,
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "iso" | "tgz" | "deb" | "rpm" => {
            ARCHIVE_CATEGORY
        }
        _ => FILE_CATEGORY,
    }
}

/// 纯函数：解析 `.desktop` 文件内容，命中图标关键词返回具体类别，否则 None
pub fn classify_launcher_content(content: &str) -> Option<String> {
    let lower = content.to_lowercase();
    for (kw, cat) in APP_KEYWORDS {
        if lower.contains(kw) {
            return Some((*cat).into());
        }
    }
    None
}

/// 分类桌面图标（需要文件系统：判断目录 / 读取 .desktop 内容）
pub fn classify_desktop_item(file_name: &str, path: &Path) -> String {
    let lower = file_name.to_lowercase();

    // 我的电脑 / 回收站
    if let Some(cat) = classify_special_name(&lower) {
        return cat.into();
    }

    // .desktop 启动器 -> 应用（命中关键词则细分）
    if file_name.ends_with(".desktop") {
        if let Some(cat) = std::fs::read_to_string(path)
            .ok()
            .and_then(|c| classify_launcher_content(&c))
        {
            return cat;
        }
        return APP_CATEGORY.into();
    }

    // 目录
    if path.is_dir() {
        if let Some(cat) = classify_dir_name(&lower) {
            return cat.into();
        }
        return FOLDER_CATEGORY.into();
    }

    // 文件按扩展名
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    classify_file_by_ext(&ext).into()
}

/// 空文件夹名（整理时跳过空组）
pub fn is_empty_folder(name: &str) -> bool {
    name == "文件夹" || name == "其他"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_names_by_keyword() {
        assert_eq!(classify_special_name("computer"), Some("我的电脑"));
        assert_eq!(classify_special_name("mycomputer"), Some("我的电脑"));
        assert_eq!(classify_special_name("电脑"), Some("我的电脑"));
        assert_eq!(classify_special_name("trash-can"), Some("回收站"));
        assert_eq!(classify_special_name("recycle.bin"), Some("回收站"));
        assert_eq!(classify_special_name("firefox"), None);
        assert_eq!(classify_special_name(""), None);
    }

    #[test]
    fn special_names_are_case_insensitive() {
        assert_eq!(classify_special_name("TRASH"), Some("回收站"));
        assert_eq!(classify_special_name("Computer.svg"), Some("我的电脑"));
    }

    #[test]
    fn dir_names_map_to_special_groups() {
        assert_eq!(classify_dir_name("图片收藏"), Some(IMAGE_VIDEO_CATEGORY));
        assert_eq!(classify_dir_name("照片2026"), Some(IMAGE_VIDEO_CATEGORY));
        assert_eq!(classify_dir_name("截图"), Some(IMAGE_VIDEO_CATEGORY));
        assert_eq!(classify_dir_name("my-photos"), Some(IMAGE_VIDEO_CATEGORY));
        assert_eq!(classify_dir_name("压缩备份"), Some(ARCHIVE_CATEGORY));
        assert_eq!(classify_dir_name("backup-2026"), Some(ARCHIVE_CATEGORY));
        assert_eq!(classify_dir_name("资料"), None);
        assert_eq!(classify_dir_name(""), None);
    }

    #[test]
    fn file_extensions_grouping() {
        // 图片视频
        for ext in [
            "jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico", "tif", "tiff", "mp4", "avi",
            "mkv", "mov", "mp3", "wav",
        ] {
            assert_eq!(classify_file_by_ext(ext), IMAGE_VIDEO_CATEGORY, "ext={ext}");
        }
        // 压缩包
        for ext in [
            "zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "tgz", "deb", "rpm",
        ] {
            assert_eq!(classify_file_by_ext(ext), ARCHIVE_CATEGORY, "ext={ext}");
        }
        // 普通文件
        assert_eq!(classify_file_by_ext("txt"), FILE_CATEGORY);
        assert_eq!(classify_file_by_ext("docx"), FILE_CATEGORY);
        assert_eq!(classify_file_by_ext(""), FILE_CATEGORY);
        assert_eq!(classify_file_by_ext("none"), FILE_CATEGORY);
    }

    #[test]
    fn launcher_content_keyword_match() {
        assert_eq!(
            classify_launcher_content("[Desktop Entry]\nIcon=firefox\n"),
            Some("浏览器".into())
        );
        assert_eq!(
            classify_launcher_content("[Desktop Entry]\nName=微信\n"),
            Some("微信".into())
        );
        assert_eq!(
            classify_launcher_content("[Desktop Entry]\nExec=notepad\n"),
            None
        );
        assert_eq!(classify_launcher_content(""), None);
    }

    #[test]
    fn empty_folder_names() {
        assert!(is_empty_folder("文件夹"));
        assert!(is_empty_folder("其他"));
        assert!(!is_empty_folder("应用"));
        assert!(!is_empty_folder(""));
    }

    #[test]
    fn classify_desktop_item_uses_fs_for_dir_and_file() {
        let dir = tempfile::tempdir().unwrap();

        // 目录
        let normal_dir = dir.path().join("项目资料");
        std::fs::create_dir_all(&normal_dir).unwrap();
        assert_eq!(
            classify_desktop_item("项目资料", &normal_dir),
            FOLDER_CATEGORY
        );

        let photo_dir = dir.path().join("旅行照片");
        std::fs::create_dir_all(&photo_dir).unwrap();
        assert_eq!(
            classify_desktop_item("旅行照片", &photo_dir),
            IMAGE_VIDEO_CATEGORY
        );

        // 普通文件按扩展名
        let txt = dir.path().join("notes.txt");
        std::fs::write(&txt, "hi").unwrap();
        assert_eq!(classify_desktop_item("notes.txt", &txt), FILE_CATEGORY);

        let archive = dir.path().join("release.zip");
        std::fs::write(&archive, "x").unwrap();
        assert_eq!(
            classify_desktop_item("release.zip", &archive),
            ARCHIVE_CATEGORY
        );

        // .desktop 命中关键词
        let app = dir.path().join("firefox.desktop");
        std::fs::write(&app, "[Desktop Entry]\nIcon=firefox\n").unwrap();
        assert_eq!(
            classify_desktop_item("firefox.desktop", &app),
            "浏览器".into()
        );

        // .desktop 未命中 -> 应用
        let plain = dir.path().join("tool.desktop");
        std::fs::write(&plain, "[Desktop Entry]\nExec=tool\n").unwrap();
        assert_eq!(classify_desktop_item("tool.desktop", &plain), "应用".into());
    }
}
