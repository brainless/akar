# Epic 004: Canvas

**Status:** Planned
**Goal:** Add an infinite-pan / zoom canvas surface as a first-class component. A `Canvas` occupies a taffy leaf node — the layout system allocates it a pixel rect like any other component. Inside that rect the developer controls a world coordinate system (pan offset + zoom). The public API is `canvas_begin` / `canvas_end`, a scoped painter (`CanvasPainter`) that enforces world-space drawing, and a pure `is_visible_world` culling helper. Grid rendering and C ABI bindings are deferred.

**Prerequisite:** Epic 003 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean before Task 1 begins.

---

## Architecture Decision Records

### ADR-009: CanvasPainter Owns a Buffer — No Lifetime on the Draw List Borrow

**Decision:** `CanvasPainter` is a fully owned value — it carries a `Vec<QuadCall>` and a baked-in `CanvasTransform`. It holds no reference into `AkarCore`. `canvas_end` drains the buffer into the real draw list and pops the scissor rect.

**Rationale:** The natural alternative — having `CanvasPainter<'a>` hold `&'a mut DrawList` extracted from `AkarCore` — creates a borrow that blocks all access to `AkarCore` for the duration of the canvas scope. This matters because the developer needs `core.input.mouse_pos` (world-space hit testing against the canvas's transforms) between `canvas_begin` and `canvas_end`. A scoped lifetime would force the developer to capture all needed input state before calling `canvas_begin`, which is workable but fragile. The owned-buffer approach costs one `Vec` allocation per canvas per frame; this is negligible — the main draw list already does the same.

**Draw order invariant:** `canvas_end` drains the buffer into `core.draw_list.push_quad()` while the canvas scissor rect is still active on the scissor stack, so AABB culling fires correctly for each quad. The scissor is popped after the drain.

**Consequences:**
- `canvas_begin(core, layout, node_id, state, config) -> (CanvasResponse, CanvasPainter)` — consistent with all existing component signatures; no new `Ui` wrapper needed.
- `canvas_end(core: &mut AkarCore, painter: CanvasPainter)` — consumes the painter (preventing use-after-end) and borrows `core` only momentarily to drain and pop scissor.
- `AkarCore` is fully accessible between `canvas_begin` and `canvas_end`, including `core.input` for hit tests.
- Draw list changes: **none**. The draw list stays a flat, stateless list of resolved screen-space quads.

---

### ADR-010: Caller-Owned CanvasState; CanvasResponse Carries Per-Frame Facts

**Decision:** `CanvasState { pan: Vec2, zoom: f32, is_panning: bool }` is declared and stored by the application, not by the toolkit. `canvas_begin` mutates it in place (pan, zoom, is_panning) based on this frame's input. `CanvasResponse` carries per-frame read-only facts: whether a drag or zoom happened, the precomputed `world_to_screen` and `screen_to_world` transforms, and the precomputed `visible_world_rect`.

**Pan handling:** `is_panning` in `CanvasState` is the retained flag that distinguishes "drag started inside this canvas" from "button held while mouse happens to be nearby." `canvas_begin` sets it when the pan button is first pressed inside the canvas rect, keeps it while the button is held regardless of current mouse position, and clears it on button release. This correctly handles the "started inside, dragged outside" case without any per-widget ID storage in the toolkit.

**Zoom formula:** The zoom is cursor-anchored — the world point under the cursor stays at the same screen position after the zoom change. The correct formula (derived and verified):
```
world_pos = (screen_cursor - canvas_center) / old_zoom + pan_old
new_zoom  = clamp(old_zoom * zoom_factor, zoom_min, zoom_max)
pan_new   = world_pos - (screen_cursor - canvas_center) / new_zoom
```
This differs from sugacode's `zoom_at_point`, whose formula does not mathematically anchor the cursor point. The akar formula is verified by the unit test in Task 4.

**`CanvasTransform` type:** A simple `{ offset: Vec2, scale: f32 }` in `akar-layout`. The world→screen transform is a uniform scale + translate (no rotation), so a full affine matrix is unnecessary. `apply(pt: Vec2) -> Vec2` computes `pt * scale + offset`. The world→screen instance has `offset = canvas_center - pan * zoom` and `scale = zoom`. The screen→world instance has `offset = pan - canvas_center / zoom` and `scale = 1.0 / zoom`.

**`Rect` type:** `{ min: Vec2, max: Vec2 }` in `akar-layout`. Used for world-space geometry. The existing screen-space `[f32; 4]` (x, y, w, h) convention is kept for draw list calls.

---

### ADR-011: Deferred Items

**C ABI (`akar_canvas_begin`, `akar_canvas_end`, `AkarCanvasResponse`):** Deferred. The Rust API must be stable before the C surface is worth specifying. Canvas via FFI requires deciding how `CanvasPainter` is represented across the ABI boundary — likely as an opaque heap-allocated handle with explicit alloc/free, which is a non-trivial addition to `akar-c-api`.

**Grid rendering:** Deferred. The developer can draw grid lines as world-space quads using `CanvasPainter::push_quad`. A built-in `canvas_grid` helper that computes visible grid lines from pan/zoom and pushes them to the painter is a logical Epic 005 addition, but is not required to prove the canvas model.

**`CanvasPainter::push_text`:** Deferred. Text in world space requires `TextPipeline` access inside `canvas_end` to issue the glyphon prepare call. This makes `canvas_end` more complex and is not needed for Epic 004's acceptance criteria. World-space text is the primary use case for the first follow-up.

**Canvas-in-Canvas:** Disallowed by design. ADR-008 from Epic 003 documents this constraint. `CanvasPainter` has no `canvas_begin` method.

---

## Tasks

### Task 1: `akar-layout` — `Rect` and `CanvasTransform`

**Goal:** Two new geometric types used by the canvas — `Rect` for world-space bounding boxes and `CanvasTransform` for the scale+translate coordinate mapping — plus three pure constructor functions. No changes to existing `akar-layout` code.

**File:** `crates/akar-layout/src/rect.rs`

```rust
use glam::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self {
            min: Vec2::new(x, y),
            max: Vec2::new(x + w, y + h),
        }
    }

    pub fn intersects(self, other: Rect) -> bool {
        other.max.x >= self.min.x
            && other.min.x <= self.max.x
            && other.max.y >= self.min.y
            && other.min.y <= self.max.y
    }
}
```

**File:** `crates/akar-layout/src/canvas_transform.rs`

```rust
use glam::Vec2;
use crate::Rect;

#[derive(Clone, Copy, Debug)]
pub struct CanvasTransform {
    pub offset: Vec2,  // additive term after scale
    pub scale: f32,
}

impl CanvasTransform {
    pub fn apply(self, pt: Vec2) -> Vec2 {
        pt * self.scale + self.offset
    }

    /// Transforms a world-space `Rect` to a screen-space `[x, y, w, h]`.
    /// Also scales `corner_radii` by the zoom so world-space radii appear
    /// proportionally correct at all zoom levels.
    pub fn apply_rect(self, rect: Rect) -> [f32; 4] {
        let min = self.apply(rect.min);
        let max = self.apply(rect.max);
        [min.x, min.y, max.x - min.x, max.y - min.y]
    }

    pub fn scale_radius(self, radius: f32) -> f32 {
        radius * self.scale
    }
}

/// Returns the world→screen transform for a canvas with the given pan/zoom
/// and screen-space rect `[x, y, w, h]`.
///
/// Formula: `screen = (world - pan) * zoom + canvas_center`
/// Implemented as: `screen = world * zoom + (canvas_center - pan * zoom)`
pub fn make_world_to_screen(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> CanvasTransform {
    let canvas_center = Vec2::new(
        canvas_rect[0] + canvas_rect[2] * 0.5,
        canvas_rect[1] + canvas_rect[3] * 0.5,
    );
    CanvasTransform {
        offset: canvas_center - pan * zoom,
        scale: zoom,
    }
}

/// Returns the screen→world transform.
///
/// Formula: `world = (screen - canvas_center) / zoom + pan`
/// Implemented as: `world = screen * (1/zoom) + (pan - canvas_center / zoom)`
pub fn make_screen_to_world(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> CanvasTransform {
    let canvas_center = Vec2::new(
        canvas_rect[0] + canvas_rect[2] * 0.5,
        canvas_rect[1] + canvas_rect[3] * 0.5,
    );
    CanvasTransform {
        offset: pan - canvas_center / zoom,
        scale: 1.0 / zoom,
    }
}

/// Unprojects the four corners of `canvas_rect` into world space to give the
/// axis-aligned bounding rect of what is currently visible.
pub fn compute_visible_world_rect(pan: Vec2, zoom: f32, canvas_rect: [f32; 4]) -> Rect {
    let s2w = make_screen_to_world(pan, zoom, canvas_rect);
    let tl = Vec2::new(canvas_rect[0], canvas_rect[1]);
    let br = Vec2::new(canvas_rect[0] + canvas_rect[2], canvas_rect[1] + canvas_rect[3]);
    Rect {
        min: s2w.apply(tl),
        max: s2w.apply(br),
    }
}
```

**`crates/akar-layout/src/lib.rs`** — add at the top:

```rust
mod rect;
pub use rect::Rect;

mod canvas_transform;
pub use canvas_transform::{
    CanvasTransform,
    make_world_to_screen,
    make_screen_to_world,
    compute_visible_world_rect,
};
```

**Acceptance criteria:** `cargo test -p akar-layout` passes. Unit tests (added in Task 4) cover: `apply` identity, `apply_rect` dimensions, `intersects` all four cases, `compute_visible_world_rect` at default pan/zoom, round-trip world→screen→world.

---

### Task 2: `akar-components` — Canvas Types

**Goal:** The five public types that make up the canvas API. No GPU interaction, no draw list calls. All state types implement `Default`.

**File:** `crates/akar-components/src/canvas.rs`

```rust
use glam::Vec2;
use akar_core::QuadCall;
use akar_layout::{Rect, CanvasTransform, make_world_to_screen};
use crate::color::color_to_f32;

// ── Configuration ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PanButton {
    Middle,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct CanvasConfig {
    pub pan_button: PanButton,
    pub zoom_sensitivity: f32,  // zoom_factor = 1.0 + scroll_y * sensitivity
    pub zoom_min: f32,
    pub zoom_max: f32,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            pan_button: PanButton::Middle,
            zoom_sensitivity: 0.005,
            zoom_min: 0.1,
            zoom_max: 5.0,
        }
    }
}

// ── Per-frame caller state ────────────────────────────────────────────────────

pub struct CanvasState {
    pub pan: Vec2,
    pub zoom: f32,
    pub is_panning: bool,   // managed by canvas_begin; set true when pan button
                            // first pressed inside canvas, cleared on release
}

impl CanvasState {
    pub fn new() -> Self {
        Self { pan: Vec2::ZERO, zoom: 1.0, is_panning: false }
    }

    /// Adjusts pan so that `screen_pos` stays fixed over the same world point
    /// after applying `zoom_factor` to the current zoom.
    pub fn zoom_at_point(
        &mut self,
        screen_pos: Vec2,
        canvas_rect: [f32; 4],
        zoom_factor: f32,
        zoom_min: f32,
        zoom_max: f32,
    ) {
        let canvas_center = Vec2::new(
            canvas_rect[0] + canvas_rect[2] * 0.5,
            canvas_rect[1] + canvas_rect[3] * 0.5,
        );
        let world_pos = (screen_pos - canvas_center) / self.zoom + self.pan;
        let new_zoom = (self.zoom * zoom_factor).clamp(zoom_min, zoom_max);
        // Solve: screen_pos = (world_pos - pan_new) * new_zoom + canvas_center
        self.pan = world_pos - (screen_pos - canvas_center) / new_zoom;
        self.zoom = new_zoom;
    }
}

impl Default for CanvasState {
    fn default() -> Self { Self::new() }
}

// ── Per-frame response ────────────────────────────────────────────────────────

pub struct CanvasResponse {
    pub dragged: bool,
    pub zoomed: bool,
    /// World→screen transform valid for this frame. Use to convert world-space
    /// positions to screen positions when needed outside the painter scope.
    pub world_to_screen: CanvasTransform,
    /// Screen→world transform. Use to convert `core.input.mouse_pos` into world
    /// space for hit testing between canvas_begin and canvas_end.
    pub screen_to_world: CanvasTransform,
    /// Axis-aligned world-space rect that is currently visible inside the canvas.
    /// Pass as `viewport` to `is_visible_world` to cull off-screen objects before
    /// calling `painter.push_quad`.
    pub visible_world_rect: Rect,
}

// ── Painter ───────────────────────────────────────────────────────────────────

pub struct CanvasPainter {
    pub(crate) buffer: Vec<QuadCall>,
    pub(crate) world_to_screen: CanvasTransform,
}

impl CanvasPainter {
    /// Draws a solid quad at `world_rect`.
    ///
    /// `corner_radii` are in world-space units and are scaled by the current
    /// zoom before being handed to the quad pipeline, so they appear
    /// proportionally correct at all zoom levels.
    ///
    /// `z` has the same semantics as in the main draw list.
    pub fn push_quad(
        &mut self,
        world_rect: Rect,
        fill: u32,
        border_color: u32,
        border_width: f32,
        corner_radii: [f32; 4],
        z: f32,
    ) {
        let screen_rect = self.world_to_screen.apply_rect(world_rect);
        let scaled_radii = corner_radii.map(|r| self.world_to_screen.scale_radius(r));
        self.buffer.push(QuadCall {
            rect: screen_rect,
            fill: color_to_f32(fill),
            border_color: color_to_f32(border_color),
            border_width,
            corner_radii: scaled_radii,
            z,
            _pad: [0.0; 2],
        });
    }
}
```

Re-export from `crates/akar-components/src/lib.rs`:

```rust
pub mod canvas;
pub use canvas::{
    CanvasConfig, CanvasPainter, CanvasResponse, CanvasState, PanButton,
    canvas_begin, canvas_end, is_visible_world,
};
```

**Acceptance criteria:** `cargo check -p akar-components` passes. `CanvasState::default()` gives `pan = ZERO`, `zoom = 1.0`. `CanvasPainter::push_quad` with a world rect compiles without accessing `AkarCore`.

---

### Task 3: `akar-components` — `canvas_begin`, `canvas_end`, `is_visible_world`

**Goal:** The three public functions that complete the canvas API. All in `crates/akar-components/src/canvas.rs`, added after the type definitions from Task 2.

```rust
use akar_core::AkarCore;
use akar_layout::{Layout, NodeId, make_screen_to_world, compute_visible_world_rect};

/// Opens a canvas scope. Mutates `state` in place (pan, zoom, is_panning).
///
/// Must be paired with exactly one call to `canvas_end`. The returned
/// `CanvasPainter` is the only way to draw inside this canvas; it accepts
/// world-space coordinates and transforms them internally.
///
/// The canvas scissor rect is pushed to `core.draw_list` during this call.
/// It is popped by `canvas_end`. `AkarCore` is fully accessible between
/// the two calls, including `core.input` for world-space hit testing via
/// `response.screen_to_world`.
pub fn canvas_begin(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    state: &mut CanvasState,
    config: &CanvasConfig,
) -> (CanvasResponse, CanvasPainter) {
    let rect = layout.rect(node_id);

    // Push canvas rect as scissor so off-canvas quads are culled during canvas_end.
    core.draw_list.push_scissor(rect);

    // ── Pan input ────────────────────────────────────────────────────────────
    let pan_btn = match config.pan_button {
        PanButton::Middle => 2,
        PanButton::Right => 1,
    };

    // Start panning when the button is first pressed inside the canvas rect.
    if core.input.mouse_buttons_pressed[pan_btn] && core.input.is_hovering(rect) {
        state.is_panning = true;
    }
    // Stop panning when the button is released (regardless of cursor position).
    if !core.input.mouse_buttons[pan_btn] {
        state.is_panning = false;
    }

    let mut dragged = false;
    if state.is_panning {
        let delta = (core.input.mouse_pos - core.input.mouse_pos_prev) / state.zoom;
        if delta != Vec2::ZERO {
            state.pan -= delta;
            dragged = true;
        }
    }

    // ── Zoom input ───────────────────────────────────────────────────────────
    let mut zoomed = false;
    let scroll_y = core.input.scroll_delta.y;
    if scroll_y != 0.0 && core.input.is_hovering(rect) {
        let zoom_factor = 1.0 + scroll_y * config.zoom_sensitivity;
        if zoom_factor > 0.0 {
            state.zoom_at_point(
                core.input.mouse_pos,
                rect,
                zoom_factor,
                config.zoom_min,
                config.zoom_max,
            );
            zoomed = true;
        }
    }

    // ── Build response ───────────────────────────────────────────────────────
    let world_to_screen = make_world_to_screen(state.pan, state.zoom, rect);
    let screen_to_world = make_screen_to_world(state.pan, state.zoom, rect);
    let visible_world_rect = compute_visible_world_rect(state.pan, state.zoom, rect);

    let response = CanvasResponse { dragged, zoomed, world_to_screen, screen_to_world, visible_world_rect };
    let painter = CanvasPainter { buffer: Vec::new(), world_to_screen };

    (response, painter)
}

/// Closes the canvas scope opened by `canvas_begin`.
///
/// Drains `painter`'s buffer into `core.draw_list` (with AABB culling against
/// the active canvas scissor) then pops the scissor rect.
pub fn canvas_end(core: &mut AkarCore, painter: CanvasPainter) {
    for quad in painter.buffer {
        core.draw_list.push_quad(quad);
    }
    core.draw_list.pop_scissor();
}

/// Returns `true` if `target` overlaps `viewport` (inclusive AABB intersection).
///
/// Both arguments are in world-space coordinates. Pass `response.visible_world_rect`
/// as `viewport` to skip `painter.push_quad` calls for fully off-screen objects.
///
/// # Example
/// ```
/// for obj in &self.objects {
///     if is_visible_world(response.visible_world_rect, obj.bounds) {
///         painter.push_quad(obj.bounds, obj.color, 0, 0.0, [0.0; 4], 0.0);
///     }
/// }
/// ```
pub fn is_visible_world(viewport: Rect, target: Rect) -> bool {
    viewport.intersects(target)
}
```

**Acceptance criteria:** `cargo test -p akar-components` passes. Unit tests from Task 4 cover: `canvas_begin` pushes one scissor; `canvas_end` drains buffer and pops it; `is_visible_world` with inside/outside/touching cases.

---

### Task 4: Tests

**Goal:** Pure unit tests for all transform math and the canvas begin/end lifecycle. No GPU required. All tests in `#[cfg(test)]` blocks inside the relevant source files.

**`crates/akar-layout/src/canvas_transform.rs`** — add at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    const CANVAS: [f32; 4] = [0.0, 0.0, 800.0, 600.0]; // x=0, y=0, w=800, h=600
    const CENTER: Vec2 = Vec2::new(400.0, 300.0);

    #[test]
    fn world_to_screen_identity() {
        // pan=0, zoom=1 → world origin maps to canvas center
        let t = make_world_to_screen(Vec2::ZERO, 1.0, CANVAS);
        let s = t.apply(Vec2::ZERO);
        assert!((s - CENTER).length() < 0.001, "got {s}");
    }

    #[test]
    fn world_to_screen_with_pan() {
        // pan=(100,0), zoom=1 → world (100,0) maps to canvas center
        let t = make_world_to_screen(Vec2::new(100.0, 0.0), 1.0, CANVAS);
        let s = t.apply(Vec2::new(100.0, 0.0));
        assert!((s - CENTER).length() < 0.001, "got {s}");
    }

    #[test]
    fn world_to_screen_with_zoom() {
        // pan=0, zoom=2 → world (1,0) is 2px right of canvas center
        let t = make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let s = t.apply(Vec2::new(1.0, 0.0));
        assert!((s - Vec2::new(402.0, 300.0)).length() < 0.001, "got {s}");
    }

    #[test]
    fn world_to_screen_off_center_canvas() {
        // Canvas rect offset by 200px (sidebar). World origin must map to *that* canvas's center.
        let canvas = [200.0, 0.0, 600.0, 600.0];
        let expected_center = Vec2::new(500.0, 300.0);
        let t = make_world_to_screen(Vec2::ZERO, 1.0, canvas);
        let s = t.apply(Vec2::ZERO);
        assert!((s - expected_center).length() < 0.001, "got {s}");
    }

    #[test]
    fn round_trip() {
        // screen_to_world(world_to_screen(p)) ≈ p for arbitrary pan/zoom
        let canvas = [50.0, 100.0, 700.0, 500.0];
        let pan = Vec2::new(123.0, -45.0);
        let zoom = 1.7;
        let world = Vec2::new(200.0, -80.0);
        let w2s = make_world_to_screen(pan, zoom, canvas);
        let s2w = make_screen_to_world(pan, zoom, canvas);
        let back = s2w.apply(w2s.apply(world));
        assert!((back - world).length() < 0.001, "round-trip error: {back}");
    }

    #[test]
    fn visible_world_rect_identity() {
        // pan=0, zoom=1 → visible rect is ±(half_w, half_h) around world origin
        let v = compute_visible_world_rect(Vec2::ZERO, 1.0, CANVAS);
        assert!((v.min - Vec2::new(-400.0, -300.0)).length() < 0.001);
        assert!((v.max - Vec2::new(400.0, 300.0)).length() < 0.001);
    }

    #[test]
    fn visible_world_rect_zoom2() {
        // zoom=2 → visible world area is halved
        let v = compute_visible_world_rect(Vec2::ZERO, 2.0, CANVAS);
        assert!((v.min - Vec2::new(-200.0, -150.0)).length() < 0.001);
        assert!((v.max - Vec2::new(200.0, 150.0)).length() < 0.001);
    }

    #[test]
    fn apply_rect_dimensions() {
        // A 10×10 world rect at zoom=2 should produce a 20×20 screen rect.
        let t = make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let world_rect = crate::Rect { min: Vec2::new(-5.0, -5.0), max: Vec2::new(5.0, 5.0) };
        let [_x, _y, w, h] = t.apply_rect(world_rect);
        assert!((w - 20.0).abs() < 0.001);
        assert!((h - 20.0).abs() < 0.001);
    }
}
```

**`crates/akar-layout/src/rect.rs`** — add at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
        Rect { min: Vec2::new(x0, y0), max: Vec2::new(x1, y1) }
    }

    #[test]
    fn intersects_inside()   { assert!(r(-10., -10., 10., 10.).intersects(r(-5., -5., 5., 5.))); }
    #[test]
    fn intersects_outside()  { assert!(!r(-10., -10., 10., 10.).intersects(r(20., 20., 30., 30.))); }
    #[test]
    fn intersects_touching()  { assert!(r(0., 0., 10., 10.).intersects(r(10., 0., 20., 10.))); }
    #[test]
    fn intersects_partial()  { assert!(r(0., 0., 10., 10.).intersects(r(5., 5., 15., 15.))); }
}
```

