use akar_core::{AkarCore, QuadCall};
use akar_layout::length;
use akar_layout::{Display, FlexDirection, Layout, NodeId, Size, Style};

use crate::box_style::BoxStyle;
use crate::color::color_to_f32;
use crate::AkarTheme;

pub struct CardSlots {
    pub header: Option<NodeId>,
    pub body: NodeId,
    pub footer: Option<NodeId>,
}

pub struct CardLayout {
    pub direction: FlexDirection,
    pub gap: f32,
    pub padding: f32,
    pub has_header: bool,
    pub has_footer: bool,
}

impl CardLayout {
    pub fn body_only(theme: &AkarTheme) -> Self {
        Self {
            direction: FlexDirection::Column,
            gap: 0.0,
            padding: theme.padding_x,
            has_header: false,
            has_footer: false,
        }
    }

    pub fn with_header_footer(theme: &AkarTheme) -> Self {
        Self {
            direction: FlexDirection::Column,
            gap: 0.0,
            padding: theme.padding_x,
            has_header: true,
            has_footer: true,
        }
    }
}

pub struct CardStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_color: u32,
    pub shadow_offset: [f32; 2],
    pub separator_color: u32,
}

impl CardStyle {
    pub fn default(theme: &AkarTheme) -> Self {
        let bx = BoxStyle::card(theme);
        Self {
            background: bx.fill,
            border_color: bx.border_color,
            border_width: bx.border_width,
            corner_radii: bx.corner_radii,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: 0,
            shadow_offset: [0.0; 2],
            separator_color: theme.base_300,
        }
    }
}

pub fn card_layout(layout: &mut Layout, node_id: NodeId, options: &CardLayout) -> CardSlots {
    let children_count = 1 + options.has_header as usize + options.has_footer as usize;
    let mut children: Vec<NodeId> = Vec::with_capacity(children_count);

    let header = if options.has_header {
        let node = layout.new_leaf(Style::default());
        children.push(node);
        Some(node)
    } else {
        None
    };

    let body = layout.new_leaf(Style::default());
    children.push(body);

    let footer = if options.has_footer {
        let node = layout.new_leaf(Style::default());
        children.push(node);
        Some(node)
    } else {
        None
    };

    layout.set_children(node_id, &children);

    layout.set_style(
        node_id,
        Style {
            display: Display::Flex,
            flex_direction: options.direction,
            padding: akar_layout::Rect {
                top: length(options.padding),
                right: length(options.padding),
                bottom: length(options.padding),
                left: length(options.padding),
            },
            gap: Size {
                width: length(options.gap),
                height: length(options.gap),
            },
            ..Default::default()
        },
    );

    CardSlots {
        header,
        body,
        footer,
    }
}

fn push_separator(core: &mut AkarCore, y: f32, x: f32, w: f32, color: u32) {
    core.draw_list.push_quad(QuadCall {
        rect: [x, y, w, 1.0],
        fill: color_to_f32(color),
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });
}

