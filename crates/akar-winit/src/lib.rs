use akar_core::InputState;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

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
                    input.push_char(c);
                }
            }
            if event.state == ElementState::Pressed {
                match &event.logical_key {
                    winit::keyboard::Key::Named(name) => {
                        let key = match name {
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
                            _ => return,
                        };
                        input.push_key(key);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}
