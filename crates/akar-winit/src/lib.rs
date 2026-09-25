use akar_core::{FileDragInput, InputState, Key, KeyEvent, Modifiers};
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

fn modifiers(state: winit::keyboard::ModifiersState) -> Modifiers {
    Modifiers {
        shift: state.shift_key(),
        control: state.control_key(),
        alt: state.alt_key(),
        super_key: state.super_key(),
    }
}

fn logical_key(key: &winit::keyboard::Key) -> Option<Key> {
    match key {
        winit::keyboard::Key::Named(name) => named_key(name),
        winit::keyboard::Key::Character(text) => {
            let mut chars = text.chars();
            let c = chars.next()?;
            (c.is_ascii_alphabetic() && chars.next().is_none())
                .then_some(Key::Character(c.to_ascii_lowercase()))
        }
        _ => None,
    }
}

fn physical_latin_key(key: winit::keyboard::PhysicalKey) -> Option<Key> {
    let winit::keyboard::PhysicalKey::Code(code) = key else {
        return None;
    };
    let c = match code {
        winit::keyboard::KeyCode::KeyA => 'a',
        winit::keyboard::KeyCode::KeyB => 'b',
        winit::keyboard::KeyCode::KeyC => 'c',
        winit::keyboard::KeyCode::KeyD => 'd',
        winit::keyboard::KeyCode::KeyE => 'e',
        winit::keyboard::KeyCode::KeyF => 'f',
        winit::keyboard::KeyCode::KeyG => 'g',
        winit::keyboard::KeyCode::KeyH => 'h',
        winit::keyboard::KeyCode::KeyI => 'i',
        winit::keyboard::KeyCode::KeyJ => 'j',
        winit::keyboard::KeyCode::KeyK => 'k',
        winit::keyboard::KeyCode::KeyL => 'l',
        winit::keyboard::KeyCode::KeyM => 'm',
        winit::keyboard::KeyCode::KeyN => 'n',
        winit::keyboard::KeyCode::KeyO => 'o',
        winit::keyboard::KeyCode::KeyP => 'p',
        winit::keyboard::KeyCode::KeyQ => 'q',
        winit::keyboard::KeyCode::KeyR => 'r',
        winit::keyboard::KeyCode::KeyS => 's',
        winit::keyboard::KeyCode::KeyT => 't',
        winit::keyboard::KeyCode::KeyU => 'u',
        winit::keyboard::KeyCode::KeyV => 'v',
        winit::keyboard::KeyCode::KeyW => 'w',
        winit::keyboard::KeyCode::KeyX => 'x',
        winit::keyboard::KeyCode::KeyY => 'y',
        winit::keyboard::KeyCode::KeyZ => 'z',
        _ => return None,
    };
    Some(Key::Character(c))
}

/// Translates winit input into akar input. In winit 0.30, file events carry
/// paths but no drag position, so drops stay window-level and cannot target a
/// layout node. Hosts with native drag coordinates can submit `FileDragInput`
/// directly through `InputState::push_file_drag`.
pub fn process_window_event(input: &mut InputState, event: &WindowEvent) {
    match event {
        WindowEvent::DroppedFile(path) => {
            input.push_file_drag(FileDragInput::UnpositionedDrop {
                paths: vec![path.clone()],
            });
        }
        WindowEvent::HoveredFileCancelled => input.push_file_drag(FileDragInput::Leave),
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
                let key = logical_key(&event.logical_key)
                    .or_else(|| physical_latin_key(event.physical_key));
                if let Some(key) = key {
                    input.push_key_event(KeyEvent {
                        key,
                        modifiers: input.modifiers,
                        repeat: event.repeat,
                    });
                }
            }
        }
        WindowEvent::ModifiersChanged(state) => input.modifiers = modifiers(state.state()),
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

    #[test]
    fn logical_shortcut_keys_are_layout_aware_and_lowercase() {
        assert_eq!(
            logical_key(&winit::keyboard::Key::Character("A".into())),
            Some(Key::Character('a'))
        );
        assert_eq!(
            logical_key(&winit::keyboard::Key::Character("é".into())),
            None
        );
    }

    #[test]
    fn physical_latin_fallback_covers_shortcut_letters() {
        assert_eq!(
            physical_latin_key(winit::keyboard::PhysicalKey::Code(
                winit::keyboard::KeyCode::KeyC
            )),
            Some(Key::Character('c'))
        );
    }

    #[test]
    fn modifier_snapshot_is_independent_of_later_changes() {
        let mut input = InputState::new();
        input.modifiers = Modifiers {
            super_key: true,
            ..Modifiers::default()
        };
        input.push_key_event(KeyEvent {
            key: Key::Character('v'),
            modifiers: input.modifiers,
            repeat: false,
        });
        input.modifiers = Modifiers::default();
        assert!(input.key_events[0].modifiers.super_key);
    }

    #[test]
    fn winit_file_drop_remains_unpositioned_despite_last_pointer_position() {
        let mut input = InputState::new();
        input.set_mouse_pos(42.0, 17.0);
        process_window_event(&mut input, &WindowEvent::HoveredFile("first".into()));
        process_window_event(&mut input, &WindowEvent::DroppedFile("first".into()));
        process_window_event(&mut input, &WindowEvent::DroppedFile("second".into()));

        assert!(input.file_drag_position.is_none());
        assert!(input.file_drop_events.is_empty());
        assert_eq!(
            input.unpositioned_file_drops,
            [
                vec![std::path::PathBuf::from("first")],
                vec![std::path::PathBuf::from("second")],
            ]
        );
        assert!(input.claim_file_drops(|_| true).is_empty());
    }

    #[test]
    fn native_host_positioned_events_are_independent_of_winit_pointer_state() {
        let mut input = InputState::new();
        input.set_mouse_pos(500.0, 500.0);
        input.push_file_drag(FileDragInput::Enter {
            position: [12.0, 24.0],
            paths: vec!["first".into()],
        });
        assert_eq!(input.file_drag_position, Some([12.0, 24.0]));
        input.push_file_drag(FileDragInput::Drop {
            position: [14.0, 26.0],
            paths: vec!["first".into(), "second".into()],
        });
        assert!(input
            .claim_file_drops(|position| position == [500.0, 500.0])
            .is_empty());
        assert_eq!(
            input.claim_file_drops(|position| position == [14.0, 26.0]),
            [
                std::path::PathBuf::from("first"),
                std::path::PathBuf::from("second"),
            ]
        );
    }
}
