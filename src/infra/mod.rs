//! 平台能力层：进程、自启、文件/应用操作。依赖 `core`，不依赖 UI。

pub mod autostart;
pub mod launch;
pub mod single_instance;
