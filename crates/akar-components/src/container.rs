use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub fn container(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    background: u32,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }
    if background == 0 {
        return;
    }
    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(background),
        border_color: color_to_f32(theme.base_300),
        border_width: 0.0,
        corner_radii: [theme.radius_box; 4],
        z: 0.0,
        _pad: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn transparent_background_pushes_no_quad() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(100.0),
                height: akar_layout::length(100.0),
            },
            ..Default::default()
        });
        let mut core = AkarCore::mock();

        container(&mut core, &layout, node_id, 0x00000000, &AKAR_THEME_DARK);

        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn non_zero_background_pushes_one_quad() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(100.0),
                height: akar_layout::length(100.0),
            },
            ..Default::default()
        });
        layout.compute(node_id, (Some(200.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        let mut core = AkarCore::mock();

        container(&mut core, &layout, node_id, 0xff0000ff, &AKAR_THEME_DARK);

        assert_eq!(core.draw_list.sorted_quads().len(), 1);
    }

    #[test]
    fn zero_area_pushes_no_quad() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        container(&mut core, &layout, node_id, 0xff0000ff, &AKAR_THEME_DARK);

        assert!(core.draw_list.sorted_quads().is_empty());
    }
}
