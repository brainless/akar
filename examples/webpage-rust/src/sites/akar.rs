use akar_core::AkarCore;
use akar_layout::{Dimension, Layout, NodeId, Size, Style};

use crate::site::Site;

pub struct AkarSite {
    root: NodeId,
}

impl AkarSite {
    pub fn new() -> Self {
        Self {
            root: NodeId::new(0),
        }
    }
}

impl Site for AkarSite {
    fn name(&self) -> &str {
        "akar"
    }

    fn root(&self) -> NodeId {
        self.root
    }

    fn build_layout(&mut self, layout: &mut Layout) {
        self.root = layout.new_leaf(Style {
            size: Size {
                width: Dimension::percent(1.0),
                height: Dimension::percent(1.0),
            },
            ..Default::default()
        });
    }

    fn render(&mut self, core: &mut AkarCore, _layout: &Layout, viewport_rect: [f32; 4]) {
        core.draw_list.push_quad(akar_core::QuadCall {
            rect: viewport_rect,
            fill: [0.98, 0.98, 0.98, 1.0],
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

        let buf = core.text_pipeline.set_text(
            Some(10000),
            "Coming soon",
            glyphon::Metrics::new(32.0, 32.0 * 1.4),
            None,
            None,
            None,
        );
        let measured = core.text_pipeline.measure(buf, None);
        let text_x = viewport_rect[0] + (viewport_rect[2] - measured.x) / 2.0;
        let text_y = viewport_rect[1] + (viewport_rect[3] - measured.y) / 2.0;
        core.draw_list.push_text(akar_core::TextCall {
            buffer_id: buf,
            x: text_x,
            y: text_y,
            clip: viewport_rect,
            color: [0.33, 0.33, 0.33, 1.0],
            z: 0.0,
        });
    }
}
