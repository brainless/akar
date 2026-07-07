use akar_core::AkarCore;

pub struct ScrollAreaResponse {
    pub content_y: f32,
}

pub fn scroll_area_begin(
    core: &mut AkarCore,
    rect: [f32; 4],
    scroll_y: &mut f32,
    content_height: f32,
) -> ScrollAreaResponse {
    let [_, y, _, h] = rect;

    if core.input.is_hovering(rect) {
        *scroll_y -= core.input.scroll_delta.y;
    }

    let max_scroll = (content_height - h).max(0.0);
    *scroll_y = scroll_y.clamp(0.0, max_scroll);

    core.draw_list.push_scissor(rect);

    ScrollAreaResponse {
        content_y: y - *scroll_y,
    }
}

pub fn scroll_area_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}

#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::AkarCore;

    fn make_core() -> AkarCore {
        AkarCore::mock()
    }

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        scroll_area_begin(&mut core, [0.0, 0.0, 200.0, 400.0], &mut 0.0, 1000.0);
        assert!(core.draw_list.active_scissor().is_some());
        scroll_area_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }

    #[test]
    fn scroll_y_clamped_to_zero() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        let mut scroll_y = -50.0f32;
        scroll_area_begin(&mut core, [0.0, 0.0, 200.0, 400.0], &mut scroll_y, 1000.0);
        scroll_area_end(&mut core);
        assert_eq!(scroll_y, 0.0);
    }

    #[test]
    fn scroll_y_clamped_to_max() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        let mut scroll_y = 9999.0f32;
        scroll_area_begin(&mut core, [0.0, 0.0, 200.0, 400.0], &mut scroll_y, 1000.0);
        scroll_area_end(&mut core);
        assert_eq!(scroll_y, 600.0);
    }

    #[test]
    fn content_y_reflects_scroll() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        let mut scroll_y = 100.0f32;
        let resp = scroll_area_begin(&mut core, [0.0, 50.0, 200.0, 400.0], &mut scroll_y, 1000.0);
        scroll_area_end(&mut core);
        assert_eq!(resp.content_y, -50.0);
    }
}
