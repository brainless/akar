# Epic 012: Form Controls and Text Input

**Status:** Planned
**Goal:** All interactive input components. Discrete controls (Checkbox, Radio, Switch, Slider, Select) come first because they require only existing input infrastructure. Text input components (TextInput, Textarea) come last because they require extending `InputState` and `akar-winit` with named key events.

**Prerequisite:** Epic 011 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean.

---

## Scope

### Part A: Discrete Controls (no key event changes needed)

#### Checkbox
A square toggle with a checkmark. `checkbox(ctx, layout, node_id, checked: &mut bool, label: &str, theme) -> bool` — returns true if the value changed this frame. Renders a bordered square; when checked, renders a filled box or an X/check mark glyph via label.

#### Radio
A set of mutually exclusive options. The caller passes the full list of labels and the active index. `radio_group(ctx, layout, nodes: &[NodeId], labels: &[&str], selected: &mut usize, theme) -> bool` — returns true if the selection changed.

#### Switch (Toggle)
A sliding on/off toggle. `switch(ctx, layout, node_id, on: &mut bool, theme) -> bool`. Renders as a rounded track with a thumb that sits at the left (off) or right (on) end. No animation — the thumb position is binary. Animated thumb is deferred.

#### Slider
A horizontal range input. `slider(ctx, layout, node_id, value: &mut f32, min: f32, max: f32, theme) -> bool`. Renders a track and a draggable thumb. Drag interaction uses `core.input.mouse_buttons` and `core.input.mouse_pos`. Returns true if the value changed.

#### Select
A dropdown selector built on the `dropdown_begin/end` primitive from Epic 011. `select(ctx, layout, node_id, options: &[&str], selected: &mut usize, theme) -> bool`. When clicked, opens a dropdown below the node rect. When an option is selected, closes and updates `*selected`.

The caller must pass a `viewport_rect` so the dropdown can position correctly. The open/closed state is owned by the caller (a `bool`), matching the same pattern as drawer and modal.

---

### Part B: Named Key Events

Before TextInput and Textarea can be implemented, `InputState` needs structured key events beyond printable characters.

**Extension to `akar-core/src/input.rs`:**

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
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

