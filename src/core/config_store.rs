//! 配置持久化：原子写入 `config.json`（默认 `~/.config/kylin-desktop-organizer/`）
//!
//! 重构说明：原 `model.rs` 用全局函数 + 固定路径，无法测试。
//! 这里抽象出 `ConfigStore`，目录可注入（测试可指向临时目录），
//! 并提供 `from_env()` 覆盖默认行为，保持对外调用方式不变。

use crate::core::config::Config;
use std::fs;
use std::path::{Path, PathBuf};

/// 应用标识（目录名 / 进程名 / 自启文件名）
pub const APP_ID: &str = "kylin-desktop-organizer";

/// 可注入配置目录的持久化存储
#[derive(Clone, Debug)]
pub struct ConfigStore {
    dir: PathBuf,
}

impl ConfigStore {
    /// 指定目录构造（主要用于测试注入临时目录）
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    /// 默认目录：`~/.config/kylin-desktop-organizer`
    pub fn from_env() -> Self {
        let dir = dirs::config_dir()
            .map(|d| d.join(APP_ID))
            .unwrap_or_else(|| PathBuf::from(".").join(".kylin-desktop-organizer"));
        Self { dir }
    }

    pub fn config_dir(&self) -> &Path {
        &self.dir
    }

    pub fn config_path(&self) -> PathBuf {
        self.dir.join("config.json")
    }

    /// 单实例 PID 文件路径
    pub fn pid_path(&self) -> PathBuf {
        self.dir.join("app.pid")
    }

    /// 读取配置：文件不存在或损坏时回退默认值
    pub fn load(&self) -> Config {
        match fs::read_to_string(self.config_path()) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    /// 原子保存配置：先写临时文件再改名，避免写一半损坏
    pub fn save(&self, cfg: &Config) {
        if let Err(e) = fs::create_dir_all(&self.dir) {
            log::warn!("无法创建配置目录: {e}");
            return;
        }
        match serde_json::to_string_pretty(cfg) {
            Ok(s) => {
                let tmp = self.config_dir().join("config.json.tmp");
                if fs::write(&tmp, s).is_ok() {
                    let _ = fs::rename(&tmp, self.config_path());
                }
            }
            Err(e) => log::warn!("配置序列化失败: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_returns_default() {
        let dir = tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_path_buf());
        assert_eq!(store.load(), Config::default());
    }

    #[test]
    fn save_then_load_roundtrip() {
        let dir = tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_path_buf());
        let mut cfg = Config::default();
        cfg.first_run = false;
        cfg.theme = "dark".into();
        cfg.box_seq = 42;
        store.save(&cfg);

        assert!(store.config_path().exists());
        let loaded = store.load();
        assert!(!loaded.first_run);
        assert_eq!(loaded.theme, "dark");
        assert_eq!(loaded.box_seq, 42);
    }

    #[test]
    fn corrupt_json_falls_back_to_default() {
        let dir = tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_path_buf());
        std::fs::write(store.config_path(), "{ not json !!!").unwrap();
        assert_eq!(store.load(), Config::default());
    }

    #[test]
    fn save_creates_missing_directory() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b");
        let store = ConfigStore::new(nested.clone());
        store.save(&Config::default());
        assert!(nested.join("config.json").exists());
    }

    #[test]
    fn pid_path_is_under_config_dir() {
        let dir = tempdir().unwrap();
        let store = ConfigStore::new(dir.path().to_path_buf());
        assert_eq!(store.pid_path(), dir.path().join("app.pid"));
    }
}
