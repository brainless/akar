# Epic 006: Text Pipeline

**Status:** Planned
**Goal:** Fix the text buffer memory leak, add a standalone `label` component, expose text via the C ABI, and update the Rust demo to show real labeled UI. The GPU text pipeline (`TextPipeline`, `DrawList::push_text`, `AkarCore::end_frame` render path) is already complete — this epic wires it correctly into the component layer.

**Prerequisite:** Epic 005 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean before Task 1 begins.

---

## Architecture Decision Records

### ADR-015: NodeId as Text Buffer Key — No Separate Cache

**Decision:** `TextPipeline::set_text` is called with `Some(u64::from(node_id))` instead of `None`. This reuses the existing `HashMap<u64, glyphon::Buffer>` in `TextPipeline` as the buffer cache, keyed directly by the node's numeric ID. No new data structure is introduced.

**Rationale:** `TextPipeline` already deduplicates by buffer ID: if `Some(id)` is passed for an existing key, it updates the buffer in place; if `Some(id)` is passed for a new key, it inserts. Using the node ID directly as the buffer ID makes the cache implicit and zero-overhead — no HashMap lookup beyond what `set_text` already does. The only requirement is that NodeId maps losslessly to `u64`, which is guaranteed by `taffy::NodeId`'s internal representation and the existing `From<u64>` / `Into<u64>` impls.

**Buffer lifetime:** Buffers persist until `TextPipeline::remove_buffer(buffer_id)` is called. Components are not responsible for calling this — it is the caller's responsibility to remove nodes from the layout tree when they are no longer needed, and a future `akar_layout_remove` C function (or a `Layout::remove_all` call between scenes) will clean up the corresponding text buffers. For the scope of this epic, buffer cleanup is out of scope; fixing the leak (creating a new buffer per frame) is the priority.

**Consequences:**
- `button.rs`: `set_text(None, ...)` → `set_text(Some(u64::from(node_id)), ...)` — one line change.
- `label.rs`: same pattern from the start.
- The `// TODO: cache buffer per node_id` comment in `button.rs` is removed.

---

### ADR-016: `label` is a Quad-Free Component

**Decision:** The `label` component renders only a `TextCall` — no background quad, no border. It is the simplest possible text component: text positioned inside its layout rect, clipped to that rect.

**Rationale:** Any colored background or border is the responsibility of a `container` wrapping the label, or the caller pushing a quad separately before calling `label`. Merging background and text into a single component creates variant proliferation (label with bg, label without bg, label with border, etc.). The immediate-mode model already solves this — the caller stacks calls.

**Consequences:**
- `label(core, layout, node_id, text, color, theme)` — six parameters. `color` is a `u32` RGBA hex, so the caller controls text color without needing a full theme override.
- No state enum return — label is purely visual; it has no interaction.
- The label's layout rect is used as the text clip rect, matching button behavior.

---

### ADR-017: Deferred Items

**Text centering / alignment:** `glyphon::Buffer` supports horizontal alignment via `glyphon::Attrs` and per-run attributes. Centering button labels and arbitrary text alignment are deferred — the current left-aligned rendering is correct for an MVP.

**Canvas `push_text`:** `CanvasPainter::push_text` (world-space text rendering) is deferred per ADR-011. It requires `TextPipeline` access inside `canvas_end`, which is a non-trivial change to the end-frame sequence.

**Font loading:** Custom font files are deferred. `glyphon::FontSystem::new()` loads system fonts. Explicit font loading (`font_system.db_mut().load_font_data(...)`) is a future task.

**Text input component:** Deferred to a later epic covering interactive form components.

---

## Tasks

### Task 1: Fix Text Buffer Leak in `button`

**Goal:** Change `button.rs` to pass `Some(node_id_as_u64)` to `set_text`, eliminating the per-frame buffer creation.

**File:** `crates/akar-components/src/button.rs`

Replace:
```rust
// TODO: cache buffer per node_id
let buffer_id = core.text_pipeline.set_text(
    None,
    label,
    glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
    Some(rect[2]),
    None,
);
```

With:
```rust
let buffer_id = core.text_pipeline.set_text(
    Some(node_id.into()),
    label,
    glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
    Some(rect[2]),
    None,
);
```

`node_id.into()` converts `taffy::NodeId` to `u64`, which is the key `TextPipeline` uses internally.

