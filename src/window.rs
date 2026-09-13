//! 桌面层主窗口：透明 ARGB 桌面窗口 + 图标 + 盒子 + 拖拽 + 动画 + 右键菜单

use crate::categorize;
use crate::icon::{short_name, DesktopIcon, IconKind};
use crate::icons_util;
use crate::launch;
use crate::model::{BoxState, Config, IconState};
use crate::settings::SettingsDlg;
use crate::theme::Palette;
use gtk::prelude::*;
use gtk::{gdk, Fixed, Inhibit, Window, WindowType};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
pub const BOX_PADDING: i32 = 8;
pub const BOX_TITLE_H: i32 = 34;
pub const BOX_MIN_W: i32 = 96;
pub const BOX_MIN_H: i32 = 96;
const FLY_MS: u64 = 380;
const SNAP: i32 = 6;
pub const DOUBLE_CLICK_MS: u64 = 320;

pub struct DesktopWindow {
    pub win: Window,
    pub fixed: Fixed,
    pub cfg: Rc<RefCell<Config>>,
    pub pal: Rc<RefCell<Palette>>,
    pub icons: Rc<RefCell<HashMap<String, Rc<DesktopIcon>>>>,
    pub boxes: Rc<RefCell<HashMap<String, Rc<RefCell<BoxView>>>>>,
    pub desktop_rect: (i32, i32, i32, i32),
    pub sel: Rc<RefCell<Vec<String>>>,
    pub draggable: Rc<RefCell<bool>>,
    pub press_pos: Rc<RefCell<(i32, i32)>>,
    pub press_icon: Rc<RefCell<Option<String>>>,
    pub last_click: Rc<RefCell<(u64, String)>>,
    pub organized: Rc<RefCell<bool>>,
    pub dragging: Rc<RefCell<Option<String>>>,
}

pub struct BoxView {
    pub id: String,
    pub title: String,
    pub event: gtk::EventBox,
    pub header: gtk::EventBox,
    pub title_label: gtk::Label,
    pub content: Fixed,
    pub collapse_btn: gtk::Button,
    pub close_btn: gtk::Button,
    pub color_dot: gtk::DrawingArea,
    pub fixed: Fixed,
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
    pub fn size(&self) -> (i32, i32) {
        let alloc = self.event.allocation();
        (alloc.width(), alloc.height())
    }
}

impl DesktopWindow {
    pub fn new(cfg: Config) -> Rc<Self> {
        let win = Window::new(WindowType::Toplevel);
        win.set_title("桌面整理");
        win.set_type_hint(gdk::WindowTypeHint::Desktop);
        win.set_app_paintable(true);
        win.set_decorated(false);
        win.set_keep_below(true);
        win.set_skip_taskbar_hint(true);
        win.set_skip_pager_hint(true);
        win.set_accept_focus(false);
        win.set_resizable(false);
        win.set_default_size(1, 1);

        let screen = win.screen();
        let visual = screen.rgba_visual().unwrap();
        win.set_visual(Some(&visual));

        let wa = screen.width();
        let ha = screen.height();
        let desktop_rect = (0, 0, wa, ha);

        let fixed = Fixed::new();
        win.add(&fixed);

        {
            win.connect_draw(move |_w, cr| {
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                cr.set_operator(cairo::Operator::Source);
                cr.paint().unwrap();
                glib::Propagation::Proceed
            });
        }

        let pal = Rc::new(RefCell::new(Palette::get(&cfg.theme)));
        let cfg = Rc::new(RefCell::new(cfg));
        let icons = Rc::new(RefCell::new(HashMap::new()));
        let boxes = Rc::new(RefCell::new(HashMap::new()));
        let sel = Rc::new(RefCell::new(vec![]));
        let draggable = Rc::new(RefCell::new(true));
        let press_pos = Rc::new(RefCell::new((0, 0)));
        let press_icon = Rc::new(RefCell::new(None));
        let last_click = Rc::new(RefCell::new((0, "".to_string())));
        let organized = Rc::new(RefCell::new(false));
        let dragging = Rc::new(RefCell::new(None));

        let this = Rc::new(DesktopWindow {
            win: win.clone(),
            fixed: fixed.clone(),
            cfg: cfg.clone(),
            pal: pal.clone(),
            icons,
            boxes,
            desktop_rect,
            sel,
            draggable,
            press_pos,
            press_icon,
            last_click,
            organized,
            dragging,
        });

        // 桌面空白右键菜单
        let menu = this.build_desktop_menu();
        {
            let this = this.clone();
            let menu = menu.clone();
            win.connect_button_press_event(move |_w, ev| {
                if ev.button() == 3 {
                    this.desktop_menu_popup(&menu, ev);
                    return Inhibit(true);
                }
                Inhibit(false)
            });
        }
        // 点击空白：取消选中；双击空白：快速整理
        {
            let this = this.clone();
            let blank_click = Rc::new(RefCell::new((0u64, 0i32, 0i32)));
            let blank_click2 = blank_click.clone();
            win.connect_button_press_event(move |_w, ev| {
                if ev.button() == 1 {
                    this.clear_selection();
                    let now = (glib::monotonic_time() / 1000) as u64;
                    let last = *blank_click.borrow();
                    let (x, y) = (ev.x() as i32, ev.y() as i32);
                    if last.0 != 0
                        && now - last.0 < DOUBLE_CLICK_MS
                        && (x - last.1).abs() < 8
                        && (y - last.2).abs() < 8
                    {
                        this.handle_blank_double_click(ev);
                        blank_click.borrow_mut().0 = 0;
                        return Inhibit(true);
                    }
                    blank_click.borrow_mut().0 = now;
                    blank_click.borrow_mut().1 = x;
                    blank_click.borrow_mut().2 = y;
                }
                Inhibit(false)
            });
        }

        this.restore(&cfg.borrow());

        if cfg.borrow().first_run {
            let n = this.scan_desktop(false);
            if n > 0 {
                this.organize_icons(true);
            }
            cfg.borrow_mut().first_run = false;
            crate::model::save_config(&cfg.borrow());
        }

        win.show_all();
        win.r#move(0, 0);
        win.resize(wa, ha);

        this
    }

