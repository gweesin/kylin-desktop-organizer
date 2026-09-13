//! 一键整理端到端集成测试：在临时目录构造"桌面"，走完
//! 扫描文件 → 分类 → 分组排序 → 布局规划 的完整链路（纯 core 逻辑，不依赖 GTK）。

use kylin_desktop_organizer::core::categorize::classify_desktop_item;
use kylin_desktop_organizer::core::organize::{build_groups, plan_organize};
use std::path::Path;
use tempfile::tempdir;

/// 与 window.rs 中一致的分类入口（文件名 + 路径）
fn classify(path: &str) -> String {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    classify_desktop_item(name, Path::new(path))
}

/// 在临时目录构造模拟桌面，返回全部条目路径
fn build_fake_desktop() -> (tempfile::TempDir, Vec<String>) {
    let dir = tempdir().unwrap();
    let mut paths = Vec::new();

    let app = dir.path().join("firefox.desktop");
    std::fs::write(&app, "[Desktop Entry]\nIcon=firefox\n").unwrap();
    paths.push(app.to_string_lossy().into_owned());

    let notes = dir.path().join("notes.txt");
    std::fs::write(&notes, "hi").unwrap();
    paths.push(notes.to_string_lossy().into_owned());

    let photo = dir.path().join("holiday.jpg");
    std::fs::write(&photo, "img").unwrap();
    paths.push(photo.to_string_lossy().into_owned());

    let archive = dir.path().join("backup-2026.zip");
    std::fs::write(&archive, "zip").unwrap();
    paths.push(archive.to_string_lossy().into_owned());

    let docs = dir.path().join("项目资料");
    std::fs::create_dir_all(&docs).unwrap();
    paths.push(docs.to_string_lossy().into_owned());

    // 我的电脑 / 回收站 特殊项
    let computer = dir.path().join("computer");
    std::fs::create_dir_all(&computer).unwrap();
    paths.push(computer.to_string_lossy().into_owned());

    let trash = dir.path().join("回收站");
    std::fs::create_dir_all(&trash).unwrap();
    paths.push(trash.to_string_lossy().into_owned());

    (dir, paths)
}

#[test]
fn full_pipeline_groups_and_orders_correctly() {
    let (_dir, paths) = build_fake_desktop();

    // 分组（show_files=true）
    let groups = build_groups(&paths, true, classify);
    let titles: Vec<&str> = groups.iter().map(|(t, _)| t.as_str()).collect();
    // 顺序符合 CATEGORY_ORDER：我的电脑 > 回收站 > 应用 > 文件夹 > 文件 > 图片视频 > 压缩包
    assert_eq!(
        titles,
        vec![
            "我的电脑",
            "回收站",
            "应用",
            "文件夹",
            "文件",
            "图片视频",
            "压缩包"
        ]
    );

    // 每个组包含正确的图标
    let app_items = &groups.iter().find(|(t, _)| t == "应用").unwrap().1;
    assert!(app_items.iter().any(|p| p.ends_with("firefox.desktop")));
    let file_items = &groups.iter().find(|(t, _)| t == "文件").unwrap().1;
    assert!(file_items.iter().any(|p| p.ends_with("notes.txt")));
    let img_items = &groups.iter().find(|(t, _)| t == "图片视频").unwrap().1;
    assert!(img_items.iter().any(|p| p.ends_with("holiday.jpg")));
    let arc_items = &groups.iter().find(|(t, _)| t == "压缩包").unwrap().1;
    assert!(arc_items.iter().any(|p| p.ends_with("backup-2026.zip")));

    // 布局规划：每个组一个盒子，图标不丢失
    let plan = plan_organize(&groups, 1920, 48, 10, 34, 8);
    assert_eq!(plan.boxes.len(), groups.len());
    let total_icons: usize = plan.boxes.iter().map(|b| b.icons.len()).sum();
    assert_eq!(total_icons, paths.len());
    // 坐标为正且不越界
    for b in &plan.boxes {
        assert!(b.x >= 20 && b.y >= 16);
        assert!(b.w > 0 && b.h > 0);
    }
}

#[test]
fn show_files_false_filters_to_apps_only() {
    let (_dir, paths) = build_fake_desktop();
    let groups = build_groups(&paths, false, classify);
    let titles: Vec<&str> = groups.iter().map(|(t, _)| t.as_str()).collect();
    // 只保留 我的电脑 / 回收站 / 应用
    assert_eq!(titles, vec!["我的电脑", "回收站", "应用"]);
    let total: usize = groups.iter().map(|(_, v)| v.len()).sum();
    assert_eq!(total, 3);
}

#[test]
fn empty_desktop_yields_empty_plan() {
    let groups = build_groups(&[], true, classify);
    assert!(groups.is_empty());
    let plan = plan_organize(&groups, 1920, 48, 10, 34, 8);
    assert!(plan.boxes.is_empty());
}

#[test]
fn narrow_screen_wraps_boxes() {
    let (_dir, paths) = build_fake_desktop();
    let groups = build_groups(&paths, true, classify);
    // 极窄桌面：每个盒子只能占一行
    let plan = plan_organize(&groups, 320, 48, 10, 34, 8);
    let mut prev_y = i32::MIN;
    for b in &plan.boxes {
        assert!(b.y >= prev_y, "盒子应按行排列且不重叠");
        prev_y = b.y;
    }
    // 至少有一个盒子换行（7 个组不可能都挤进 320px）
    let ys: std::collections::BTreeSet<i32> = plan.boxes.iter().map(|b| b.y).collect();
    assert!(ys.len() >= 2, "极窄桌面应换行: ys={ys:?}");
}

#[test]
fn classify_handles_mixed_case_and_special_names() {
    let dir = tempdir().unwrap();
    let computer = dir.path().join("Computer");
    std::fs::create_dir_all(&computer).unwrap();
    let p = computer.to_string_lossy().into_owned();
    let cat = classify(&p);
    assert_eq!(cat, "我的电脑");
}
