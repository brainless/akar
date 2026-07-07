# Epic 012: Form Controls and Text Input

**Status:** Done
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

---

## Review Notes

### Round 1 (2026-07-07)

**Task 1 — Key enum + push_key (akar-core):**
- Added `Key` enum (11 variants: Backspace, Delete, Left, Right, Up, Down, Home, End, Enter, Escape, Tab) in `input.rs:3`
- Added `keys_pressed: Vec<Key>` field to `InputState`
- Added `push_key(&mut self, key: Key)` method
- `begin_frame()` clears `keys_pressed`
- `Key` re-exported from `akar-core/src/lib.rs`

**Task 3 — Checkbox:**
- File: `crates/akar-components/src/checkbox.rs` (118 lines)
- Signature: `checkbox(core, layout, node_id, checked: &mut bool, label: &str, theme) -> bool`
- 18×18 box + `"\u{2713}"` glyph + label text; tick toggles on click
- Zero-area guard, hover border lighten via `scale_color`

**Task 4 — Radio Group:**
- File: `crates/akar-components/src/radio.rs` (121 lines)
- Signature: `radio_group(core, layout, nodes: &[NodeId], labels: &[&str], selected: &mut usize, theme) -> bool`
- 16×16 outer circle per item, 8×8 inner fill for selected; zero-area nodes skipped

**Task 5 — Switch:**
- File: `crates/akar-components/src/switch.rs` (98 lines)
- Signature: `switch(core, layout, node_id, on: &mut bool, theme) -> bool`
- 36×20 track (pill), 16×16 thumb at left/off or right/on; click toggles

**Task 6 — Slider:**
- File: `crates/akar-components/src/slider.rs` (121 lines)
- Signature: `slider(core, layout, node_id, value: &mut f32, min: f32, max: f32, theme) -> bool`
- Full-width track + fill portion + 14×14 thumb; drag via `is_pressed`, clamps `[min, max]`

**Task 7 — Select:**
- File: `crates/akar-components/src/select.rs` (207 lines)
- Signature: `select(core, layout, node_id, options: &[&str], selected: &mut usize, open: &mut bool, theme, viewport_rect: [f32; 4]) -> bool`
- Closed: bordered rect + selected text + `"\u{25BC}"` chevron. Open: delegates to `dropdown_begin/end`, renders up to 4 items with hover highlight; click-outside closes

**`scale_color` moved to `color.rs`** (previously local to `button.rs`) — shared by checkbox, radio, switch, select.

`cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` all pass.

**Files changed in Round 1:**
- `crates/akar-core/src/input.rs`
- `crates/akar-core/src/lib.rs`
- `crates/akar-components/src/color.rs`
- `crates/akar-components/src/lib.rs`
- `crates/akar-components/src/checkbox.rs` (new)
- `crates/akar-components/src/radio.rs` (new)
- `crates/akar-components/src/switch.rs` (new)
- `crates/akar-components/src/slider.rs` (new)
- `crates/akar-components/src/select.rs` (new)

### Round 2 (2026-07-07)

**Task 2 — Winit key translation (akar-winit):**
- Extended `KeyboardInput` handler: on `ElementState::Pressed`, matches `event.logical_key` against `Key::Named(named)` for all 11 `NamedKey` variants
- Maps to `akar_core::Key` and calls `input.push_key(key)`
- Uses `logical_key` (correct for compose/IME-processed keys)

**Task 8 — TextInput:**
- File: `crates/akar-components/src/text_input.rs` (233 lines)
- `TextInputResponse { changed, submitted }`
- Focus on click, click-outside defocus; char insertion, Backspace/Delete with UTF-8 char boundary safety; Left/Right/Home/End cursor movement; Enter → `submitted`; Escape → defocus
- Cursor quad (2px, `theme.primary`) controlled by `cursor_visible`
- Placeholder text in muted color when empty + not focused
- UTF-8 char boundary helpers `prev_char_boundary`/`next_char_boundary`

**Task 9 — Textarea:**
- File: `crates/akar-components/src/textarea.rs` (288 lines)
- `TextAreaResponse { changed }`
- All TextInput key handling, plus Enter inserts `'\n'`; Up/Down cursor between lines; Home/End to line start/end
- `scroll_y` for vertical scroll (mouse wheel + caller-managed); scissor push/pop for clipping
- Cursor Y tracks line count from newlines

`cargo check --workspace` and `cargo test --workspace` pass clean.

**Files changed in Round 2:**
- `crates/akar-winit/src/lib.rs`
- `crates/akar-components/src/text_input.rs` (new)
- `crates/akar-components/src/textarea.rs` (new)
- `crates/akar-components/src/lib.rs`

### Round 3 (2026-07-07)

**Task 10 — C ABI wrappers (akar-c-api):**
- Added `pub const AKAR_KEY_BACKSPACE`..`AKAR_KEY_TAB` (11 key constants) → generate C `#define` macros in `akar.h`
- `akar_push_key(ctx, key: u32)` — routes u32 → `Key` enum → `InputState::push_key`
- `akar_checkbox(ctx, node_id, label, label_len, checked) -> bool`
- `akar_radio_group(ctx, nodes, node_count, labels, label_lengths, selected) -> bool`
- `akar_switch(ctx, node_id, on) -> bool`
- `akar_slider(ctx, node_id, value, min, max) -> bool`
- `akar_select(ctx, node_id, options, option_count, option_lengths, selected, open, viewport) -> AkarSelectResponse`
- `akar_text_input(ctx, node_id, value_buf, buf_len, cursor_pos, placeholder, cursor_visible) -> AkarTextInputResponse`
- `akar_textarea(ctx, node_id, value_buf, buf_len, cursor_pos, scroll_y, placeholder, cursor_visible) -> AkarTextAreaResponse`
- New C structs: `AkarSelectResponse`, `AkarTextInputResponse`, `AkaraTextAreaResponse`
- Import updated to `use akar_core::{AkarCore, Key}`
- `akar.h` regenerated (329 lines, up from 247)

`cargo check --workspace` and `cargo build -p akar-c-api` pass clean. Header regenerated.

**Files changed in Round 3:**
- `crates/akar-c-api/src/lib.rs`
- `akar.h` (regenerated)

### Round 4 (2026-07-07)

**Task 11 — Demo Form tab (demo-rust):**
- Added 21 new state fields for form values, cursor state, and layout nodes
- Created form layout nodes in `resumed()`: container (flex-column), name input (40px), textarea (100px), checkbox row (32px), radio group row (flex-row, 32px, Dark/Light), switch (32px), slider (32px), select (40px), submit button (120×36px, centered)
- Extended tab bar labels to `["List", "Canvas", "Stats", "Form"]`
- Added `3 =>` arm to panel children switching
- Form rendering: cursor blink (30-frame interval), "Form Demo" title, label texts, all 7 form controls bound to state fields
- Submit button: success toast if agreed, warning toast if not

`cargo check --workspace` passes clean.

**Files changed in Round 4:**
- `examples/demo-rust/src/main.rs`

---

### Epic Complete

All acceptance criteria verified:
- [x] `Key` enum + `push_key` on `InputState`
- [x] `akar-winit` translates all named keys
- [x] `checkbox`, `radio_group`, `switch`, `slider`, `select`, `text_input`, `textarea` implemented
- [x] Cursor blink timer owned by caller (`cursor_visible: bool`)
- [x] `cursor_pos: &mut usize` updated correctly
- [x] All components exposed in `akar.h`
- [x] `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass clean
- [x] No text selection, clipboard, or IME (deferred)
- [x] Demo updated with Form tab containing all controls
