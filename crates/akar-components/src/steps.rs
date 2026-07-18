use crate::color::color_to_f32;
use crate::AkarTheme;
use akar_core::{AkarCore, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};

pub fn steps(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    labels: &[&str],
    current: usize,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    if labels.is_empty() {
        return;
    }

    let count = labels.len();
    let current = current.min(count - 1);
    let step_spacing = rect[2] / count as f32;

    let circle_base_diameter = 10.0;
    let circle_base_radius = circle_base_diameter / 2.0;
    let current_diameter = 14.0;
    let current_radius = current_diameter / 2.0;

    for (i, label) in labels.iter().enumerate() {
        let circle_center_x = rect[0] + step_spacing * i as f32 + step_spacing / 2.0;

        let is_current = i == current;
        let (diameter, radius) = if is_current {
            (current_diameter, current_radius)
        } else {
            (circle_base_diameter, circle_base_radius)
        };

        let circle_x = circle_center_x - radius;

        if i > 0 {
            let prev_center_x = rect[0] + step_spacing * (i - 1) as f32 + step_spacing / 2.0;
            let prev_radius = if (i - 1) == current {
                current_radius
            } else {
                circle_base_radius
            };
            let conn_x = prev_center_x + prev_radius;
            let conn_w = circle_x - conn_x;

            if conn_w > 0.0 {
                let conn_color = if i <= current {
                    theme.primary
                } else {
                    theme.base_300
                };
                let circle_center_y = rect[1] + circle_base_radius + 2.0;
                let conn_y = circle_center_y - 1.0;

                core.draw_list.push_quad(QuadCall {
                    rect: [conn_x, conn_y, conn_w, 2.0],
                    fill: color_to_f32(conn_color),
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
        }

        let circle_center_y = rect[1] + circle_base_radius + 2.0;
        let circle_y = circle_center_y - radius;
        let circle_fill = if i <= current {
            theme.primary
        } else {
            theme.base_300
        };

        core.draw_list.push_quad(QuadCall {
            rect: [circle_x, circle_y, diameter, diameter],
            fill: color_to_f32(circle_fill),
            border_color: [0.0; 4],
            corner_radii: [radius; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });

        let label_y = rect[1] + circle_base_diameter + 2.0 + 4.0;
        let label_color = if is_current {
            theme.primary
        } else if i < current {
            theme.base_content
        } else {
            theme.base_300
        };

        let buffer_id = core.text_pipeline.set_text(
            Some(layout.widget_id(node_id) + i as u64),
            label,
            glyphon::Metrics::new(theme.font_size_sm, theme.font_size_sm * 1.2),
            Some(step_spacing),
            None,
        );

        core.draw_list.push_text(TextCall {
            buffer_id,
            x: circle_center_x - step_spacing / 2.0,
            y: label_y,
            clip: [
                circle_center_x - step_spacing / 2.0,
                label_y,
                step_spacing,
                theme.font_size_sm * 1.2,
            ],
            color: color_to_f32(label_color),
            z: 0.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::{length, Layout, Size, Style};

    fn node_400x60(layout: &mut Layout) -> NodeId {
        let n = layout.new_leaf(Style {
            size: Size {
                width: length(400.0),
                height: length(60.0),
            },
            ..Default::default()
        });
        layout.compute(n, (Some(400.0), Some(60.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        n
    }

    #[test]
    fn empty_labels_does_nothing() {
        let mut layout = Layout::new();
        let node = node_400x60(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        steps(&mut core, &layout, node, &[], 0, &AKAR_THEME_DARK);

        assert_eq!(core.draw_list.len(), 0);
    }

    #[test]
    fn two_steps_all_completed() {
        let mut layout = Layout::new();
        let node = node_400x60(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        steps(
            &mut core,
            &layout,
            node,
            &["Step 1", "Step 2"],
            2,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 3);
        let primary = color_to_f32(AKAR_THEME_DARK.primary);
        for q in &quads {
            assert_eq!(q.fill, primary);
        }
    }

    #[test]
    fn first_step_current() {
        let mut layout = Layout::new();
        let node = node_400x60(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);

        steps(
            &mut core,
            &layout,
            node,
            &["Step 1", "Step 2"],
            0,
            &AKAR_THEME_DARK,
        );

        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 3);
        let primary = color_to_f32(AKAR_THEME_DARK.primary);
        let base_300 = color_to_f32(AKAR_THEME_DARK.base_300);
        assert_eq!(quads[0].fill, primary);
        assert_eq!(quads[1].fill, base_300);
        assert_eq!(quads[2].fill, base_300);
    }
}