    // ---------------- 恢复 ----------------

    fn restore(&self, cfg: &Config) {
        for bs in &cfg.boxes {
            self.create_box(bs);
        }
        for st in &cfg.desktop_icons {
            let size = self.cfg.borrow().icon_size;
            if let Some(ic) = self.make_icon(&st.path, IconKind::Desktop, size) {
                self.add_icon(&ic, st.x, st.y);
                self.icons.borrow_mut().insert(st.path.clone(), ic);
            }
        }
    }

    // ---------------- 扫描桌面 ----------------

    pub fn scan_desktop(&self, only_missing: bool) -> usize {
        let home = dirs::desktop_dir().unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join("Desktop"))
                .unwrap_or_else(|| Path::new("/root/Desktop").to_path_buf())
        });
        let Ok(rd) = std::fs::read_dir(&home) else {
            return 0;
        };
        let gap = self.cfg.borrow().grid_gap;
        let size = self.cfg.borrow().icon_size;
        let total = icons_util::icon_total(size);
        let cell = total + gap;

        let mut row = 0;
        let mut col = 0;
        let max_per_row = ((self.desktop_rect.2 - 80) / cell).max(1) as i32;

        let entries: Vec<_> = rd.flatten().collect();
        let mut new_items: Vec<(String, bool)> = vec![];
        for entry in entries {
            let p = entry.path();
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            if name.starts_with('.') {
                continue;
            }
            let path_str = p.to_string_lossy().into_owned();
            let is_dir = p.is_dir();
            if self.icons.borrow().contains_key(&path_str) || self.is_in_box(&path_str) {
                continue;
            }
            new_items.push((path_str, is_dir));
        }
        new_items.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let mut added = 0;
        for (path, _is_dir) in new_items {
            let x = 20 + col * cell;
            let y = 16 + row * (total + 30);
            if let Some(ic) = self.make_icon(&path, IconKind::Desktop, size) {
                self.add_icon(&ic, x, y);
                self.icons.borrow_mut().insert(path.clone(), ic);
                added += 1;
            }
            col += 1;
            if col >= max_per_row {
                col = 0;
                row += 1;
            }
        }
        if added > 0 {
            self.persist();
        }
        added
    }

    pub fn is_in_box(&self, path: &str) -> bool {
        self.boxes
            .borrow()
            .values()
            .any(|b| b.borrow().icons.contains(&path.to_string()))
    }

    // ---------------- 图标 ----------------

    pub fn make_icon(&self, path: &str, kind: IconKind, size: i32) -> Option<Rc<DesktopIcon>> {
        let p = Path::new(path);
        if !p.exists() {
            return None;
        }
        let is_dir = p.is_dir();
        let ic = DesktopIcon::new(path.to_string(), is_dir, kind, size);

        {
            let this = self.clone();
            let ic = ic.clone();
            let path = path.to_string();
            ic.widget.connect_button_press_event(move |_w, ev| {
                if ev.button() == 1 {
                    if ev.event_type() == gdk::EventType::DoubleButtonPress {
                        this.double_click(&path, &ic);
                        return Inhibit(true);
                    }
                    let now = glib::monotonic_time() / 1000;
                    let last = this.last_click.borrow().clone();
                    if last.0 != 0 && now - last.0 < DOUBLE_CLICK_MS && last.1 == path {
                        return Inhibit(true);
                    }
                    this.select_icon(&path);
                    this.press_pos.replace((ev.x() as i32, ev.y() as i32));
                    this.press_icon.replace(Some(path.clone()));
                    this.last_click.replace((now, path));
                }
                Inhibit(false)
            });
        }
        {
            let this = self.clone();
            let ic = ic.clone();
            let path = path.to_string();
            ic.widget.connect_button_press_event(move |w, ev| {
                if ev.button() == 3 {
                    this.select_icon(&path);
                    let menu = this.build_icon_menu(&path, &ic);
                    menu.popup_at_pointer(Some(ev));
                    return Inhibit(true);
                }
                Inhibit(false)
            });
        }
        {
            let this = self.clone();
            let ic = ic.clone();
            let path = path.to_string();
            ic.widget.connect_motion_notify_event(move |_w, ev| {
                this.icon_motion(&path, ev);
                Inhibit(false)
            });
        }
        {
            let this = self.clone();
            let ic = ic.clone();
            let path = path.to_string();
            ic.widget.connect_button_release_event(move |_w, ev| {
                if ev.button() == 1 {
                    this.icon_release(&path, &ic);
                }
                Inhibit(false)
            });
        }

        Some(ic)
    }

    pub fn add_icon(&self, ic: &Rc<DesktopIcon>, x: i32, y: i32) {
        self.fixed.put(ic.widget.clone(), x, y);
        let w = icons_util::icon_total(self.cfg.borrow().icon_size) + 4;
        self.fixed
            .child_set_property(ic.widget.clone(), "width-request", &w);
    }

    // ---------------- 选择 / 拖拽 ----------------

    pub fn clear_selection(&self) {
        let sels: Vec<String> = self.sel.borrow().clone();
        for p in &sels {
            if let Some(ic) = self.icons.borrow().get(p) {
                ic.set_selected(false);
            }
        }
        self.sel.borrow_mut().clear();
    }

    pub fn select_icon(&self, path: &str) {
        if !self.sel.borrow().iter().any(|s| s == path) {
            self.clear_selection();
            if let Some(ic) = self.icons.borrow().get(path) {
                ic.set_selected(true);
                self.sel.borrow_mut().push(path.to_string());
            }
        }
    }

    fn icon_motion(&self, path: &str, ev: &gdk::EventMotion) {
        if !*self.draggable.borrow() {
            return;
        }
        if self.press_icon.borrow().as_deref() != Some(path) {
            return;
        }
        let start = *self.press_pos.borrow();
        let dx = (ev.x() as i32 - start.0).abs();
        let dy = (ev.y() as i32 - start.1).abs();
        if dx + dy < 4 {
            return;
        }
        let ic = self.icons.borrow().get(path).cloned();
        let Some(ic) = ic else { return };

        if let Some(bid) = self.box_containing(&ic) {
            self.remove_icon_from_box(&bid, path);
            let (cx, cy) = ic.position(&self.fixed);
            self.fixed.move_(ic.widget.clone(), cx, cy);
        }

        let (cx, cy) = ic.position(&self.fixed);
        let nx = (cx + ev.x() as i32 - start.0).max(0);
        let ny = (cy + ev.y() as i32 - start.1).max(0);
        self.fixed.move_(ic.widget.clone(), nx, ny);
        self.press_pos.replace((ev.x() as i32, ev.y() as i32));
        *self.dragging.borrow_mut() = Some(path.to_string());
        self.persist();
    }

    fn icon_release(&self, path: &str, ic: &Rc<DesktopIcon>) {
        if self.dragging.borrow().as_deref() == Some(path) {
            self.snap_icon(ic);
            self.dragging.borrow_mut().take();
            self.persist();
        }
        self.press_icon.replace(None);
    }

    /// 图标吸附网格（水平对齐）
    pub fn snap_icon(&self, ic: &Rc<DesktopIcon>) {
        let gap = self.cfg.borrow().grid_gap;
        let total = icons_util::icon_total(self.cfg.borrow().icon_size);
        let cell = total + gap;
        let (x, y) = ic.position(&self.fixed);
        let nx = ((x as f64 / cell as f64).round() as i32) * cell;
        let ny = y;
        if (nx - x).abs() < SNAP + gap / 2 {
            self.fixed.move_(ic.widget.clone(), nx, ny);
        }
    }

    fn box_containing(&self, ic: &Rc<DesktopIcon>) -> Option<String> {
        let (x, y) = ic.position(&self.fixed);
        let total = icons_util::icon_total(self.cfg.borrow().icon_size);
        let cx = x + total / 2;
        let cy = y + total / 2;
        for (id, b) in self.boxes.borrow().iter() {
            let b = b.borrow();
            if b.minimized || b.collapsed {
                continue;
            }
            let (bx, by) = b.pos;
            let (bw, bh) = b.size();
            if cx >= bx && cx <= bx + bw && cy >= by && cy <= by + bh {
                return Some(id.clone());
            }
        }
        None
    }

    /// 把图标加入盒子（自动重排）
    pub fn add_icon_to_box(&self, box_id: &str, path: &str) {
        let Some(ic) = self.icons.borrow().get(path).cloned() else {
            return;
        };
        let mut box_ = self.boxes.borrow().get(box_id).cloned();
        let Some(bv) = box_ else { return };
        // 若在别的盒子里，先移除
        let other: Vec<String> = self
            .boxes
            .borrow()
            .iter()
            .filter(|(id, _)| id.as_str() != box_id)
            .filter(|(_, b)| b.borrow().icons.contains(&path.to_string()))
            .map(|(id, _)| id.clone())
            .collect();
        for oid in other {
            self.remove_icon_from_box(&oid, path);
        }
        bv.borrow_mut().icons.push(path.to_string());
        // 若图标在桌面层，从桌面层移除
        if let Some(ic) = self.icons.borrow().get(path).cloned() {
            self.fixed.remove(&ic.widget.clone());
        }
        self.reflow_box(&bv);
        self.persist();
    }

    // ---------------- 盒子创建 ----------------

    pub fn create_box(&self, bs: &BoxState) {
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

        let content = Fixed::new();

        vbox.pack_start(&header, false, false, 0);
        vbox.pack_start(&content, true, true, 0);

        event.add(&vbox);
        event.set_above_child(true);
        event.set_visible_window(true);
        event.set_app_paintable(true);

        let bv = Rc::new(RefCell::new(BoxView {
            id: bs.id.clone(),
            title: bs.title.clone(),
            event: event.clone(),
            header: header.clone(),
            title_label: title_label.clone(),
            content: content.clone(),
            collapse_btn: collapse_btn.clone(),
            close_btn: close_btn.clone(),
            color_dot: color_dot.clone(),
            fixed: self.fixed.clone(),
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
        }));

        // 绘制盒子外观
        {
            let pal = self.pal.clone();
            let opacity = self.cfg.borrow().box_opacity;
            event.connect_draw(move |w, cr| {
                let alloc = w.allocation();
                let (w_, h_) = (alloc.width() as f64, alloc.height() as f64);
                let p = pal.borrow();
                // 半透明背景
                let (r, g, b) = p.box_bg;
                cr.set_source_rgba(r, g, b, opacity);
                icons_util::round_rect(cr, 1.0, 1.0, w_ - 2.0, h_ - 2.0, 10.0);
                cr.fill();
                // 边框
                let (br, bg, bb) = p.box_border;
                cr.set_source_rgba(br, bg, bb, 0.9);
                icons_util::round_rect(cr, 1.0, 1.0, w_ - 2.0, h_ - 2.0, 10.0);
                cr.set_line_width(1.2);
                cr.stroke();
                glib::Propagation::Proceed
            });
        }

        // 标题栏拖拽
        {
            let this = self.clone();
            let bv = bv.clone();
            header.connect_button_press_event(move |_w, ev| {
                if ev.button() == 1 {
                    bv.borrow_mut().drag_start = (ev.x(), ev.y());
                }
                Inhibit(false)
            });
            header.connect_motion_notify_event(move |_w, ev| {
                if ev.state().contains(gdk::ModifierType::BUTTON1_MASK) {
                    let start = bv.borrow().drag_start;
                    let (x, y) = bv.borrow().pos;
                    let nx = x + ev.x() as i32 - start.0 as i32;
                    let ny = y + ev.y() as i32 - start.1 as i32;
                    bv.borrow_mut().pos = (nx.max(0), ny.max(0));
                    this.move_box(&bv, nx.max(0), ny.max(0));
                }
                Inhibit(false)
            });
        }

        // 关闭按钮
        {
            let this = self.clone();
            let bv = bv.clone();
            close_btn.connect_clicked(move |_b| {
                this.close_box(&bv);
            });
        }
        // 折叠按钮
        {
            let this = self.clone();
            let bv = bv.clone();
            collapse_btn.connect_clicked(move |_b| {
                this.toggle_collapse(&bv);
            });
        }

        // 标题栏右键菜单
        {
            let this = self.clone();
            let bv = bv.clone();
            header.connect_button_press_event(move |_w, ev| {
                if ev.button() == 3 {
                    let menu = this.build_box_menu(&bv);
                    menu.popup_at_pointer(Some(ev));
                    return Inhibit(true);
                }
                Inhibit(false)
            });
        }

        // 加入容器
        self.fixed.put(event.clone(), bs.x, bs.y);
        let (tw, th) = (bs.w.max(BOX_MIN_W), bs.h.max(BOX_MIN_H));
        event.set_size_request(tw, th);

        // 恢复盒子内图标
        for st in &bs.icons {
            let size = self.cfg.borrow().icon_size;
            if let Some(ic) = self.make_icon(&st.path, IconKind::Box, size) {
                let (ix, iy) = (st.x, st.y);
                bv.borrow().content.put(ic.widget.clone(), ix, iy);
                let w = icons_util::icon_total(size) + 4;
                bv.borrow()
                    .content
                    .child_set_property(ic.widget.clone(), "width-request", &w);
                // 记录到桌面层索引（供拖拽/打开复用）
                self.icons.borrow_mut().insert(st.path.clone(), ic);
            }
        }

        // 若折叠，应用折叠高度
        if bs.collapsed {
            let h = BOX_TITLE_H + 4;
            event.set_size_request(tw, h);
        }

        self.boxes.borrow_mut().insert(bs.id.clone(), bv);
    }

    pub fn move_box(&self, bv: &Rc<RefCell<BoxView>>, x: i32, y: i32) {
        self.fixed.move_(bv.borrow().event.clone(), x, y);
        bv.borrow_mut().pos = (x, y);
        self.persist();
    }

    /// 移除盒子（含其中图标回到桌面层）
    pub fn close_box(&self, bv: &Rc<RefCell<BoxView>>) {
        let id = bv.borrow().id.clone();
        let paths: Vec<String> = bv.borrow().icons.clone();
        for p in &paths {
            self.remove_icon_from_box(&id, p);
            if let Some(ic) = self.icons.borrow().get(p).cloned() {
                let (x, y) = ic.position(&self.fixed);
                self.fixed.move_(ic.widget.clone(), x, y);
            }
        }
        self.fixed.remove(&bv.borrow().event.clone());
        self.boxes.borrow_mut().remove(&id);
        self.persist();
    }

    pub fn toggle_collapse(&self, bv: &Rc<RefCell<BoxView>>) {
        let mut b = bv.borrow_mut();
        b.collapsed = !b.collapsed;
        let (w, _) = b.size();
        if b.collapsed {
            b.event.set_size_request(w, BOX_TITLE_H + 6);
            b.collapse_btn.set_label("▢");
        } else {
            b.event.set_size_request(w, b.target_h.max(BOX_MIN_H));
            b.collapse_btn.set_label("─");
        }
        drop(b);
        self.persist();
    }

    /// 图标从盒子移除（回到桌面层）
    pub fn remove_icon_from_box(&self, box_id: &str, path: &str) {
        let b = self.boxes.borrow().get(box_id).cloned();
        let Some(bv) = b else { return };
        {
            let mut bb = bv.borrow_mut();
            if let Some(pos) = bb.icons.iter().position(|p| p == path) {
                bb.icons.remove(pos);
                // 从 content 移除 widget
                if let Some(ic) = self.icons.borrow().get(path).cloned() {
                    bb.content.remove(&ic.widget.clone());
                }
            }
        }
        self.reflow_box(&bv);
        self.persist();
    }

    /// 重排盒子内图标（网格）
    pub fn reflow_box(&self, bv: &Rc<RefCell<BoxView>>) {
        let size = self.cfg.borrow().icon_size;
        let gap = self.cfg.borrow().grid_gap;
        let total = icons_util::icon_total(size);
        let cell = total + gap;
        let max_per_row = (((bv.borrow().size().0 - BOX_PADDING * 2) / cell).max(1)) as i32;

        let paths: Vec<String> = bv.borrow().icons.clone();
        let mut col = 0;
        let mut row = 0;
        for p in &paths {
            if let Some(ic) = self.icons.borrow().get(p).cloned() {
                let x = BOX_PADDING + col * cell;
                let y = BOX_PADDING + row * (total + 30);
                bv.borrow().content.move_(ic.widget.clone(), x, y);
            }
            col += 1;
            if col >= max_per_row {
                col = 0;
                row += 1;
            }
        }
        // 自适应高度
        let rows = if paths.is_empty() { 1 } else { row + 1 };
        let need_h = BOX_TITLE_H + BOX_PADDING * 2 + rows * (total + 30) + 4;
        if need_h > bv.borrow().target_h {
            bv.borrow_mut().target_h = need_h;
            if !bv.borrow().collapsed {
                let w = bv.borrow().size().0;
                bv.borrow().event.set_size_request(w, need_h);
            }
        }
    }

    // ---------------- 一键整理 ----------------

    pub fn organize_icons(&self, animate: bool) {
        let gap = self.cfg.borrow().grid_gap;
        let size = self.cfg.borrow().icon_size;
        let total = icons_util::icon_total(size);
        let cell = total + gap;

        // 清空现有盒子布局：所有图标先回桌面层（保留盒子本体）
        let box_ids: Vec<String> = self.boxes.borrow().keys().cloned().collect();
        let all_paths: Vec<String> = self.icons.borrow().keys().cloned().collect();
        for p in &all_paths {
            for bid in &box_ids {
                if self
                    .boxes
                    .borrow()
                    .get(bid)
                    .map(|b| b.borrow().icons.contains(p))
                    .unwrap_or(false)
                {
                    self.remove_icon_from_box(bid, p);
                }
            }
            if let Some(ic) = self.icons.borrow().get(p).cloned() {
                self.fixed.remove(&ic.widget.clone());
            }
        }

        // 重新扫描桌面，保证拿到最新文件
        self.scan_desktop(true);

        // 分组
        let show_files = self.cfg.borrow().show_files;
        let mut groups: HashMap<String, Vec<String>> = HashMap::new();
        let paths: Vec<String> = self.icons.borrow().keys().cloned().collect();
        for p in &paths {
            // 关闭"显示文件"时，文件/图片/压缩包类图标直接隐藏，不参与整理
            if !show_files {
                let cat = categorize::classify_desktop_item(
                    Path::new(p)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(""),
                    Path::new(p),
                );
                if matches!(cat.as_str(), "文件" | "图片视频" | "压缩包" | "文件夹") {
                    continue;
                }
            }
            let cat = categorize::classify_desktop_item(
                Path::new(p)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(""),
                Path::new(p),
            );
            groups.entry(cat).or_default().push(p.clone());
        }

        // 忽略空的"文件夹/其他"组
        groups.retain(|k, v| !(categorize::is_empty_folder(k) && v.is_empty()));
        // 排序组
        let mut order: Vec<(String, Vec<String>)> = groups.into_iter().collect();
        order.sort_by(|a, b| {
            let ai = categorize::CATEGORY_ORDER
                .iter()
                .position(|c| *c == a.0)
                .unwrap_or(usize::MAX);
            let bi = categorize::CATEGORY_ORDER
                .iter()
                .position(|c| *c == b.0)
                .unwrap_or(usize::MAX);
            ai.cmp(&bi)
        });

        let max_per_row = ((self.desktop_rect.2 - 80) / cell).max(2) as i32;
        let mut x = 20;
        let mut y = 16;

        let box_ids: Vec<String> = self.boxes.borrow().keys().cloned().collect();
        let mut used_box = 0;

        for (cat, items) in &order {
            if items.is_empty() {
                continue;
            }
            // 每类建一个盒子
            let n = items.len();
            let cols = n.min(max_per_row as usize);
            let rows = (n as f64 / max_per_row as f64).ceil().max(1.0) as i32;
            let bw = cols as i32 * cell + BOX_PADDING * 2;
            let bh = BOX_TITLE_H + BOX_PADDING * 2 + rows * (total + 30) + 4;

            let (bid, bv) = if used_box < box_ids.len() {
                // 复用旧盒子（按顺序）
                let id = box_ids[used_box].clone();
                let b = self.boxes.borrow().get(&id).cloned().unwrap();
                b.borrow_mut().title = cat.clone();
                b.borrow_mut().title_label.set_text(cat);
                b.borrow_mut().target_w = bw;
                b.borrow_mut().target_h = bh;
                b.borrow_mut().collapsed = false;
                b.borrow_mut().collapse_btn.set_label("─");
                b.borrow().event.set_size_request(bw, bh);
                (id, b)
            } else {
                // 新建盒子
                let seq = {
                    let mut c = self.cfg.borrow_mut();
                    c.box_seq += 1;
                    c.box_seq
                };
                let id = format!("auto-{seq}");
                let bs = BoxState {
                    id: id.clone(),
                    title: cat.clone(),
                    x,
                    y,
                    w: bw,
                    h: bh,
                    collapsed: false,
                    color: None,
                    icons: vec![],
                };
                self.create_box(&bs);
                let b = self.boxes.borrow().get(&id).cloned().unwrap();
                (id, b)
            };
            used_box += 1;

            // 移动盒子到目标位置
            self.fixed.move_(bv.borrow().event.clone(), x, y);
            bv.borrow_mut().pos = (x, y);

            // 把图标加入盒子
            let mut col = 0;
            let mut r = 0;
            for p in items {
                let ic = self.icons.borrow().get(p).cloned().unwrap();
                // 从桌面层移除，加入盒子
                self.fixed.remove(&ic.widget.clone());
                let ix = BOX_PADDING + col * cell;
                let iy = BOX_PADDING + r * (total + 30);
                bv.borrow().content.put(ic.widget.clone(), ix, iy);
                bv.borrow_mut().icons.push(p.clone());
                col += 1;
                if col >= max_per_row as usize {
                    col = 0;
                    r += 1;
                }
            }

            // 动画：盒子与图标飞入（简化：盒子位置动画）
            if animate {
                let target = (x, y);
                let start = (x - 60, y);
                self.fixed
                    .move_(bv.borrow().event.clone(), start.0, start.1);
                let bv2 = bv.clone();
                let this = self.clone();
                let ev = bv.borrow().event.clone();
                glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
                    this.animate_box(&bv2, &ev, start, target);
                    glib::ControlFlow::Break
                });
            }

            x += bw + 16;
            if x + bw > self.desktop_rect.2 - 40 {
                x = 20;
                y += bh + 20;
            }
        }

        // 多余的旧盒子删除
        while used_box < box_ids.len() {
            let id = box_ids[used_box].clone();
            if let Some(b) = self.boxes.borrow().get(&id).cloned() {
                self.close_box(&b);
            }
            used_box += 1;
        }

        *self.organized.borrow_mut() = true;
        self.persist();
    }

    fn is_special(&self, cat: &str) -> bool {
        cat == "我的电脑" || cat == "回收站"
    }

    fn animate_box(
        &self,
        bv: &Rc<RefCell<BoxView>>,
        ev: &gtk::EventBox,
        from: (i32, i32),
        to: (i32, i32),
    ) {
        // 简易两步动画
        let (fx, fy) = from;
        let (tx, ty) = to;
        let mx = fx + (tx - fx) * 2 / 3;
        let my = fy + (ty - fy) * 2 / 3;
        self.fixed.move_(ev.clone(), mx, my);
        bv.borrow_mut().pos = (mx, my);
        let ev2 = ev.clone();
        let bv2 = bv.clone();
        let this = self.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(FLY_MS / 2), move || {
            this.fixed.move_(ev2.clone(), tx, ty);
            bv2.borrow_mut().pos = (tx, ty);
            this.persist();
            glib::ControlFlow::Break
        });
    }

    // ---------------- 事件：双击 / 菜单 ----------------

    fn double_click(&self, path: &str, _ic: &Rc<DesktopIcon>) {
        if !*self.draggable.borrow() {
            return;
        }
        launch::open_path(path);
    }

    fn build_desktop_menu(&self) -> gtk::Menu {
        let menu = gtk::Menu::new();
        let this = self.clone();

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

    fn desktop_menu_popup(&self, menu: &gtk::Menu, ev: &gdk::EventButton) {
        menu.popup_at_pointer(Some(ev));
    }

    pub fn new_box(&self, title: &str) {
        let seq = {
            let mut c = self.cfg.borrow_mut();
            c.box_seq += 1;
            c.box_seq
        };
        let id = format!("box-{seq}");
        let (x, y) = (40, 40);
        let bs = BoxState {
            id: id.clone(),
            title: title.to_string(),
            x,
            y,
            w: 320,
            h: 240,
            collapsed: false,
            color: None,
            icons: vec![],
        };
        self.create_box(&bs);
        self.persist();
        let _ = id;
    }

    // ---------------- 菜单：图标 / 盒子 ----------------

    fn build_icon_menu(&self, path: &str, _ic: &Rc<DesktopIcon>) -> gtk::Menu {
        let menu = gtk::Menu::new();
        let this = self.clone();
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
        let box_ids: Vec<String> = self.boxes.borrow().keys().cloned().collect();
        if box_ids.is_empty() {
            let it = gtk::MenuItem::with_label("(暂无盒子)");
            it.set_sensitive(false);
            sub.append(&it);
        } else {
            for id in &box_ids {
                let it =
                    gtk::MenuItem::with_label(&self.boxes.borrow().get(id).unwrap().borrow().title);
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

    fn build_box_menu(&self, bv: &Rc<RefCell<BoxView>>) -> gtk::Menu {
        let menu = gtk::Menu::new();
        let this = self.clone();

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

    // ---------------- 图标操作 ----------------

    fn rename_icon(&self, path: &str) {
        let old = path.to_string();
        let dir = Path::new(&old).parent().map(|p| p.to_path_buf());
        let old_name = Path::new(&old)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let dlg = gtk::Dialog::with_buttons(
            Some("重命名"),
            Some(&self.win),
            gtk::DialogFlags::MODAL,
            &[
                ("取消", gtk::ResponseType::Cancel),
                ("确定", gtk::ResponseType::Ok),
            ],
        );
        let entry = gtk::Entry::new();
        entry.set_text(&old_name);
        dlg.content_area().pack_start(&entry, false, false, 0);
        dlg.show_all();
        let resp = dlg.run();
        let new_name = entry.text().to_string();
        dlg.close();
        if resp == gtk::ResponseType::Ok && !new_name.is_empty() && new_name != old_name {
            if let Some(dir) = dir {
                let new_path = dir.join(&new_name);
                if std::fs::rename(&old, &new_path).is_ok() {
                    self.update_icon_path(&old, &new_path.to_string_lossy());
                }
            }
        }
    }

    fn copy_icon(&self, path: &str) {
        let gio_f = gio::File::for_path(path);
        let dest = format!("{path}.copy");
        let dest_f = gio::File::for_path(&dest);
        let _ = gio_f.copy(
            &dest_f,
            gio::FileCopyFlags::NONE,
            gio::Cancellable::NONE,
            None::<fn(f64, f64)>,
        );
        self.scan_desktop(false);
    }

    fn delete_icon(&self, path: &str) {
        let dlg = gtk::MessageDialog::new(
            Some(&self.win),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Warning,
            gtk::ButtonsType::YesNo,
            &format!("确定删除「{}」？", short_name(path)),
        );
        let resp = dlg.run();
        dlg.close();
        if resp == gtk::ResponseType::Yes {
            if launch::delete_file(path) {
                self.remove_icon_widget(path);
                self.persist();
            }
        }
    }

    fn icon_properties(&self, path: &str) {
        let p = Path::new(path);
        let meta = p.metadata();
        let (kind, size, modified) = match &meta {
            Ok(m) => (
                if m.is_dir() { "文件夹" } else { "文件" },
                if m.is_dir() {
                    String::new()
                } else {
                    format!("{} 字节", m.len())
                },
                m.modified().map(|t| format!("{t:?}")).unwrap_or_default(),
            ),
            Err(e) => (e.to_string(), String::new(), String::new()),
        };
        let msg = format!(
            "名称：{}
路径：{}
类型：{}
大小：{}
修改时间：{}",
            short_name(path),
            path,
            kind,
            size,
            modified
        );
        let dlg = gtk::MessageDialog::new(
            Some(&self.win),
            gtk::DialogFlags::MODAL,
            gtk::MessageType::Info,
            gtk::ButtonsType::Ok,
            &msg,
        );
        dlg.run();
        dlg.close();
    }

    /// 图标路径变更后更新索引与持久化
    pub fn update_icon_path(&self, old: &str, new: &str) {
        if let Some(ic) = self.icons.borrow().get(old).cloned() {
            let mut ic = ic;
            self.icons.borrow_mut().remove(old);
            self.icons.borrow_mut().insert(new.to_string(), ic);
            // 更新控件标签
            if let Some(ic) = self.icons.borrow().get(new) {
                ic.label.set_text(&short_name(new));
            }
        }
        self.persist();
    }

    pub fn remove_icon_widget(&self, path: &str) {
        if let Some(ic) = self.icons.borrow().get(path).cloned() {
            self.fixed.remove(&ic.widget.clone());
            self.icons.borrow_mut().remove(path);
        }
        self.sel.borrow_mut().retain(|p| p != path);
    }

    // ---------------- 通知 ----------------

    pub fn notify(&self, msg: &str) {
        let label = gtk::Label::new(Some(msg));
        let pop = gtk::Window::new(gtk::WindowType::Popup);
        pop.set_app_paintable(true);
        let screen = pop.screen();
        if let Some(v) = screen.rgba_visual() {
            pop.set_visual(Some(&v));
        }
        let f = gtk::Fixed::new();
        pop.add(&f);
        f.put(&label, 0, 0);
        pop.r#move(self.desktop_rect.2 / 2 - 150, 60);
        pop.set_default_size(300, 40);
        pop.show_all();
        let pop2 = pop.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(1800), move || {
            pop2.close();
            glib::ControlFlow::Break
        });
    }

    // ---------------- 设置 ----------------

    pub fn open_settings(&self) {
        let dlg = SettingsDlg::new(self);
        dlg.run();
    }

    /// 图标大小改变后重建所有图标（surface 尺寸跟随图标大小）
    pub fn rebuild_icons(&self) {
        let size = self.cfg.borrow().icon_size;
        let paths: Vec<String> = self.icons.borrow().keys().cloned().collect();
        let mut positions: HashMap<String, (i32, i32)> = HashMap::new();

        // 收集当前位置
        for p in &paths {
            if let Some(ic) = self.icons.borrow().get(p) {
                if let Some(b) = self.box_containing(ic) {
                    let bv = self.boxes.borrow().get(&b).cloned().unwrap();
                    let (x, y) = ic.position(&bv.borrow().content);
                    positions.insert(p.clone(), (x, y));
                    // 从盒子移除 widget
                    bv.borrow().content.remove(&ic.widget.clone());
                } else {
                    let (x, y) = ic.position(&self.fixed);
                    positions.insert(p.clone(), (x, y));
                    self.fixed.remove(&ic.widget.clone());
                }
            }
        }

        // 重建
        let old: Vec<String> = self.icons.borrow().keys().cloned().collect();
        for p in &old {
            self.icons.borrow_mut().remove(p);
        }
        let mut restored: Vec<(String, i32, i32, IconKind)> = vec![];
        for (p, (x, y)) in &positions {
            let kind = if self.is_in_box(p) {
                IconKind::Box
            } else {
                IconKind::Desktop
            };
            if let Some(ic) = self.make_icon(p, kind, size) {
                restored.push((p.clone(), *x, *y, kind));
            }
        }
        for (p, x, y, kind) in restored {
            if let Some(ic) = self.icons.borrow().get(&p).cloned() {
                if kind == IconKind::Box {
                    // 放回盒子（需找到所属盒子）
                    if let Some(bid) = self.box_of_icon(&p) {
                        let bv = self.boxes.borrow().get(&bid).cloned().unwrap();
                        bv.borrow().content.put(ic.widget.clone(), x, y);
                        bv.borrow().content.child_set_property(
                            ic.widget.clone(),
                            "width-request",
                            &(icons_util::icon_total(size) + 4),
                        );
                    } else {
                        self.add_icon(&ic, x, y);
                    }
                } else {
                    self.add_icon(&ic, x, y);
                }
            }
        }
    }

    /// 图标所属盒子
    fn box_of_icon(&self, path: &str) -> Option<String> {
        self.boxes
            .borrow()
            .iter()
            .find(|(_, b)| b.borrow().icons.contains(&path.to_string()))
            .map(|(id, _)| id.clone())
    }

    /// 应用"显示桌面图标"开关：隐藏/显示盒子外的图标
    pub fn apply_show_desktop_icons(&self) {
        let show = self.cfg.borrow().show_desktop_icons;
        let paths: Vec<String> = self.icons.borrow().keys().cloned().collect();
        for p in &paths {
            if self.is_in_box(&p) {
                continue;
            }
            if let Some(ic) = self.icons.borrow().get(&p).cloned() {
                if show {
                    if ic.widget.parent().is_none() {
                        let (x, y) = ic.position(&self.fixed);
                        self.add_icon(&ic, x, y);
                    }
                } else if let Some(par) = ic.widget.parent() {
                    self.fixed.remove(&ic.widget.clone());
                }
            }
        }
    }

    /// 双击桌面空白处整理（在 main 里通过窗口级按钮事件调用）
    pub fn handle_blank_double_click(&self, ev: &gdk::EventButton) {
        if ev.button() != 1 {
            return;
        }
        // 只有空白处（没点到任何图标/盒子）才触发
        if !*self.draggable.borrow() {
            return;
        }
        // 检查点击位置是否在盒子区域内（盒子内部不算空白）
        let (px, py) = (ev.x() as i32, ev.y() as i32);
        for (_, b) in self.boxes.borrow().iter() {
            let b = b.borrow();
            let (bx, by) = b.pos;
            let (bw, bh) = b.size();
            if px >= bx && px <= bx + bw && py >= by && py <= by + bh {
                return;
            }
        }
        if self.cfg.borrow().double_click_organize {
            self.organize_icons(true);
        }
    }

    pub fn persist(&self) {
        let cfg = self.cfg.clone();
        let icons = self.icons.clone();
        let boxes = self.boxes.clone();
        let fixed = self.fixed.clone();

        let mut c = cfg.borrow_mut();
        c.desktop_icons.clear();
        for (path, ic) in icons.borrow().iter() {
            let (x, y) = ic.position(&fixed);
            c.desktop_icons.push(IconState {
                path: path.clone(),
                x,
                y,
            });
        }
        c.boxes.clear();
        for (id, b) in boxes.borrow().iter() {
            let b = b.borrow();
            let mut icons = vec![];
            for p in &b.icons {
                if let Some(ic) = self.icons.borrow().get(p) {
                    let (x, y) = ic.position(&b.content);
                    icons.push(IconState {
                        path: p.clone(),
                        x,
                        y,
                    });
                }
            }
            c.boxes.push(BoxState {
                id: id.clone(),
                title: b.title.clone(),
                x: b.pos.0,
                y: b.pos.1,
                w: b.target_w,
                h: b.target_h,
                collapsed: b.collapsed,
                color: None,
                icons,
            });
        }
        drop(c);
        crate::model::save_config(&cfg.borrow());
    }

    pub fn rename_box(&self, bv: &Rc<RefCell<BoxView>>) {
        let dlg = gtk::Dialog::with_buttons(
            Some("重命名盒子"),
            Some(&self.win),
            gtk::DialogFlags::MODAL,
            &[
                ("取消", gtk::ResponseType::Cancel),
                ("确定", gtk::ResponseType::Ok),
            ],
        );
        let entry = gtk::Entry::new();
        entry.set_text(&bv.borrow().title);
        dlg.content_area().pack_start(&entry, false, false, 0);
        dlg.show_all();
        let resp = dlg.run();
        let text = entry.text().to_string();
        dlg.close();
        if resp == gtk::ResponseType::Ok && !text.is_empty() {
            bv.borrow_mut().title = text.clone();
            bv.borrow_mut().title_label.set_text(&text);
            self.persist();
        }
    }
}
