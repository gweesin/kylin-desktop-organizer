//! 布局几何计算（纯函数，无 GTK 依赖）
//!
//! 重构说明：网格位置、吸附、盒子尺寸等原本内嵌在 `window.rs` 的 widget 操作中，
//! 既无法测试也无法复用。此处集中为纯函数，`window.rs` 只负责「算好坐标 → 移动控件」。

/// 桌面图标区左 / 上边距
pub const DESKTOP_MARGIN_X: i32 = 20;
pub const DESKTOP_MARGIN_Y: i32 = 16;
/// 图标行高（图标 + 文字标签占位）
pub const ICON_ROW_H: i32 = 30;
/// 整理盒之间的水平间距 / 换行判断用到的基准
pub const BOX_GAP_X: i32 = 16;
pub const BOX_GAP_Y: i32 = 20;

/// 图标最终绘制边长（图标本体 + 内边距）
pub fn icon_total(icon_size: i32) -> i32 {
    icon_size + 16
}

/// 一个网格单元（图标边长 + 间隙）
pub fn cell(icon_size: i32, gap: i32) -> i32 {
    icon_total(icon_size) + gap
}

/// 一行最多放几个图标（可用宽度已扣掉两边留白）
pub fn max_per_row(avail_w: i32, icon_size: i32, gap: i32) -> i32 {
    (avail_w / cell(icon_size, gap)).max(1)
}

/// 桌面网格坐标（col/row 从 0 开始）
pub fn desktop_pos(col: i32, row: i32, icon_size: i32, gap: i32) -> (i32, i32) {
    let c = cell(icon_size, gap);
    (
        DESKTOP_MARGIN_X + col * c,
        DESKTOP_MARGIN_Y + row * (icon_total(icon_size) + ICON_ROW_H),
    )
}

/// 水平吸附：距网格线小于阈值则返回吸附后的 x，否则 None
pub fn snap_x(x: i32, icon_size: i32, gap: i32, snap: i32) -> Option<i32> {
    let c = cell(icon_size, gap);
    let nx = ((x as f64 / c as f64).round() as i32) * c;
    if (nx - x).abs() < snap + gap / 2 {
        Some(nx)
    } else {
        None
    }
}

/// 盒子内图标网格坐标（index 从 0 开始，按行优先排布）
pub fn box_icon_pos(
    index: i32,
    max_per_row: i32,
    icon_size: i32,
    gap: i32,
    padding: i32,
) -> (i32, i32) {
    let c = cell(icon_size, gap);
    let mpr = max_per_row.max(1);
    let col = index.rem_euclid(mpr);
    let row = index.div_euclid(mpr);
    (
        padding + col * c,
        padding + row * (icon_total(icon_size) + ICON_ROW_H),
    )
}

/// 按最大每行数计算盒子需要的行数（空盒子至少 1 行）
pub fn box_rows(count: usize, max_per_row: i32) -> usize {
    let mpr = max_per_row.max(1);
    if count == 0 {
        return 1;
    }
    ((count as i32 + mpr - 1) / mpr).max(1) as usize
}

/// 盒子宽度（图标数量 + 左右内边距）
pub fn box_width(count: usize, max_per_row: i32, icon_size: i32, gap: i32, padding: i32) -> i32 {
    let cols = (count as i32).min(max_per_row.max(1)).max(1);
    cols * cell(icon_size, gap) + padding * 2
}

/// 盒子高度（标题栏 + 图标行 + 上下内边距 + 边线）
pub fn box_height(
    count: usize,
    max_per_row: i32,
    icon_size: i32,
    title_h: i32,
    padding: i32,
) -> i32 {
    let rows = box_rows(count, max_per_row) as i32;
    title_h + padding * 2 + rows * (icon_total(icon_size) + ICON_ROW_H) + 4
}

