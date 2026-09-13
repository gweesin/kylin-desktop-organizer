//! 整理盒控件：结构体与控件构建（原 `window.rs` 中 `BoxView` 部分）
//!
//! 重构说明：把 `BoxView` 从 1500 行的 `window.rs` 中拆出，控件在这里构建，
//! 事件接线（拖拽/按钮/右键）由 `window.rs` 负责，保持「控件 vs 交互」分离。

use crate::core::config::BoxState;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct BoxView {
    pub id: String,
    pub title: String,
    pub event: gtk::EventBox,
    pub header: gtk::EventBox,
    pub title_label: gtk::Label,
    pub content: gtk::Fixed,
    pub collapse_btn: gtk::Button,
    pub close_btn: gtk::Button,
    pub color_dot: gtk::DrawingArea,
    pub fixed: gtk::Fixed,
    pub collapsed: bool,
    pub minimized: bool,
    pub drag_start: (f64, f64),
    pub icons: Vec<String>,
    pub hover_close: bool,
    pub hover_btn: bool,
    pub target_w: i32,
    pub target_h: i32,
    pub resize_from: Option<(i32, i32)>,
    pub pos: (i32, i32),
}

impl BoxView {
    /// 按 `BoxState` 构建控件树（不含事件接线）
    pub fn build(bs: &BoxState, fixed: gtk::Fixed) -> Rc<RefCell<Self>> {
        let event = gtk::EventBox::new();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        event.add(&vbox);

        // 标题栏
        let header = gtk::EventBox::new();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        hbox.set_margin_start(10);
        hbox.set_margin_end(6);
        hbox.set_margin_top(4);
        hbox.set_margin_bottom(4);
        header.add(&hbox);

        let color_dot = gtk::DrawingArea::new();
        color_dot.set_size_request(10, 10);

        let title_label = gtk::Label::new(Some(&bs.title));
        title_label.set_xalign(0.0);
        title_label.set_selectable(false);

        // 折叠按钮
        let collapse_btn = gtk::Button::new_with_label("─");
        collapse_btn.set_relief(gtk::ReliefStyle::None);
        collapse_btn.set_focus_on_click(false);
        collapse_btn.set_size_request(22, 22);
        // 关闭按钮
        let close_btn = gtk::Button::new_with_label("✕");
        close_btn.set_relief(gtk::ReliefStyle::None);
        close_btn.set_focus_on_click(false);
        close_btn.set_size_request(22, 22);

        hbox.pack_start(&color_dot, false, false, 0);
        hbox.pack_start(&title_label, true, true, 0);
        hbox.pack_start(&collapse_btn, false, false, 0);
        hbox.pack_start(&close_btn, false, false, 0);

        let content = gtk::Fixed::new();

        vbox.pack_start(&header, false, false, 0);
        vbox.pack_start(&content, true, true, 0);

        event.add(&vbox);
        event.set_above_child(true);
        event.set_visible_window(true);
        event.set_app_paintable(true);

        Rc::new(RefCell::new(BoxView {
            id: bs.id.clone(),
            title: bs.title.clone(),
            event: event.clone(),
            header: header.clone(),
            title_label: title_label.clone(),
            content: content.clone(),
            collapse_btn: collapse_btn.clone(),
            close_btn: close_btn.clone(),
            color_dot: color_dot.clone(),
            fixed: fixed.clone(),
            collapsed: bs.collapsed,
            minimized: false,
            drag_start: (0.0, 0.0),
            icons: bs.icons.iter().map(|i| i.path.clone()).collect(),
            hover_close: false,
            hover_btn: false,
            target_w: bs.w,
            target_h: bs.h,
            resize_from: None,
            pos: (bs.x, bs.y),
        }))
    }

    pub fn size(&self) -> (i32, i32) {
        let alloc = self.event.allocation();
        (alloc.width(), alloc.height())
    }
}
