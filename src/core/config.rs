//! 配置数据模型（纯数据，无任何平台依赖，可独立单元测试）
//!
//! 重构说明：原 `model.rs` 把「数据模型 + 文件 IO + 单实例 PID + 自启」混在一个文件里，
//! 此处只保留纯数据模型，并补充类型化枚举 `Theme` / `BoxStyle`，
//! 所有字段带 `#[serde(default)]`，旧版配置文件缺字段时按字段补齐而不是整体回退默认。

use serde::{Deserialize, Serialize};

/// 桌面图标状态（路径 + 桌面坐标）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct IconState {
    pub path: String,
    pub x: i32,
    pub y: i32,
}

/// 整理盒状态（几何 + 包含的图标）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

/// 界面主题
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Light,
    Dark,
}

impl Theme {
    pub fn parse(s: &str) -> Self {
        match s {
            "dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

/// 盒子样式
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoxStyle {
    Card,
    Solid,
    Plain,
}

impl BoxStyle {
    pub fn parse(s: &str) -> Self {
        match s {
            "solid" => BoxStyle::Solid,
            "plain" => BoxStyle::Plain,
            _ => BoxStyle::Card,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BoxStyle::Card => "card",
            BoxStyle::Solid => "solid",
            BoxStyle::Plain => "plain",
        }
    }
}

/// 全局配置（对应 `~/.config/kylin-desktop-organizer/config.json`）
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_true")]
    pub first_run: bool,
    #[serde(default = "default_icon_size")]
    pub icon_size: i32,
    #[serde(default = "default_grid_gap")]
    pub grid_gap: i32,
    #[serde(default = "default_true")]
    pub double_click_organize: bool,
    #[serde(default)]
    pub auto_organize: bool,
    #[serde(default = "default_true")]
    pub show_files: bool,
    #[serde(default = "default_true")]
    pub show_desktop_icons: bool,
    #[serde(default)]
    pub theme: String, // "light" | "dark"
    #[serde(default)]
    pub box_style: String, // "card" | "solid" | "plain"
    #[serde(default = "default_opacity")]
    pub box_opacity: f64,
    #[serde(default)]
    pub desktop_icons: Vec<IconState>,
    #[serde(default)]
    pub boxes: Vec<BoxState>,
    #[serde(default)]
    pub box_seq: u64,
    #[serde(default)]
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

impl Config {
    /// 类型化主题
    pub fn theme(&self) -> Theme {
        Theme::parse(&self.theme)
    }

    /// 类型化盒子样式
    pub fn box_style(&self) -> BoxStyle {
        BoxStyle::parse(&self.box_style)
    }
}

fn default_true() -> bool {
    true
}
fn default_icon_size() -> i32 {
    48
}
fn default_grid_gap() -> i32 {
    10
}
fn default_opacity() -> f64 {
    0.88
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sane_values() {
        let c = Config::default();
        assert!(c.first_run);
        assert_eq!(c.icon_size, 48);
        assert_eq!(c.grid_gap, 10);
        assert!(c.double_click_organize);
        assert!(c.show_files);
        assert!(c.show_desktop_icons);
        assert_eq!(c.box_opacity, 0.88);
        assert!(c.desktop_icons.is_empty());
        assert!(c.boxes.is_empty());
        assert_eq!(c.box_seq, 0);
    }

    #[test]
    fn serde_round_trip_preserves_all_fields() {
        let c = Config {
            first_run: false,
            icon_size: 56,
            grid_gap: 12,
            double_click_organize: false,
            auto_organize: true,
            show_files: false,
            show_desktop_icons: false,
            theme: "dark".into(),
            box_style: "plain".into(),
            box_opacity: 0.5,
            desktop_icons: vec![IconState {
                path: "/home/u/Desktop/a.txt".into(),
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
                collapsed: false,
                color: Some("#ff0000".into()),
                icons: vec![IconState {
                    path: "/home/u/Desktop/firefox.desktop".into(),
                    x: 1,
                    y: 2,
                }],
            }],
            box_seq: 7,
            hidden: vec!["secret".into()],
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back.first_run, c.first_run);
        assert_eq!(back.icon_size, c.icon_size);
        assert_eq!(back.grid_gap, c.grid_gap);
        assert_eq!(back.double_click_organize, c.double_click_organize);
        assert_eq!(back.auto_organize, c.auto_organize);
        assert_eq!(back.show_files, c.show_files);
        assert_eq!(back.show_desktop_icons, c.show_desktop_icons);
        assert_eq!(back.theme, "dark");
        assert_eq!(back.box_style, "plain");
        assert_eq!(back.box_opacity, c.box_opacity);
        assert_eq!(back.desktop_icons, c.desktop_icons);
        assert_eq!(back.boxes, c.boxes);
        assert_eq!(back.box_seq, 7);
        assert_eq!(back.hidden, c.hidden);
    }

    #[test]
    fn missing_fields_get_defaults_instead_of_failing() {
        // 只写部分字段，模拟旧版本配置升级
        let json = r#"{"icon_size": 40, "theme": "dark"}"#;
        let c: Config = serde_json::from_str(json).unwrap();
        assert_eq!(c.icon_size, 40);
        assert_eq!(c.theme, "dark");
        // 未提供的字段应取默认值，而不是整个解析失败
        assert!(c.first_run);
        assert_eq!(c.box_seq, 0);
        assert!(c.boxes.is_empty());
    }

    #[test]
    fn typed_theme_and_box_style() {
        assert_eq!(Theme::parse("dark"), Theme::Dark);
        assert_eq!(Theme::parse("light"), Theme::Light);
        assert_eq!(Theme::parse("unknown"), Theme::Light);
        assert_eq!(Theme::parse("").as_str(), "light");
        assert_eq!(Theme::Dark.as_str(), "dark");

        assert_eq!(BoxStyle::parse("solid"), BoxStyle::Solid);
        assert_eq!(BoxStyle::parse("plain"), BoxStyle::Plain);
        assert_eq!(BoxStyle::parse("card"), BoxStyle::Card);
        assert_eq!(BoxStyle::parse("whatever"), BoxStyle::Card);
        assert_eq!(BoxStyle::Solid.as_str(), "solid");

        let c = Config::default();
        assert_eq!(c.theme(), Theme::Light);
        assert_eq!(c.box_style(), BoxStyle::Card);
    }
}