// Add to InputState:
pub keys_pressed: Vec<Key>,
```

`push_key(key: Key)` is the new `InputState` method for named key events.

**Extension to `akar-winit/src/lib.rs`:**

The existing `KeyboardInput` branch handles only `event.text` (printable characters). Extend to also translate `winit::keyboard::KeyCode` to `akar_core::Key` for the named keys listed above, calling `input.push_key(key)` for each.

The mapping is straightforward:
- `winit::keyboard::Key::Named(NamedKey::Backspace)` → `Key::Backspace`
- `winit::keyboard::Key::Named(NamedKey::ArrowLeft)` → `Key::Left`
- ... and so on for all members of the `Key` enum.

---

### Part C: Text Input Components

#### TextInput (single-line)

`text_input(ctx, layout, node_id, value: &mut String, placeholder: &str, theme) -> TextInputResponse`

`TextInputResponse { changed: bool, submitted: bool }` — `changed` is true if the text changed; `submitted` is true if Enter was pressed.

**Behavior:**
- Click to focus: sets `core.input.focused_id = Some(node_id_as_u64)`.
- When focused: printable chars from `core.input.chars` are appended at the cursor. Named keys: Backspace deletes the character before the cursor; Delete deletes the character after; Left/Right move the cursor; Home/End jump to start/end; Enter sets `submitted`; Escape clears focus.
- Renders: a bordered rect (focus ring when focused, using `theme.primary` as border color), the current text value, and a blinking cursor quad. Cursor position tracks the byte index in the string.

**Cursor blink:** Caller-managed. akar does not own a timer. The caller passes `cursor_visible: bool` to `text_input` (based on their own elapsed time modulo blink interval). This keeps akar stateless and predictable.

**v1 scope:** No text selection, no clipboard, no IME. Single-line only. All of these are deferred.

#### Textarea (multi-line)

`textarea(ctx, layout, node_id, value: &mut String, placeholder: &str, theme) -> TextAreaResponse`

Same key handling as TextInput except Enter inserts a newline rather than submitting. The text wraps within the node width. Scroll within the text area is vertical only and caller-managed (same `scroll_y: &mut f32` pattern as `scroll_area_begin`).

**v1 scope:** No selection, no clipboard, no IME. Line wrap is display-only — the text value is a flat string with embedded newlines.

---

## Key Design Decisions

### Cursor State is Caller-Owned

The cursor byte position (`cursor_pos: usize`) is passed by the caller as `&mut usize` alongside the string. This matches the caller-owned state pattern established by `scroll_y` in Epic 008. The caller initializes it to `0` and akar updates it.

Rationale: cursor position is persistent state that survives across frames. Retaining it inside akar would require a HashMap<node_id, usize>. Caller ownership is simpler, explicit, and avoids any state-invalidation edge cases when the text value changes externally.

### Focus is Already in InputState

`InputState.focused_id: Option<u64>` already exists from the initial design. Text input sets `focused_id` to its node ID on click (left button press inside the rect) and clears it on Escape or click-outside. Click-outside detection: if `core.input.mouse_buttons_pressed[0]` is true and the press is not inside the text input rect, clear focus.

### `akar-winit` Key Translation

`winit`'s keyboard event model distinguishes between `Key::Named(NamedKey)` (semantic keys) and `Key::Character(SmolStr)` (printable characters). The existing `event.text` path handles printable characters already. The new code handles `Key::Named` for the nine keys in the `Key` enum. Both paths run for every `WindowEvent::KeyboardInput` event — they are not mutually exclusive.

### Select Uses Dropdown from Epic 011

`select` is a thin wrapper over `dropdown_begin/end`. It renders the closed state (a bordered rect with the selected option text and a chevron icon) and delegates the open state to the dropdown primitive. No new rendering infrastructure needed.

---

## C ABI

All discrete controls: `akar_checkbox`, `akar_radio_group`, `akar_switch`, `akar_slider`, `akar_select`.

Key event: `akar_push_key(ctx, key: u32)` — numeric constants for each `Key` variant defined in `akar.h`.

Text input: `akar_text_input(ctx, node_id, value_buf, buf_len, cursor_pos, placeholder, cursor_visible) -> AkarTextInputResponse`

`AkarTextInputResponse { changed: bool, submitted: bool, new_cursor_pos: u32 }`.

Textarea: `akar_textarea(ctx, node_id, value_buf, buf_len, cursor_pos, scroll_y, placeholder, cursor_visible) -> AkarTextAreaResponse`.

---

## Demo

The demo gains a "Form" tab (added to the tab bar from Epic 010) containing:
- A text input for a name field with a placeholder.
- A textarea for notes.
- A checkbox for an agreement toggle.
- A radio group for theme selection (Dark / Light).
- A switch for enabling notifications.
- A slider for font size.
- A select for language.
- A submit button that shows a success toast on click.

All form values are owned by the demo's app state struct.

---

## Acceptance Criteria

- [ ] `Key` enum and `push_key` method added to `InputState`; `begin_frame` clears `keys_pressed`.
- [ ] `akar-winit` translates all nine named keys from winit `KeyCode` to `akar_core::Key`.
- [ ] `checkbox` toggles its value on click; renders check glyph when true.
- [ ] `radio_group` selects the clicked option; only one option active at a time.
- [ ] `switch` toggles on click; thumb position matches the boolean state.
- [ ] `slider` tracks mouse drag; value clamps to `[min, max]`; returns `changed = true` on drag.
- [ ] `select` opens a dropdown on click; selecting an option closes it and updates `*selected`.
- [ ] `text_input` inserts printable chars; Backspace deletes; Left/Right move cursor; Enter sets `submitted`.
- [ ] `textarea` inserts printable chars and newlines; Backspace/Delete work; cursor tracks correctly.
- [ ] `cursor_visible: bool` controls cursor quad rendering — akar does not blink internally.
- [ ] `cursor_pos: &mut usize` is updated correctly after all key operations.
- [ ] All components exposed in `akar.h` with the signatures above.
- [ ] `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass clean.
- [ ] No text selection, no clipboard, no IME — deferred per this epic's v1 scope.