**`crates/akar-components/src/canvas.rs`** — add at the bottom:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    const CANVAS: [f32; 4] = [0.0, 0.0, 800.0, 600.0];

    #[test]
    fn zoom_at_point_anchors_cursor() {
        // The world point under the cursor stays at the same screen position after zoom.
        let mut state = CanvasState::new();
        let cursor = Vec2::new(500.0, 300.0);
        let canvas_center = Vec2::new(400.0, 300.0);

        // Record the world position under the cursor before zoom.
        let world_before = (cursor - canvas_center) / state.zoom + state.pan;

        state.zoom_at_point(cursor, CANVAS, 2.0, 0.1, 5.0);

        // After zoom, the same world position should project to the same screen position.
        let screen_after = (world_before - state.pan) * state.zoom + canvas_center;
        assert!((screen_after - cursor).length() < 0.001, "got {screen_after}");
    }

    #[test]
    fn zoom_clamps_at_min() {
        let mut state = CanvasState { pan: Vec2::ZERO, zoom: 0.15, is_panning: false };
        state.zoom_at_point(Vec2::new(400.0, 300.0), CANVAS, 0.1, 0.1, 5.0);
        assert!(state.zoom >= 0.1);
    }

    #[test]
    fn zoom_clamps_at_max() {
        let mut state = CanvasState { pan: Vec2::ZERO, zoom: 4.9, is_panning: false };
        state.zoom_at_point(Vec2::new(400.0, 300.0), CANVAS, 10.0, 0.1, 5.0);
        assert!(state.zoom <= 5.0);
    }

    #[test]
    fn is_visible_world_cases() {
        let viewport = Rect { min: Vec2::new(-100.0, -100.0), max: Vec2::new(100.0, 100.0) };
        let inside   = Rect { min: Vec2::new(-50.0, -50.0),   max: Vec2::new(50.0, 50.0) };
        let outside  = Rect { min: Vec2::new(200.0, 200.0),   max: Vec2::new(300.0, 300.0) };
        let touching = Rect { min: Vec2::new(100.0, -50.0),   max: Vec2::new(200.0, 50.0) };
        let partial  = Rect { min: Vec2::new(50.0, 50.0),     max: Vec2::new(150.0, 150.0) };
        assert!(is_visible_world(viewport, inside));
        assert!(!is_visible_world(viewport, outside));
        assert!(is_visible_world(viewport, touching));
        assert!(is_visible_world(viewport, partial));
    }

    #[test]
    fn push_quad_transforms_rect() {
        // CanvasPainter::push_quad should store a screen-space QuadCall.
        // At pan=0, zoom=2, canvas [0,0,800,600]:
        // world rect (-5,-5)→(5,5) should become screen rect (390,290)→(410,310),
        // i.e. 20×20 centered on canvas center (400,300).
        let w2s = akar_layout::make_world_to_screen(Vec2::ZERO, 2.0, CANVAS);
        let mut painter = CanvasPainter { buffer: Vec::new(), world_to_screen: w2s };
        let world_rect = Rect { min: Vec2::new(-5.0, -5.0), max: Vec2::new(5.0, 5.0) };
        painter.push_quad(world_rect, 0xFF0000FF, 0x00000000, 0.0, [0.0; 4], 0.0);
        assert_eq!(painter.buffer.len(), 1);
        let [x, y, w, h] = painter.buffer[0].rect;
        assert!((x - 390.0).abs() < 0.001, "x={x}");
        assert!((y - 290.0).abs() < 0.001, "y={y}");
        assert!((w - 20.0).abs() < 0.001,  "w={w}");
        assert!((h - 20.0).abs() < 0.001,  "h={h}");
    }
}
```

**Acceptance criteria:** `cargo test --workspace` passes. New tests: 8 in `canvas_transform`, 4 in `rect`, 5 in canvas component = 17 tests total. All pure (no GPU).

---

### Task 5: Update `examples/demo-rust` — Canvas Demo

**Goal:** Replace the two-column split in `page.main` with a canvas that fills the entire main area. Show four colored world-space rectangles at fixed world positions. Pan with middle mouse; scroll to zoom. Demonstrate that the existing header and sidebar are unaffected.

**Layout change in `examples/demo-rust/src/main.rs`:**

Remove the `two_column` call and `btn_node`. Make `page.main` the canvas node directly — it is already a taffy leaf with `flex-grow: 1`. No new taffy nodes are needed.

```rust
// Remove:
// let two_col = layout.two_column(page.main, 0.5, 1.0);
// let btn_node = layout.new_leaf(...);
// layout.add_child(two_col.right, btn_node);