pub fn card(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    slots: &CardSlots,
    style: &CardStyle,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let (shadow_color, shadow_offset, shadow_blur, shadow_spread) = (
        color_to_f32(style.shadow_color),
        style.shadow_offset,
        style.shadow_blur,
        style.shadow_spread,
    );

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(style.background),
        border_color: color_to_f32(style.border_color),
        corner_radii: style.corner_radii,
        border_width: style.border_width,
        z: 0.0,
        shadow_blur,
        shadow_spread,
        shadow_color,
        shadow_offset,
        _pad: [0.0; 2],
    });

    let pad_left = style.border_width + style.corner_radii[3];
    let pad_right = rect[2] - style.border_width - style.corner_radii[1];
    let sep_w = pad_right - pad_left;

    if let Some(header_id) = slots.header {
        let header_rect = layout.rect(header_id);
        let header_bottom = header_rect[1] + header_rect[3];
        if header_bottom > rect[1] && header_bottom < rect[1] + rect[3] {
            push_separator(
                core,
                header_bottom,
                rect[0] + pad_left,
                sep_w,
                style.separator_color,
            );
        }
    }

    if let Some(footer_id) = slots.footer {
        let footer_rect = layout.rect(footer_id);
        if footer_rect[1] > rect[1] && footer_rect[1] < rect[1] + rect[3] {
            push_separator(
                core,
                footer_rect[1],
                rect[0] + pad_left,
                sep_w,
                style.separator_color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;

    fn sized_root(layout: &mut akar_layout::Layout) -> NodeId {
        layout.new_leaf(Style {
            size: akar_layout::Size {
                width: length(300.0_f32),
                height: length(200.0_f32),
            },
            ..Default::default()
        })
    }

    #[test]
    fn card_layout_creates_body_only() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout::body_only(&AKAR_THEME_DARK);
        let slots = card_layout(&mut layout, root, &opts);

        assert!(slots.header.is_none());
        assert!(slots.footer.is_none());
        assert_eq!(layout.rect(slots.body).len(), 4);
    }

    #[test]
    fn card_layout_creates_header_body_footer() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout::with_header_footer(&AKAR_THEME_DARK);
        let slots = card_layout(&mut layout, root, &opts);

        assert!(slots.header.is_some());
        assert!(slots.footer.is_some());
    }

    #[test]
    fn card_layout_returns_distinct_slot_ids() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout::with_header_footer(&AKAR_THEME_DARK);
        let slots = card_layout(&mut layout, root, &opts);

        let h = slots.header.unwrap();
        let b = slots.body;
        let f = slots.footer.unwrap();
        assert_ne!(h, b);
        assert_ne!(b, f);
        assert_ne!(h, f);
    }

    #[test]
    fn zero_area_does_nothing() {
        let mut layout = akar_layout::Layout::new();
        let root = layout.new_leaf(Style::default());
        layout.compute(root, (Some(0.0), Some(0.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();
        let opts = CardLayout::body_only(&AKAR_THEME_DARK);
        let slots = card_layout(&mut layout, root, &opts);
        let style = CardStyle::default(&AKAR_THEME_DARK);
        card(&mut core, &layout, root, &slots, &style);
        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn card_renders_background() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout::body_only(&AKAR_THEME_DARK);
        let slots = card_layout(&mut layout, root, &opts);
        layout.compute(root, (Some(300.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let mut core = AkarCore::mock();
        let style = CardStyle::default(&AKAR_THEME_DARK);
        card(&mut core, &layout, root, &slots, &style);

        let quads = core.draw_list.sorted_quads();
        assert!(!quads.is_empty(), "card should emit at least one quad");
    }

    #[test]
    fn card_separator_between_header_body() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout {
            direction: FlexDirection::Column,
            gap: 0.0,
            padding: 12.0,
            has_header: true,
            has_footer: false,
        };
        let slots = card_layout(&mut layout, root, &opts);

        let header_child = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: length(276.0_f32),
                height: length(30.0_f32),
            },
            ..Default::default()
        });
        layout.add_child(slots.header.unwrap(), header_child);

        layout.compute(root, (Some(300.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let mut core = AkarCore::mock();
        let style = CardStyle::default(&AKAR_THEME_DARK);
        card(&mut core, &layout, root, &slots, &style);

        let quads = core.draw_list.sorted_quads();
        assert!(
            quads.len() >= 2,
            "expected background + separator, got {}",
            quads.len()
        );
    }

    #[test]
    fn card_no_separator_body_only() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout::body_only(&AKAR_THEME_DARK);
        let slots = card_layout(&mut layout, root, &opts);
        layout.compute(root, (Some(300.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let mut core = AkarCore::mock();
        let style = CardStyle::default(&AKAR_THEME_DARK);
        card(&mut core, &layout, root, &slots, &style);

        let quads = core.draw_list.sorted_quads();
        assert_eq!(
            quads.len(),
            1,
            "body-only card should have only background quad"
        );
    }

    #[test]
    fn card_separator_between_body_footer() {
        let mut layout = akar_layout::Layout::new();
        let root = sized_root(&mut layout);
        let opts = CardLayout {
            direction: FlexDirection::Column,
            gap: 0.0,
            padding: 12.0,
            has_header: false,
            has_footer: true,
        };
        let slots = card_layout(&mut layout, root, &opts);

        let footer_child = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: length(276.0_f32),
                height: length(20.0_f32),
            },
            ..Default::default()
        });
        layout.add_child(slots.footer.unwrap(), footer_child);

        layout.compute(root, (Some(300.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });

        let mut core = AkarCore::mock();
        let style = CardStyle::default(&AKAR_THEME_DARK);
        card(&mut core, &layout, root, &slots, &style);

        let quads = core.draw_list.sorted_quads();
        assert!(
            quads.len() >= 2,
            "expected background + separator, got {}",
            quads.len()
        );
    }
}
