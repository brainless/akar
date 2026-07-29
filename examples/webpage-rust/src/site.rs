use akar_core::AkarCore;
use akar_layout::{Layout, NodeId};

pub trait Site {
    fn name(&self) -> &str;
    fn root(&self) -> NodeId;
    fn build_layout(&mut self, layout: &mut Layout);
    fn render(&mut self, core: &mut AkarCore, layout: &Layout, viewport_rect: [f32; 4]);
}
