//! 主题与样式：颜色 / 字号 / 盒子风格（原 `theme.rs` 迁移）

use gtk::gdk::RGBA;

#[derive(Clone, Copy)]
pub struct Palette {
    pub is_dark: bool,
    pub text: RGBA,
    pub label_shadow: bool,
    pub box_bg: (f64, f64, f64),
    pub box_border: (f64, f64, f64),
    pub box_header: (f64, f64, f64),
    pub accent: (f64, f64, f64),
    pub sel_bg: (f64, f64, f64),
}

impl Palette {
    pub fn light() -> Self {
        Palette {
            is_dark: false,
            text: RGBA::new(0.10, 0.10, 0.12, 1.0),
            label_shadow: false,
            box_bg: (1.0, 1.0, 1.0),
            box_border: (0.78, 0.82, 0.88),
            box_header: (0.96, 0.97, 0.99),
            accent: (0.11, 0.53, 0.95),
            sel_bg: (0.22, 0.55, 0.95),
        }
    }
    pub fn dark() -> Self {
        Palette {
            is_dark: true,
            text: RGBA::new(0.93, 0.94, 0.96, 1.0),
            label_shadow: true,
            box_bg: (0.16, 0.17, 0.20),
            box_border: (0.30, 0.32, 0.36),
            box_header: (0.20, 0.21, 0.24),
            accent: (0.30, 0.62, 0.97),
            sel_bg: (0.27, 0.60, 0.98),
        }
    }
    pub fn get(theme: &str) -> Self {
        match crate::core::config::Theme::parse(theme) {
            crate::core::config::Theme::Dark => Self::dark(),
            crate::core::config::Theme::Light => Self::light(),
        }
    }
}
