use std::time::{Duration, Instant};

use akar_core::{
    InputState, Key, KeyEvent, Modifiers, Shortcut, ShortcutModifiers, TextEditKeybindings,
};
use akar_layout::Layout;

#[derive(Clone, Debug, PartialEq)]
pub enum HoverTarget {
    Coords(f32, f32),
    Label(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScriptStep {
    Hover(HoverTarget),
    Press(MouseButton),
    Release(MouseButton),
    Click(HoverTarget),
    Scroll(f32, f32),
    Key(KeyEvent),
    TextBindings(TextEditKeybindings),
    Type(String),
    Paste(HoverTarget, String),
    Delay(f64),
    Screenshot(String),
}

fn button_index(b: MouseButton) -> usize {
    match b {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}

fn parse_button(s: &str) -> Result<MouseButton, String> {
    match s {
        "left" => Ok(MouseButton::Left),
        "middle" => Ok(MouseButton::Middle),
        "right" => Ok(MouseButton::Right),
        other => Err(format!("unknown mouse button '{other}'")),
    }
}

fn parse_key(s: &str) -> Result<Key, String> {
    match s {
        "Backspace" => Ok(Key::Backspace),
        "Delete" => Ok(Key::Delete),
        "Left" => Ok(Key::Left),
        "Right" => Ok(Key::Right),
        "Up" => Ok(Key::Up),
        "Down" => Ok(Key::Down),
        "Home" => Ok(Key::Home),
        "End" => Ok(Key::End),
        "Enter" => Ok(Key::Enter),
        "Escape" => Ok(Key::Escape),
        "Tab" => Ok(Key::Tab),
        "PageUp" => Ok(Key::PageUp),
        "PageDown" => Ok(Key::PageDown),
        other if other.chars().count() == 1 => {
            let character = other.chars().next().unwrap();
            Ok(Key::Character(character.to_ascii_lowercase()))
        }
        other => Err(format!("unknown key '{other}'")),
    }
}

fn parse_shortcut(s: &str) -> Result<(Shortcut, Modifiers), String> {
    let mut modifiers = ShortcutModifiers::NONE;
    let mut event_modifiers = Modifiers::default();
    let mut key = None;
    for part in s.split('+') {
        match part {
            "Primary" => {
                modifiers |= ShortcutModifiers::PRIMARY;
                if cfg!(target_os = "macos") {
                    event_modifiers.super_key = true;
                } else {
                    event_modifiers.control = true;
                }
            }
            "Control" => {
                modifiers |= ShortcutModifiers::CONTROL;
                event_modifiers.control = true;
            }
            "Super" => {
                modifiers |= ShortcutModifiers::SUPER;
                event_modifiers.super_key = true;
            }
            "Alt" => {
                modifiers |= ShortcutModifiers::ALT;
                event_modifiers.alt = true;
            }
            "Shift" => {
                modifiers |= ShortcutModifiers::SHIFT;
                event_modifiers.shift = true;
            }
            other => {
                if key.replace(parse_key(other)?).is_some() {
                    return Err(format!("shortcut '{s}' has more than one key"));
                }
            }
        }
    }
    let key = key.ok_or_else(|| format!("shortcut '{s}' requires a key"))?;
    Ok((Shortcut::new(modifiers, key), event_modifiers))
}

fn parse_quoted(line: &str) -> Result<String, String> {
    let start = line
        .find('"')
        .ok_or_else(|| "type command requires a quoted string".to_string())?;
    let rest = &line[start + 1..];
    let end = rest
        .find('"')
        .ok_or_else(|| "type command requires a closing quote".to_string())?;
    let mut parsed = String::new();
    let mut chars = rest[..end].chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            parsed.push(c);
            continue;
        }
        let escaped = chars
            .next()
            .ok_or_else(|| "quoted string ends with an escape".to_string())?;
        parsed.push(match escaped {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '\\' => '\\',
            '"' => '"',
            other => return Err(format!("unsupported escape '\\{other}'")),
        });
    }
    Ok(parsed)
}

pub fn parse_script(input: &str) -> Result<Vec<ScriptStep>, String> {
    let mut steps = Vec::new();
    for (i, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("type") {
            steps.push(ScriptStep::Type(parse_quoted(rest)?));
            continue;
        }
        if let Some(rest) = line.strip_prefix("paste") {
            let mut parts = rest.split_whitespace();
            let target = parts
                .next()
                .ok_or_else(|| format!("line {}: paste requires a target", i + 1))?;
            let label = target
                .strip_prefix('@')
                .ok_or_else(|| format!("line {}: paste target must be a label", i + 1))?;
            steps.push(ScriptStep::Paste(
                HoverTarget::Label(label.to_string()),
                parse_quoted(rest)?,
            ));
            continue;
        }
        let mut parts = line.split_whitespace();
        let cmd = parts.next().unwrap();
        let step = match cmd {
            "hover" | "click" => {
                let arg = parts
                    .next()
                    .ok_or_else(|| format!("line {}: {} requires a target", i + 1, cmd))?;
                let target = if let Some(label) = arg.strip_prefix('@') {
                    if parts.next().is_some() {
                        return Err(format!(
                            "line {}: unexpected token after label target",
                            i + 1
                        ));
                    }
                    HoverTarget::Label(label.to_string())
                } else {
                    let x = arg
                        .parse::<f32>()
                        .map_err(|_| format!("line {}: invalid x coordinate", i + 1))?;
                    let y = parts
                        .next()
                        .ok_or_else(|| format!("line {}: {} requires y coordinate", i + 1, cmd))?
                        .parse::<f32>()
                        .map_err(|_| format!("line {}: invalid y coordinate", i + 1))?;
                    HoverTarget::Coords(x, y)
                };
                if cmd == "hover" {
                    ScriptStep::Hover(target)
                } else {
                    ScriptStep::Click(target)
                }
            }
            "press" | "release" => {
                let btn = parse_button(parts.next().unwrap_or("left"))?;
                if cmd == "press" {
                    ScriptStep::Press(btn)
                } else {
                    ScriptStep::Release(btn)
                }
            }
            "scroll" => {
                let dx = parts
                    .next()
                    .ok_or_else(|| format!("line {}: scroll requires dx", i + 1))?
                    .parse::<f32>()
                    .map_err(|_| format!("line {}: invalid dx", i + 1))?;
                let dy = parts
                    .next()
                    .ok_or_else(|| format!("line {}: scroll requires dy", i + 1))?
                    .parse::<f32>()
                    .map_err(|_| format!("line {}: invalid dy", i + 1))?;
                ScriptStep::Scroll(dx, dy)
            }
            "key" => {
                let name = parts
                    .next()
                    .ok_or_else(|| format!("line {}: key requires a name", i + 1))?;
                let (shortcut, modifiers) = parse_shortcut(name)?;
                ScriptStep::Key(KeyEvent {
                    key: shortcut.key,
                    modifiers,
                    repeat: false,
                })
            }
            "text-bindings" => {
                let select_all = parts.next().ok_or_else(|| {
                    format!("line {}: text-bindings requires three shortcuts", i + 1)
                })?;
                let copy = parts.next().ok_or_else(|| {
                    format!("line {}: text-bindings requires three shortcuts", i + 1)
                })?;
                let paste = parts.next().ok_or_else(|| {
                    format!("line {}: text-bindings requires three shortcuts", i + 1)
                })?;
                if parts.next().is_some() {
                    return Err(format!(
                        "line {}: text-bindings accepts exactly three shortcuts",
                        i + 1
                    ));
                }
                ScriptStep::TextBindings(TextEditKeybindings {
                    select_all: parse_shortcut(select_all)?.0,
                    copy: parse_shortcut(copy)?.0,
                    paste: parse_shortcut(paste)?.0,
                })
            }
            "delay" => {
                let secs = parts
                    .next()
                    .ok_or_else(|| format!("line {}: delay requires seconds", i + 1))?
                    .parse::<f64>()
                    .map_err(|_| format!("line {}: invalid seconds", i + 1))?;
                ScriptStep::Delay(secs)
            }
            "screenshot" => {
                let path = parts
                    .next()
                    .ok_or_else(|| format!("line {}: screenshot requires a path", i + 1))?;
                ScriptStep::Screenshot(path.to_string())
            }
            other => return Err(format!("line {}: unknown command '{other}'", i + 1)),
        };
        steps.push(step);
    }
    Ok(steps)
}

fn apply_target(input: &mut InputState, target: &HoverTarget, layout: &Layout) {
    match target {
        HoverTarget::Coords(x, y) => input.set_mouse_pos(*x, *y),
        HoverTarget::Label(name) => {
            if let Some(node) = layout.resolve_label(name) {
                let r = layout.rect(node);
                input.set_mouse_pos(r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
            }
        }
    }
}

pub struct ScriptRunner {
    steps: Vec<ScriptStep>,
    cursor: usize,
    delay_deadline: Option<Instant>,
}

impl ScriptRunner {
    pub fn new(steps: Vec<ScriptStep>) -> Self {
        Self {
            steps,
            cursor: 0,
            delay_deadline: None,
        }
    }

    pub fn advance(
        &mut self,
        input: &mut InputState,
        text_edit_keybindings: &mut TextEditKeybindings,
        layout: &Layout,
        now: Instant,
    ) -> Option<String> {
        while let Some(ScriptStep::Delay(secs)) = self.steps.get(self.cursor) {
            let deadline = self
                .delay_deadline
                .get_or_insert_with(|| now + Duration::from_secs_f64(*secs));
            if *deadline > now {
                return None;
            }
            self.delay_deadline = None;
            self.cursor += 1;
        }

        if self.cursor >= self.steps.len() {
            return None;
        }

        let step = self.steps[self.cursor].clone();
        self.cursor += 1;

        match step {
            ScriptStep::Screenshot(path) => Some(path),
            ScriptStep::Hover(target) => {
                apply_target(input, &target, layout);
                None
            }
            ScriptStep::Click(target) => {
                apply_target(input, &target, layout);
                input.push_mouse_button(0, true);
                input.push_mouse_button(0, false);
                None
            }
            ScriptStep::Press(b) => {
                input.push_mouse_button(button_index(b), true);
                None
            }
            ScriptStep::Release(b) => {
                input.push_mouse_button(button_index(b), false);
                None
            }
            ScriptStep::Scroll(dx, dy) => {
                input.push_scroll(dx, dy);
                None
            }
            ScriptStep::Key(event) => {
                input.push_key_event(event);
                None
            }
            ScriptStep::TextBindings(bindings) => {
                *text_edit_keybindings = bindings;
                None
            }
            ScriptStep::Type(s) => {
                for c in s.chars() {
                    input.push_char(c);
                }
                None
            }
            ScriptStep::Paste(HoverTarget::Label(name), text) => {
                if let Some(node) = layout.resolve_label(&name) {
                    input.push_paste(layout.widget_id(node), text);
                }
                None
            }
            ScriptStep::Paste(HoverTarget::Coords(_, _), _) => unreachable!(),
            ScriptStep::Delay(_) => unreachable!(),
        }
    }
}
