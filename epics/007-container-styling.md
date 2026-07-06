# Epic 007: Container Styling

**Status:** In Progress
**Goal:** Give `container` HTML/CSS-level visual capabilities: configurable fill, border, per-corner radii, alpha, and soft drop shadow — all driven by a new `BoxStyle` type. Add `Layout::set_padding` and `Layout::set_margin` so padding and margin can be adjusted after node creation. Expose the new surface in the C ABI.

**Prerequisite:** Epic 006 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean before Task 1 begins.

---

## Architecture Decision Records

### ADR-018: Shadow in the Fragment Shader — Single Draw Call

**Decision:** Drop shadow is rendered in the same fragment shader invocation as the main box, not as a separate draw call. The vertex shader expands the rendered quad to cover the shadow region; the fragment shader composites shadow underneath the main box using SDF.

**Rationale:** A two-quad approach (shadow quad + main quad) requires a z-ordering contract between the two quads and doubles the draw call count per shadowed container. The GPUI pattern — expanding vertex bounds and doing everything in the fragment shader — is cleaner, has no z-ordering issue, and costs one extra SDF evaluation per fragment, which is negligible. The shader already has the SDF infrastructure for rounded corners.

**Shadow SDF:** The shadow box is a rounded rect centered at `local_pos - shadow_offset` with `half_size + shadow_spread` and the same corner radii as the main box. Soft edges are produced by dividing the distance by `shadow_blur` before clamping. When `shadow_blur == 0` this degrades to a hard-edge copy of the box.

**Vertex expansion:** To cover the shadow region, the vertex shader computes per-side padding:
```
pad_left   = max(0, -offset.x + blur + spread)
pad_right  = max(0,  offset.x + blur + spread)
pad_top    = max(0, -offset.y + blur + spread)
pad_bottom = max(0,  offset.y + blur + spread)
```
The expanded rect is passed to the rasterizer. `local_pos` is kept relative to the ORIGINAL box center so the box SDF remains correct.

**"No shadow" sentinel:** `shadow_color.a == 0.0` disables all shadow computation and vertex expansion collapses to zero extra area. Existing components that zero `shadow_color` pay no cost.

**Consequences:**
- `QuadCall` gains 4 shadow fields (shadow_blur, shadow_spread, shadow_color, shadow_offset) — struct grows from 80 to 112 bytes. All `QuadCall` construction sites must zero the new fields for non-shadowed quads.
- WGSL `QuadInstance` struct and `VertexOutput` struct are updated to match.
- `quad_pipeline.rs` is unchanged — it reads `QuadCall` generically via bytemuck.

---

### ADR-019: `BoxStyle` Owns Visual Properties; Taffy Owns Layout

**Decision:** `BoxStyle` captures all visual properties (fill, border, radii, shadow). Padding and margin are NOT in `BoxStyle` — they are set on the taffy node via `Layout::set_padding` / `Layout::set_margin`, and resolved by taffy before the container renders. The container renders exactly the taffy-resolved rect including the padding region; children are positioned by taffy inside that rect automatically.

**Rationale:** This matches CSS box model semantics: the background covers the padding area, and children render inside the content area. Taffy already handles this correctly. Duplicating padding in `BoxStyle` would create a second source of truth with no benefit.

**Consequences:**
- `container(core, layout, node_id, &BoxStyle)` — no theme parameter, no padding parameter. Theme presets are factory methods on `BoxStyle` (`BoxStyle::card(theme)`, `BoxStyle::panel(theme)`, `BoxStyle::surface(theme)`).
- **Breaking change:** The existing `container(core, layout, node_id, background: u32, theme: &AkarTheme)` signature changes. The two call sites in `examples/demo-rust` are updated as part of Task 5.

---

### ADR-020: `Layout::set_padding` and `Layout::set_margin` in `akar-layout`

**Decision:** Add `set_padding(node, top, right, bottom, left)` and `set_margin(node, top, right, bottom, left)` to `Layout`. Both read the current taffy style, update the relevant fields, and call `set_style`. Dimensions use `taffy::style::LengthPercentage::Length` (logical pixels).

**Rationale:** The C ABI caller needs a way to set padding/margin after node creation without receiving a taffy `Style` struct. These two helpers are the minimum needed for HTML/CSS-level layout control.

---

### ADR-021: Deferred Items

