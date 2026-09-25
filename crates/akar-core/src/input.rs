use glam;
use std::ops::{BitOr, BitOrAssign};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum FileDragInput {
    Enter {
        position: [f32; 2],
        paths: Vec<PathBuf>,
    },
    Move {
        position: [f32; 2],
    },
    Drop {
        position: [f32; 2],
        paths: Vec<PathBuf>,
    },
    Leave,
    UnpositionedDrop {
        paths: Vec<PathBuf>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileDropEvent {
    pub position: [f32; 2],
    pub paths: Vec<PathBuf>,
    claimed: bool,
}

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
    PageUp,
    PageDown,
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
    pub file_drag_position: Option<[f32; 2]>,
    pub file_drag_paths: Vec<PathBuf>,
    pub file_drop_events: Vec<FileDropEvent>,
    pub unpositioned_file_drops: Vec<Vec<PathBuf>>,
    file_hover_claimed: bool,
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
            file_drag_position: None,
            file_drag_paths: Vec::new(),
            file_drop_events: Vec::new(),
            unpositioned_file_drops: Vec::new(),
            file_hover_claimed: false,
            modifiers: Modifiers::default(),
            focused_id: None,
        }
    }

    /// Clears one-frame events after rendering, or before host input is submitted for a frame.
    /// Submit file events after an explicit input reset such as `akar_input_begin`.
    pub fn begin_frame(&mut self) {
        self.mouse_pos_prev = self.mouse_pos;
        self.mouse_buttons_pressed = [false; 5];
        self.mouse_buttons_released = [false; 5];
        self.scroll_delta = glam::Vec2::ZERO;
        self.chars.clear();
        self.keys_pressed.clear();
        self.key_events.clear();
        self.paste_events.clear();
        self.file_drop_events.clear();
        self.unpositioned_file_drops.clear();
        self.file_hover_claimed = false;
    }

    /// Queues a host file event. Coordinates must use the layout's window-local input space.
    /// An invalid drop coordinate is retained only as an unpositioned window-level drop.
    pub fn push_file_drag(&mut self, event: FileDragInput) {
        match event {
            FileDragInput::Enter { position, paths } => {
                self.file_drag_position = valid_file_position(position);
                self.file_drag_paths = paths;
            }
            FileDragInput::Move { position } => {
                self.file_drag_position = valid_file_position(position);
            }
            FileDragInput::Drop { position, paths } => {
                self.file_drag_position = None;
                self.file_drag_paths.clear();
                if valid_file_position(position).is_some() {
                    self.file_drop_events.push(FileDropEvent {
                        position,
                        paths,
                        claimed: false,
                    });
                } else {
                    self.unpositioned_file_drops.push(paths);
                }
            }
            FileDragInput::Leave => {
                self.file_drag_position = None;
                self.file_drag_paths.clear();
            }
            FileDragInput::UnpositionedDrop { paths } => {
                self.file_drag_position = None;
                self.file_drag_paths.clear();
                self.unpositioned_file_drops.push(paths);
            }
        }
    }

    pub fn claim_file_hover(&mut self, eligible: bool) -> bool {
        if !eligible || self.file_drag_position.is_none() || self.file_hover_claimed {
            return false;
        }
        self.file_hover_claimed = true;
        true
    }

    pub fn claim_file_drops(&mut self, mut eligible: impl FnMut([f32; 2]) -> bool) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for event in &mut self.file_drop_events {
            if !event.claimed && eligible(event.position) {
                event.claimed = true;
                paths.append(&mut event.paths);
            }
        }
        paths
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