**Acceptance criteria:** `cargo test -p akar-components` passes. The button test in `button.rs` continues to pass. Running `demo-rust` for multiple frames shows stable buffer count (no growth in `text_pipeline.buffers.len()`).

**Review:** Done. `set_text(None, ...)` → `set_text(Some(node_id.into()), ...)`, TODO removed. Clippy clean, all 10 tests pass.

---

### Task 2: `label` Component

**Goal:** A standalone text rendering component. No background. Uses the same buffer-caching pattern as the fixed button.

**File:** `crates/akar-components/src/label.rs`

```rust
use akar_core::{AkarCore, TextCall};
use akar_layout::{Layout, NodeId};
use crate::color::color_to_f32;
use crate::AkarTheme;

/// Renders text at the node's layout rect, clipped to that rect.
/// No background or border — wrap in `container` if a background is needed.
pub fn label(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    color: u32,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);

    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let buffer_id = core.text_pipeline.set_text(
        Some(node_id.into()),
        text,
        glyphon::Metrics::new(theme.font_size_base, theme.font_size_base * 1.2),
        Some(rect[2]),
        None,
    );

    core.draw_list.push_text(TextCall {
        buffer_id,
        x: rect[0],
        y: rect[1],
        clip: rect,
        color: color_to_f32(color),
        z: 0.0,
    });
}
```

**`crates/akar-components/src/lib.rs`** — add:

```rust
pub mod label;
pub use label::label;
```

**Tests** — add `#[cfg(test)]` block at the bottom of `label.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::AKAR_THEME_DARK;
    use akar_layout::Style;

    #[test]
    fn zero_area_does_not_push_text() {
        let mut layout = akar_layout::Layout::new();
        let node_id = layout.new_leaf(Style::default());

        let mut core = AkarCore::mock();
        // Zero-area node — label must early-return and push nothing.
        label(&mut core, &layout, node_id, "Hello", 0xFFFFFFFF, &AKAR_THEME_DARK);

        // DrawList should have no calls (no quads, no text).
        assert_eq!(core.draw_list.len(), 0);
    }
}
```

This requires `DrawList::len() -> usize` to be exposed. Add it to `draw_list.rs` if not already present:

```rust
pub fn len(&self) -> usize {
    self.calls.len()
}
```

**Acceptance criteria:** `cargo test -p akar-components` passes including the new label test.

**Review:** Done. `label.rs` created (quad-free, text-only), `DrawList::len()` + `is_empty()` added, `lib.rs` exports `akar_label`. Clippy clean, 11 component tests + 11 core tests pass.

---

### Task 3: C ABI — `akar_label`

**Goal:** Expose the label component over the C ABI. Mirrors `akar_button` in structure.

**Add to `crates/akar-c-api/src/lib.rs`:**

```rust
#[no_mangle]
pub unsafe extern "C" fn akar_label(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const c_char,
    text_len: i32,
    color: u32,
) {
    let ctx = unsafe { &mut *ctx };

    if text.is_null() || text_len <= 0 {
        return;
    }

    let bytes = unsafe { std::slice::from_raw_parts(text as *const u8, text_len as usize) };
    let Ok(text_str) = std::str::from_utf8(bytes) else {
        return;
    };

    let nid: akar_layout::NodeId = node_id.into();
    akar_components::label(
        &mut ctx.core,
        &ctx.layout,
        nid,
        text_str,
        color,
        &ctx.theme,
    );
}
```

**Acceptance criteria:** `cargo check -p akar-c-api` passes. `cargo build -p akar-c-api` regenerates `akar.h` with `akar_label` present.

**Review:** Done. `akar_label` added with text+text_len+color params, null-safe, mirrors `akar_button` pattern. Clippy clean, build succeeds.

---

### Task 4: Update `examples/demo-rust` — Show Labels

**Goal:** Add two labeled buttons and a standalone label below the canvas to demonstrate real text rendering. The canvas area is unchanged.

The demo currently has a canvas filling `page.main`. Add a small flex-column row below the canvas:
- Two `button` nodes side by side (labels: `"Zoom In"`, `"Zoom Out"`)
- One `label` node beneath them (`"Pan: middle-mouse drag | Zoom: scroll"`)

**Layout addition in `main.rs`:**

The existing `page.main` fills `body`. Replace `page.main` as the canvas target with a new flex-column child of body that contains:
1. A canvas node (`flex_grow: 1`) — fills remaining space.
2. A controls strip (`flex_shrink: 0`, fixed height 48px) — holds the buttons and label.