**Inset (inner) shadow:** CSS `inset` box-shadow requires the shadow to be drawn INSIDE the box border. This inverts the SDF culling logic. Deferred — outer shadow covers the dominant use case.

**Multiple shadows per box:** CSS allows comma-separated `box-shadow` lists. Requires either multiple QuadCall structs per box or a shadow array in the shader. Deferred.

**World-space shadow in CanvasPainter:** Shadow offset and blur would need to be scaled by zoom. Deferred. `CanvasPainter::push_quad` zeros shadow fields.

**`akar_set_style` (full taffy Style from C):** Exposing the full `Style` struct over the C ABI requires an `AkarStyle` struct that mirrors taffy. Deferred. `akar_set_padding` and `akar_set_margin` cover the primary use case.

---

## Tasks

### Task 1: Extend `QuadCall` and `quad.wgsl` for Shadow

**Goal:** Grow `QuadCall` from 80 to 112 bytes by adding shadow fields, update the WGSL shader to compute and composite the shadow, and verify that existing non-shadowed quads produce identical output (zero shadow fields = no change in appearance).

**File:** `crates/akar-core/src/draw_list.rs` — replace `QuadCall`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
// Layout (112 bytes, 7 × 16-byte chunks):
//   offset  0 — rect           vec4
//   offset 16 — fill           vec4
//   offset 32 — border_color   vec4
//   offset 48 — corner_radii   vec4  (must be 16-byte aligned)
//   offset 64 — border_width, z, shadow_blur, shadow_spread   (4 × f32)
//   offset 80 — shadow_color   vec4
//   offset 96 — shadow_offset  vec2, _pad vec2
pub struct QuadCall {
    pub rect: [f32; 4],
    pub fill: [f32; 4],
    pub border_color: [f32; 4],
    pub corner_radii: [f32; 4],
    pub border_width: f32,
    pub z: f32,
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_color: [f32; 4],
    pub shadow_offset: [f32; 2],
    pub _pad: [f32; 2],
}
```

**File:** `crates/akar-core/src/shaders/quad.wgsl` — full replacement:

```wgsl
struct Params {
    screen_resolution: vec2<u32>,
}

