# Epic 003: Layout Containers

**Status:** Planned
**Goal:** Build the layout container primitives — `Container`, `Separator`, `TwoColumn`, `ThreeColumn`, and `Page` — so developers can compose typical application shells without writing raw taffy code. Add a `responsive_columns` utility. Update the demo to show a full page shell with a two-column main area. No new GPU pipelines or draw-list changes are required; all work builds on Epic 002's infrastructure.

**Prerequisite:** Epic 002 improvements (`epics/002-improvements.md`) are applied and `cargo clippy --workspace -- -D warnings` passes clean before Task 1 begins.

---

## Architecture Decision Records

### ADR-005: Container Model

**Decision:** A `Container` is a styled box — a background quad and optional border quad — whose children are drawn by the developer after the container call returns. There is no closure-based or ownership-based child API.

The developer's call order each frame mirrors the taffy tree order:
1. Call `container()` for a node → background quad is pushed to the draw list.
2. Call component functions for child nodes → they draw on top.

Clipping (scissor push/pop) is NOT part of `Container` in this epic. Scroll containers that clip their children are a future epic. Developers who need clipping now call `draw_list.push_scissor` / `pop_scissor` manually.

**Rationale:** Closure-based containers (`ui.container(|ui| { ... })`) do not compose with C FFI and create borrow conflicts when the developer needs to interleave input queries with drawing. The flat, ordered call model matches how Dear ImGui and Nuklear work and is trivially wrappable in C. The developer is responsible for calling children in tree order — the same contract they already have with the button component.

**Consequences:** Developers must draw container background before children. A helper note in the API docs is sufficient — no enforcement mechanism needed.

---

### ADR-006: Layout Builders Live in `akar-layout`; Components Live in `akar-components`

**Decision:** Functions that create taffy subtrees (`two_column`, `three_column`, `page`) are methods on `Layout` in `akar-layout`. Functions that submit draw calls (`container`, `separator`) are free functions in `akar-components`. These two concerns must not be mixed.

**Rationale:** `akar-layout` must not depend on `akar-components` (it has no knowledge of draw lists or themes). `akar-components` depends on `akar-layout` to read rects. This matches the crate boundary table in `CLAUDE.md`.

**Separator thickness** flows from the caller: layout builders accept `separator_thickness: f32` (logical pixels) so the caller can pass `theme.border_width` without `akar-layout` importing `AkarTheme`.

---

### ADR-007: Separator Is Visual-Only in v1

**Decision:** `separator()` draws a single quad that fills its taffy-allocated rect. It is not interactive and cannot be dragged to resize adjacent columns. Resizable separators are deferred beyond v1.

**Rationale:** Drag-to-resize requires storing drag state across frames and calling `layout.set_style()` on sibling nodes mid-frame, which forces a layout recompute within the frame. This is a non-trivial invariant to get right and is not needed to prove the container model.

---

### ADR-008: Grid and Canvas Are Deferred

**Grid (`display: grid`)** is deferred to its own epic after Epic 003. Taffy's grid API requires explicit track-size definitions (`TrackSizingFunction`, `GridPlacement`) whose ergonomic surface is substantially larger than flex. Building it before the component library has more breadth would produce an undertested API. `TwoColumn` and `ThreeColumn` cover the common split-pane cases without grid. This choice will be revisited after the component library reaches five or more components.

**Canvas (infinite pan/zoom surface)** is Epic 004. The design is settled:
- Canvas is a **taffy leaf node** — the layout system allocates it a pixel rect like any other component.
- Inside that rect, Canvas manages its own **world coordinate system** (pan offset `Vec2` + zoom `f32`). Children of a Canvas are positioned in world coordinates, not taffy nodes.
- Canvas can contain other Containers (rendered in world space). Containers cannot contain Canvas. Canvas cannot contain Canvas.
- The public API will be `canvas_begin` / `canvas_end` with an `is_visible_world` culling helper.

---

## Tasks

### Task 1: Apply Epic 002 improvements

