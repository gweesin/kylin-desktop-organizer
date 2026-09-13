//! 桌面整理助手入口（麒麟 V10 / GTK3）

mod categorize;
mod icon;
mod icons_util;
mod launch;
mod model;
mod settings;
mod theme;
mod window;

fn main() {
    env_logger::init();
    model::write_pid();

    // 单实例：已有实例则唤醒后退出
    if model::other_instance_running() {
        eprintln!("桌面整理已在运行");
        return;
    }

    // 初始化 GTK（麒麟默认 X11）
    if gtk::init().is_err() {
        eprintln!("GTK 初始化失败，请确认已安装 GTK3 图形环境");
        std::process::exit(1);
    }

    let cfg = model::load_config();
    let _desktop = window::DesktopWindow::new(cfg);

    gtk::main();
}
