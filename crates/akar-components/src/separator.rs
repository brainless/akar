use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub fn separator(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }
    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(theme.base_300),
        border_color: [0.0; 4],
        corner_radii: [0.0; 4],
        border_width: 0.0,
        z: 0.0,
        _pad: [0.0; 2],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_pushes_no_quad() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();

        separator(&mut core, &layout, node_id, &AKAR_THEME_DARK);

        assert!(core.draw_list.sorted_quads().is_empty());
    }
}
