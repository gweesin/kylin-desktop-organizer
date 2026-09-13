//! 核心领域层：纯数据 / 纯算法 / 可注入 IO，不依赖任何 GTK 与平台 API，全部可单元测试。
//!
//! 依赖方向：本层不依赖 `infra` / `ui` / `window`。

pub mod categorize;
pub mod config;
pub mod config_store;
pub mod layout;
pub mod organize;
