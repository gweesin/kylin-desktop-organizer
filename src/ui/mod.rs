//! GTK 控件层：图标 / 盒子 / 菜单 / 设置面板 / 主题与渲染。
//! 依赖 `core` / `infra`；不包含应用编排逻辑（编排在 `window`）。

pub mod box_view;
pub mod desktop_icon;
pub mod icon_surface;
pub mod menu;
pub mod settings;
pub mod theme;