/// 判断点是否落在矩形内（含边界）
pub fn point_in_rect(px: i32, py: i32, rx: i32, ry: i32, rw: i32, rh: i32) -> bool {
    px >= rx && px <= rx + rw && py >= ry && py <= ry + rh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_total_and_cell() {
        assert_eq!(icon_total(48), 64);
        assert_eq!(icon_total(40), 56);
        assert_eq!(cell(48, 10), 74);
        assert_eq!(cell(40, 5), 61);
    }

    #[test]
    fn max_per_row_never_below_one() {
        assert_eq!(max_per_row(1000, 48, 10), 13); // 920/74 = 12.43 -> 12
        assert_eq!(max_per_row(0, 48, 10), 1);
        assert_eq!(max_per_row(10, 48, 10), 1);
    }

    #[test]
    fn desktop_pos_layout() {
        // icon_size=48 gap=10 -> cell=74, 行高 = 64+30 = 94
        assert_eq!(desktop_pos(0, 0, 48, 10), (20, 16));
        assert_eq!(desktop_pos(1, 0, 48, 10), (94, 16));
        assert_eq!(desktop_pos(0, 1, 48, 10), (20, 110));
        assert_eq!(desktop_pos(2, 3, 48, 10), (20 + 2 * 74, 16 + 3 * 94));
    }

    #[test]
    fn snap_at_grid_is_kept() {
        // 48/10 -> cell=74, SNAP=6 -> 阈值 6+5=11
        assert_eq!(snap_x(74, 48, 10, 6), Some(74));
        assert_eq!(snap_x(80, 48, 10, 6), Some(74)); // 差 6 < 11
        assert_eq!(snap_x(64, 48, 10, 6), Some(74)); // 差 10 < 11
        assert_eq!(snap_x(90, 48, 10, 6), None); // 差 16 >= 11
                                                 // 负坐标也能吸附到 0
        assert_eq!(snap_x(-3, 48, 10, 6), Some(0));
    }

    #[test]
    fn box_icon_pos_wraps_by_row() {
        let padding = 8;
        let (i0, _) = box_icon_pos(0, 4, 48, 10, padding);
        assert_eq!(i0, 8);
        let (x, y) = box_icon_pos(4, 4, 48, 10, padding);
        // index 4 -> col 0, row 1
        assert_eq!(x, 8);
        assert_eq!(y, padding + 1 * (64 + 30));
        let (x5, _) = box_icon_pos(5, 4, 48, 10, padding);
        assert_eq!(x5, padding + 74);
    }

    #[test]
    fn box_rows_calculation() {
        assert_eq!(box_rows(0, 4), 1);
        assert_eq!(box_rows(1, 4), 1);
        assert_eq!(box_rows(4, 4), 1);
        assert_eq!(box_rows(5, 4), 2);
        assert_eq!(box_rows(8, 4), 2);
        assert_eq!(box_rows(9, 4), 3);
        assert_eq!(box_rows(10, 0), 10); // 防御 max_per_row<=0
    }

    #[test]
    fn box_dimensions() {
        let w = box_width(3, 4, 48, 10, 8);
        assert_eq!(w, 3 * 74 + 16); // 238
        assert_eq!(box_width(0, 4, 48, 10, 8), 74 + 16); // 至少一个单元格宽
        let h = box_height(3, 4, 48, 34, 8);
        assert_eq!(h, 34 + 16 + 1 * 94 + 4); // 148
        let h2 = box_height(5, 4, 48, 34, 8);
        assert_eq!(h2, 34 + 16 + 2 * 94 + 4); // 242
    }

    #[test]
    fn point_in_rect_bounds() {
        assert!(point_in_rect(0, 0, 0, 0, 100, 100));
        assert!(point_in_rect(100, 100, 0, 0, 100, 100));
        assert!(!point_in_rect(101, 100, 0, 0, 100, 100));
        assert!(!point_in_rect(50, 101, 0, 0, 100, 100));
    }
}
