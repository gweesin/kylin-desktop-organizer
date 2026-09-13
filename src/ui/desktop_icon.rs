//! 桌面图标控件：GTK 容器（图标 surface + 文字标签），支持拖拽 / 选中 / 双击
//! （原 `icon.rs` 迁移）

use crate::core::layout::icon_total;
use crate::ui::icon_surface;
use gtk::prelude::*;
use gtk::{gdk, Fixed, Label};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IconKind {
    Desktop,
    Box,
}

pub struct DesktopIcon {
    pub path: String,
    pub is_dir: bool,
    pub widget: gtk::EventBox,
    pub label: Label,
    pub surface: Rc<RefCell<cairo::ImageSurface>>,
    pub kind: IconKind,
    pub selected: Rc<RefCell<bool>>,
}

impl DesktopIcon {
    pub fn new(path: String, is_dir: bool, kind: IconKind, icon_size: i32) -> Rc<Self> {
        let eb = gtk::EventBox::new();
        eb.set_above_child(false);

        let total = icon_total(icon_size);

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let da = gtk::DrawingArea::new();
        da.set_size_request(total, total);
        let label = Label::new(Some(short_name(&path).as_str()));
        label.set_max_width_chars(9);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_line_wrap(true);
        label.set_justify(gtk::Justification::Center);
        label.set_selectable(false);

        let surface = Rc::new(RefCell::new(icon_surface::build_icon_surface(
            &path, icon_size, is_dir,
        )));

        {
            let surface = surface.clone();
            da.connect_draw(move |_w, cr| {
                let s = surface.borrow();
                cr.set_source_surface(&*s, 0.0, 0.0).unwrap();
                cr.paint().unwrap();
                glib::Propagation::Proceed
            });
        }

        let selected = Rc::new(RefCell::new(false));

        // 高亮层：选中时在 drawing area 上叠加半透明高亮（重绘实现）
        {
            let selected = selected.clone();
            da.connect_draw(move |_w, cr| {
                if *selected.borrow() {
                    cr.set_source_rgba(0.25, 0.53, 0.96, 0.28);
                    cr.paint().unwrap();
                }
                glib::Propagation::Proceed
            });
        }

        box_.pack_start(&da, false, false, 0);
        box_.pack_start(&label, false, false, 0);

        eb.add(&box_);
        eb.set_size_request(total + 4, total + 4 + 30);

        // 文字颜色随主题
        apply_label_theme(&label);

        let this = Rc::new(DesktopIcon {
            path,
            is_dir,
            widget: eb,
            label,
            surface,
            kind,
            selected,
        });

        // 重绘选中态
        {
            let this = this.clone();
            this.selected.replace(false);
        }

        this
    }

    pub fn set_selected(&self, sel: bool) {
        self.selected.replace(sel);
        if let Some(da) = find_da(&self.widget) {
            da.queue_draw();
        }
        // 选中态用主题色标签
        if sel {
            self.label.override_color(
                gtk::StateFlags::NORMAL,
                Some(&gdk::RGBA::new(0.16, 0.42, 0.83, 1.0)),
            );
        } else {
            self.label.override_color(gtk::StateFlags::NORMAL, None);
            apply_label_theme(&self.label);
        }
    }

    pub fn is_selected(&self) -> bool {
        *self.selected.borrow()
    }

    /// 位置（用于布局）
    pub fn position(&self, fixed: &Fixed) -> (i32, i32) {
        let x = fixed.child_x(&self.widget);
        let y = fixed.child_y(&self.widget);
        (x, y)
    }
}

fn find_da(widget: &gtk::Widget) -> Option<gtk::DrawingArea> {
    if let Ok(da) = widget.clone().downcast::<gtk::DrawingArea>() {
        return Some(da);
    }
    widget.children().iter().find_map(|c| find_da(c.as_ref()))
}

pub fn short_name(path: &str) -> String {
    let base = path.rsplit('/').next().unwrap_or(path);
    if let Some(stripped) = base.strip_suffix(".desktop") {
        // 去掉中文快捷方式常见的 "desktop" 尾巴保持简洁；显示原名更符合腾讯风格
        return stripped.to_string();
    }
    base.to_string()
}

pub fn apply_label_theme(label: &Label) {
    // 默认浅色环境使用深色文字；深色主题由 theme.rs 统一调整
    let _ = label;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_name_strips_desktop_suffix() {
        assert_eq!(short_name("/home/u/Desktop/firefox.desktop"), "firefox");
        assert_eq!(short_name("/home/u/Desktop/notes.txt"), "notes.txt");
        assert_eq!(short_name("/a/b"), "b");
        assert_eq!(short_name(""), "");
    }
}