```rust
// Replace the single canvas-in-main with a column split:
let col = layout.new_leaf(Style {
    display: Display::Flex,
    flex_direction: FlexDirection::Column,
    size: Size { width: Dimension::percent(1.0), height: Dimension::percent(1.0) },
    ..Default::default()
});
let canvas_node = layout.new_leaf(Style {
    flex_grow: 1.0,
    ..Default::default()
});
let strip = layout.new_leaf(Style {
    flex_shrink: 0.0,
    display: Display::Flex,
    flex_direction: FlexDirection::Row,
    gap: taffy::prelude::Size { width: length(8.0), height: zero() },
    size: Size { width: Dimension::percent(1.0), height: length(48.0) },
    padding: Rect { left: length(8.0), right: length(8.0), top: length(4.0), bottom: length(4.0) },
    ..Default::default()
});
let btn_zoom_in  = layout.new_leaf(Style { flex_grow: 0.0, flex_shrink: 0.0,
    size: Size { width: length(80.0), height: length(36.0) }, ..Default::default() });
let btn_zoom_out = layout.new_leaf(Style { flex_grow: 0.0, flex_shrink: 0.0,
    size: Size { width: length(80.0), height: length(36.0) }, ..Default::default() });
let hint_label   = layout.new_leaf(Style { flex_grow: 1.0, ..Default::default() });

layout.add_child(strip, btn_zoom_in);
layout.add_child(strip, btn_zoom_out);
layout.add_child(strip, hint_label);
layout.add_child(col, canvas_node);
layout.add_child(col, strip);
layout.set_children(page.main, &[col]);  // page.main is now a container
```

**Per-frame draw sequence additions:**

```rust
// After canvas_end(&mut core, painter):

container(&mut core, &layout, strip, AKAR_THEME_DARK.base_300, &AKAR_THEME_DARK);

let zoom_in_result  = button(&mut core, &layout, btn_zoom_in,  "Zoom In",  ButtonVariant::Solid,  &AKAR_THEME_DARK);
let zoom_out_result = button(&mut core, &layout, btn_zoom_out, "Zoom Out", ButtonVariant::Outline, &AKAR_THEME_DARK);

if zoom_in_result.clicked {
    canvas_state.zoom_at_point(
        Vec2::new(layout.rect(canvas_node)[0] + layout.rect(canvas_node)[2] * 0.5,
                  layout.rect(canvas_node)[1] + layout.rect(canvas_node)[3] * 0.5),
        layout.rect(canvas_node),
        1.2,
        config.zoom_min,
        config.zoom_max,
    );
}
if zoom_out_result.clicked {
    canvas_state.zoom_at_point(
        Vec2::new(layout.rect(canvas_node)[0] + layout.rect(canvas_node)[2] * 0.5,
                  layout.rect(canvas_node)[1] + layout.rect(canvas_node)[3] * 0.5),
        layout.rect(canvas_node),
        1.0 / 1.2,
        config.zoom_min,
        config.zoom_max,
    );
}

label(&mut core, &layout, hint_label,
      "Pan: middle-mouse drag  |  Zoom: scroll",
      AKAR_THEME_DARK.base_content,
      &AKAR_THEME_DARK);
```

**Acceptance criteria:** `cargo run --manifest-path examples/demo-rust/Cargo.toml` opens a window showing:
- Canvas fills most of the main area; five colored rectangles are visible and pan/zoom correctly.
- A 48px strip at the bottom of the main area shows two buttons (`Zoom In`, `Zoom Out`) with visible text labels.
- `Zoom In` / `Zoom Out` buttons click and zoom the canvas centered on the canvas center.
- The hint label text renders to the right of the buttons.
- No panics on resize, hover, or rapid clicking.

---

## Acceptance Criteria for Epic 006

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero errors.
- [ ] `cargo test --workspace` passes. New tests: 1 in label component (zero-area guard).
- [ ] `button.rs` no longer has the `// TODO: cache buffer per node_id` comment; `set_text` is called with `Some(node_id.into())`.
- [ ] `label` component is exported from `akar-components` and documented in `lib.rs`.
- [ ] `akar_label` is present in `akar.h` after `cargo build -p akar-c-api`.
- [ ] `demo-rust` shows visible text on buttons and a visible hint label.
- [ ] No text centering or font loading — deferred per ADR-017.
- [ ] No `CanvasPainter::push_text` — deferred per ADR-017.
- [ ] No text input component — deferred per ADR-017.
