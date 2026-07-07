use crate::color::color_to_f32;
use crate::AkarTheme;
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

pub struct ProgressStyle {
    pub track_color: u32,
    pub fill_color: u32,
    pub corner_radius: f32,
}

impl ProgressStyle {
    pub fn from_theme(theme: &AkarTheme) -> Self {
        Self {
            track_color: theme.base_300,
            fill_color: theme.primary,
            corner_radius: theme.radius_field / 2.0,
        }
    }
}

pub fn progress(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: f32,
    style: &ProgressStyle,
) {
    let rect = layout.rect(node_id);
    progress_at(core, rect, value, style);
}

pub fn progress_at(core: &mut AkarCore, rect: [f32; 4], value: f32, style: &ProgressStyle) {
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }
    let value = value.clamp(0.0, 1.0);
    let radii = [style.corner_radius; 4];

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(style.track_color),
        border_color: [0.0; 4],
        corner_radii: radii,
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    if value > 0.0 {
        let fill_rect = [rect[0], rect[1], rect[2] * value, rect[3]];
        core.draw_list.push_quad(QuadCall {
            rect: fill_rect,
            fill: color_to_f32(style.fill_color),
            border_color: [0.0; 4],
            corner_radii: radii,
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

#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::AkarCore;
    use akar_layout::{length, Layout, Size, Style};

    fn node_100x20(layout: &mut Layout) -> NodeId {
        let n = layout.new_leaf(Style {
            size: Size {
                width: length(100.0),
                height: length(20.0),
            },
            ..Default::default()
        });
        layout.compute(n, (Some(200.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        n
    }

    #[test]
    fn full_value_pushes_two_quads() {
        let mut layout = Layout::new();
        let node = node_100x20(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let style = ProgressStyle {
            track_color: 0xccccccff,
            fill_color: 0x0000ffff,
            corner_radius: 4.0,
        };
        progress(&mut core, &layout, node, 1.0, &style);
        assert_eq!(core.draw_list.len(), 2);
    }

    #[test]
    fn zero_value_pushes_only_track() {
        let mut layout = Layout::new();
        let node = node_100x20(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let style = ProgressStyle {
            track_color: 0xccccccff,
            fill_color: 0x0000ffff,
            corner_radius: 4.0,
        };
        progress(&mut core, &layout, node, 0.0, &style);
        assert_eq!(core.draw_list.len(), 1);
    }

    #[test]
    fn value_clamped_above_one() {
        let mut layout = Layout::new();
        let node = node_100x20(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let style = ProgressStyle {
            track_color: 0xccccccff,
            fill_color: 0x0000ffff,
            corner_radius: 0.0,
        };
        progress(&mut core, &layout, node, 5.0, &style);
        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads[0].rect[2], quads[1].rect[2]);
    }
}
