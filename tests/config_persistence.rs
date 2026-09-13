//! 配置持久化集成测试：完整生命周期（保存 → 重载）、旧配置字段迁移、损坏配置兜底。

use kylin_desktop_organizer::core::config::{BoxState, Config, IconState};
use kylin_desktop_organizer::core::config_store::ConfigStore;
use tempfile::tempdir;

fn make_config() -> Config {
    Config {
        first_run: false,
        icon_size: 56,
        grid_gap: 14,
        double_click_organize: false,
        auto_organize: true,
        show_files: false,
        show_desktop_icons: false,
        theme: "dark".into(),
        box_style: "solid".into(),
        box_opacity: 0.6,
        desktop_icons: vec![IconState {
            path: "/home/t/Desktop/a.desktop".into(),
            x: 10,
            y: 20,
        }],
        boxes: vec![BoxState {
            id: "box-1".into(),
            title: "应用".into(),
            x: 5,
            y: 6,
            w: 300,
            h: 200,
            collapsed: true,
            color: None,
            icons: vec![IconState {
                path: "/home/t/Desktop/firefox.desktop".into(),
                x: 1,
                y: 2,
            }],
        }],
        box_seq: 9,
        hidden: vec!["secret".into()],
    }
}

#[test]
fn full_lifecycle_save_and_reload() {
    let dir = tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    assert_eq!(store.load(), Config::default());

    store.save(&make_config());

    let loaded = store.load();
    assert!(!loaded.first_run);
    assert_eq!(loaded.icon_size, 56);
    assert_eq!(loaded.grid_gap, 14);
    assert!(!loaded.double_click_organize);
    assert!(loaded.auto_organize);
    assert!(!loaded.show_files);
    assert!(!loaded.show_desktop_icons);
    assert_eq!(loaded.theme, "dark");
    assert_eq!(loaded.box_style, "solid");
    assert_eq!(loaded.box_opacity, 0.6);
    assert_eq!(loaded.box_seq, 9);
    assert_eq!(loaded.hidden, vec!["secret".to_string()]);

    assert_eq!(loaded.desktop_icons.len(), 1);
    assert_eq!(loaded.desktop_icons[0].path, "/home/t/Desktop/a.desktop");
    assert_eq!(
        (loaded.desktop_icons[0].x, loaded.desktop_icons[0].y),
        (10, 20)
    );

    assert_eq!(loaded.boxes.len(), 1);
    let b = &loaded.boxes[0];
    assert_eq!(b.id, "box-1");
    assert_eq!(b.title, "应用");
    assert!(b.collapsed);
    assert_eq!(b.icons.len(), 1);
    assert_eq!(b.icons[0].path, "/home/t/Desktop/firefox.desktop");
}

#[test]
fn legacy_config_migrates_with_field_defaults() {
    let dir = tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    // 模拟旧版本配置：只含部分字段（缺少 hidden / box_seq / show_desktop_icons 等）
    let legacy = r#"{
        "first_run": false,
        "icon_size": 40,
        "grid_gap": 12,
        "double_click_organize": true,
        "auto_organize": false,
        "show_files": true,
        "theme": "light",
        "box_style": "plain",
        "box_opacity": 0.8,
        "desktop_icons": [],
        "boxes": []
    }"#;
    std::fs::write(store.config_path(), legacy).unwrap();

    let c = store.load();
    // 已有字段被保留
    assert_eq!(c.icon_size, 40);
    assert_eq!(c.grid_gap, 12);
    assert_eq!(c.box_style, "plain");
    // 缺失字段用默认值补齐，而不是整体回退默认
    assert_eq!(c.box_seq, 0);
    assert!(c.show_desktop_icons);
    assert!(c.hidden.is_empty());
}

#[test]
fn corrupt_config_falls_back_to_default() {
    let dir = tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());
    std::fs::write(store.config_path(), "{ not valid json").unwrap();
    assert_eq!(store.load(), Config::default());
}

#[test]
fn save_overwrites_previous_config() {
    let dir = tempdir().unwrap();
    let store = ConfigStore::new(dir.path().to_path_buf());

    let mut c = Config::default();
    c.box_seq = 1;
    store.save(&c);

    c.box_seq = 2;
    c.theme = "dark".into();
    store.save(&c);

    let loaded = store.load();
    assert_eq!(loaded.box_seq, 2);
    assert_eq!(loaded.theme, "dark");
}