**Goal:** The four clippy errors and the soundness issue from `002-improvements.md` must be fixed before any Epic 003 work begins. This task has no new files — it is a series of small edits.

**Edits:**

**`crates/akar-core/src/input.rs`** — add after `impl InputState`:
```rust
impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
```

**`crates/akar-core/src/draw_list.rs`** — add after `impl DrawList`:
```rust
impl Default for DrawList {
    fn default() -> Self {
        Self::new()
    }
}
```

**`crates/akar-layout/src/lib.rs`** — add after `impl Layout`:
```rust
impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
```

**`crates/akar-core/src/quad_pipeline.rs:135`** — replace:
```rust
// before
let required_size = quads.len() * mem::size_of::<QuadCall>();

// after
let required_size = std::mem::size_of_val(quads);
```

**`crates/akar-c-api/src/lib.rs:222`** — replace `from_utf8_unchecked`:
```rust
// before
let label_str = unsafe { std::str::from_utf8_unchecked(label_bytes) };

// after
let Ok(label_str) = std::str::from_utf8(label_bytes) else {
    return AkarButtonResult { clicked: false, hovered: false, pressed: false };
};
```

**`crates/akar-components/src/button.rs`** — remove `lighten_color`, rename `dim_color` to `scale_color`, update all call sites:
```rust
fn scale_color(c: u32, factor: f32) -> u32 {
    let r = (((c >> 24) & 0xFF) as f32 * factor).min(255.0) as u32;
    let g = (((c >> 16) & 0xFF) as f32 * factor).min(255.0) as u32;
    let b = (((c >> 8) & 0xFF) as f32 * factor).min(255.0) as u32;
    let a = c & 0xFF;
    (r << 24) | (g << 16) | (b << 8) | a
}
```
Call sites: `scale_color(theme.primary, 0.8)` for press-dim, `scale_color(theme.primary, 1.1)` for hover-lighten.

**Acceptance criteria:** `cargo clippy --workspace -- -D warnings` passes. `cargo test --workspace` still passes. No functional change.

---

### Task 2: `akar-components` — Separator

**Goal:** The simplest component: a single quad that fills its taffy rect. Used as the visual divider between columns and sections.

**File:** `crates/akar-components/src/separator.rs`

```rust
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};
use crate::AkarTheme;

pub fn separator(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }
    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(theme.base_300),
        border_color: [0.0; 4],
        border_width: 0.0,
        corner_radii: [0.0; 4],
        z: 0.0,
        _pad: 0.0,
    });
}
```

`color_to_f32` is the same helper already in `button.rs`. Move it to a private `crates/akar-components/src/color.rs` module and re-use it from both `button.rs` and `separator.rs`:

```rust
// crates/akar-components/src/color.rs
pub(crate) fn color_to_f32(c: u32) -> [f32; 4] {
    [
        ((c >> 24) & 0xFF) as f32 / 255.0,
        ((c >> 16) & 0xFF) as f32 / 255.0,
        ((c >> 8) & 0xFF) as f32 / 255.0,
        (c & 0xFF) as f32 / 255.0,
    ]
}
```

Re-export from `crates/akar-components/src/lib.rs`:
```rust
mod color;
pub mod separator;
pub use separator::separator;
```

**Acceptance criteria:** `cargo test -p akar-components` passes. Unit test: separator with zero-area rect does not push any quad to the draw list.

---

### Task 3: `akar-components` — Container

**Goal:** A styled box that draws a background quad and optional border. The developer calls this before drawing children. No clipping.

**File:** `crates/akar-components/src/container.rs`

```rust
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};
use crate::{color::color_to_f32, AkarTheme};

pub fn container(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    background: u32,    // packed RGBA; 0x00000000 = fully transparent (no quad pushed)
    theme: &AkarTheme,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 {
        return;
    }
    if background == 0 {
        return;
    }
    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(background),
        border_color: color_to_f32(theme.base_300),
        border_width: 0.0,      // no border by default; caller sets non-zero if needed
        corner_radii: [theme.radius_box; 4],
        z: 0.0,
        _pad: 0.0,
    });
}
```

