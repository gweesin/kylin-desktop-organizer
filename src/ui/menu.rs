//! 右键菜单构建：桌面 / 图标 / 盒子（原 `window.rs` 菜单部分）
//!
//! 重构说明：把三个菜单构建函数从 `window.rs` 拆出，
//! 以 `&Rc<DesktopWindow>` 作为入参，解除对 `DesktopWindow` 私有方法的依赖。

use crate::infra::launch;
use crate::ui::box_view::BoxView;
use crate::window::DesktopWindow;
use gtk::gdk;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// 桌面空白处右键菜单
pub fn build_desktop_menu(win: &Rc<DesktopWindow>) -> gtk::Menu {
    let menu = gtk::Menu::new();
    let this = win.clone();

    let it = gtk::MenuItem::with_label("一键整理");
    {
        let this = this.clone();
        it.connect_activate(move |_| this.organize_icons(true));
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("新建盒子");
    {
        let this = this.clone();
        it.connect_activate(move |_| this.new_box("新建盒子"));
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("扫描桌面");
    {
        let this = this.clone();
        it.connect_activate(move |_| {
            let n = this.scan_desktop(false);
            this.notify(&format!("已添加 {n} 个桌面图标"));
        });
    }
    menu.append(&it);

    menu.append(&gtk::SeparatorMenuItem::new());

    let it = gtk::MenuItem::with_label("设置");
    {
        let this = this.clone();
        it.connect_activate(move |_| this.open_settings());
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("退出");
    {
        let this = this.clone();
        it.connect_activate(move |_| {
            this.persist();
            gtk::main_quit();
        });
    }
    menu.append(&it);

    menu.show_all();
    menu
}

/// 图标右键菜单
pub fn build_icon_menu(win: &Rc<DesktopWindow>, path: &str) -> gtk::Menu {
    let menu = gtk::Menu::new();
    let this = win.clone();
    let path = path.to_string();

    let it = gtk::MenuItem::with_label("打开");
    {
        let p = path.clone();
        it.connect_activate(move |_| launch::open_path(&p));
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("在文件管理器中显示");
    {
        let p = path.clone();
        it.connect_activate(move |_| launch::show_in_file_manager(&p));
    }
    menu.append(&it);

    menu.append(&gtk::SeparatorMenuItem::new());

    // 移入盒子子菜单
    let sub = gtk::Menu::new();
    let box_ids: Vec<String> = this.boxes.borrow().keys().cloned().collect();
    if box_ids.is_empty() {
        let it = gtk::MenuItem::with_label("(暂无盒子)");
        it.set_sensitive(false);
        sub.append(&it);
    } else {
        for id in &box_ids {
            let it =
                gtk::MenuItem::with_label(&this.boxes.borrow().get(id).unwrap().borrow().title);
            let bid = id.clone();
            let p = path.clone();
            it.connect_activate(move |_| this.add_icon_to_box(&bid, &p));
            sub.append(&it);
        }
    }
    let mi = gtk::MenuItem::with_label("移入盒子");
    mi.set_submenu(Some(&sub));
    menu.append(&mi);

    let it = gtk::MenuItem::with_label("重命名");
    {
        let this = this.clone();
        let p = path.clone();
        it.connect_activate(move |_| this.rename_icon(&p));
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("复制");
    {
        let p = path.clone();
        it.connect_activate(move |_| this.copy_icon(&p));
    }
    menu.append(&it);

    menu.append(&gtk::SeparatorMenuItem::new());

    let it = gtk::MenuItem::with_label("删除");
    {
        let p = path.clone();
        it.connect_activate(move |_| this.delete_icon(&p));
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("属性");
    {
        let p = path.clone();
        it.connect_activate(move |_| this.icon_properties(&p));
    }
    menu.append(&it);

    menu.show_all();
    menu
}

/// 盒子标题栏右键菜单
pub fn build_box_menu(win: &Rc<DesktopWindow>, bv: &Rc<RefCell<BoxView>>) -> gtk::Menu {
    let menu = gtk::Menu::new();
    let this = win.clone();

    let it = gtk::MenuItem::with_label("重命名盒子");
    {
        let bv = bv.clone();
        it.connect_activate(move |_| this.rename_box(&bv));
    }
    menu.append(&it);

    let it = gtk::MenuItem::with_label("添加盒子");
    {
        let this = this.clone();
        it.connect_activate(move |_| this.new_box("新建盒子"));
    }
    menu.append(&it);

    menu.append(&gtk::SeparatorMenuItem::new());

    let it = gtk::MenuItem::with_label("移除盒子");
    {
        let bv = bv.clone();
        it.connect_activate(move |_| this.close_box(&bv));
    }
    menu.append(&it);

    menu.show_all();
    menu
}

/// 在指针位置弹出菜单
pub fn popup(menu: &gtk::Menu, ev: &gdk::EventButton) {
    menu.popup_at_pointer(Some(ev));
}
