use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldRect {
    pub min: Vec2,
    pub max: Vec2,
}

impl WorldRect {
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        }
    }

    pub fn intersects(self, other: WorldRect) -> bool {
        other.max.x >= self.min.x
            && other.min.x <= self.max.x
            && other.max.y >= self.min.y
            && other.min.y <= self.max.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> WorldRect {
        WorldRect { min: Vec2::new(x0, y0), max: Vec2::new(x1, y1) }
    }

    #[test]
    fn intersects_inside()   { assert!(r(-10., -10., 10., 10.).intersects(r(-5., -5., 5., 5.))); }
    #[test]
    fn intersects_outside()  { assert!(!r(-10., -10., 10., 10.).intersects(r(20., 20., 30., 30.))); }
    #[test]
    fn intersects_touching()  { assert!(r(0., 0., 10., 10.).intersects(r(10., 0., 20., 10.))); }
    #[test]
    fn intersects_partial()  { assert!(r(0., 0., 10., 10.).intersects(r(5., 5., 15., 15.))); }
}
