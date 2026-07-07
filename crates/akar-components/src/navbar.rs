use akar_core::AkarCore;
use akar_layout::{
    AlignItems, Dimension, Display, FlexDirection, JustifyContent, Layout, NodeId, Size, Style,
};

use crate::box_style::BoxStyle;
use crate::container::container;
use crate::AkarTheme;

pub struct NavbarSlots {
    pub start: NodeId,
    pub center: NodeId,
    pub end: NodeId,
}

pub fn navbar(
    core: &mut AkarCore,
    layout: &mut Layout,
    node_id: NodeId,
    theme: &AkarTheme,
) -> NavbarSlots {
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
    container(core, layout, node_id, &BoxStyle::panel(theme));

    NavbarSlots { start, center, end }
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_layout::length;

    #[test]
    fn navbar_creates_three_slots() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();
        let slots = navbar(&mut core, &mut layout, node, &crate::AKAR_THEME_DARK);

        assert_ne!(slots.start, slots.center);
        assert_ne!(slots.center, slots.end);
        assert_ne!(slots.start, slots.end);
    }

    #[test]
    fn navbar_background_rendered() {
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
        navbar(&mut core, &mut layout, node, &crate::AKAR_THEME_DARK);
        assert!(!core.draw_list.sorted_quads().is_empty());
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
        let mut core = AkarCore::mock();
        let slots = navbar(&mut core, &mut layout, node, &crate::AKAR_THEME_DARK);

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
}
