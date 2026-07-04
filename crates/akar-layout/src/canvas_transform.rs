use glam::Vec2;
use crate::Rect;

#[derive(Clone, Copy, Debug)]
pub struct CanvasTransform {
    pub offset: Vec2,
    pub scale: f32,
}

impl CanvasTransform {
    pub fn apply(self, pt: Vec2) -> Vec2 {
        pt * self.scale + self.offset
    }

    pub fn apply_rect(self, rect: Rect) -> [f32; 4] {
        let min = self.apply(rect.min);
        let max = self.apply(rect.max);
        [min.x, min.y, max.x - min.x, max.y - min.y]
    }

    pub fn scale_radius(self, radius: f32) -> f32 {
        radius * self.scale
    }
}

pub fn make_world_to_screen(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> CanvasTransform {
    let canvas_center = Vec2::new(
        canvas_rect[0] + canvas_rect[2] * 0.5,
        canvas_rect[1] + canvas_rect[3] * 0.5,
    );
    CanvasTransform {
        offset: canvas_center - pan * zoom,
        scale: zoom,
    }
}

pub fn make_screen_to_world(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> CanvasTransform {
    let canvas_center = Vec2::new(
        canvas_rect[0] + canvas_rect[2] * 0.5,
        canvas_rect[1] + canvas_rect[3] * 0.5,
    );
    CanvasTransform {
        offset: pan - canvas_center / zoom,
        scale: 1.0 / zoom,
    }
}

pub fn compute_visible_world_rect(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> Rect {
    let s2w = make_screen_to_world(pan, zoom, canvas_rect);
    let tl = Vec2::new(canvas_rect[0], canvas_rect[1]);
    let br = Vec2::new(canvas_rect[0] + canvas_rect[2], canvas_rect[1] + canvas_rect[3]);
    Rect {
        min: s2w.apply(tl),
        max: s2w.apply(br),
    }
}