> Note: `border_width: 0.0` is intentional — the border color is set to `base_300` as a cheap default but is not visible until `border_width > 0`. A future `ContainerStyle` struct can expose this when more components need it. For now the simplest signature wins.

Re-export from `lib.rs`:
```rust
pub mod container;
pub use container::container;
```

**Acceptance criteria:** Unit test: calling `container` with `background = 0x00000000` pushes no quad. Calling it with a non-zero color and non-zero rect pushes exactly one quad.

---

### Task 4: `akar-layout` — `TwoColumn` and `ThreeColumn` builders

**Goal:** Builder methods on `Layout` that create flex-row subtrees with column nodes and separator nodes, so the developer doesn't have to write raw taffy Style code for a split-pane layout.

**File:** `crates/akar-layout/src/lib.rs` — add to `impl Layout`

**Types (add to `lib.rs`):**
```rust
pub struct TwoColumnLayout {
    pub left: NodeId,
    pub separator: NodeId,
    pub right: NodeId,
}

pub struct ThreeColumnLayout {
    pub left: NodeId,
    pub sep_left: NodeId,
    pub middle: NodeId,
    pub sep_right: NodeId,
    pub right: NodeId,
}
```

**Methods:**
```rust
impl Layout {
    /// Populates `parent` as a horizontal flex container with left column, separator, and right column.
    ///
    /// `left_fraction` — proportion of remaining width (after separator) given to the left column.
    ///   E.g. 0.3 gives left 30%, right 70%.
    /// `separator_thickness` — fixed logical-pixel width of the separator leaf (pass `theme.border_width`).
    ///
    /// Sets `parent` style to `display: flex, flex-direction: row`.
    /// Caller must not set children on `parent` after this — the builder owns the child list.
    pub fn two_column(
        &mut self,
        parent: NodeId,
        left_fraction: f32,
        separator_thickness: f32,
    ) -> TwoColumnLayout

    /// Same as `two_column` but with three content columns and two separators.
    ///
    /// `fractions` — relative flex-grow weights for [left, middle, right]. Need not sum to 1.0;
    ///   taffy normalises them. E.g. [1.0, 2.0, 1.0] gives 25/50/25 split.
    /// `separator_thickness` — fixed logical-pixel width of each separator leaf.
    pub fn three_column(
        &mut self,
        parent: NodeId,
        fractions: [f32; 3],
        separator_thickness: f32,
    ) -> ThreeColumnLayout
}
```

**Implementation of `two_column`:**
```rust
pub fn two_column(
    &mut self,
    parent: NodeId,
    left_fraction: f32,
    separator_thickness: f32,
) -> TwoColumnLayout {
    self.set_style(parent, Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        size: Size { width: Dimension::Percent(1.0), height: Dimension::Percent(1.0) },
        ..Default::default()
    });

    let right_fraction = 1.0 - left_fraction.clamp(0.0, 1.0);
    let left_fraction = left_fraction.clamp(0.0, 1.0);

    let left = self.new_leaf(Style {
        flex_grow: left_fraction,
        flex_shrink: 1.0,
        ..Default::default()
    });
    let separator = self.new_leaf(Style {
        flex_grow: 0.0,
        flex_shrink: 0.0,
        size: Size { width: length(separator_thickness), height: Dimension::Auto },
        ..Default::default()
    });
    let right = self.new_leaf(Style {
        flex_grow: right_fraction,
        flex_shrink: 1.0,
        ..Default::default()
    });

    self.set_children(parent, &[left, separator, right]);

    TwoColumnLayout { left, separator, right }
}
```

**Implementation of `three_column`:** same pattern with five children: `[left, sep_left, middle, sep_right, right]`. Each content column gets `flex_grow = fractions[i]`.

