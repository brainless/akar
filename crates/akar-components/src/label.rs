use akar_core::{AkarCore, TextCall};
use akar_layout::{Layout, NodeId};

use crate::color::color_to_f32;
use crate::AkarTheme;

pub fn label(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    color: u32,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let buffer_id = core.text_pipeline.set_text(
        Some(layout.widget_id(node_id)),
        text,
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
        Some(rect[2]),
        None,
    );

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0],
        y: rect[1],
        clip: rect,
        color: color_to_f32(color),
        z: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_does_not_push_text() {
        let mut layout = Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        label(
            &mut core,
            &layout,
            node_id,
            "Hello",
            0xFFFFFFFF,
            &AKAR_THEME_DARK,
        );

        assert_eq!(core.draw_list.len(), 0);
    }
}
