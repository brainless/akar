use glam;

pub struct InputState {
    pub mouse_pos: glam::Vec2,
    pub mouse_pos_prev: glam::Vec2,
    pub mouse_buttons: [bool; 5],
    pub mouse_buttons_pressed: [bool; 5],
    pub mouse_buttons_released: [bool; 5],
    pub scroll_delta: glam::Vec2,
    pub chars: Vec<char>,
    pub focused_id: Option<u64>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            mouse_pos: glam::Vec2::ZERO,
            mouse_pos_prev: glam::Vec2::ZERO,
            mouse_buttons: [false; 5],
            mouse_buttons_pressed: [false; 5],
            mouse_buttons_released: [false; 5],
            scroll_delta: glam::Vec2::ZERO,
            chars: Vec::new(),
            focused_id: None,
        }
    }

    pub fn begin_frame(&mut self) {
        self.mouse_pos_prev = self.mouse_pos;
        self.mouse_buttons_pressed = [false; 5];
        self.mouse_buttons_released = [false; 5];
        self.scroll_delta = glam::Vec2::ZERO;
        self.chars.clear();
    }

    pub fn set_mouse_pos(&mut self, x: f32, y: f32) {
        self.mouse_pos = glam::Vec2::new(x, y);
    }

    pub fn push_mouse_button(&mut self, button: usize, pressed: bool) {
        if button < 5 {
            let was_down = self.mouse_buttons[button];
            self.mouse_buttons[button] = pressed;
            if pressed && !was_down {
                self.mouse_buttons_pressed[button] = true;
            } else if !pressed && was_down {
                self.mouse_buttons_released[button] = true;
            }
        }
    }

    pub fn push_scroll(&mut self, dx: f32, dy: f32) {
        self.scroll_delta += glam::Vec2::new(dx, dy);
    }

    pub fn push_char(&mut self, c: char) {
        self.chars.push(c);
    }

    pub fn is_hovering(&self, rect: [f32; 4]) -> bool {
        let [x, y, w, h] = rect;
        self.mouse_pos.x >= x
            && self.mouse_pos.x <= x + w
            && self.mouse_pos.y >= y
            && self.mouse_pos.y <= y + h
    }

    pub fn is_clicked(&self, rect: [f32; 4]) -> bool {
        self.mouse_buttons_released[0] && self.is_hovering(rect)
    }

    pub fn is_pressed(&self, rect: [f32; 4]) -> bool {
        self.mouse_buttons[0] && self.is_hovering(rect)
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hovering_inside_rect() {
        let mut input = InputState::new();
        input.set_mouse_pos(50.0, 50.0);
        assert!(input.is_hovering([0.0, 0.0, 100.0, 100.0]));
    }

    #[test]
    fn hovering_outside_rect() {
        let mut input = InputState::new();
        input.set_mouse_pos(150.0, 50.0);
        assert!(!input.is_hovering([0.0, 0.0, 100.0, 100.0]));
    }

    #[test]
    fn clicked_after_press_release_inside() {
        let mut input = InputState::new();
        input.set_mouse_pos(50.0, 50.0);
        input.push_mouse_button(0, true);
        input.begin_frame();
        input.push_mouse_button(0, false);
        assert!(input.is_clicked([0.0, 0.0, 100.0, 100.0]));
    }

    #[test]
    fn not_clicked_when_released_outside() {
        let mut input = InputState::new();
        input.set_mouse_pos(50.0, 50.0);
        input.push_mouse_button(0, true);
        input.begin_frame();
        input.set_mouse_pos(150.0, 50.0);
        input.push_mouse_button(0, false);
        assert!(!input.is_clicked([0.0, 0.0, 100.0, 100.0]));
    }

    #[test]
    fn pressed_when_held_inside() {
        let mut input = InputState::new();
        input.set_mouse_pos(50.0, 50.0);
        input.push_mouse_button(0, true);
        assert!(input.is_pressed([0.0, 0.0, 100.0, 100.0]));
    }
}