**Acceptance criteria:** `cargo test -p akar-layout` passes. Unit test: `two_column` with `left_fraction = 0.5` and `separator_thickness = 1.0` on a 401px-wide parent produces `left.width ≈ 200.0` and `right.width ≈ 200.0` after `compute`. Unit test: `three_column` with `fractions = [1.0, 2.0, 1.0]` on a 402px parent produces `left.width ≈ 100.0`, `middle.width ≈ 200.0`, `right.width ≈ 100.0`.

---

### Task 5: `akar-layout` — `Page` layout builder

**Goal:** A builder that creates the standard full-window application shell: optional header, body row (optional sidebars + main area), optional footer. Returns named `NodeId`s so the developer can draw each region independently.

**File:** `crates/akar-layout/src/lib.rs` — add to `impl Layout`

**Types:**
```rust
pub struct PageConfig {
    pub header_height: Option<f32>,       // logical pixels; None = no header node created
    pub footer_height: Option<f32>,
    pub sidebar_left_width: Option<f32>,
    pub sidebar_right_width: Option<f32>,
}

pub struct PageLayout {
    pub root: NodeId,                      // full-viewport flex-column root
    pub header: Option<NodeId>,
    pub body: NodeId,                      // flex-row; contains sidebars and main
    pub sidebar_left: Option<NodeId>,
    pub main: NodeId,                      // flex-grow: 1 in body row
    pub sidebar_right: Option<NodeId>,
    pub footer: Option<NodeId>,
}
```

**Method:**
```rust
impl Layout {
    /// Creates a page shell taffy subtree rooted at a new node.
    ///
    /// The caller must attach `page.root` to their own root node (or compute layout directly on it).
    /// All sizes are in logical pixels. The root node is sized to fill available space via
    /// `size: 100%` on both axes — set `available` to window dimensions when calling `compute`.
    pub fn page(&mut self, config: PageConfig) -> PageLayout
}
```

**Taffy tree structure:**
```
root  [flex-column, width: 100%, height: 100%]
  header?  [flex-shrink: 0, height: fixed(header_height), width: 100%]
  body     [flex-grow: 1, flex-direction: row, width: 100%]
    sidebar_left?   [flex-shrink: 0, width: fixed(sidebar_left_width), height: 100%]
    main            [flex-grow: 1, height: 100%]
    sidebar_right?  [flex-shrink: 0, width: fixed(sidebar_right_width), height: 100%]
  footer?  [flex-shrink: 0, height: fixed(footer_height), width: 100%]
```

**Implementation notes:**
- Only create nodes for present regions (check `config.header_height.is_some()` etc.).
- Body children list is built conditionally: `[sidebar_left?, main, sidebar_right?]`.
- Root children list: `[header?, body, footer?]`.

**Acceptance criteria:** `cargo test -p akar-layout` passes. Unit test: `page` with `header_height: Some(60.0)`, `sidebar_left_width: Some(200.0)`, no footer, no right sidebar, computed at `(800.0, 600.0)` produces:
- `header.height == 60.0`, `header.width == 800.0`
- `sidebar_left.width == 200.0`, `sidebar_left.height == 540.0`
- `main.width == 600.0`, `main.height == 540.0`

---

### Task 6: `akar-layout` — `responsive_columns`

**Goal:** A pure free function that returns the appropriate column count for a given window width, given a breakpoint table. Zero dependencies, zero state.

**File:** `crates/akar-layout/src/responsive.rs`

