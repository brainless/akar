use glam::Vec2;
use crate::WorldRect;

#[derive(Clone, Copy, Debug)]
pub struct CanvasTransform {
    pub offset: Vec2,
    pub scale: f32,
}

impl CanvasTransform {
    pub fn apply(self, pt: Vec2) -> Vec2 {
        pt * self.scale + self.offset
    }

    pub fn apply_rect(self, rect: WorldRect) -> [f32; 4] {
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

pub fn compute_visible_world_rect(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> WorldRect {
    let s2w = make_screen_to_world(pan, zoom, canvas_rect);
    let tl = Vec2::new(canvas_rect[0], canvas_rect[1]);
    let br = Vec2::new(canvas_rect[0] + canvas_rect[2], canvas_rect[1] + canvas_rect[3]);
    WorldRect {
        min: s2w.apply(tl),
        max: s2w.apply(br),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    const CANVAS: [f32; 4] = [0.0, 0.0, 800.0, 600.0];
    const CENTER: Vec2 = Vec2::new(400.0, 300.0);

    #[test]
    fn world_to_screen_identity() {
        let t = make_world_to_screen(Vec2::ZERO, 1.0, CANVAS);
        let s = t.apply(Vec2::ZERO);
        assert!((s - CENTER).length() < 0.001, "got {s}");
    }

    #[test]
    fn world_to_screen_with_pan() {
        let t = make_world_to_screen(Vec2::new(100.0, 0.0), 1.0, CANVAS);
        let s = t.apply(Vec2::new(100.0, 0.0));
        assert!((s - CENTER).length() < 0.001, "got {s}");
    }

    #[test]
    fn world_to_screen_with_zoom() {
        let t = make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let s = t.apply(Vec2::new(1.0, 0.0));
        assert!((s - Vec2::new(402.0, 300.0)).length() < 0.001, "got {s}");
    }

    #[test]
    fn world_to_screen_off_center_canvas() {
        let canvas = [200.0, 0.0, 600.0, 600.0];
        let expected_center = Vec2::new(500.0, 300.0);
        let t = make_world_to_screen(Vec2::ZERO, 1.0, canvas);
        let s = t.apply(Vec2::ZERO);
        assert!((s - expected_center).length() < 0.001, "got {s}");
    }

    #[test]
    fn round_trip() {
        let canvas = [50.0, 100.0, 700.0, 500.0];
        let pan = Vec2::new(123.0, -45.0);
        let zoom = 1.7;
        let world = Vec2::new(200.0, -80.0);
        let w2s = make_world_to_screen(pan, zoom, canvas);
        let s2w = make_screen_to_world(pan, zoom, canvas);
        let back = s2w.apply(w2s.apply(world));
        assert!((back - world).length() < 0.001, "round-trip error: {back}");
    }

    #[test]
    fn visible_world_rect_identity() {
        let v = compute_visible_world_rect(Vec2::ZERO, 1.0, CANVAS);
        assert!((v.min - Vec2::new(-400.0, -300.0)).length() < 0.001);
        assert!((v.max - Vec2::new(400.0, 300.0)).length() < 0.001);
    }

    #[test]
    fn visible_world_rect_zoom2() {
        let v = compute_visible_world_rect(Vec2::ZERO, 2.0, CANVAS);
        assert!((v.min - Vec2::new(-200.0, -150.0)).length() < 0.001);
        assert!((v.max - Vec2::new(200.0, 150.0)).length() < 0.001);
    }

    #[test]
    fn apply_rect_dimensions() {
        let t = make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let world_rect = crate::WorldRect { min: Vec2::new(-5.0, -5.0), max: Vec2::new(5.0, 5.0) };
        let [_x, _y, w, h] = t.apply_rect(world_rect);
        assert!((w - 20.0).abs() < 0.001);
        assert!((h - 20.0).abs() < 0.001);
    }
}
