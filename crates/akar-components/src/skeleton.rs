use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SkeletonVariant {
    Text,
    Card,
    Circle,
}

pub fn skeleton(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    variant: SkeletonVariant,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let corner_radius = match variant {
        SkeletonVariant::Text => theme.radius_field / 2.0,
        SkeletonVariant::Card => theme.radius_box,
        SkeletonVariant::Circle => rect[3] / 2.0,
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(theme.base_300),
        border_color: [0.0; 4],
        corner_radii: [corner_radius; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_pushes_nothing() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        skeleton(&mut core, &layout, node_id, SkeletonVariant::Text, &AKAR_THEME_DARK);

        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn text_variant_pushes_one_quad() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(200.0),
                height: akar_layout::length(20.0),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(200.0), Some(20.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();

        skeleton(&mut core, &layout, node_id, SkeletonVariant::Text, &AKAR_THEME_DARK);

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].corner_radii[0], AKAR_THEME_DARK.radius_field / 2.0);
    }

    #[test]
    fn circle_variant_computes_radius() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(48.0),
                height: akar_layout::length(48.0),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(48.0), Some(48.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();

        skeleton(&mut core, &layout, node_id, SkeletonVariant::Circle, &AKAR_THEME_DARK);

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].corner_radii[0], 24.0);
    }
}
