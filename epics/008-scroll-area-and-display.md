# Epic 008: ScrollArea, Canvas-Container Bridge, and Display Components

**Status:** In Progress (Tasks 1–2 Done)
**Goal:** Four things in priority order:
1. Rename `akar_layout::Rect` → `WorldRect` (naming cleanup, carried from the TBD placeholder).
2. Add `Layout::rect_offset` and `list_clip` — the two helpers that make the canvas-container "Figma frame" pattern usable: a container positioned in world space whose internal components are laid out by an independent taffy tree.
3. Add `ScrollArea` — the first component that manages scissor state and caller-owned scroll position, enabling vertically scrollable content with virtualization.
4. Add `Progress` and `Badge` — the first static display components beyond button and label.

**Prerequisite:** Epic 007 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean before Task 1 begins.

---

## Architecture Decision Records

### ADR-022: Caller-Owned Scroll State

**Decision:** `scroll_area_begin` takes `scroll_y: &mut f32`. The caller owns the scroll position across frames. akar reads the mouse wheel input, updates the value in place, clamps it to the valid range, and returns the y-coordinate of the content origin for the frame.

**Rationale:** Immediate mode — state lives where the developer puts it. A retained scroll state HashMap keyed by node_id (like egui's Memory) adds indirection and lifecycle complexity with no benefit at this stage. The caller naming their own `f32` is explicit, cheap, and unsurprising.

**Consequences:** The C ABI equivalent takes a `float*` so C callers can mutate it from their frame loop. There is no lookup by ID.

---

### ADR-023: `list_clip` as a Pure Free Function in `akar-core`

**Decision:** `pub fn list_clip(total: usize, item_height: f32, scroll_y: f32, viewport_height: f32) -> std::ops::Range<usize>` lives in `akar-core/src/lib.rs`, re-exported from the crate root.

**Rationale:** A pure O(1) computation with no dependencies on layout or components. `akar-core` is the right home — it is the foundational crate and this function is conceptually adjacent to scissor clipping (also in core). The caller uses it directly before the scroll area render loop to compute which items to submit.

**Formula:** `first = floor(scroll_y / item_height)`, `last = ceil((scroll_y + viewport_height) / item_height)`, both clamped to `[0, total]`.

**Deferred:** A 2D variant `grid_clip` for virtualized grids. Same principle, later epic.

---

### ADR-024: Canvas-Container Bridge via `rect_offset` and Deferred Explicit-Rect Components

**Decision:** Add `Layout::rect_offset(node: NodeId, origin: [f32; 2]) -> [f32; 4]` as the immediate bridge helper. Full "component-at-explicit-rect" variants (where a component bypasses its node_id and takes a raw rect directly) are deferred to the epic when a concrete canvas application is built.

**Rationale:** `rect_offset` is useful right now for any caller that is manually computing screen positions from a world→screen transform and needs to place quads or compute hit-test regions. It costs nothing. Full explicit-rect variants of every component double the API surface and should only be introduced when there is a concrete use case driving the design — not speculatively.

**Usage pattern (for documentation):**
```
// Canvas container placement — world to screen:
let screen_rect = canvas_transform.world_to_screen(world_rect);
// Push scissor for the container viewport.
core.draw_list.push_scissor(screen_rect);
// Render container background directly (raw quad, not via taffy).
core.draw_list.push_quad(QuadCall { rect: screen_rect, ... });
// Internal layout tree: independent Layout instance, computed against screen_rect size.
frame_layout.compute(frame_root, (Some(screen_rect[2]), Some(screen_rect[3])), measure_fn);
// Offset internal rects to screen space.
let label_rect = frame_layout.rect_offset(label_node, [screen_rect[0], screen_rect[1]]);
// ... manual rendering at label_rect ...
core.draw_list.pop_scissor();
```

---

### ADR-025: ScrollArea Renders No Visual Chrome

**Decision:** `scroll_area_begin` / `scroll_area_end` only manage scissor state and scroll position. They do not render a scrollbar track, scrollbar thumb, background, or border.

**Rationale:** The visual frame around a scroll area is a `container` call that the developer makes before `scroll_area_begin`. The scrollbar is a separate problem — its design (thin overlay vs. gutter) is contentious and can be a discrete component added later. Keeping the scroll area behavior-only avoids baking visual decisions that the developer may want to override.

---

## Tasks

### Task 1: Rename `akar_layout::Rect` → `WorldRect`

**Status:** Done (commit a175dcc)

**Review note:** Clean mechanical rename across 5 files. No remaining uses of `akar_layout::Rect`. Clippy and tests pass.

**Goal:** Eliminate the naming ambiguity between `akar_layout::Rect` (world-space bounding box) and `taffy::geometry::Rect` (padding/margin type, in scope via `pub use taffy::prelude::*`).

**Rename:** `Rect` → `WorldRect` everywhere it refers to the canvas world-space bounding box.

| File | Change |
|---|---|
| `crates/akar-layout/src/rect.rs` | `pub struct Rect` → `pub struct WorldRect` |
| `crates/akar-layout/src/canvas_transform.rs` | `Rect` in all function signatures and return types; `compute_visible_world_rect` return type |
| `crates/akar-layout/src/lib.rs` | `pub use rect::Rect` → `pub use rect::WorldRect` |
| `crates/akar-components/src/canvas.rs` | `CanvasResponse.visible_world_rect` type, `is_visible_world` arguments, `CanvasPainter::push_quad` argument, test `use` imports |
| `examples/canvas-basic-rust/src/main.rs` | `use akar_layout::Rect` import |
| `examples/demo-rust/src/main.rs` | `use akar_layout::Rect` import if present |

No logic changes — identifier rename only.

**Acceptance criteria:** `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass clean after the rename. No uses of the old name remain (verify with `grep -r "akar_layout::Rect\b" .`).

---

### Task 2: Canvas-Container Bridge Helpers

**Status:** Done (commit c1798c3)

**Review note:** `rect_offset` placed above `rect` in Layout impl. `list_clip` defined as free fn in `akar-core/src/lib.rs` with its test module. All tests pass (26 total for workspace, 5 list_clip + 1 rect_offset).

**Goal:** Add `Layout::rect_offset` and the `list_clip` free function.

#### `Layout::rect_offset`

**File:** `crates/akar-layout/src/lib.rs` — add to the `impl Layout` block:

```rust
/// Returns the screen-space rect of `node` offset by `origin`.
///
/// Used in the canvas-container pattern: when an independent layout tree is
/// computed against a canvas container's screen-space size, the tree's internal
/// rects are relative to (0, 0). `origin` is the container's screen-space top-left
/// corner (from the world→screen transform), and this method shifts all coordinates
/// into screen space.
pub fn rect_offset(&self, node: NodeId, origin: [f32; 2]) -> [f32; 4] {
    let [x, y, w, h] = self.rect(node);
    [origin[0] + x, origin[1] + y, w, h]
}
```

**Test** — add to the existing `#[cfg(test)]` block:

```rust
#[test]
fn rect_offset_shifts_by_origin() {
    let mut layout = Layout::new();
    let child = layout.new_leaf(Style {
        size: Size { width: length(40.0), height: length(20.0) },
        ..Default::default()
    });
    let root = layout.new_with_children(Style::default(), &[child]);
    layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

    let r = layout.rect_offset(child, [100.0, 50.0]);
    assert_eq!(r[0], 100.0);
    assert_eq!(r[1], 50.0);
    assert_eq!(r[2], 40.0);
    assert_eq!(r[3], 20.0);
}
```

#### `list_clip`

**File:** `crates/akar-core/src/lib.rs` — add as a public free function:

```rust
/// Returns the range of item indices visible in a virtualized vertical list.
///
/// Given a list of `total` items each with identical `item_height`, and the
/// current `scroll_y` offset (in logical pixels from the top of the content),
/// returns the half-open range `[first, last)` of items that intersect the
/// viewport. The caller renders only items in this range.
///
/// One extra item is included on each side to avoid pop-in at the boundary.
/// Both ends are clamped to `[0, total]`.
pub fn list_clip(
    total: usize,
    item_height: f32,
    scroll_y: f32,
    viewport_height: f32,
) -> std::ops::Range<usize> {
    if total == 0 || item_height <= 0.0 {
        return 0..0;
    }
    let first = ((scroll_y / item_height).floor() as isize - 1).max(0) as usize;
    let last = ((( scroll_y + viewport_height) / item_height).ceil() as usize + 1).min(total);
    first..last
}
```

**File:** `crates/akar-core/src/lib.rs` — re-export from crate root:

```rust
pub use list_clip::list_clip;  // or define inline if no separate module
```

**Tests** — add to `akar-core`:

```rust
#[cfg(test)]
mod list_clip_tests {
    use super::list_clip;

    #[test]
    fn empty_list_returns_empty() {
        assert_eq!(list_clip(0, 50.0, 0.0, 400.0), 0..0);
    }

    #[test]
    fn zero_item_height_returns_empty() {
        assert_eq!(list_clip(100, 0.0, 0.0, 400.0), 0..0);
    }

    #[test]
    fn top_of_list_includes_first_items() {
        let range = list_clip(100, 50.0, 0.0, 200.0);
        assert_eq!(range.start, 0);
        assert!(range.end >= 4); // at least 4 visible at 50px each in 200px viewport
        assert!(range.end <= 6); // with ±1 buffer
    }

    #[test]
    fn scrolled_mid_list() {
        let range = list_clip(100, 50.0, 250.0, 200.0);
        assert!(range.start <= 4); // item 5 starts at y=250, minus buffer
        assert!(range.end >= 9);   // item 9 ends at y=450, plus buffer
        assert!(range.end <= 100);
    }

    #[test]
    fn near_end_clamps_to_total() {
        let range = list_clip(10, 50.0, 400.0, 200.0);
        assert_eq!(range.end, 10);
    }
}
```

**Acceptance criteria:** `cargo test --workspace` passes with all new tests green.

---

### Task 3: `ScrollArea` Component

**Goal:** A behavior-only component that reads scroll input, updates the caller's scroll position, pushes a scissor rect, and returns the content y-origin. The matching `scroll_area_end` pops the scissor.

**File:** `crates/akar-components/src/scroll_area.rs` (new):

```rust
use akar_core::AkarCore;

pub struct ScrollAreaResponse {
    /// Y-coordinate of the content origin for this frame.
    /// Item `i` in a uniform list renders at `content_y + i as f32 * item_height`.
    pub content_y: f32,
}

/// Begins a scroll area at `rect` (logical pixels).
///
/// `scroll_y` is the caller-owned vertical scroll offset. This function:
/// 1. Reads `core.input.scroll_delta.y` when the mouse is inside `rect` and
///    updates `*scroll_y` accordingly.
/// 2. Clamps `*scroll_y` to `[0, max(0, content_height - rect[3])]`.
/// 3. Pushes a scissor rect matching `rect` onto the draw list.
/// 4. Returns a `ScrollAreaResponse` containing the y-origin of the content.
///
/// Must be followed by exactly one `scroll_area_end` call per frame.
pub fn scroll_area_begin(
    core: &mut AkarCore,
    rect: [f32; 4],
    scroll_y: &mut f32,
    content_height: f32,
) -> ScrollAreaResponse {
    let [_, y, _, h] = rect;

    if core.input.is_hovering(rect) {
        *scroll_y -= core.input.scroll_delta.y;
    }

    let max_scroll = (content_height - h).max(0.0);
    *scroll_y = scroll_y.clamp(0.0, max_scroll);

    core.draw_list.push_scissor(rect);

    ScrollAreaResponse {
        content_y: y - *scroll_y,
    }
}

/// Ends a scroll area begun with `scroll_area_begin`. Pops the scissor rect.
pub fn scroll_area_end(core: &mut AkarCore) {
    core.draw_list.pop_scissor();
}
```

**Tests** — at the bottom of `scroll_area.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::AkarCore;

    fn make_core() -> AkarCore {
        AkarCore::mock()
    }

    #[test]
    fn scissor_pushed_and_popped() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        scroll_area_begin(&mut core, [0.0, 0.0, 200.0, 400.0], &mut 0.0, 1000.0);
        assert!(core.draw_list.active_scissor().is_some());
        scroll_area_end(&mut core);
        assert!(core.draw_list.active_scissor().is_none());
    }

    #[test]
    fn scroll_y_clamped_to_zero() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        let mut scroll_y = -50.0f32;
        scroll_area_begin(&mut core, [0.0, 0.0, 200.0, 400.0], &mut scroll_y, 1000.0);
        scroll_area_end(&mut core);
        assert_eq!(scroll_y, 0.0);
    }

    #[test]
    fn scroll_y_clamped_to_max() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        let mut scroll_y = 9999.0f32;
        scroll_area_begin(&mut core, [0.0, 0.0, 200.0, 400.0], &mut scroll_y, 1000.0);
        scroll_area_end(&mut core);
        assert_eq!(scroll_y, 600.0); // 1000 content - 400 viewport
    }

    #[test]
    fn content_y_reflects_scroll() {
        let mut core = make_core();
        core.draw_list.begin_frame(1.0);
        let mut scroll_y = 100.0f32;
        let resp = scroll_area_begin(&mut core, [0.0, 50.0, 200.0, 400.0], &mut scroll_y, 1000.0);
        scroll_area_end(&mut core);
        // rect.y=50, scroll_y=100 → content_y = 50 - 100 = -50
        assert_eq!(resp.content_y, -50.0);
    }
}
```

**File:** `crates/akar-components/src/lib.rs` — add:

```rust
pub mod scroll_area;
pub use scroll_area::{ScrollAreaResponse, scroll_area_begin, scroll_area_end};
```

**Acceptance criteria:** `cargo test -p akar-components` passes with all four new tests green.

---

### Task 4: `Progress` and `Badge` Components

#### Progress

**File:** `crates/akar-components/src/progress.rs` (new):

```rust
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};
use crate::color::color_to_f32;
use crate::AkarTheme;

pub struct ProgressStyle {
    pub track_color: u32,
    pub fill_color: u32,
    /// Applied to all four corners of both track and fill.
    pub corner_radius: f32,
}

impl ProgressStyle {
    pub fn from_theme(theme: &AkarTheme) -> Self {
        Self {
            track_color: theme.base_300,
            fill_color: theme.primary,
            corner_radius: theme.radius_field / 2.0,
        }
    }
}

/// Renders a horizontal progress bar in the taffy-resolved rect of `node_id`.
/// `value` is clamped to `[0.0, 1.0]`.
pub fn progress(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    value: f32,
    style: &ProgressStyle,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }
    let value = value.clamp(0.0, 1.0);
    let radii = [style.corner_radius; 4];

    // Track
    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(style.track_color),
        border_color: [0.0; 4],
        corner_radii: radii,
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    // Fill
    if value > 0.0 {
        let fill_rect = [rect[0], rect[1], rect[2] * value, rect[3]];
        core.draw_list.push_quad(QuadCall {
            rect: fill_rect,
            fill: color_to_f32(style.fill_color),
            border_color: [0.0; 4],
            corner_radii: radii,
            border_width: 0.0,
            z: 0.0,
            shadow_blur: 0.0,
            shadow_spread: 0.0,
            shadow_color: [0.0; 4],
            shadow_offset: [0.0; 2],
            _pad: [0.0; 2],
        });
    }
}
```

**Tests** inside `progress.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use akar_core::AkarCore;
    use akar_layout::{Layout, Style, Size, length};

    fn node_100x20(layout: &mut Layout) -> NodeId {
        let n = layout.new_leaf(Style {
            size: Size { width: length(100.0), height: length(20.0) },
            ..Default::default()
        });
        layout.compute(n, (Some(200.0), Some(200.0)), |_, _, _, _, _| akar_layout::Size::ZERO);
        n
    }

    #[test]
    fn full_value_pushes_two_quads() {
        let mut layout = Layout::new();
        let node = node_100x20(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let style = ProgressStyle { track_color: 0xccccccff, fill_color: 0x0000ffff, corner_radius: 4.0 };
        progress(&mut core, &layout, node, 1.0, &style);
        assert_eq!(core.draw_list.len(), 2);
    }

    #[test]
    fn zero_value_pushes_only_track() {
        let mut layout = Layout::new();
        let node = node_100x20(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let style = ProgressStyle { track_color: 0xccccccff, fill_color: 0x0000ffff, corner_radius: 4.0 };
        progress(&mut core, &layout, node, 0.0, &style);
        assert_eq!(core.draw_list.len(), 1);
    }

    #[test]
    fn value_clamped_above_one() {
        let mut layout = Layout::new();
        let node = node_100x20(&mut layout);
        let mut core = AkarCore::mock();
        core.draw_list.begin_frame(1.0);
        let style = ProgressStyle { track_color: 0xccccccff, fill_color: 0x0000ffff, corner_radius: 0.0 };
        progress(&mut core, &layout, node, 5.0, &style);
        let quads = core.draw_list.sorted_quads();
        // Fill width must equal track width (value=1.0 after clamp).
        assert_eq!(quads[0].rect[2], quads[1].rect[2]);
    }
}
```

#### Badge

**File:** `crates/akar-components/src/badge.rs` (new):

```rust
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};
use crate::color::color_to_f32;
use crate::AkarTheme;
use crate::label::label;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeVariant {
    Default,
    Primary,
    Success,
    Warning,
    Error,
    Info,
}

/// Renders a pill-shaped badge with `text` inside the taffy-resolved rect of `node_id`.
/// Background and text colors are chosen from `theme` based on `variant`.
pub fn badge(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    text: &str,
    variant: BadgeVariant,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }

    let (bg, fg) = match variant {
        BadgeVariant::Default  => (theme.base_300,   theme.base_content),
        BadgeVariant::Primary  => (theme.primary,     theme.primary_content),
        BadgeVariant::Success  => (theme.success,     theme.success_content),
        BadgeVariant::Warning  => (theme.warning,     theme.warning_content),
        BadgeVariant::Error    => (theme.error,       theme.error_content),
        BadgeVariant::Info     => (theme.info,        theme.info_content),
    };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(bg),
        border_color: [0.0; 4],
        corner_radii: [theme.radius_field; 4],
        border_width: 0.0,
        z: 0.0,
        shadow_blur: 0.0,
        shadow_spread: 0.0,
        shadow_color: [0.0; 4],
        shadow_offset: [0.0; 2],
        _pad: [0.0; 2],
    });

    label(core, layout, node_id, text, fg, theme.font_size_sm);
}
```

Note: `badge` delegates to `label` for text rendering. The `label` function already centers text within a rect and handles the text pipeline. Verify `label`'s current signature and adapt if needed.

**File:** `crates/akar-components/src/lib.rs` — add:

```rust
pub mod progress;
pub use progress::{ProgressStyle, progress as akar_progress};

pub mod badge;
pub use badge::{BadgeVariant, badge as akar_badge};
```

**Acceptance criteria:** `cargo test -p akar-components` passes with all new tests green. `cargo clippy -p akar-components -- -D warnings` passes clean.

---

### Task 5: C ABI Exposure and Demo Update

**Goal:** Expose `list_clip`, `scroll_area_begin/end`, `progress`, and `badge` in `akar.h`. Update `demo-rust` to demonstrate a scrollable list with a progress bar and badges.

#### C ABI additions — `crates/akar-c-api/src/lib.rs`

**`AkarRange`** (new repr(C) struct — range returned by `list_clip`):

```rust
#[repr(C)]
pub struct AkarRange {
    pub start: u32,
    pub end: u32,
}
```

**`akar_list_clip`:**

```rust
#[no_mangle]
pub extern "C" fn akar_list_clip(
    total: u32,
    item_height: f32,
    scroll_y: f32,
    viewport_height: f32,
) -> AkarRange {
    let r = akar_core::list_clip(total as usize, item_height, scroll_y, viewport_height);
    AkarRange { start: r.start as u32, end: r.end as u32 }
}
```

**`akar_scroll_area_begin` / `akar_scroll_area_end`:**

```rust
/// Returns the content_y offset for the current frame.
#[no_mangle]
pub unsafe extern "C" fn akar_scroll_area_begin(
    ctx: *mut AkarCtx,
    rect: *const f32,   // [x, y, w, h]
    scroll_y: *mut f32,
    content_height: f32,
) -> f32 {
    let ctx = unsafe { &mut *ctx };
    let rect = unsafe { *(rect as *const [f32; 4]) };
    let resp = akar_components::scroll_area_begin(
        &mut ctx.core,
        rect,
        unsafe { &mut *scroll_y },
        content_height,
    );
    resp.content_y
}

#[no_mangle]
pub unsafe extern "C" fn akar_scroll_area_end(ctx: *mut AkarCtx) {
    let ctx = unsafe { &mut *ctx };
    akar_components::scroll_area_end(&mut ctx.core);
}
```

**`akar_progress`:**

```rust
#[no_mangle]
pub unsafe extern "C" fn akar_progress(
    ctx: *mut AkarCtx,
    node_id: u64,
    value: f32,
    track_color: u32,
    fill_color: u32,
    corner_radius: f32,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    let style = akar_components::ProgressStyle { track_color, fill_color, corner_radius };
    akar_components::akar_progress(&mut ctx.core, &ctx.layout, nid, value, &style);
}
```

**`akar_badge`:**

```rust
#[no_mangle]
pub unsafe extern "C" fn akar_badge(
    ctx: *mut AkarCtx,
    node_id: u64,
    text: *const std::ffi::c_char,
    variant: u32,  // 0=Default, 1=Primary, 2=Success, 3=Warning, 4=Error, 5=Info
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    let text = unsafe { std::ffi::CStr::from_ptr(text) }.to_str().unwrap_or("");
    let variant = match variant {
        1 => akar_components::BadgeVariant::Primary,
        2 => akar_components::BadgeVariant::Success,
        3 => akar_components::BadgeVariant::Warning,
        4 => akar_components::BadgeVariant::Error,
        5 => akar_components::BadgeVariant::Info,
        _ => akar_components::BadgeVariant::Default,
    };
    akar_components::akar_badge(&mut ctx.core, &ctx.layout, nid, text, variant, &ctx.theme);
}
```

**Regenerate `akar.h`:** run `cargo build -p akar-c-api` and verify the header contains `AkarRange`, `akar_list_clip`, `akar_scroll_area_begin`, `akar_scroll_area_end`, `akar_progress`, `akar_badge`.

#### Demo update — `examples/demo-rust/src/main.rs`

Add a scrollable list panel to the demo's main area:
- 50 synthetic items, each 48px tall.
- Each item: a card-style container, a label with the item name, and a progress bar showing `i / 50.0`.
- Two badges at the top of the panel: one `Success` and one `Warning`, to show variant colors.
- Use `list_clip` to render only visible items.
- Scroll with the mouse wheel.

This exercises the full Epic 008 feature set in one visible demo.

**Acceptance criteria:**
- `cargo run --manifest-path examples/demo-rust/Cargo.toml` opens and shows the scrollable list.
- Mouse wheel scrolls the list; items clip correctly at the viewport boundary.
- Progress bars render with varying widths per item.
- Badges render with correct variant colors.
- `cargo clippy --workspace -- -D warnings` passes clean.
- `cargo test --workspace` passes with all new tests.

---

## Acceptance Criteria for Epic 008

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero warnings.
- [ ] `cargo test --workspace` passes. New test count: ≥ 10 (rect_offset × 1, list_clip × 5, scroll_area × 4, progress × 3).
- [ ] No uses of `akar_layout::Rect` remain (confirmed by grep).
- [ ] `list_clip` is a public free function exported from `akar-core`.
- [ ] `Layout::rect_offset` is a public method on `Layout`.
- [ ] `scroll_area_begin` / `scroll_area_end` manage scissor state correctly; nested calls are safe.
- [ ] `progress` renders one quad for value=0, two quads for value>0; fill width is proportional.
- [ ] `badge` renders background quad + centered text in the correct variant color.
- [ ] `AkarRange`, `akar_list_clip`, `akar_scroll_area_begin`, `akar_scroll_area_end`, `akar_progress`, `akar_badge` are in `akar.h`.
- [ ] Demo shows a 50-item scrollable list with progress bars and badges.
- [ ] No scrollbar visual — deferred per ADR-025.
- [ ] No explicit-rect component variants — deferred per ADR-024.
