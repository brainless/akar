# Epic 018: Text Editing, Keybindings, and Clipboard Interop

**Status:** In Progress
**Goal:** Make `text_input` and `textarea` reliable, configurable text editors with selection, platform-standard select/copy/paste shortcuts, and a platform-neutral clipboard boundary.

**Prerequisite:** Epic 017 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean.

---

## Context

Epic 012 delivered initial text editing: focus, character insertion, cursor movement, Backspace/Delete, Home/End, and multiline editing. It explicitly deferred selection and clipboard support.

The implementation now needs a proper text-editing model before adding clipboard behavior:

- A selection must be represented independently of the caret. Select all, copy, paste, typed insertion, and deletion all operate on the same selected range.
- Keyboard shortcuts require named/logical keys and modifier state. Committed text (`InputState::chars`) is not a shortcut API.
- Clipboard access is platform-owned. `akar-core` and `akar-components` must not gain a windowing or OS clipboard dependency.
- Downstream applications need one context-level default binding configuration. Per-widget overrides are deferred.

### Existing Backspace Regression

`akar-winit::process_window_event` currently forwards every `KeyboardInput.event.text` character through `InputState::push_char` before translating named keys. Winit can report non-printable/special keys such as Backspace through its text path. This lets a Backspace event insert a control character and then delete that just-inserted character, leaving the visible value unchanged.

The text bridge must only forward committed printable text. Named editing keys must reach widgets solely through structured key events. This regression must be verified in the real `demo-rust` Form component, not only with direct `InputState::push_key` unit tests.

---

## Design Decisions

### Context-Level Keybinding Defaults

`AkarCore` owns one `TextEditKeybindings` value. Its default is platform standard:

| Action | macOS | Windows/Linux |
|---|---|---|
| Select all | Cmd+A | Ctrl+A |
| Copy | Cmd+C | Ctrl+C |
| Paste | Cmd+V | Ctrl+V |

Applications configure the complete default once on their `AkarCore`/`AkarCtx`. Individual widget calls do not receive a bindings parameter in this epic.

The binding representation uses a semantic primary modifier rather than exposing platform conditionals to every application:

```rust
pub struct Shortcut {
    pub modifiers: ShortcutModifiers,
    pub key: Key,
}

pub struct TextEditKeybindings {
    pub select_all: Shortcut,
    pub copy: Shortcut,
    pub paste: Shortcut,
}
```

`ShortcutModifiers::PRIMARY` matches Command on macOS and Control on Windows/Linux. Explicit Control, Super, Alt, and Shift modifiers remain available for downstream custom bindings. Matching uses the modifier state captured with the key event, not the final modifier state for the frame.

### Input Events

Extend the input model with modifier-aware, logical key events. A representative shape is:

```rust
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
    pub repeat: bool,
}
```

`Key` gains logical character variants needed by configurable shortcuts, while committed text remains a separate input path. `akar-winit` updates modifier state from `WindowEvent::ModifiersChanged` and emits a `KeyEvent` for every keyboard press. The modifier snapshot is necessary when a modifier is pressed and released in the same rendered frame.

Bindings are logical, layout-aware shortcuts. The winit bridge should retain a physical-key fallback for Latin shortcut keys when a keyboard layout cannot provide a logical Latin character, following egui's integration approach.

### Caller-Owned Selection State

Replace cursor-only text-edit state with caller-owned selection state:

```rust
pub struct TextEditState {
    pub cursor: usize,
    pub anchor: usize,
}
```

Both positions are valid UTF-8 byte boundaries. A collapsed selection has `cursor == anchor`; otherwise the selected range is `min(cursor, anchor)..max(cursor, anchor)`.

All edits normalize externally supplied state to valid character boundaries before indexing the string.

### Editing Semantics

- Select all sets `anchor = 0` and `cursor = value.len()`.
- Copy requests the selected substring only. Copy with an empty selection is a no-op.
- Typed text and pasted text replace the current selected range, then collapse the selection after inserted text.
- Backspace and Delete delete the selected range first; with no selection, they retain their existing one-character behavior.
- Left collapses a nonempty selection to its start; Right collapses it to its end. Existing navigation applies when selection is empty.
- Textarea uses the same text-range rules across newline boundaries. Its Home, End, Up, and Down behavior continues to operate on the resulting caret.
- Text input pastes as a single line: normalize CRLF/CR to LF, then replace newlines with spaces. Textarea preserves normalized newlines.

Selection is visible. Render selection backgrounds before text and the caret after text. Geometry must be derived from glyphon's shaped layout, not the current byte-index width estimate, so Unicode, proportional fonts, wrapping, and multiline selections are correct.

### Clipboard Boundary

Clipboard APIs remain outside akar's core/component crates. Components produce requests; the application supplies paste text back as an input event. This keeps all APIs synchronous and does not require a window, runtime, callback, or clipboard crate.

```rust
pub struct TextEditResponse {
    pub changed: bool,
    pub submitted: bool,
    pub copy_text: Option<String>,
    pub request_paste: bool,
}

InputState::push_paste(target: u64, text: impl Into<String>);
```

When the focused widget recognizes its configured Paste shortcut, it returns `request_paste: true`. The host reads the system clipboard and pushes a paste payload for that widget before the next frame. Targeted paste prevents delayed text from landing in a different widget after focus changes.

`akar-winit` remains an input translator, not a clipboard owner. A later optional adapter may translate its platform clipboard directly into these requests/events, but it is not required by this epic.

