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

    pub fn is_exhausted(&self) -> bool {
        self.cursor >= self.steps.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hover_and_click_coords() {
        let steps = parse_script("hover 100 200\nclick 50 60\n").unwrap();
        assert_eq!(
            steps,
            vec![
                ScriptStep::Hover(HoverTarget::Coords(100.0, 200.0)),
                ScriptStep::Click(HoverTarget::Coords(50.0, 60.0)),
            ]
        );
    }

    #[test]
    fn parse_label_targets() {
        let steps = parse_script("hover @form_submit\nclick @navbar_btn\n").unwrap();
        assert_eq!(
            steps,
            vec![
                ScriptStep::Hover(HoverTarget::Label("form_submit".to_string())),
                ScriptStep::Click(HoverTarget::Label("navbar_btn".to_string())),
            ]
        );
    }

    #[test]
    fn parse_press_release_default_button() {
        let steps = parse_script("press\nrelease right\npress middle\n").unwrap();
        assert_eq!(
            steps,
            vec![
                ScriptStep::Press(MouseButton::Left),
                ScriptStep::Release(MouseButton::Right),
                ScriptStep::Press(MouseButton::Middle),
            ]
        );
    }

    #[test]
    fn parse_scroll_key_type() {
        let steps =
            parse_script("scroll -10 20\nkey Enter\nkey Escape\ntype \"hello world\"\n").unwrap();
        assert_eq!(
            steps,
            vec![
                ScriptStep::Scroll(-10.0, 20.0),
                ScriptStep::Key(KeyEvent {
                    key: Key::Enter,
                    modifiers: Modifiers::default(),
                    repeat: false,
                }),
                ScriptStep::Key(KeyEvent {
                    key: Key::Escape,
                    modifiers: Modifiers::default(),
                    repeat: false,
                }),
                ScriptStep::Type("hello world".to_string()),
            ]
        );
    }

    #[test]
    fn parse_and_inject_targeted_paste() {
        let steps = parse_script("paste @field \"hello clipboard\"\n").unwrap();
        assert_eq!(
            steps,
            vec![ScriptStep::Paste(
                HoverTarget::Label("field".to_string()),
                "hello clipboard".to_string()
            )]
        );

        use akar_layout::{Layout as L, Style};
        let mut layout = L::new();
        let node = layout.new_leaf(Style::default());
        layout.register_label("field", node);
        let target = layout.widget_id(node);
        let mut runner = ScriptRunner::new(steps);
        let mut input = InputState::new();
        let mut bindings = TextEditKeybindings::default();
        runner.advance(&mut input, &mut bindings, &layout, Instant::now());
        assert_eq!(
            input.pastes_for(target).collect::<Vec<_>>(),
            ["hello clipboard"]
        );
    }

    #[test]
    fn parse_delay_and_screenshot() {
        let steps = parse_script("delay 0.25\nscreenshot /tmp/out.png\n").unwrap();
        assert_eq!(
            steps,
            vec![
                ScriptStep::Delay(0.25),
                ScriptStep::Screenshot("/tmp/out.png".to_string()),
            ]
        );
    }

    #[test]
    fn parse_ignores_comments_and_blanks() {
        let steps = parse_script("# a comment\n\n   \nhover 1 2\n").unwrap();
        assert_eq!(
            steps,
            vec![ScriptStep::Hover(HoverTarget::Coords(1.0, 2.0))]
        );
    }

    #[test]
    fn parse_all_keys() {
        for name in [
            "Backspace",
            "Delete",
            "Left",
            "Right",
            "Up",
            "Down",
            "Home",
            "End",
            "Enter",
            "Escape",
            "Tab",
        ] {
            let line = format!("key {name}");
            assert!(parse_script(&line).is_ok(), "should parse key {name}");
        }
    }

    #[test]
    fn parse_modifier_shortcuts_and_custom_bindings() {
        let steps = parse_script("key Primary+A\nkey Control+c\ntext-bindings Alt+a Alt+c Alt+v\n")
            .unwrap();
        assert!(matches!(
            steps[0],
            ScriptStep::Key(KeyEvent {
                key: Key::Character('a'),
                ..
            })
        ));
        assert!(matches!(steps[2], ScriptStep::TextBindings(_)));
    }

    #[test]
    fn parse_paste_decodes_newline_escapes() {
        let steps = parse_script("paste @field \"first\\r\\nsecond\"\n").unwrap();
        assert!(matches!(
            &steps[0],
            ScriptStep::Paste(_, text) if text == "first\r\nsecond"
        ));
    }

    #[test]
    fn parse_malformed_lines_error() {
        assert!(parse_script("hover onlyone").is_err());
        assert!(parse_script("click @missing x").is_err());
        assert!(parse_script("scroll 1").is_err());
        assert!(parse_script("key Wibble").is_err());
        assert!(parse_script("frobnicate 1 2").is_err());
        assert!(parse_script("delay notanumber").is_err());
        assert!(parse_script("type").is_err());
        assert!(parse_script("type noquotes").is_err());
        assert!(parse_script("type \"unterminated").is_err());
        assert!(parse_script("paste \"text\"").is_err());
        assert!(parse_script("paste 10 20 \"text\"").is_err());
    }

    #[test]
    fn runner_fires_click_same_frame() {
        let layout = Layout::new();
        let mut input = InputState::new();
        let mut bindings = TextEditKeybindings::default();
        let steps = parse_script("click 10 10\n").unwrap();
        let mut runner = ScriptRunner::new(steps);
        let path = runner.advance(&mut input, &mut bindings, &layout, Instant::now());
        assert!(path.is_none());
        assert!(input.is_clicked([0.0, 0.0, 20.0, 20.0]));
    }

    #[test]
    fn runner_delay_blocks_then_advances() {
        use std::thread;
        let steps = parse_script("delay 0.05\nhover 5 5\n").unwrap();
        let mut runner = ScriptRunner::new(steps);
        let mut input = InputState::new();
        let mut bindings = TextEditKeybindings::default();
        let layout = Layout::new();

        let t0 = Instant::now();
        assert!(runner
            .advance(&mut input, &mut bindings, &layout, t0)
            .is_none());
        assert!(!runner.is_exhausted());

        thread::sleep(Duration::from_millis(60));
        let t1 = Instant::now();
        runner.advance(&mut input, &mut bindings, &layout, t1);
        assert!(runner.is_exhausted());
    }

    #[test]
    fn runner_screenshot_returns_path() {
        let steps = parse_script("screenshot /tmp/x.png\n").unwrap();
        let mut runner = ScriptRunner::new(steps);
        let mut input = InputState::new();
        let mut bindings = TextEditKeybindings::default();
        let layout = Layout::new();
        let path = runner.advance(&mut input, &mut bindings, &layout, Instant::now());
        assert_eq!(path, Some("/tmp/x.png".to_string()));
        assert!(runner.is_exhausted());
    }

    #[test]
    fn runner_resolves_label_center() {
        use akar_layout::{length, Display, Layout as L, Size, Style};
        let mut layout = L::new();
        let node = layout.new_leaf(Style {
            display: Display::Flex,
            size: Size {
                width: length(40.0),
                height: length(20.0),
            },
            ..Default::default()
        });
        let root = layout.new_with_children(Style::default(), &[node]);
        layout.compute(root, (Some(100.0), Some(100.0)), |_, _, _, _, _| Size::ZERO);
        layout.register_label("box", node);
        let mut input = InputState::new();
        let mut bindings = TextEditKeybindings::default();
        let steps = parse_script("hover @box\n").unwrap();
        let mut runner = ScriptRunner::new(steps);
        runner.advance(&mut input, &mut bindings, &layout, Instant::now());
        assert_eq!(input.mouse_pos.x, 20.0);
        assert_eq!(input.mouse_pos.y, 10.0);
    }

    #[test]
    fn parse_fixture_file() {
        let contents = include_str!("../scripts/open_dropdown.txt");
        let steps = parse_script(contents).expect("fixture should parse");
        assert_eq!(
            steps,
            vec![
                ScriptStep::Hover(HoverTarget::Label("navbar_dropdown".to_string())),
                ScriptStep::Delay(0.1),
                ScriptStep::Click(HoverTarget::Label("navbar_dropdown".to_string())),
                ScriptStep::Delay(0.2),
                ScriptStep::Hover(HoverTarget::Label("navbar_dropdown".to_string())),
                ScriptStep::Screenshot("/tmp/dropdown_open.png".to_string()),
            ]
        );
    }
}
