use glam;
use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Character(char),
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Enter,
    Escape,
    Tab,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasteEvent {
    pub target: u64,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShortcutModifiers(u8);

impl ShortcutModifiers {
    pub const NONE: Self = Self(0);
    pub const PRIMARY: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const SUPER: Self = Self(1 << 2);
    pub const ALT: Self = Self(1 << 3);
    pub const SHIFT: Self = Self(1 << 4);

    fn matches(self, modifiers: Modifiers) -> bool {
        let mut expected = self.0;
        if expected & Self::PRIMARY.0 != 0 {
            expected &= !Self::PRIMARY.0;
            #[cfg(target_os = "macos")]
            {
                expected |= Self::SUPER.0;
            }
            #[cfg(not(target_os = "macos"))]
            {
                expected |= Self::CONTROL.0;
            }
        }

        let actual = (if modifiers.control {
            Self::CONTROL.0
        } else {
            0
        }) | (if modifiers.super_key {
            Self::SUPER.0
        } else {
            0
        }) | (if modifiers.alt { Self::ALT.0 } else { 0 })
            | (if modifiers.shift { Self::SHIFT.0 } else { 0 });
        actual == expected
    }
}

impl BitOr for ShortcutModifiers {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ShortcutModifiers {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: ShortcutModifiers,
    pub key: Key,
}

impl Shortcut {
    pub const fn new(modifiers: ShortcutModifiers, key: Key) -> Self {
        Self { modifiers, key }
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        self.key == event.key && self.modifiers.matches(event.modifiers)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextEditKeybindings {
    pub select_all: Shortcut,
    pub copy: Shortcut,
    pub paste: Shortcut,
}

impl TextEditKeybindings {
    pub const fn platform_default() -> Self {
        Self {
            select_all: Shortcut::new(ShortcutModifiers::PRIMARY, Key::Character('a')),
            copy: Shortcut::new(ShortcutModifiers::PRIMARY, Key::Character('c')),
            paste: Shortcut::new(ShortcutModifiers::PRIMARY, Key::Character('v')),
        }
    }

    pub fn matches_select_all(&self, event: &KeyEvent) -> bool {
        self.select_all.matches(event)
    }

    pub fn matches_copy(&self, event: &KeyEvent) -> bool {
        self.copy.matches(event)
    }

    pub fn matches_paste(&self, event: &KeyEvent) -> bool {
        self.paste.matches(event)
    }
}

impl Default for TextEditKeybindings {
    fn default() -> Self {
        Self::platform_default()
    }
}

pub struct InputState {
    pub mouse_pos: glam::Vec2,
    pub mouse_pos_prev: glam::Vec2,
    pub mouse_buttons: [bool; 5],
    pub mouse_buttons_pressed: [bool; 5],
    pub mouse_buttons_released: [bool; 5],
    pub scroll_delta: glam::Vec2,
    pub chars: Vec<char>,
    pub keys_pressed: Vec<Key>,
    pub key_events: Vec<KeyEvent>,
    pub paste_events: Vec<PasteEvent>,
    pub modifiers: Modifiers,
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
            keys_pressed: Vec::new(),
            key_events: Vec::new(),
            paste_events: Vec::new(),
            modifiers: Modifiers::default(),
            focused_id: None,
        }
    }

    pub fn begin_frame(&mut self) {
        self.mouse_pos_prev = self.mouse_pos;
        self.mouse_buttons_pressed = [false; 5];
        self.mouse_buttons_released = [false; 5];
        self.scroll_delta = glam::Vec2::ZERO;
        self.chars.clear();
        self.keys_pressed.clear();
        self.key_events.clear();
        self.paste_events.clear();
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

    pub fn push_key(&mut self, key: Key) {
        self.keys_pressed.push(key);
        self.key_events.push(KeyEvent {
            key,
            modifiers: self.modifiers,
            repeat: false,
        });
    }

    pub fn push_key_event(&mut self, event: KeyEvent) {
        self.keys_pressed.push(event.key);
        self.key_events.push(event);
    }

    pub fn push_paste(&mut self, target: u64, text: impl Into<String>) {
        self.paste_events.push(PasteEvent {
            target,
            text: text.into(),
        });
    }

    pub fn pastes_for(&self, target: u64) -> impl Iterator<Item = &str> {
        self.paste_events
            .iter()
            .filter(move |event| event.target == target)
            .map(|event| event.text.as_str())
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

    #[test]
    fn key_event_keeps_modifier_snapshot_and_clears_per_frame() {
        let mut input = InputState::new();
        input.modifiers = Modifiers {
            control: true,
            ..Modifiers::default()
        };
        input.push_key_event(KeyEvent {
            key: Key::Character('a'),
            modifiers: input.modifiers,
            repeat: false,
        });
        input.modifiers = Modifiers::default();
        assert_eq!(input.key_events[0].modifiers.control, true);
        assert!(!input.modifiers.control);
        input.begin_frame();
        assert!(input.key_events.is_empty());
        assert!(input.keys_pressed.is_empty());
    }

    #[test]
    fn paste_events_are_targeted_and_clear_per_frame() {
        let mut input = InputState::new();
        input.push_paste(7, "first");
        input.push_paste(9, "other");
        input.push_paste(7, "second");

        assert_eq!(input.pastes_for(7).collect::<Vec<_>>(), ["first", "second"]);
        assert_eq!(input.pastes_for(9).collect::<Vec<_>>(), ["other"]);

        input.begin_frame();
        assert!(input.paste_events.is_empty());
    }

    #[test]
    fn platform_defaults_use_primary_modifier() {
        let bindings = TextEditKeybindings::platform_default();
        assert_eq!(bindings.select_all.key, Key::Character('a'));
        assert_eq!(bindings.copy.key, Key::Character('c'));
        assert_eq!(bindings.paste.key, Key::Character('v'));

        #[cfg(target_os = "macos")]
        assert!(bindings.matches_select_all(&KeyEvent {
            key: Key::Character('a'),
            modifiers: Modifiers {
                super_key: true,
                ..Modifiers::default()
            },
            repeat: false,
        }));
        #[cfg(not(target_os = "macos"))]
        assert!(bindings.matches_select_all(&KeyEvent {
            key: Key::Character('a'),
            modifiers: Modifiers {
                control: true,
                ..Modifiers::default()
            },
            repeat: false,
        }));
    }

    #[test]
    fn matching_requires_exact_modifiers_and_supports_custom_bindings() {
        let bindings = TextEditKeybindings {
            select_all: Shortcut::new(
                ShortcutModifiers::ALT | ShortcutModifiers::SHIFT,
                Key::Character('x'),
            ),
            ..TextEditKeybindings::default()
        };
        let event = |modifiers| KeyEvent {
            key: Key::Character('x'),
            modifiers,
            repeat: false,
        };
        assert!(bindings.matches_select_all(&event(Modifiers {
            alt: true,
            shift: true,
            ..Modifiers::default()
        })));
        assert!(!bindings.matches_select_all(&event(Modifiers {
            alt: true,
            ..Modifiers::default()
        })));
        assert!(!bindings.matches_select_all(&event(Modifiers {
            alt: true,
            shift: true,
            control: true,
            ..Modifiers::default()
        })));
    }
}