struct QuadInstance {
    rect: vec4<f32>,           // x, y, w, h in logical pixels
    fill: vec4<f32>,           // RGBA
    border_color: vec4<f32>,   // RGBA
    corner_radii: vec4<f32>,   // tl, tr, br, bl
    border_width: f32,
    z: f32,
    shadow_blur: f32,
    shadow_spread: f32,
    shadow_color: vec4<f32>,   // RGBA — alpha 0 disables shadow
    shadow_offset: vec2<f32>,  // dx, dy in logical pixels
    _pad: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> quads: array<QuadInstance>;
@group(0) @binding(1) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) half_size: vec2<f32>,
    @location(2) fill: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) @interpolate(flat) border_width: f32,
    @location(5) @interpolate(flat) corner_radii: vec4<f32>,
    @location(6) @interpolate(flat) shadow_color: vec4<f32>,
    @location(7) @interpolate(flat) shadow_params: vec4<f32>, // offset.x, offset.y, blur, spread
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let q = quads[instance_index];

    // Expand rendered quad to cover the shadow region.
    // When shadow_color.a == 0 all pads are 0 and expansion is a no-op.
    let pad_l = max(0.0, -q.shadow_offset.x + q.shadow_blur + q.shadow_spread);
    let pad_r = max(0.0,  q.shadow_offset.x + q.shadow_blur + q.shadow_spread);
    let pad_t = max(0.0, -q.shadow_offset.y + q.shadow_blur + q.shadow_spread);
    let pad_b = max(0.0,  q.shadow_offset.y + q.shadow_blur + q.shadow_spread);

    let ex = vec4<f32>(
        q.rect.x - pad_l,
        q.rect.y - pad_t,
        q.rect.z + pad_l + pad_r,
        q.rect.w + pad_t + pad_b,
    );

    var uv: vec2<f32>;
    switch vertex_index {
        case 0u: { uv = vec2<f32>(0.0, 0.0); }
        case 1u: { uv = vec2<f32>(1.0, 0.0); }
        case 2u: { uv = vec2<f32>(0.0, 1.0); }
        case 3u: { uv = vec2<f32>(0.0, 1.0); }
        case 4u: { uv = vec2<f32>(1.0, 0.0); }
        default: { uv = vec2<f32>(1.0, 1.0); }
    }

    let pixel_pos = ex.xy + uv * ex.zw;
    let clip_pos = 2.0 * pixel_pos / vec2<f32>(params.screen_resolution) - 1.0;

    // local_pos is relative to the ORIGINAL box center, not the expanded quad.
    let box_center = q.rect.xy + q.rect.zw * 0.5;

    var out: VertexOutput;
    out.position       = vec4<f32>(clip_pos.x, -clip_pos.y, q.z, 1.0);
    out.local_pos      = pixel_pos - box_center;
    out.half_size      = q.rect.zw * 0.5;
    out.fill           = q.fill;
    out.border_color   = q.border_color;
    out.border_width   = q.border_width;
    out.corner_radii   = q.corner_radii;
    out.shadow_color   = q.shadow_color;
    out.shadow_params  = vec4<f32>(q.shadow_offset.x, q.shadow_offset.y, q.shadow_blur, q.shadow_spread);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let abs_pos      = abs(in.local_pos);
    let shad_offset  = in.shadow_params.xy;
    let shad_blur    = in.shadow_params.z;
    let shad_spread  = in.shadow_params.w;

    // Pick corner radius for the quadrant this fragment sits in.
    var corner_radius: f32;
    if in.local_pos.x < 0.0 && in.local_pos.y < 0.0 {
        corner_radius = in.corner_radii.x;  // TL
    } else if in.local_pos.x >= 0.0 && in.local_pos.y < 0.0 {
        corner_radius = in.corner_radii.y;  // TR
    } else if in.local_pos.x >= 0.0 && in.local_pos.y >= 0.0 {
        corner_radius = in.corner_radii.z;  // BR
    } else {
        corner_radius = in.corner_radii.w;  // BL
    }

    // Fast path: solid fill, no rounded corners, no border, no shadow.
    if corner_radius == 0.0 && in.border_width <= 0.0 && in.shadow_color.a == 0.0 {
        return in.fill;
    }

    // ── Main box SDF ─────────────────────────────────────────────────────────
    let d = abs_pos - in.half_size + vec2<f32>(corner_radius);
    let outer_dist  = length(max(vec2<f32>(0.0), d)) + min(0.0, max(d.x, d.y)) - corner_radius;
    let outer_alpha = saturate(0.5 - outer_dist);

    // ── Shadow ────────────────────────────────────────────────────────────────
    var shadow_a = 0.0;
    if in.shadow_color.a > 0.0 {
        // Shadow box: same radii, center shifted by shadow_offset, expanded by spread.
        let s_pos  = abs(in.local_pos - shad_offset);
        let s_half = in.half_size + vec2<f32>(shad_spread);
        let s_d    = s_pos - s_half + vec2<f32>(corner_radius);
        let s_dist = length(max(vec2<f32>(0.0), s_d)) + min(0.0, max(s_d.x, s_d.y)) - corner_radius;
        shadow_a   = in.shadow_color.a * clamp(0.5 - s_dist / max(shad_blur, 0.001), 0.0, 1.0);
        // Shadow is hidden under the opaque parts of the main box.
        shadow_a  *= (1.0 - outer_alpha);
    }

    // Discard if nothing is visible.
    if outer_alpha <= 0.0 && shadow_a <= 0.0 {
        discard;
    }

    // ── Main box color (fill + optional border) ───────────────────────────────
    var main_rgb = vec3<f32>(0.0);
    var main_a   = 0.0;
    if outer_alpha > 0.0 {
        if in.border_width <= 0.0 {
            main_rgb = in.fill.rgb;
            main_a   = in.fill.a * outer_alpha;
        } else {
            let inner_corner = max(0.0, corner_radius - in.border_width);
            let inner_half   = max(vec2<f32>(0.0), in.half_size - vec2<f32>(in.border_width));
            let di           = abs_pos - inner_half + vec2<f32>(inner_corner);
            let inner_dist   = length(max(vec2<f32>(0.0), di)) + min(0.0, max(di.x, di.y)) - inner_corner;
            let inner_alpha  = saturate(0.5 - inner_dist);
            let color        = mix(in.border_color, in.fill, inner_alpha);
            main_rgb         = color.rgb;
            main_a           = color.a * outer_alpha;
        }
    }

    // ── Composite: shadow behind, box in front ────────────────────────────────
    let out_a = main_a + shadow_a * (1.0 - main_a);
    if out_a <= 0.0 { discard; }
    let out_rgb = (main_rgb * main_a + in.shadow_color.rgb * shadow_a * (1.0 - main_a)) / out_a;
    return vec4<f32>(out_rgb, out_a);
}
```

**Update all `QuadCall` construction sites** — these are in `akar-components`. Each must add the new shadow fields zeroed:

```rust
QuadCall {
    rect,
    fill: color_to_f32(fill),
    border_color: color_to_f32(border_color),
    corner_radii,
    border_width,
    z: 0.0,
    shadow_blur: 0.0,
    shadow_spread: 0.0,
    shadow_color: [0.0; 4],
    shadow_offset: [0.0; 2],
    _pad: [0.0; 2],
}
```

Files to update: `button.rs`, `container.rs`, `separator.rs`, `canvas.rs` (`CanvasPainter::push_quad`).

**Acceptance criteria:** `cargo test --workspace` passes. Visually: the demo looks identical to before (no shadow on existing quads).

**Review:** Done. `QuadCall` expanded to 112 bytes with `shadow_blur`, `shadow_spread`, `shadow_color`, `shadow_offset`. Compile-time `size_of` assert added. `quad.wgsl` fully replaced with SDF shadow compositing in fragment shader; vertex shader expands quad for shadow region. `push_quad` scales shadow fields by `scale_factor`. All 4 construction sites (`button.rs`, `container.rs`, `separator.rs`, `canvas.rs`) zero the new fields. Clippy clean, 50 tests pass.

---

### Task 2: `BoxStyle` and `BoxShadow` Types

**Goal:** Two new types in `akar-components` plus three themed presets.

**File:** `crates/akar-components/src/box_style.rs`

```rust
use crate::AkarTheme;