```rust
/// Returns the column count for `window_width` given a breakpoint table.
///
/// `breakpoints` is a slice of `(min_width, cols)` pairs. They need not be sorted.
/// The function selects the entry with the largest `min_width` that is still ≤ `window_width`.
/// Returns 1 if no breakpoint matches (window is narrower than all breakpoints).
///
/// # Example
/// ```
/// let cols = responsive_columns(1024.0, &[(600.0, 2), (900.0, 3), (1200.0, 4)]);
/// assert_eq!(cols, 3);
/// ```
pub fn responsive_columns(window_width: f32, breakpoints: &[(f32, usize)]) -> usize {
    breakpoints
        .iter()
        .filter(|(min_w, _)| *min_w <= window_width)
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, cols)| *cols)
        .unwrap_or(1)
}
```

Re-export from `crates/akar-layout/src/lib.rs`:
```rust
mod responsive;
pub use responsive::responsive_columns;
```

**Acceptance criteria:** Unit tests:
- `responsive_columns(500.0, &[(600.0, 2), (900.0, 3)])` → `1`
- `responsive_columns(700.0, &[(600.0, 2), (900.0, 3)])` → `2`
- `responsive_columns(1000.0, &[(600.0, 2), (900.0, 3)])` → `3`
- `responsive_columns(700.0, &[])` → `1`
- Breakpoints given in unsorted order return the correct result.

---

### Task 7: Update `examples/demo-rust` to showcase layout containers

**Goal:** Replace the single-button demo with a page shell that demonstrates all Epic 003 primitives. The demo remains a single Rust binary; no new crate is needed.

**File:** `examples/demo-rust/src/main.rs` — full rewrite

**Layout structure to build:**
```
page.root  (800 × 600)
  page.header  (800 × 48)
  page.body
    page.sidebar_left  (200 × 552)
    page.main  (600 × 552)
      two_col.left   (~297 × 552)
      two_col.separator   (1 × 552)
      two_col.right   (~302 × 552) — wider because right_fraction = 0.508...
        btn_node   (160 × 48, positioned via flex-start)
```

**Setup (in `resumed`):**
```rust
let mut layout = Layout::new();

let page = layout.page(PageConfig {
    header_height: Some(48.0),
    footer_height: None,
    sidebar_left_width: Some(200.0),
    sidebar_right_width: None,
});

// Split main area into two columns with a 1px separator
let two_col = layout.two_column(page.main, 0.5, 1.0);

// Button inside right column
let btn_node = layout.new_leaf(Style {
    size: Size { width: length(160.0), height: length(48.0) },
    ..Default::default()
});
layout.add_child(two_col.right, btn_node);
```

**Per-frame draw order** (in `RedrawRequested` handler):
```
layout.compute(page.root, (Some(width as f32), Some(height as f32)), |_,_,_,_,_| Size::ZERO);

// Header
container(&mut core, &layout, page.header, AKAR_THEME_DARK.base_200, &AKAR_THEME_DARK);
// Sidebar
container(&mut core, &layout, page.sidebar_left, AKAR_THEME_DARK.base_200, &AKAR_THEME_DARK);
// Main — no background (transparent)
// Left column — no background
// Right column — no background
// Separator
separator(&mut core, &layout, two_col.separator, &AKAR_THEME_DARK);
// Button
let result = button(&mut core, &layout, btn_node, "Click me", ButtonVariant::Solid, &AKAR_THEME_DARK);
if result.clicked {
    println!("clicked!");
}
```

**Acceptance criteria:** `cargo run --manifest-path examples/demo-rust/Cargo.toml` compiles and opens a window showing:
- A dark header band across the top.
- A dark sidebar on the left.
- A vertical separator line dividing the right area in half.
- A button in the right half that changes appearance on hover and prints `"clicked!"` on click.
- No panics on window resize (layout recomputes to new dimensions each frame).

---

## Acceptance Criteria for Epic 003

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero errors (including Epic 002 fixes from Task 1).
- [ ] `cargo test --workspace` passes. New tests from Tasks 2–6 cover: separator zero-area, container zero-area and transparent, `two_column` and `three_column` flex widths, `page` region sizes, all five `responsive_columns` cases.
- [ ] `cargo check --manifest-path examples/demo-rust/Cargo.toml` passes.
- [ ] The demo renders a visible page shell with header, sidebar, separator, and a working button.
- [ ] No windowing or event loop code exists in `akar-core`, `akar-layout`, or `akar-components`.
- [ ] No `unsafe` outside `crates/akar-c-api/src/lib.rs`.
- [ ] Grid (`display: grid`) is not implemented; its deferral is documented in ADR-008.
- [ ] Canvas is not implemented; its design contract is documented in ADR-008.

---

## Review Notes

*(Filled during implementation)*
