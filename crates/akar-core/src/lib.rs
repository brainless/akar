pub mod draw_list;
pub use draw_list::{DrawCall, DrawList, QuadCall, TextCall};

pub mod input;
pub use input::{InputState, Key};

pub mod quad_pipeline;
pub use quad_pipeline::QuadPipeline;

pub mod text_pipeline;
pub use text_pipeline::TextPipeline;

pub mod context;
pub use context::AkarCore;

pub const Z_BASE: f32 = 0.0;
pub const Z_SCRIM: f32 = 0.5;
pub const Z_FLOAT: f32 = 1.0;
pub const Z_OVERLAY: f32 = 2.0;

pub fn list_clip(
    total: usize,
    item_height: f32,
    scroll_y: f32,
    viewport_height: f32,
) -> std::ops::Range<usize> {
    if total == 0 || item_height <= 0.0 {
        return 0..0;
    }
    let first = ((scroll_y / item_height).floor() as isize - 1).max(0) as usize;
    let last = (((scroll_y + viewport_height) / item_height).ceil() as usize + 1).min(total);
    first..last
}

#[cfg(test)]
mod list_clip_tests {
    use super::list_clip;

    #[test]
    fn empty_list_returns_empty() {
        assert_eq!(list_clip(0, 50.0, 0.0, 400.0), 0..0);
    }

    #[test]
    fn zero_item_height_returns_empty() {
        assert_eq!(list_clip(100, 0.0, 0.0, 400.0), 0..0);
    }

    #[test]
    fn top_of_list_includes_first_items() {
        let range = list_clip(100, 50.0, 0.0, 200.0);
        assert_eq!(range.start, 0);
        assert!(range.end >= 4);
        assert!(range.end <= 6);
    }

    #[test]
    fn scrolled_mid_list() {
        let range = list_clip(100, 50.0, 250.0, 200.0);
        assert!(range.start <= 4);
        assert!(range.end >= 9);
        assert!(range.end <= 100);
    }

    #[test]
    fn near_end_clamps_to_total() {
        let range = list_clip(10, 50.0, 400.0, 200.0);
        assert_eq!(range.end, 10);
    }
}