#[derive(Clone, Copy, Debug)]
pub struct BoxShadow {
    /// RGBA hex shadow color (e.g. `0x00000066` for 40% black).
    pub color: u32,
    /// Horizontal and vertical shadow offset in logical pixels.
    pub offset: [f32; 2],
    /// Soft-edge radius in logical pixels. 0.0 = hard edge.
    pub blur: f32,
    /// Expands the shadow rect beyond the box bounds in all directions.
    pub spread: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BoxStyle {
    /// Background fill color (RGBA hex). `0x00000000` = transparent, skips quad.
    pub fill: u32,
    pub border_color: u32,
    pub border_width: f32,
    /// Per-corner radii `[TL, TR, BR, BL]`. Use `[r; 4]` for uniform radius.
    pub corner_radii: [f32; 4],
    pub shadow: Option<BoxShadow>,
}

impl BoxStyle {
    /// Flat background with no border, no radius, no shadow.
    pub fn flat(fill: u32) -> Self {
        Self {
            fill,
            border_color: 0,
            border_width: 0.0,
            corner_radii: [0.0; 4],
            shadow: None,
        }
    }

    /// Surface background — fills with `base_100`, rounded box radius, no border, no shadow.
    pub fn surface(theme: &AkarTheme) -> Self {
        Self {
            fill: theme.base_100,
            border_color: 0,
            border_width: 0.0,
            corner_radii: [theme.radius_box; 4],
            shadow: None,
        }
    }

    /// Panel — `base_200` background with a subtle border. Typical sidebar / header fill.
    pub fn panel(theme: &AkarTheme) -> Self {
        Self {
            fill: theme.base_200,
            border_color: theme.base_300,
            border_width: theme.border_width,
            corner_radii: [theme.radius_box; 4],
            shadow: None,
        }
    }

