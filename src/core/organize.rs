//! 一键整理规划（纯算法，无 GTK 依赖）
//!
//! 重构说明：原 `window.rs::organize_icons` 把「分组」「布局计算」「控件操作」混在一起。
//! 这里抽出纯算法部分：`build_groups` 负责分类聚合排序，`plan_organize` 负责计算
//! 每个分类盒子的几何与归属图标。`window.rs` 只负责把方案应用到 GTK 控件上。

use crate::core::categorize::{is_empty_folder, CATEGORY_ORDER};
use crate::core::layout;
use std::collections::BTreeMap;

/// 分类盒子计划（纯数据）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryPlan {
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub icons: Vec<String>,
}

/// 一次一键整理的完整布局方案
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrganizePlan {
    pub boxes: Vec<CategoryPlan>,
}

/// 整理时桌面宽度对应的每行最大盒子内图标数（与原实现一致：至少 2 列）
pub fn organize_max_per_row(desktop_w: i32, icon_size: i32, gap: i32) -> i32 {
    ((desktop_w - 80) / layout::cell(icon_size, gap)).max(2)
}

/// 纯算法：把路径列表按分类聚合（可注入分类函数，便于测试与替换规则）
///
/// - `show_files=false` 时跳过文件/图片视频/压缩包/文件夹类图标
/// - 返回结果已按 `CATEGORY_ORDER` 排序，未知分类排在最后
pub fn build_groups<F>(
    paths: &[String],
    show_files: bool,
    classify: F,
) -> Vec<(String, Vec<String>)>
where
    F: Fn(&str) -> String,
{
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in paths {
        let cat = classify(p);
        if !show_files && matches!(cat.as_str(), "文件" | "图片视频" | "压缩包" | "文件夹")
        {
            continue;
        }
        groups.entry(cat).or_default().push(p.clone());
    }

    // 忽略空的「文件夹 / 其他」组
    groups.retain(|k, v| !(is_empty_folder(k) && v.is_empty()));

    let mut out: Vec<(String, Vec<String>)> = groups.into_iter().collect();
    out.sort_by(|a, b| {
        let ai = CATEGORY_ORDER
            .iter()
            .position(|c| *c == a.0)
            .unwrap_or(usize::MAX);
        let bi = CATEGORY_ORDER
            .iter()
            .position(|c| *c == b.0)
            .unwrap_or(usize::MAX);
        ai.cmp(&bi)
    });
    out
}