// State to add to the application struct (or local variable in resumed):
let mut canvas_state = CanvasState::new();
```

**Demo objects (five world-space rects to render):**

```rust
struct DemoObject {
    bounds: Rect,
    fill: u32,
}

let objects = [
    DemoObject { bounds: Rect::from_xywh(-180.0, -80.0, 120.0, 60.0), fill: 0x3B82F6FF }, // blue
    DemoObject { bounds: Rect::from_xywh(80.0,  -80.0, 120.0, 60.0),  fill: 0x10B981FF }, // green
    DemoObject { bounds: Rect::from_xywh(-60.0,  40.0, 120.0, 60.0),  fill: 0xF59E0BFF }, // amber
    DemoObject { bounds: Rect::from_xywh(-280.0, 60.0,  80.0, 80.0),  fill: 0xEF4444FF }, // red
    DemoObject { bounds: Rect::from_xywh(200.0,  20.0, 100.0, 100.0), fill: 0x8B5CF6FF }, // purple
];
```

**Per-frame draw sequence (in `RedrawRequested` handler):**

```rust
layout.compute(page.root, (Some(width as f32), Some(height as f32)), |_, _, _, _, _| Size::ZERO);

// Static chrome (unchanged from Epic 003):
container(&mut core, &layout, page.header,       AKAR_THEME_DARK.base_200, &AKAR_THEME_DARK);
container(&mut core, &layout, page.sidebar_left, AKAR_THEME_DARK.base_200, &AKAR_THEME_DARK);

