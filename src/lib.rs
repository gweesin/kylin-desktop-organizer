//! 桌面整理应用（麒麟桌面助手）库入口
//!
//! 依赖方向（自上而下单向）：
//! - `window`（应用编排：窗口 + 事件 + 把方案应用到控件）
//! - `ui`（GTK 控件层：图标/盒子/菜单/设置/渲染）→ 依赖 `core` / `infra`
//! - `infra`（平台能力：进程/自启/文件操作）→ 依赖 `core`
//! - `core`（纯领域逻辑：数据模型/分类/布局/整理算法，零 GTK 依赖，可独立测试）

pub mod core;
pub mod infra;
pub mod ui;
pub mod window;
