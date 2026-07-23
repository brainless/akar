use akar_core::InputState;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

fn is_committed_text_char(c: char) -> bool {
    !c.is_control()
        && !matches!(
            c,
            '\u{E000}'..='\u{F8FF}'
                | '\u{F0000}'..='\u{FFFFD}'
                | '\u{100000}'..='\u{10FFFD}'
        )
}

fn named_key(name: &winit::keyboard::NamedKey) -> Option<akar_core::Key> {
    Some(match name {
        winit::keyboard::NamedKey::Backspace => akar_core::Key::Backspace,
        winit::keyboard::NamedKey::Delete => akar_core::Key::Delete,
        winit::keyboard::NamedKey::ArrowLeft => akar_core::Key::Left,
        winit::keyboard::NamedKey::ArrowRight => akar_core::Key::Right,
        winit::keyboard::NamedKey::ArrowUp => akar_core::Key::Up,
        winit::keyboard::NamedKey::ArrowDown => akar_core::Key::Down,
        winit::keyboard::NamedKey::Home => akar_core::Key::Home,
        winit::keyboard::NamedKey::End => akar_core::Key::End,
        winit::keyboard::NamedKey::Enter => akar_core::Key::Enter,
        winit::keyboard::NamedKey::Escape => akar_core::Key::Escape,
        winit::keyboard::NamedKey::Tab => akar_core::Key::Tab,
        _ => return None,
    })
}

pub fn process_window_event(input: &mut InputState, event: &WindowEvent) {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            input.set_mouse_pos(position.x as f32, position.y as f32);
        }
        WindowEvent::MouseInput { state, button, .. } => {
            let btn = match button {
                MouseButton::Left => 0,
                MouseButton::Right => 1,
                MouseButton::Middle => 2,
                _ => return,
            };
            input.push_mouse_button(btn, *state == ElementState::Pressed);
        }
        WindowEvent::MouseWheel { delta, .. } => match delta {
            MouseScrollDelta::LineDelta(x, y) => input.push_scroll(*x * 20.0, *y * 20.0),
            MouseScrollDelta::PixelDelta(p) => input.push_scroll(p.x as f32, p.y as f32),
        },
        WindowEvent::KeyboardInput { event, .. } => {
            if let Some(text) = &event.text {
                for c in text.chars() {
                    if is_committed_text_char(c) {
                        input.push_char(c);
                    }
                }
            }
            if event.state == ElementState::Pressed {
                if let winit::keyboard::Key::Named(name) = &event.logical_key {
                    if let Some(key) = named_key(name) {
                        input.push_key(key);
                    }
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_text_filters_editing_and_control_characters() {
        assert!(!is_committed_text_char('\u{08}'));
        assert!(!is_committed_text_char('\u{7F}'));
        assert!(!is_committed_text_char('\n'));
        assert!(!is_committed_text_char('\t'));
        assert!(!is_committed_text_char('\u{E000}'));
        assert!(!is_committed_text_char('\u{F0000}'));
        assert!(!is_committed_text_char('\u{100000}'));
        assert!(is_committed_text_char('A'));
        assert!(is_committed_text_char('é'));
        assert!(is_committed_text_char('😀'));
        assert!(is_committed_text_char(' '));
    }

    #[test]
    fn named_editing_keys_are_preserved() {
        assert_eq!(
            named_key(&winit::keyboard::NamedKey::Backspace),
            Some(akar_core::Key::Backspace)
        );
        assert_eq!(
            named_key(&winit::keyboard::NamedKey::Delete),
            Some(akar_core::Key::Delete)
        );
        assert_eq!(
            named_key(&winit::keyboard::NamedKey::Enter),
            Some(akar_core::Key::Enter)
        );
        assert_eq!(
            named_key(&winit::keyboard::NamedKey::Tab),
            Some(akar_core::Key::Tab)
        );
        assert_eq!(
            named_key(&winit::keyboard::NamedKey::ArrowLeft),
            Some(akar_core::Key::Left)
        );
    }
}