// Canvas in page.main:
let config = CanvasConfig::default();
let (response, mut painter) = canvas_begin(&mut core, &layout, page.main, &mut canvas_state, &config);

for obj in &objects {
    if is_visible_world(response.visible_world_rect, obj.bounds) {
        painter.push_quad(obj.bounds, obj.fill, 0x00000000, 0.0, [8.0; 4], 0.0);
    }
}

canvas_end(&mut core, painter);
```

**Acceptance criteria:** `cargo run --manifest-path examples/demo-rust/Cargo.toml` compiles and opens a window showing:
- Dark header band and dark sidebar (unchanged from Epic 003).
- Five colored rounded rectangles in the canvas area at `zoom = 1.0`.
- Middle-mouse drag pans the canvas — rectangles move together.
- Scroll wheel zooms centered on the cursor — rectangles scale and translate correctly.
- All five rectangles disappear from the draw list (not just the screen) when scrolled fully off canvas — confirmed by the `is_visible_world` culling path.
- No panics on window resize (layout recomputes each frame).
- No panics when mouse is outside the canvas while scrolling (zoom only triggers when `is_hovering` is true).

---

## Acceptance Criteria for Epic 004

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero errors.
- [ ] `cargo test --workspace` passes. New tests: 4 in `Rect`, 8 in `CanvasTransform`, 5 in canvas component = 17 new tests, all pure.
- [ ] `cargo check --manifest-path examples/demo-rust/Cargo.toml` passes.
- [ ] Demo renders five world-space objects with correct pan and cursor-anchored zoom.
- [ ] Panning started outside the canvas rect does not move the canvas (is_panning gate).
- [ ] Scrolling outside the canvas rect does not zoom the canvas (is_hovering gate).
- [ ] `is_visible_world` culling is exercised: zoom out until objects leave the visible world rect and confirm they are absent from `painter.buffer`.
- [ ] No windowing or event loop code added to `akar-core`, `akar-layout`, or `akar-components`.
- [ ] No `unsafe` outside `crates/akar-c-api/src/lib.rs`.
- [ ] Canvas-in-Canvas is not possible: `CanvasPainter` has no `canvas_begin` method.
- [ ] C ABI surface for canvas is not implemented; its deferral is noted in ADR-011.
- [ ] Grid rendering is not implemented; its deferral is noted in ADR-011.

---

## Review Notes

### Task 1 — `akar-layout`: Rect and CanvasTransform
- Created `rect.rs` (Rect with from_xywh, intersects) and `canvas_transform.rs` (CanvasTransform with apply/apply_rect/scale_radius, plus make_world_to_screen/make_screen_to_world/compute_visible_world_rect)
- Updated `lib.rs` with mod declarations and pub use re-exports
- Code matches epic spec exactly; no existing code modified
- `cargo check -p akar-layout` and `cargo clippy -p akar-layout -- -D warnings` both pass clean

### Task 2 — `akar-components`: Canvas types
- Created `canvas.rs` with PanButton, CanvasConfig, CanvasState (with zoom_at_point), CanvasResponse, CanvasPainter (with push_quad)
- Updated `lib.rs` with canvas module and re-exports
- Added `glam.workspace = true` to akar-components/Cargo.toml
- `cargo check -p akar-components` and `cargo clippy -p akar-components -- -D warnings` both pass clean

### Task 3 — `akar-components`: canvas_begin, canvas_end, is_visible_world
- Added canvas_begin (scissor push, pan/zoom input handling, response/painter construction), canvas_end (buffer drain, scissor pop), is_visible_world (delegates to Rect::intersects)
- Updated lib.rs re-exports to include the three functions
- `cargo check -p akar-components` and `cargo clippy -p akar-components -- -D warnings` both pass clean