### C ABI

The C API mirrors Rust's state and context defaults:

- `AkarTextEditState { cursor, anchor }`
- `AkarShortcut` and `AkarTextEditKeybindings`
- `akar_set_text_edit_keybindings(ctx, bindings)`
- `akar_push_paste(ctx, widget_id, utf8, utf8_len)`

Text input and textarea responses report selection state and clipboard requests. Copy output must use a documented caller-provided buffer or context accessor; it must never return a borrowed Rust pointer.

While touching these functions, replace `value_buf + buf_len` with explicit `value_len + value_capacity`. The existing signature cannot distinguish the meaningful UTF-8 length from trailing spare bytes and cannot safely report a capacity-limited paste result. All writes must remain valid UTF-8 and NUL-terminate when capacity permits.

---

## Tasks

### Task 1 — Fix Winit Text Filtering and Backspace Regression

**Status:** Done

- Filter control, private-use, newline, tab, and other non-committed special text before `InputState::push_char`.
- Preserve printable Unicode text input and named editing key delivery.
- Add `akar-winit` tests covering Backspace, Delete, Enter, Tab, and a normal printable character.
- Add a demo script that focuses both form fields, types text, sends Backspace, and records screenshots/frame dumps proving the character was removed.

### Task 2 — Modifier and Key Event Infrastructure

- Add `Modifiers`, `KeyEvent`, character shortcut keys, and per-frame event clearing to `akar-core`.
- Update `akar-winit` for `ModifiersChanged`, logical key translation, modifier snapshots, and physical Latin-key fallback.
- Preserve or deliberately migrate existing `keys_pressed` consumers across the workspace.
- Unit-test same-frame modifier changes and shortcut matching on macOS and non-macOS configurations.

### Task 3 — Context-Level Text Edit Keybindings

- Add `Shortcut`, `TextEditKeybindings`, `platform_default`, and matching helpers.
- Store bindings on `AkarCore` with a clear configuration method.
- Implement default Primary+A/C/V behavior and arbitrary downstream replacement bindings.
- Document that bindings are context-wide in this version.

### Task 4 — Shared Selection and Editing Engine

- Introduce `TextEditState` and shared UTF-8-safe range/edit helpers used by both widgets.
- Migrate TextInput and Textarea from `cursor_pos: &mut usize`.
- Implement select-all, selection deletion/replacement, collapse-on-arrow behavior, and multiline-safe paste normalization.
- Test ASCII and multi-byte Unicode selections, empty values, selection across newlines, Backspace/Delete, and external invalid cursor/anchor positions.

### Task 5 — Selection and Caret Rendering

- Obtain caret and selection geometry from shaped glyphon/cosmic-text layout.
- Render selection backgrounds beneath text and caret above text under the active scissor.
- Verify single-line, wrapped textarea, multiline, scrolled, Unicode, and zero-area cases with `MockDrawList` where possible and component screenshots where visual layout matters.

### Task 6 — Clipboard Request and Paste Injection

- Extend text edit responses with copy output and paste request state.
- Add target-addressed paste injection to `InputState`.
- Ensure copy and paste only affect the focused matching widget and copy never exposes unselected text.
- Add demo clipboard simulation to the scripted-input tool; no OS clipboard access is required for screenshots or unit tests.

### Task 7 — C ABI and Generated Header

- Add binding and selection structs, keybinding setter, and targeted paste input to `akar-c-api`.
- Move C text buffers to explicit logical length and capacity semantics.
- Regenerate `akar.h` with cbindgen; do not hand-edit it.
- Add C integration tests for custom bindings, select-all/copy/paste response behavior, UTF-8 capacity handling, and no-selection copy.

### Task 8 — Demo, Documentation, and Verification

- Configure the Form demo with platform defaults and show visible selection state.
- Extend scripts with modifier shortcuts and injected paste text.
- Capture TextInput and Textarea through `--component form` or a focused isolation target for Backspace, select-all, copy request, paste, and customized bindings.
- Run `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.

---

## Acceptance Criteria

- [ ] Backspace removes the preceding visible character in both Form text widgets through the real winit/demo input path.
- [ ] TextInput and Textarea support selectable UTF-8 text with visible selection rendering.
- [ ] Default Select All, Copy, and Paste use Cmd on macOS and Ctrl on Windows/Linux.
- [ ] Downstream applications can change all three bindings once at context level.
- [ ] Copy operates only on selected text; paste and typed text replace selected text.
- [ ] Clipboard reads/writes remain host-owned; akar core/components have no platform clipboard dependency.
- [ ] C callers can configure bindings, pass paste text, receive copy requests safely, and use explicit buffer length/capacity.
- [ ] Unit, C ABI, scripted demo, and visual verification cover the new behavior.

---

## Notes for Future Work

- Per-widget keybinding overrides and scoped binding stacks.
- Cut, undo/redo, word navigation, Shift-extended keyboard selection, pointer drag selection, double/triple click selection, and select-word/select-line.
- IME composition, dead-key handling, input-method caret rect reporting, and mobile virtual keyboard integration.
- Optional clipboard convenience adapters/callbacks in `akar-winit` or language bindings, built over the request/injection contract.
- Password-field policies that suppress copy/cut and adjust selection rendering.
- Rich-text editing, syntax highlighting, undo history, and large-document virtualization.
- Clipboard command aliases such as Shift+Insert / Ctrl+Insert where platform conventions require them.