    /// Card — `base_100` background, border, and a soft drop shadow. Typical card component.
    pub fn card(theme: &AkarTheme) -> Self {
        Self {
            fill: theme.base_100,
            border_color: theme.base_300,
            border_width: theme.border_width,
            corner_radii: [theme.radius_box; 4],
            shadow: Some(BoxShadow {
                color: 0x00000040,
                offset: [0.0, 4.0],
                blur: 12.0,
                spread: 0.0,
            }),
        }
    }
}
```

**`crates/akar-components/src/lib.rs`** — add:

```rust
pub mod box_style;
pub use box_style::{BoxShadow, BoxStyle};
```

**Acceptance criteria:** `cargo check -p akar-components` passes.

**Review:** Done. `box_style.rs` created with `BoxShadow`, `BoxStyle`, and presets `flat`, `surface`, `panel`, `card`. Re-exported from `lib.rs`. Clippy clean, 50 tests pass.

---

### Task 3: Upgrade `container` to Use `BoxStyle`

**Goal:** Replace the `(background: u32, theme: &AkarTheme)` signature with `(style: &BoxStyle)`. Render shadow when present.

**File:** `crates/akar-components/src/container.rs` — full replacement:

```rust
use akar_core::{AkarCore, QuadCall};
use akar_layout::{Layout, NodeId};

use crate::box_style::BoxStyle;
use crate::color::color_to_f32;

pub fn container(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    style: &BoxStyle,
) {
    let rect = layout.rect(node_id);
    if rect[2] == 0.0 || rect[3] == 0.0 || style.fill == 0 {
        return;
    }

    let (shadow_color, shadow_offset, shadow_blur, shadow_spread) =
        match &style.shadow {
            Some(s) => (
                color_to_f32(s.color),
                s.offset,
                s.blur,
                s.spread,
            ),
            None => ([0.0; 4], [0.0; 2], 0.0, 0.0),
        };

    core.draw_list.push_quad(QuadCall {
        rect,
        fill: color_to_f32(style.fill),
        border_color: color_to_f32(style.border_color),
        corner_radii: style.corner_radii,
        border_width: style.border_width,
        z: 0.0,
        shadow_blur,
        shadow_spread,
        shadow_color,
        shadow_offset,
        _pad: [0.0; 2],
    });
}
```

**Tests** — update the existing tests in `container.rs` to use `BoxStyle`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::box_style::BoxStyle;
    use akar_layout::Style;

    fn sized_node(layout: &mut akar_layout::Layout) -> NodeId {
        let node = layout.new_leaf(Style {
            size: akar_layout::Size {
                width: akar_layout::length(100.0),
                height: akar_layout::length(100.0),
            },
            ..Default::default()
        });
        layout.compute(node, (Some(200.0), Some(200.0)), |_, _, _, _, _| {
            akar_layout::Size::ZERO
        });
        node
    }

    #[test]
    fn transparent_fill_pushes_no_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();
        container(&mut core, &layout, node, &BoxStyle::flat(0x00000000));
        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn solid_fill_pushes_one_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();
        container(&mut core, &layout, node, &BoxStyle::flat(0xFF0000FF));
        assert_eq!(core.draw_list.sorted_quads().len(), 1);
    }

    #[test]
    fn zero_area_pushes_no_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = layout.new_leaf(Style::default());
        let mut core = AkarCore::mock();
        container(&mut core, &layout, node, &BoxStyle::flat(0xFF0000FF));
        assert!(core.draw_list.sorted_quads().is_empty());
    }

    #[test]
    fn shadow_fields_propagate_to_quad() {
        let mut layout = akar_layout::Layout::new();
        let node = sized_node(&mut layout);
        let mut core = AkarCore::mock();
        let style = BoxStyle {
            fill: 0xFFFFFFFF,
            border_color: 0,
            border_width: 0.0,
            corner_radii: [0.0; 4],
            shadow: Some(crate::box_style::BoxShadow {
                color: 0x00000080,
                offset: [2.0, 4.0],
                blur: 8.0,
                spread: 0.0,
            }),
        };
        container(&mut core, &layout, node, &style);
        let quads = core.draw_list.sorted_quads();
        assert_eq!(quads.len(), 1);
        assert!(quads[0].shadow_blur > 0.0);
        assert!(quads[0].shadow_color[3] > 0.0);
    }
}
```

**Acceptance criteria:** `cargo test -p akar-components` passes. The demo compiles (call sites updated in Task 5).

**Review:** Done. `container` signature changed to `(style: &BoxStyle)`, shadow fields propagate to `QuadCall`. New test `shadow_fields_propagate_to_quad` added (4 container tests total). All call sites in `demo-rust` (6 sites) and `canvas-basic-rust` (1 site) migrated to `BoxStyle::panel`/`surface`/`flat`. Clippy clean, 51 tests pass.

---

### Task 4: `Layout::set_padding` and `Layout::set_margin`

**Goal:** Two new methods on `Layout` in `akar-layout` that let callers update padding and margin after node creation.

**File:** `crates/akar-layout/src/lib.rs` — add to the `impl Layout` block:

```rust
/// Sets the padding for `node`. Values are in logical pixels.
pub fn set_padding(&mut self, node: NodeId, top: f32, right: f32, bottom: f32, left: f32) {
    let mut style = self.tree.style(node).unwrap().clone();
    style.padding = Rect {
        top:    length(top),
        right:  length(right),
        bottom: length(bottom),
        left:   length(left),
    };
    self.tree.set_style(node, style).unwrap();
}

/// Sets the margin for `node`. Values are in logical pixels.
pub fn set_margin(&mut self, node: NodeId, top: f32, right: f32, bottom: f32, left: f32) {
    let mut style = self.tree.style(node).unwrap().clone();
    style.margin = Rect {
        top:    length(top),
        right:  length(right),
        bottom: length(bottom),
        left:   length(left),
    };
    self.tree.set_style(node, style).unwrap();
}
```

Note: `Rect` here is `taffy::geometry::Rect`, not `akar_layout::Rect` (the world-space type from Epic 004). The taffy `Rect` is already in scope via `pub use taffy::prelude::*`.

**Tests** — add to the existing `#[cfg(test)]` block in `lib.rs`:

```rust
#[test]
fn set_padding_affects_child_position() {
    let mut layout = Layout::new();
    let child = layout.new_leaf(Style {
        size: Size { width: length(50.0), height: length(50.0) },
        ..Default::default()
    });
    let root = layout.new_with_children(
        Style {
            display: Display::Flex,
            size: Size { width: length(200.0), height: length(200.0) },
            ..Default::default()
        },
        &[child],
    );
    layout.set_padding(root, 20.0, 20.0, 20.0, 20.0);
    layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

    let r = layout.rect(child);
    // Child should be offset by the 20px padding on the root.
    assert!((r[0] - 20.0).abs() < 1.0, "child.x = {}", r[0]);
    assert!((r[1] - 20.0).abs() < 1.0, "child.y = {}", r[1]);
}

#[test]
fn set_margin_pushes_node() {
    let mut layout = Layout::new();
    let child = layout.new_leaf(Style {
        size: Size { width: length(50.0), height: length(50.0) },
        ..Default::default()
    });
    let root = layout.new_with_children(
        Style {
            display: Display::Flex,
            ..Default::default()
        },
        &[child],
    );
    layout.set_margin(child, 10.0, 0.0, 0.0, 15.0);
    layout.compute(root, (Some(200.0), Some(200.0)), |_, _, _, _, _| Size::ZERO);

    let r = layout.rect(child);
    assert!((r[0] - 15.0).abs() < 1.0, "child.x = {}", r[0]);
    assert!((r[1] - 10.0).abs() < 1.0, "child.y = {}", r[1]);
}
```

**Acceptance criteria:** `cargo test -p akar-layout` passes including the two new tests.

---

### Task 5: C ABI — `AkarBoxStyle`, `akar_container`, `akar_set_padding`, `akar_set_margin`

**Goal:** Expose `BoxStyle` and the two layout helpers in the C ABI. Regenerate `akar.h`.

**Add to `crates/akar-c-api/src/lib.rs`:**

```rust
/// Visual style for a container: fill, border, corner radii, and optional shadow.
/// Set `shadow_color` to `0x00000000` (transparent) to disable shadow.
#[repr(C)]
pub struct AkarBoxStyle {
    pub fill: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow_color: u32,
    pub shadow_offset: [f32; 2],
    pub shadow_blur: f32,
    pub shadow_spread: f32,
}

#[no_mangle]
pub unsafe extern "C" fn akar_container(
    ctx: *mut AkarCtx,
    node_id: u64,
    style: AkarBoxStyle,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();

    let shadow = if (style.shadow_color & 0xFF) > 0 {
        Some(akar_components::BoxShadow {
            color: style.shadow_color,
            offset: style.shadow_offset,
            blur: style.shadow_blur,
            spread: style.shadow_spread,
        })
    } else {
        None
    };

    let box_style = akar_components::BoxStyle {
        fill: style.fill,
        border_color: style.border_color,
        border_width: style.border_width,
        corner_radii: style.corner_radii,
        shadow,
    };

    akar_components::container(&mut ctx.core, &ctx.layout, nid, &box_style);
}

#[no_mangle]
pub unsafe extern "C" fn akar_set_padding(
    ctx: *mut AkarCtx,
    node_id: u64,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    ctx.layout.set_padding(nid, top, right, bottom, left);
}

#[no_mangle]
pub unsafe extern "C" fn akar_set_margin(
    ctx: *mut AkarCtx,
    node_id: u64,
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
) {
    let ctx = unsafe { &mut *ctx };
    let nid: akar_layout::NodeId = node_id.into();
    ctx.layout.set_margin(nid, top, right, bottom, left);
}
```

**Update `cbindgen.toml`** — add `AkarBoxStyle` to the export list:

```toml
[export]
include = ["AkarCtx", "AkarButtonResult", "AkarRect", "AkarBoxStyle"]
```

**Run `cargo build -p akar-c-api`** to regenerate `akar.h`. Verify `akar.h` contains `AkarBoxStyle`, `akar_container`, `akar_set_padding`, `akar_set_margin`.

**Acceptance criteria:** `cargo check -p akar-c-api` and `cargo clippy -p akar-c-api -- -D warnings` pass clean. Updated `akar.h` committed.

---

### Task 6: Update `examples/demo-rust` and Demonstrate Shadow

**Goal:** Migrate existing `container` call sites to `BoxStyle`. Add a card with a visible drop shadow to prove the shadow pipeline end-to-end.

**Existing call sites** — in `examples/demo-rust/src/main.rs`:

```rust
// Before:
container(&mut core, &layout, page.header,       AKAR_THEME_DARK.base_200, &AKAR_THEME_DARK);
container(&mut core, &layout, page.sidebar_left,  AKAR_THEME_DARK.base_200, &AKAR_THEME_DARK);

// After:
container(&mut core, &layout, page.header,       &BoxStyle::panel(&AKAR_THEME_DARK));
container(&mut core, &layout, page.sidebar_left,  &BoxStyle::panel(&AKAR_THEME_DARK));
```

**Add a card to the control strip** (from Epic 006's strip layout). Wrap the two buttons and the label inside a card-styled container:

```rust
// The strip node already exists from Epic 006.
// Just change its style:
container(&mut core, &layout, strip, &BoxStyle::card(&AKAR_THEME_DARK));
```

This makes the control strip visually distinct from the background with a border and shadow, demonstrating the full `BoxStyle` capability.

**Imports to add to `main.rs`:**

```rust
use akar_components::{BoxStyle, /* existing imports... */};
```

**Acceptance criteria:** `cargo run --manifest-path examples/demo-rust/Cargo.toml` opens a window where:
- Header and sidebar render with `BoxStyle::panel` appearance (background fill, subtle border).
- The control strip at the bottom of the main area renders as a card with a visible drop shadow above the canvas background.
- Buttons retain their existing appearance and behavior.
- No panics on resize or interaction.

---

## Acceptance Criteria for Epic 007

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero errors.
- [ ] `cargo test --workspace` passes. New tests: 1 in `container` (shadow fields propagate), 2 in `akar-layout` (padding and margin affect child layout).
- [ ] `QuadCall` is 112 bytes; `std::mem::size_of::<QuadCall>() == 112` (add a compile-time assert in `draw_list.rs`).
- [ ] WGSL struct byte layout matches `QuadCall` exactly — verified by correct visual output.
- [ ] Quads with `shadow_color.a == 0` render identically to before (fast path or zero shadow alpha).
- [ ] `BoxStyle::card` renders a visible soft drop shadow in `demo-rust`.
- [ ] `BoxStyle::panel` renders a border without shadow in `demo-rust`.
- [ ] `BoxStyle::flat(0)` / transparent fill → no quad pushed.
- [ ] `Layout::set_padding` and `Layout::set_margin` are exported from `akar-layout`.
- [ ] `AkarBoxStyle`, `akar_container`, `akar_set_padding`, `akar_set_margin` are in `akar.h`.
- [ ] No inset shadow, no multiple shadows — deferred per ADR-021.
- [ ] `CanvasPainter::push_quad` zeros all shadow fields — no world-space shadow in Epic 007.
- [ ] No changes to `akar-winit`.