/// 纯算法：把已排序的分组排布成盒子布局（自左向右，超宽换行）
pub fn plan_organize(
    groups: &[(String, Vec<String>)],
    desktop_w: i32,
    icon_size: i32,
    gap: i32,
    title_h: i32,
    padding: i32,
) -> OrganizePlan {
    let mpr = organize_max_per_row(desktop_w, icon_size, gap);
    let mut plans = Vec::new();
    let mut x = layout::DESKTOP_MARGIN_X;
    let mut y = layout::DESKTOP_MARGIN_Y;

    for (cat, items) in groups {
        if items.is_empty() {
            continue;
        }
        let w = layout::box_width(items.len(), mpr, icon_size, gap, padding);
        let h = layout::box_height(items.len(), mpr, icon_size, title_h, padding);
        plans.push(CategoryPlan {
            title: cat.clone(),
            x,
            y,
            w,
            h,
            icons: items.clone(),
        });

        x += w + layout::BOX_GAP_X;
        if x + w > desktop_w - 40 {
            x = layout::DESKTOP_MARGIN_X;
            y += h + layout::BOX_GAP_Y;
        }
    }

    OrganizePlan { boxes: plans }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试用的固定分类函数
    fn classify(cat: &str) -> impl Fn(&str) -> String + '_ {
        move |_: &str| cat.to_string()
    }

    #[test]
    fn empty_input_yields_empty_plan() {
        let groups = build_groups(&[], true, classify("应用"));
        assert!(groups.is_empty());
        let plan = plan_organize(&groups, 1920, 48, 10, 34, 8);
        assert!(plan.boxes.is_empty());
    }

    #[test]
    fn groups_are_sorted_by_category_order() {
        let paths = vec![
            "/d/txt.txt".into(),
            "/d/firefox.desktop".into(),
            "/d/电脑".into(),
            "/d/weird.unknownext".into(),
        ];
        let groups = build_groups(&paths, true, |p| {
            let name = p.rsplit('/').next().unwrap_or("");
            match name {
                "电脑" => "我的电脑".into(),
                "firefox.desktop" => "应用".into(),
                _ if name.ends_with(".txt") => "文件".into(),
                _ => "未知类".into(),
            }
        });
        let titles: Vec<&str> = groups.iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(titles, vec!["我的电脑", "应用", "文件", "未知类"]);
    }

    #[test]
    fn hide_files_filters_file_groups() {
        let paths = vec![
            "/d/a.txt".into(),
            "/d/pic.jpg".into(),
            "/d/arch.zip".into(),
            "/d/资料".into(),
            "/d/firefox.desktop".into(),
        ];
        let groups = build_groups(&paths, false, |p| {
            let name = p.rsplit('/').next().unwrap_or("");
            match name {
                "firefox.desktop" => "应用".into(),
                "资料" => "文件夹".into(),
                _ if name.ends_with(".jpg") => "图片视频".into(),
                _ if name.ends_with(".zip") => "压缩包".into(),
                _ => "文件".into(),
            }
        });
        let titles: Vec<&str> = groups.iter().map(|(t, _)| t.as_str()).collect();
        // 文件/图片视频/压缩包/文件夹 全被过滤，只剩 应用
        assert_eq!(titles, vec!["应用"]);
    }

    #[test]
    fn empty_folder_group_is_dropped() {
        let groups = build_groups(&["/d/x.desktop".into()], true, classify("文件夹"));
        // 只有一个「文件夹」组且它是空组 -> 被剔除
        assert!(groups.is_empty());
    }

    #[test]
    fn non_empty_folder_group_is_kept() {
        let groups = build_groups(&["/d/a".into()], true, classify("文件夹"));
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "文件夹");
    }

    #[test]
    fn plan_places_boxes_left_to_right_and_wraps() {
        // desktop_w=800, icon 48/gap 10 -> cell 74; mpr = max(2, 720/74)=max(2,9)=9
        // 每组 3 个图标 -> box_width = 3*74+16 = 238
        let groups = vec![
            ("应用".to_string(), vec!["a".into(), "b".into(), "c".into()]),
            ("文件".to_string(), vec!["d".into(), "e".into(), "f".into()]),
            (
                "文件夹".to_string(),
                vec!["g".into(), "h".into(), "i".into()],
            ),
        ];
        let plan = plan_organize(&groups, 800, 48, 10, 34, 8);
        assert_eq!(plan.boxes.len(), 3);
        assert_eq!(plan.boxes[0].x, 20);
        assert_eq!(plan.boxes[0].y, 16);
        assert_eq!(plan.boxes[0].w, 238);
        // 第二个盒子紧随其后
        assert_eq!(plan.boxes[1].x, 20 + 238 + 16);
        assert_eq!(plan.boxes[1].y, 16);
        // 第三个盒子如果放不下就换行（x+238 > 800-40=760?）
        // 前两个占 20+238+16+238=512 < 760，第三个仍在同一行
        assert_eq!(plan.boxes[2].x, 20 + 2 * (238 + 16));
        assert_eq!(plan.boxes[2].y, 16);
    }

    #[test]
    fn plan_wraps_to_next_row_when_too_wide() {
        // 每个盒子 480 宽，桌面 800：一行只能放 1 个
        let groups = vec![
            (
                "应用".to_string(),
                (0..10).map(|i| format!("a{i}")).collect(),
            ),
            (
                "文件".to_string(),
                (0..10).map(|i| format!("b{i}")).collect(),
            ),
        ];
        let plan = plan_organize(&groups, 800, 48, 10, 34, 8);
        assert_eq!(plan.boxes.len(), 2);
        // 第二个盒子换行
        assert_eq!(plan.boxes[1].x, 20);
        assert!(plan.boxes[1].y > plan.boxes[0].y);
    }

    #[test]
    fn plan_icons_fit_multiple_rows() {
        // 10 个图标，mpr=4（桌面很窄） -> 3 行
        let groups = vec![(
            "文件".to_string(),
            (0..10).map(|i| format!("f{i}")).collect(),
        )];
        let plan = plan_organize(&groups, 400, 48, 10, 34, 8);
        assert_eq!(plan.boxes.len(), 1);
        assert_eq!(plan.boxes[0].icons.len(), 10);
        // 3 行 -> 高度 = 34+16+3*94+4 = 336
        assert_eq!(plan.boxes[0].h, 336);
    }

    #[test]
    fn organize_max_per_row_minimum_is_two() {
        assert_eq!(organize_max_per_row(1, 48, 10), 2);
        assert_eq!(organize_max_per_row(100, 48, 10), 2);
    }
}
