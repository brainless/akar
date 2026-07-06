use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq)]
pub enum DrawCall {
    Quad(QuadCall),
    Text(TextCall),
}

#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct QuadCall {
    pub rect: [f32; 4],
    pub fill: [f32; 4],
    pub border_color: [f32; 4],
    pub corner_radii: [f32; 4],
    pub border_width: f32,
    pub z: f32,
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_color: [f32; 4],
    pub shadow_offset: [f32; 2],
    pub _pad: [f32; 2],
}

const _: () = assert!(std::mem::size_of::<QuadCall>() == 112);

#[derive(Clone, Debug, PartialEq)]
pub struct TextCall {
    pub buffer_id: u64,
    pub x: f32,
    pub y: f32,
    pub clip: [f32; 4],
    pub color: [f32; 4],
    pub z: f32,
}

pub struct DrawList {
    calls: Vec<DrawCall>,
    scissor_stack: Vec<[f32; 4]>,
    scale_factor: f32,
}

impl DrawList {
    pub fn new() -> Self {
        Self {
            calls: Vec::new(),
            scissor_stack: Vec::new(),
            scale_factor: 1.0,
        }
    }

    pub fn begin_frame(&mut self, scale_factor: f32) {
        self.calls.clear();
        self.scissor_stack.clear();
        self.scale_factor = scale_factor;
    }

    pub fn push_scissor(&mut self, rect: [f32; 4]) {
        let physical = [
            rect[0] * self.scale_factor,
            rect[1] * self.scale_factor,
            rect[2] * self.scale_factor,
            rect[3] * self.scale_factor,
        ];
        if let Some(&top) = self.scissor_stack.last() {
            let x = top[0].max(physical[0]);
            let y = top[1].max(physical[1]);
            let w = (top[0] + top[2]).min(physical[0] + physical[2]) - x;
            let h = (top[1] + top[3]).min(physical[1] + physical[3]) - y;
            if w <= 0.0 || h <= 0.0 {
                self.scissor_stack.push([x, y, 0.0, 0.0]);
            } else {
                self.scissor_stack.push([x, y, w, h]);
            }
        } else {
            self.scissor_stack.push(physical);
        }
    }

    pub fn pop_scissor(&mut self) {
        self.scissor_stack.pop();
    }

    pub fn active_scissor(&self) -> Option<[f32; 4]> {
        self.scissor_stack.last().copied()
    }

    pub fn push_quad(&mut self, mut call: QuadCall) {
        call.rect[0] *= self.scale_factor;
        call.rect[1] *= self.scale_factor;
        call.rect[2] *= self.scale_factor;
        call.rect[3] *= self.scale_factor;
        call.border_width *= self.scale_factor;
        for r in &mut call.corner_radii {
            *r *= self.scale_factor;
        }
        call.shadow_blur *= self.scale_factor;
        call.shadow_spread *= self.scale_factor;
        call.shadow_offset[0] *= self.scale_factor;
        call.shadow_offset[1] *= self.scale_factor;
        if let Some(scissor) = self.active_scissor() {
            if !intersects(call.rect, scissor) {
                return;
            }
        }
        self.calls.push(DrawCall::Quad(call));
    }

    pub fn push_text(&mut self, call: TextCall) {
        if let Some(scissor) = self.active_scissor() {
            if !intersects(call.clip, scissor) {
                return;
            }
        }
        self.calls.push(DrawCall::Text(call));
    }

    pub fn sorted_quads(&mut self) -> Vec<QuadCall> {
        let mut quads: Vec<QuadCall> = self
            .calls
            .iter()
            .filter_map(|c| match c {
                DrawCall::Quad(q) => Some(*q),
                _ => None,
            })
            .collect();
        quads.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap_or(Ordering::Equal));
        quads
    }

    pub fn text_calls(&self) -> &[DrawCall] {
        &self.calls
    }

    pub fn len(&self) -> usize {
        self.calls.len()
    }

    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}

impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}

fn intersects(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && a[0] + a[2] > b[0] && a[1] < b[1] + b[3] && a[1] + a[3] > b[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_at(x: f32, y: f32, w: f32, h: f32) -> QuadCall {
        QuadCall {
            rect: [x, y, w, h],
            fill: [1.0, 0.0, 0.0, 1.0],
            border_color: [0.0; 4],
            corner_radii: [0.0; 4],
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        }
    }

    fn text_at(x: f32, y: f32, clip_w: f32, clip_h: f32) -> TextCall {
        TextCall {
            buffer_id: 0,
            x,
            y,
            clip: [x, y, clip_w, clip_h],
            color: [1.0; 4],
            z: 0.0,
        }
    }

    #[test]
    fn scissor_culls_quad_outside() {
        let mut dl = DrawList::new();
        dl.begin_frame(1.0);
        dl.push_scissor([0.0, 0.0, 100.0, 100.0]);
        dl.push_quad(quad_at(200.0, 200.0, 50.0, 50.0));
        let quads = dl.sorted_quads();
        assert!(quads.is_empty());
    }

    #[test]
    fn scissor_includes_partial_quad() {
        let mut dl = DrawList::new();
        dl.begin_frame(1.0);
        dl.push_scissor([0.0, 0.0, 100.0, 100.0]);
        dl.push_quad(quad_at(80.0, 80.0, 50.0, 50.0));
        let quads = dl.sorted_quads();
        assert_eq!(quads.len(), 1);
    }

    #[test]
    fn quads_sorted_by_z() {
        let mut dl = DrawList::new();
        dl.begin_frame(1.0);
        let mut q1 = quad_at(0.0, 0.0, 10.0, 10.0);
        q1.z = 3.0;
        let mut q2 = quad_at(0.0, 0.0, 10.0, 10.0);
        q2.z = 1.0;
        let mut q3 = quad_at(0.0, 0.0, 10.0, 10.0);
        q3.z = 2.0;
        dl.push_quad(q1);
        dl.push_quad(q2);
        dl.push_quad(q3);
        let quads = dl.sorted_quads();
        assert_eq!(quads[0].z, 1.0);
        assert_eq!(quads[1].z, 2.0);
        assert_eq!(quads[2].z, 3.0);
    }

    #[test]
    fn push_scissor_intersection_clips() {
        let mut dl = DrawList::new();
        dl.begin_frame(1.0);
        dl.push_scissor([0.0, 0.0, 200.0, 200.0]);
        dl.push_scissor([50.0, 50.0, 100.0, 100.0]);
        let s = dl.active_scissor().unwrap();
        assert_eq!(s, [50.0, 50.0, 100.0, 100.0]);
        dl.pop_scissor();
        dl.push_scissor([100.0, 100.0, 200.0, 200.0]);
        let s = dl.active_scissor().unwrap();
        assert_eq!(s, [100.0, 100.0, 100.0, 100.0]);
    }

    #[test]
    fn text_culled_outside_scissor() {
        let mut dl = DrawList::new();
        dl.begin_frame(1.0);
        dl.push_scissor([0.0, 0.0, 100.0, 100.0]);
        dl.push_text(text_at(200.0, 200.0, 50.0, 50.0));
        assert!(dl.text_calls().is_empty());
    }
}
