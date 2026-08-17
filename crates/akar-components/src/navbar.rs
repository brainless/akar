use akar_core::{AkarCore, QuadCall};
use akar_layout::{
    AlignItems, Dimension, Display, FlexDirection, JustifyContent, Layout, NodeId, Size, Style,
};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub struct NavbarSlots {
    pub start: NodeId,
    pub center: NodeId,
    pub end: NodeId,
}

pub struct NavbarStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
}

impl NavbarStyle {
    pub fn default(theme: &AkarTheme) -> Self {
        Self {
            background: theme.base_200,
            border_color: theme.base_300,
            border_width: theme.border_width,
            corner_radii: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

pub fn navbar_layout(layout: &mut Layout, node_id: NodeId, _theme: &AkarTheme) -> NavbarSlots {
    layout.set_style(
        node_id,
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::CENTER),
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::auto(),
            },
            ..Default::default()
        },
    );

    let start = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: Some(AlignItems::CENTER),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        ..Default::default()
    });

    let center = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: Some(JustifyContent::CENTER),
        align_items: Some(AlignItems::CENTER),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        ..Default::default()
    });

    let end = layout.new_leaf(Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        justify_content: Some(JustifyContent::FLEX_END),
        align_items: Some(AlignItems::CENTER),
        flex_grow: 0.0,
        flex_shrink: 0.0,
        ..Default::default()
    });

    layout.set_children(node_id, &[start, center, end]);

    NavbarSlots { start, center, end }
}

pub fn navbar(core: &mut AkarCore, layout: &Layout, node_id: NodeId, style: &NavbarStyle) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(style.background),
        border_color: color_to_f32(style.border_color),
        corner_radii: style.corner_radii,
        border_width: style.border_width,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });
}

pub fn navbar_combined(
    core: &mut AkarCore,
    layout: &mut Layout,
    node_id: NodeId,
    theme: &AkarTheme,
) -> NavbarSlots {
    let slots = navbar_layout(layout, node_id, theme);
    let style = NavbarStyle::default(theme);
    navbar(core, layout, node_id, &style);
    slots
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_layout::length;

    #[test]
    fn navbar_layout_creates_three_slots() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style::default());
        let slots = navbar_layout(&mut layout, node, &crate::AKAR_THEME_DARK);

        assert_ne!(slots.start, slots.center);
        assert_ne!(slots.center, slots.end);
        assert_ne!(slots.start, slots.end);
    }

    #[test]
    fn navbar_layout_does_not_draw() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(800.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        let _slots = navbar_layout(&mut layout, node, &crate::AKAR_THEME_DARK);
        assert!(
            core.draw_list.sorted_quads().is_empty(),
            "navbar_layout must not draw"
        );
    }

    #[test]
    fn navbar_paint_draws_background() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(800.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        let style = NavbarStyle::default(&crate::AKAR_THEME_DARK);
        navbar(&mut core, &layout, node, &style);
        let quads = core.draw_list.sorted_quads();
        assert_eq!(
            quads.len(),
            1,
            "navbar paint should emit exactly one background quad"
        );
    }

    #[test]
    fn zero_area_does_nothing() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style::default());
        layout.compute(node, (Some(0.0), Some(0.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        let style = NavbarStyle::default(&crate::AKAR_THEME_DARK);
        navbar(&mut core, &layout, node, &style);
        assert!(
            core.draw_list.sorted_quads().is_empty(),
            "zero-area navbar should not draw"
        );
    }

    #[test]
    fn navbar_preserves_caller_layout() {
        let theme = &crate::AKAR_THEME_DARK;
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(500.0),
                height: length(80.0),
            },
            ..Default::default()
        });
        let _slots = navbar_layout(&mut layout, node, theme);
        layout.compute(node, (Some(500.0), Some(80.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let rect_before = layout.rect(node);

        let mut core = AkarCore::mock();
        let style = NavbarStyle::default(theme);
        navbar(&mut core, &layout, node, &style);

        let rect_after = layout.rect(node);
        assert_eq!(
            rect_before, rect_after,
            "navbar paint must not modify layout"
        );
    }

    #[test]
    fn children_can_be_added_to_slots() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        let slots = navbar_layout(&mut layout, node, &crate::AKAR_THEME_DARK);

        let child_start = layout.new_leaf(Style {
            size: Size {
                width: length(100.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.add_child(slots.start, child_start);

        let child_center = layout.new_leaf(Style {
            size: Size {
                width: length(200.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.add_child(slots.center, child_center);

        let child_end = layout.new_leaf(Style {
            size: Size {
                width: length(100.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.add_child(slots.end, child_end);

        layout.compute(node, (Some(800.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let start_rect = layout.rect(slots.start);
        let center_rect = layout.rect(slots.center);
        let end_rect = layout.rect(slots.end);

        assert!(
            (start_rect[0] - 0.0).abs() < 1.0,
            "start.x = {}",
            start_rect[0]
        );
        assert!(
            center_rect[0] >= start_rect[0] + start_rect[2] - 1.0,
            "center.x = {}, expected >= {}",
            center_rect[0],
            start_rect[0] + start_rect[2]
        );
        assert!(
            end_rect[0] >= center_rect[0] + center_rect[2] - 1.0,
            "end.x = {}, expected >= {}",
            end_rect[0],
            center_rect[0] + center_rect[2]
        );
    }

    #[test]
    fn navbar_slots_mirror_under_rtl_direction() {
        use akar_layout::AkarDirection;

        let mut layout = akar_layout::Layout::new();
        layout.set_direction(AkarDirection::Rtl);
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        let slots = navbar_layout(&mut layout, node, &crate::AKAR_THEME_DARK);

        let child_start = layout.new_leaf(Style {
            size: Size {
                width: length(100.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.add_child(slots.start, child_start);

        let child_center = layout.new_leaf(Style {
            size: Size {
                width: length(200.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.add_child(slots.center, child_center);

        let child_end = layout.new_leaf(Style {
            size: Size {
                width: length(100.0),
                height: length(40.0),
            },
            ..Default::default()
        });
        layout.add_child(slots.end, child_end);

        layout.compute(node, (Some(800.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let start_rect = layout.rect(slots.start);
        let center_rect = layout.rect(slots.center);
        let end_rect = layout.rect(slots.end);

        // Under RTL the visual order of the flex row reverses: `start` packs
        // against the right edge, `end` against the left edge, with no
        // code change to navbar.rs itself (the direction is stamped onto
        // every Style by Layout's central stamp per epic 023 Task 7).
        assert!(
            end_rect[0] <= center_rect[0] + 1.0,
            "end.x = {}, expected <= center.x = {}",
            end_rect[0],
            center_rect[0]
        );
        assert!(
            center_rect[0] <= start_rect[0] + 1.0,
            "center.x = {}, expected <= start.x = {}",
            center_rect[0],
            start_rect[0]
        );
        assert!(
            (start_rect[0] + start_rect[2] - 800.0).abs() < 1.0,
            "start should end flush with the right edge under RTL, start.x={}, start.w={}",
            start_rect[0],
            start_rect[2]
        );
    }

    #[test]
    fn navbar_combined_does_layout_and_paint() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style {
            size: Size {
                width: length(800.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(800.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        let slots = navbar_combined(&mut core, &mut layout, node, &crate::AKAR_THEME_DARK);

        assert_ne!(slots.start, slots.center);
        assert_ne!(slots.center, slots.end);
        assert!(!core.draw_list.sorted_quads().is_empty());
    }
}
