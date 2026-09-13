//! 桌面整理应用入口：单实例保护 → GTK 初始化 → 加载配置 → 启动主窗口

use kylin_desktop_organizer::core::config_store::ConfigStore;
use kylin_desktop_organizer::infra::single_instance;
use kylin_desktop_organizer::window::DesktopWindow;

fn main() {
    env_logger::init();

    // 单实例：写入当前 PID，若已有其他实例运行则退出
    single_instance::write_pid();
    if single_instance::other_instance_running() {
        eprintln!("桌面整理已在运行");
        return;
    }

    if gtk::init().is_err() {
        eprintln!("GTK 初始化失败");
        return;
    }

    let store = ConfigStore::from_env();
    let cfg = store.load();
    let _desktop = DesktopWindow::new(cfg, store);

    gtk::main();
}
