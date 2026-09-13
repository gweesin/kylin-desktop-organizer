//! 设置面板：开机自启 / 主题 / 盒子样式 / 图标大小 / 双击整理 / 显示桌面图标
//! （原 `settings.rs` 迁移；持久化统一走 `DesktopWindow::persist()`）

use crate::infra::autostart;
use crate::window::DesktopWindow;
use gtk::prelude::*;
use gtk::{Dialog, DialogFlags, ResponseType};
use std::rc::Rc;

pub struct SettingsDlg {
    pub dlg: Dialog,
}

impl SettingsDlg {
    pub fn new(win: &Rc<DesktopWindow>) -> SettingsDlg {
        let dlg = Dialog::with_buttons(
            Some("设置 - 桌面整理"),
            Some(&win.win),
            DialogFlags::MODAL,
            &[("关闭", ResponseType::Close)],
        );
        dlg.set_default_size(420, 380);

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 8);
        box_.set_margin_start(18);
        box_.set_margin_end(18);
        box_.set_margin_top(14);
        box_.set_margin_bottom(14);
        dlg.content_area().add(&box_);

        // 开机自启
        let chk_autostart = gtk::CheckButton::with_label("开机自动启动");
        {
            let enabled = autostart::autostart_path().exists();
            chk_autostart.set_active(enabled);
            let chk = chk_autostart.clone();
            chk.connect_toggled(move |c| {
                autostart::set_autostart(c.is_active());
            });
        }
        box_.pack_start(&chk_autostart, false, false, 0);

        // 主题
        let theme_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let theme_label = gtk::Label::new(Some("界面主题"));
        let theme_combo = gtk::ComboBoxText::new();
        theme_combo.append_text("浅色");
        theme_combo.append_text("深色");
        {
            let c = win.cfg.borrow();
            theme_combo.set_active(if c.theme() == crate::core::config::Theme::Dark {
                1
            } else {
                0
            });
        }
        {
            let win = win.clone();
            theme_combo.connect_changed(move |cb| {
                let is_dark = cb.active() == Some(1);
                win.cfg.borrow_mut().theme = if is_dark {
                    "dark".into()
                } else {
                    "light".into()
                };
                win.pal
                    .replace(crate::ui::theme::Palette::get(&win.cfg.borrow().theme));
                // 更新所有盒子外观
                let boxes: Vec<_> = win.boxes.borrow().values().cloned().collect();
                for b in boxes {
                    b.borrow().event.queue_draw();
                }
                win.persist();
            });
        }
        theme_box.pack_start(&theme_label, false, false, 0);
        theme_box.pack_start(&theme_combo, false, false, 0);
        box_.pack_start(&theme_box, false, false, 0);

        // 盒子样式
        let style_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let style_label = gtk::Label::new(Some("盒子样式"));
        let style_combo = gtk::ComboBoxText::new();
        style_combo.append_text("卡片（圆角半透明）");
        style_combo.append_text("实心");
        style_combo.append_text("简洁");
        {
            let c = win.cfg.borrow();
            style_combo.set_active(match c.box_style().as_str() {
                "solid" => 1,
                "plain" => 2,
                _ => 0,
            });
        }
        {
            let win = win.clone();
            style_combo.connect_changed(move |cb| {
                let s = match cb.active() {
                    Some(1) => "solid",
                    Some(2) => "plain",
                    _ => "card",
                };
                win.cfg.borrow_mut().box_style = s.into();
                let boxes: Vec<_> = win.boxes.borrow().values().cloned().collect();
                for b in boxes {
                    b.borrow().event.queue_draw();
                }
                win.persist();
            });
        }
        style_box.pack_start(&style_label, false, false, 0);
        style_box.pack_start(&style_combo, false, false, 0);
        box_.pack_start(&style_box, false, false, 0);

        // 图标大小
        let size_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let size_label = gtk::Label::new(Some("图标大小"));
        let size_combo = gtk::ComboBoxText::new();
        size_combo.append_text("小 (40)");
        size_combo.append_text("中 (48)");
        size_combo.append_text("大 (56)");
        {
            let c = win.cfg.borrow();
            size_combo.set_active(match c.icon_size {
                40 => 0,
                56 => 2,
                _ => 1,
            });
        }
        {
            let win = win.clone();
            size_combo.connect_changed(move |cb| {
                let s = match cb.active() {
                    Some(0) => 40,
                    Some(2) => 56,
                    _ => 48,
                };
                win.cfg.borrow_mut().icon_size = s;
                // 重建所有图标（尺寸变化需要重建 surface）
                win.rebuild_icons();
                win.persist();
            });
        }
        size_box.pack_start(&size_label, false, false, 0);
        size_box.pack_start(&size_combo, false, false, 0);
        box_.pack_start(&size_box, false, false, 0);

        // 双击空白整理
        let chk_dbl = gtk::CheckButton::with_label("双击桌面空白处快速整理");
        {
            let a = win.cfg.borrow().double_click_organize;
            chk_dbl.set_active(a);
            let win = win.clone();
            let chk = chk_dbl.clone();
            chk.connect_toggled(move |c| {
                win.cfg.borrow_mut().double_click_organize = c.is_active();
                win.persist();
            });
        }
        box_.pack_start(&chk_dbl, false, false, 0);

        // 显示桌面图标
        let chk_icons = gtk::CheckButton::with_label("在桌面上显示图标");
        {
            let a = win.cfg.borrow().show_desktop_icons;
            chk_icons.set_active(a);
            let win = win.clone();
            let chk = chk_icons.clone();
            chk.connect_toggled(move |c| {
                win.cfg.borrow_mut().show_desktop_icons = c.is_active();
                win.apply_show_desktop_icons();
                win.persist();
            });
        }
        box_.pack_start(&chk_icons, false, false, 0);

        // 显示普通文件
        let chk_files = gtk::CheckButton::with_label("显示文件类图标（图片/文档等）");
        {
            let a = win.cfg.borrow().show_files;
            chk_files.set_active(a);
            let win = win.clone();
            let chk = chk_files.clone();
            chk.connect_toggled(move |c| {
                win.cfg.borrow_mut().show_files = c.is_active();
                win.persist();
            });
        }
        box_.pack_start(&chk_files, false, false, 0);

        dlg.show_all();
        SettingsDlg { dlg }
    }

    pub fn run(&self) {
        self.dlg.run();
        self.dlg.close();
    }
}