fn valid_file_position([x, y]: [f32; 2]) -> Option<[f32; 2]> {
    (x.is_finite() && y.is_finite()).then_some([x, y])
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
    fn file_drag_hover_lifecycle_and_single_claim_per_frame() {
        let mut input = InputState::new();
        input.push_file_drag(FileDragInput::Enter {
            position: [10.0, 20.0],
            paths: vec![PathBuf::from("first")],
        });
        assert_eq!(input.file_drag_position, Some([10.0, 20.0]));
        assert_eq!(input.file_drag_paths, [PathBuf::from("first")]);
        assert!(!input.claim_file_hover(false));
        assert!(input.claim_file_hover(true));
        assert!(!input.claim_file_hover(true));

        input.begin_frame();
        assert_eq!(input.file_drag_position, Some([10.0, 20.0]));
        assert!(input.claim_file_hover(true));

        input.push_file_drag(FileDragInput::Move {
            position: [30.0, 40.0],
        });
        assert_eq!(input.file_drag_position, Some([30.0, 40.0]));
        input.push_file_drag(FileDragInput::Leave);
        assert_eq!(input.file_drag_position, None);
        assert!(input.file_drag_paths.is_empty());
        assert!(!input.claim_file_hover(true));
    }

    #[test]
    fn positioned_drops_retain_paths_and_positions_until_claimed() {
        let mut input = InputState::new();
        input.push_file_drag(FileDragInput::Drop {
            position: [10.0, 20.0],
            paths: vec![PathBuf::from("one"), PathBuf::from("two")],
        });
        input.push_file_drag(FileDragInput::Drop {
            position: [100.0, 200.0],
            paths: vec![PathBuf::from("three")],
        });
        assert_eq!(input.file_drop_events.len(), 2);
        assert_eq!(input.file_drop_events[0].position, [10.0, 20.0]);
        assert_eq!(input.file_drop_events[1].position, [100.0, 200.0]);
        assert_eq!(
            input.claim_file_drops(|position| position[0] > 50.0),
            [PathBuf::from("three")]
        );
        assert_eq!(
            input.claim_file_drops(|_| true),
            [PathBuf::from("one"), PathBuf::from("two")]
        );
        assert!(input.claim_file_drops(|_| true).is_empty());
    }

    #[test]
    fn invalid_coordinates_never_become_targetable() {
        let mut input = InputState::new();
        input.push_file_drag(FileDragInput::Enter {
            position: [f32::NAN, 10.0],
            paths: vec![],
        });
        assert_eq!(input.file_drag_position, None);
        assert!(!input.claim_file_hover(true));
        input.push_file_drag(FileDragInput::Move {
            position: [f32::INFINITY, 10.0],
        });
        assert_eq!(input.file_drag_position, None);
        input.push_file_drag(FileDragInput::Drop {
            position: [10.0, f32::NEG_INFINITY],
            paths: vec![PathBuf::from("invalid-position")],
        });
        assert!(input.file_drop_events.is_empty());
        assert_eq!(
            input.unpositioned_file_drops,
            [vec![PathBuf::from("invalid-position")]]
        );
        assert!(input.claim_file_drops(|_| true).is_empty());
    }

    #[test]
    fn unpositioned_drops_and_frame_cleanup() {
        let mut input = InputState::new();
        input.begin_frame();
        input.push_file_drag(FileDragInput::UnpositionedDrop {
            paths: vec![PathBuf::from("window-level")],
        });
        input.push_file_drag(FileDragInput::Drop {
            position: [1.0, 2.0],
            paths: vec![PathBuf::from("node-level")],
        });
        assert_eq!(input.unpositioned_file_drops.len(), 1);
        assert_eq!(input.file_drop_events.len(), 1);
        input.begin_frame();
        assert!(input.unpositioned_file_drops.is_empty());
        assert!(input.file_drop_events.is_empty());
        assert!(input.claim_file_drops(|_| true).is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn dropped_path_keeps_native_non_utf8_bytes() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(vec![b'f', 0xff]));
        let mut input = InputState::new();
        input.push_file_drag(FileDragInput::Drop {
            position: [1.0, 2.0],
            paths: vec![path.clone()],
        });
        assert_eq!(input.claim_file_drops(|_| true), [path]);
    }

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
        assert!(input.key_events[0].modifiers.control);
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
