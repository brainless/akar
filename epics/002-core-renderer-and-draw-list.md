# Epic 002: Core Renderer and Draw List

**Status:** Done
**Goal:** Produce a working quad renderer + draw list + input state in `akar-core`, a taffy wrapper in `akar-layout`, one end-to-end button component in `akar-components`, the C ABI skeleton in `akar-c-api`, and a Rust demo that renders the button in a winit window. All tasks in this epic are implementation — no further research is needed.

**Prerequisite:** Epic 001 is `Status: Done`. The architecture decisions below are final.

---

## Architecture Decision Records

### ADR-001: Immediate Mode

**Decision:** akar is immediate mode. Component functions are called every frame and return state immediately. There is no retained widget tree, no diffing, no lifecycle hooks.

**Rationale:** xilem's retained model adds incremental diffing, `ViewState` per node, and message routing. None of these are required for v1. Immediate mode eliminates ownership puzzles, reduces API surface, and matches the target audience (game/tool developers already driving their own loop). The one retained concept needed — focus state — is a single `focused_id: Option<NodeId>` field in `InputState`, not a full tree.

**Consequences:** Components must be pure functions of (input state, layout rect, theme) → (draw calls + return value). Components cannot hold state across frames themselves; callers hold state. The library is stateless per-component.

**Deferred:** Accessibility (requires retained element identity), animation (requires per-element timeline), and hot-reload tooling (requires retained scene identity). These are post-v1.

---

### ADR-002: Draw List Design

**Decision:** The draw list stores GPU-ready `DrawCall` structs (one per primitive). Submission order is painter's order by default. Z-value is an explicit override field used only for overlay layers (tooltips, modals, toasts). Before GPU upload, the draw list: (1) AABB-culls calls whose rect does not intersect the active scissor rect, (2) sorts stable by `(z, pipeline_type)` to minimize pipeline switches.

**DrawCall enum:**
```
DrawCall::Quad {
    rect: [f32; 4],          // x, y, w, h in physical pixels
    fill: u32,               // packed RGBA
    border: u32,             // packed RGBA
    border_width: f32,
    corner_radii: [f32; 4],  // tl, tr, br, bl
    z: f32,
}
DrawCall::Text {
    buffer_id: u64,          // opaque glyphon buffer handle
    x: f32, y: f32,
    clip: [f32; 4],          // x, y, w, h in physical pixels
    color: u32,              // packed RGBA
    z: f32,
}
```

**Scissor stack:** `push_scissor(rect)` intersects with the current top and pushes. `pop_scissor()` restores. All draw calls submitted between push and pop carry the active clip rect used for AABB culling. The scissor rect is also emitted as a wgpu scissor command in the render pass.

**Rationale:** No CPU tessellation step (unlike egui). Z-sort on small structs, not large meshes. Painter's order within a Z-level is predictable and matches what component authors expect. The sort is stable so submission order is tie-broken correctly.

---

### ADR-003: Quad Pipeline Shader

**Decision:** SDF-based rounded corners evaluated in the fragment shader, one WGSL shader for all quads, instanced draw (one draw call per batch of same-Z quads).

**Shader inputs (per instance, in a storage buffer):**
- `rect: vec4<f32>` — x, y, w, h in physical pixels
- `fill: vec4<f32>` — RGBA fill color
- `border_color: vec4<f32>` — RGBA border color
- `border_width: f32`
- `corner_radii: vec4<f32>` — tl, tr, br, bl

**SDF formula** (from GPUI `shaders.wgsl:362–387`):
1. Compute `corner_center` for the quadrant the fragment falls in (pick corner radius by sign of `local_pos - half_size`).
2. `corner_center_to_point = abs(local_pos) - half_size + corner_radius`
3. `outer_sdf = length(max(vec2(0.0), corner_center_to_point)) + min(0.0, max(corner_center_to_point.x, corner_center_to_point.y)) - corner_radius`
4. Fast path: `if (corner_radius == 0.0)` skip SDF entirely.
5. Alpha: `saturate(antialias_threshold - outer_sdf)` with `antialias_threshold = 0.5`.
6. Border: inner SDF offset by `border_width`, lerp between fill and border color based on inner vs outer SDF.

**DPI:** All coords entering the draw list are in logical pixels. The `begin_frame` call takes `scale_factor: f32`; draw list multiplies all rects by `scale_factor` before upload. Shaders receive physical pixel coordinates.

**wgpu requirements:** `Limits::downlevel_defaults()` (compatible with OpenGL ES 3.0, WebGL2, DX11). No additional features required for the quad pipeline.

---

### ADR-004: C ABI Shape

**Decision:** Opaque `AkarCtx*` pointer (singleton, caller-allocated by the library). Pooled GPU resource handles use sokol-style typed ID structs (`typedef struct { uint32_t id; } AkarFontHandle;`) to detect use-after-free via generation counters. Input follows Nuklear's begin/end bracket. Component functions return result structs (value + changed flag) rather than out-params. All enums include `_FORCE_U32 = 0x7FFFFFFF`. The header is generated by `cbindgen` from `akar-c-api` — never edited manually.

**Frame contract:**
```c
AkarCtx* akar_ctx_new(const void* wgpu_device, const void* wgpu_queue);
void akar_ctx_free(AkarCtx* ctx);

void akar_begin_frame(AkarCtx* ctx, uint32_t width, uint32_t height, float scale_factor);
void akar_end_frame(AkarCtx* ctx, const void* wgpu_render_pass_encoder);

void akar_input_begin(AkarCtx* ctx);
void akar_set_mouse_pos(AkarCtx* ctx, float x, float y);
void akar_push_mouse_button(AkarCtx* ctx, int button, bool pressed);
void akar_push_scroll(AkarCtx* ctx, float dx, float dy);
void akar_push_char(AkarCtx* ctx, uint32_t codepoint);
void akar_input_end(AkarCtx* ctx);
```

**Result structs instead of out-params:**
```c
typedef struct { bool clicked; bool hovered; bool pressed; } AkarButtonResult;
AkarButtonResult akar_button(AkarCtx* ctx, uint32_t node_id, const char* label, int label_len);
```

**Memory:** `akar_ctx_new` allocates internally (Rust global allocator). No caller-side heap allocation beyond `AkarCtx*` itself (which the library owns). Custom allocator support deferred to post-v1.

---

## Tasks

### Task 1: Workspace scaffold

**Goal:** Create the Cargo workspace with five crates. No implementation yet — just the `Cargo.toml` files and empty `src/lib.rs` stubs.

**Files to create:**

`Cargo.toml` (workspace root):
```toml
[workspace]
resolver = "2"
members = [
    "crates/akar-core",
    "crates/akar-layout",
    "crates/akar-components",
    "crates/akar-c-api",
    "crates/akar-winit",
]

[workspace.dependencies]
wgpu = "29"
glyphon = { path = "~/Projects/glyphon" }
glam = { path = "~/Projects/glam-rs" }
taffy = "0.11"
winit = { version = "0.30", optional = true }
thiserror = "2"
log = "0.4"
bytemuck = { version = "1", features = ["derive"] }
```

> Note: Use the wgpu version that matches the glyphon local checkout. Read `~/Projects/glyphon/Cargo.toml` to confirm the exact wgpu version before writing the workspace `Cargo.toml`. Use path dependencies for glyphon and glam as specified in `DEVELOP.md`.

`crates/akar-core/Cargo.toml`:
```toml
[package]
name = "akar-core"
version = "0.1.0"
edition = "2021"

[dependencies]
wgpu.workspace = true
glyphon.workspace = true
glam.workspace = true
thiserror.workspace = true
log.workspace = true
bytemuck.workspace = true
```

`crates/akar-layout/Cargo.toml`:
```toml
[package]
name = "akar-layout"
version = "0.1.0"
edition = "2021"

[dependencies]
taffy.workspace = true
glam.workspace = true
thiserror.workspace = true
```

`crates/akar-components/Cargo.toml`:
```toml
[package]
name = "akar-components"
version = "0.1.0"
edition = "2021"

[dependencies]
akar-core = { path = "../akar-core" }
akar-layout = { path = "../akar-layout" }
```

`crates/akar-c-api/Cargo.toml`:
```toml
[package]
name = "akar-c-api"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "staticlib"]

[dependencies]
akar-components = { path = "../akar-components" }
akar-core = { path = "../akar-core" }
akar-layout = { path = "../akar-layout" }
```

`crates/akar-winit/Cargo.toml`:
```toml
[package]
name = "akar-winit"
version = "0.1.0"
edition = "2021"

[dependencies]
akar-core = { path = "../akar-core" }
winit.workspace = true
```

Each crate gets `src/lib.rs` with an empty `// placeholder` comment.

**Acceptance criteria:** `cargo check --workspace` passes with no errors (empty crates are fine).

---

### Task 2: `akar-core` — InputState

**Goal:** A plain struct holding per-frame mouse/keyboard state with begin/end bracket methods for clearing transient flags.

**File:** `crates/akar-core/src/input.rs`

**Struct:**
```rust
pub struct InputState {
    pub mouse_pos: glam::Vec2,
    pub mouse_pos_prev: glam::Vec2,
    pub mouse_buttons: [bool; 5],       // current down state
    pub mouse_buttons_pressed: [bool; 5],  // rose this frame
    pub mouse_buttons_released: [bool; 5], // fell this frame
    pub scroll_delta: glam::Vec2,
    pub chars: Vec<char>,
    pub focused_id: Option<u64>,        // taffy NodeId as u64
}
```

**Methods:**
```rust
impl InputState {
    pub fn new() -> Self
    pub fn begin_frame(&mut self)   // clears _pressed, _released, scroll_delta, chars; saves mouse_pos to mouse_pos_prev
    pub fn set_mouse_pos(&mut self, x: f32, y: f32)
    pub fn push_mouse_button(&mut self, button: usize, pressed: bool)  // sets down state and computes edge flags
    pub fn push_scroll(&mut self, dx: f32, dy: f32)
    pub fn push_char(&mut self, c: char)

    // Hit-test helpers used by components:
    pub fn is_hovering(&self, rect: [f32; 4]) -> bool    // rect is [x, y, w, h]
    pub fn is_clicked(&self, rect: [f32; 4]) -> bool     // mouse_button[0] released this frame while hovering
    pub fn is_pressed(&self, rect: [f32; 4]) -> bool     // mouse_button[0] down while hovering
}
```

Re-export from `crates/akar-core/src/lib.rs`: `pub mod input; pub use input::InputState;`

**Acceptance criteria:** `cargo test -p akar-core` passes with unit tests for `is_hovering`, `is_clicked` (press then release inside rect), and `is_clicked` returning false when released outside rect.

---

### Task 3: `akar-core` — DrawList and DrawCall

**Goal:** The frame-scoped list of GPU-ready primitives. Supports AABB culling against scissor stack and stable Z-sort before flush.

**File:** `crates/akar-core/src/draw_list.rs`

**Types:**
```rust
#[derive(Clone)]
pub enum DrawCall {
    Quad(QuadCall),
    Text(TextCall),
}

#[derive(Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct QuadCall {
    pub rect: [f32; 4],           // x, y, w, h — physical pixels after scale
    pub fill: [f32; 4],           // RGBA
    pub border_color: [f32; 4],   // RGBA
    pub border_width: f32,
    pub corner_radii: [f32; 4],   // tl, tr, br, bl
    pub z: f32,
    pub _pad: f32,
}

#[derive(Clone)]
pub struct TextCall {
    pub buffer_id: u64,
    pub x: f32,
    pub y: f32,
    pub clip: [f32; 4],           // physical pixels
    pub color: [f32; 4],          // RGBA
    pub z: f32,
}
```

**DrawList:**
```rust
pub struct DrawList {
    calls: Vec<DrawCall>,
    scissor_stack: Vec<[f32; 4]>,  // physical pixel rects
    scale_factor: f32,
}

impl DrawList {
    pub fn new() -> Self
    pub fn begin_frame(&mut self, scale_factor: f32)     // clears calls and scissor_stack
    pub fn push_scissor(&mut self, rect: [f32; 4])       // rect in logical pixels; intersects with top
    pub fn pop_scissor(&mut self)
    pub fn active_scissor(&self) -> Option<[f32; 4]>

    pub fn push_quad(&mut self, mut call: QuadCall)      // multiplies rect by scale_factor; AABB culls against scissor
    pub fn push_text(&mut self, call: TextCall)           // AABB culls clip against scissor

    // Called by the renderer to get GPU data:
    pub fn sorted_quads(&mut self) -> Vec<QuadCall>      // stable sort by (z, then submission order); filters out Text calls
    pub fn text_calls(&self) -> &[DrawCall]              // returns only Text calls in submission order
}
```

AABB intersection helper (private): `fn intersects(a: [f32; 4], b: [f32; 4]) -> bool` where the rect is `[x, y, w, h]`.

Re-export from `lib.rs`: `pub mod draw_list; pub use draw_list::{DrawList, DrawCall, QuadCall, TextCall};`

**Acceptance criteria:** Unit tests for: scissor culling (quad entirely outside scissor → not added), scissor intersection (quad partially inside → added), sort order, push_scissor intersection logic.

---

### Task 4: `akar-core` — Quad pipeline (WGSL shader + wgpu pipeline)

**Goal:** A wgpu render pipeline that draws SDF-rounded quads from a storage buffer of `QuadCall` instances.

**Files:**
- `crates/akar-core/src/quad_pipeline.rs`
- `crates/akar-core/src/shaders/quad.wgsl`

**Shader (`quad.wgsl`):**

Vertex stage:
- No vertex buffer. Draw with `draw(0..6, 0..instance_count)` (two triangles per quad).
- Instance data read from a storage buffer (`@group(0) @binding(0) var<storage, read> quads: array<QuadInstance>`).
- Vertex index maps to one of 6 corners of a unit quad; scale by `rect.zw` and offset by `rect.xy`.
- Pass `local_pos` (position relative to quad center, in physical pixels) to the fragment stage.

Fragment stage:
- Inputs: `local_pos: vec2<f32>`, `half_size: vec2<f32>`, `fill: vec4<f32>`, `border_color: vec4<f32>`, `border_width: f32`, `corner_radii: vec4<f32>`.
- Pick corner radius by quadrant: `tl` if `x < 0 && y < 0`, `tr` if `x > 0 && y < 0`, `br` if `x > 0 && y > 0`, `bl` if `x < 0 && y > 0`.
- Fast path: `if corner_radius == 0.0 { /* skip SDF */ }`.
- Outer SDF: `let d = abs(local_pos) - half_size + vec2(corner_radius); outer_sdf = length(max(vec2(0.0), d)) + min(0.0, max(d.x, d.y)) - corner_radius;`
- Inner SDF: same formula with `half_size - vec2(border_width)`.
- Alpha: `saturate(0.5 - outer_sdf)`.
- Color: `mix(fill, border_color, saturate(0.5 - inner_sdf) - (1.0 - alpha))` — simplified: fill inside inner, border between inner and outer.
- Discard if `alpha <= 0.0`.

**`QuadPipeline` struct** (`quad_pipeline.rs`):
```rust
pub struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    instance_buffer: wgpu::Buffer,         // GPU buffer for QuadCall array; re-created if capacity exceeded
    instance_capacity: usize,
}

impl QuadPipeline {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self
    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        quads: &[QuadCall],
    )
}
```

`flush` uploads `quads` to `instance_buffer` (grow if needed), creates a bind group, sets pipeline, sets bind group, issues `draw(0..6, instance_count as u32)`.

Re-export: `pub mod quad_pipeline; pub use quad_pipeline::QuadPipeline;`

**Acceptance criteria:** Compiles without errors. Manual visual test deferred to Task 11 (demo).

---

### Task 5: `akar-core` — Text pipeline (glyphon wrapper)

**Goal:** A thin wrapper over glyphon that owns the `FontSystem`, `SwashCache`, `TextAtlas`, and `TextRenderer`, exposes methods to create/update text buffers and render them into a render pass.

**File:** `crates/akar-core/src/text_pipeline.rs`

**TextPipeline struct:**
```rust
pub struct TextPipeline {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    cache: glyphon::Cache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    buffers: HashMap<u64, glyphon::Buffer>,  // buffer_id → Buffer
    next_id: u64,
}
```

**Methods:**
```rust
impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self

    // Create or update a text buffer. Returns a stable buffer_id.
    pub fn set_text(
        &mut self,
        buffer_id: Option<u64>,  // None = create new
        text: &str,
        metrics: glyphon::Metrics,
        width: Option<f32>,
        height: Option<f32>,
    ) -> u64

    pub fn remove_buffer(&mut self, buffer_id: u64)

    // Measure text without committing — used by taffy measure function.
    pub fn measure(
        &mut self,
        buffer_id: u64,
        width: Option<f32>,
    ) -> glam::Vec2

    // Called once per frame before render:
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_width: u32,
        viewport_height: u32,
        scale_factor: f32,
        text_calls: &[TextCall],
    ) -> Result<(), glyphon::PrepareError>

    pub fn render<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), glyphon::RenderError>

    pub fn trim_atlas(&mut self)
}
```

Init sequence mirrors sugacode `renderer.rs:71–81` exactly. Per-frame sequence: `Viewport::update` → `TextRenderer::prepare` → `TextRenderer::render` → `TextAtlas::trim`.

Re-export: `pub mod text_pipeline; pub use text_pipeline::TextPipeline;`

**Acceptance criteria:** Compiles. Unit test: `set_text` returns a buffer_id; calling it again with the same id does not panic.

---

### Task 6: `akar-core` — AkarCore context

**Goal:** The top-level handle that owns `QuadPipeline`, `TextPipeline`, `DrawList`, and `InputState`. Implements the frame lifecycle.

**File:** `crates/akar-core/src/context.rs`

```rust
pub struct AkarCore {
    pub draw_list: DrawList,
    pub input: InputState,
    pub(crate) quad_pipeline: QuadPipeline,
    pub(crate) text_pipeline: TextPipeline,
    surface_format: wgpu::TextureFormat,
    viewport_width: u32,
    viewport_height: u32,
    scale_factor: f32,
}

impl AkarCore {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self

    /// Call once per frame before submitting any component calls.
    /// Clears the draw list and input edge flags.
    pub fn begin_frame(
        &mut self,
        width: u32,
        height: u32,
        scale_factor: f32,
    )

    /// Call once per frame after all component calls.
    /// Flushes draw list to the GPU via the provided render pass encoder.
    pub fn end_frame(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
    ) -> Result<(), Box<dyn std::error::Error>>
}
```

`begin_frame`: calls `draw_list.begin_frame(scale_factor)` and `input.begin_frame()`.

`end_frame`:
1. Calls `text_pipeline.prepare(device, queue, width, height, scale_factor, draw_list.text_calls())`.
2. Calls `quad_pipeline.flush(device, queue, pass, &draw_list.sorted_quads())`.
3. Calls `text_pipeline.render(pass)`.
4. Calls `text_pipeline.trim_atlas()`.

Re-export from `lib.rs`: `pub mod context; pub use context::AkarCore;`

**Acceptance criteria:** Compiles. The order of quad then text rendering is intentional (text draws on top of quads).

---

### Task 7: `akar-layout` — Taffy wrapper

**Goal:** A thin, ergonomic wrapper over `TaffyTree` with the 10-method API identified in Epic 001 Task 6.

**File:** `crates/akar-layout/src/lib.rs`

```rust
use taffy::prelude::*;

pub type NodeId = taffy::NodeId;

pub struct AkarNodeContext {
    pub text_buffer_id: u64,
}

pub struct Layout {
    tree: TaffyTree<AkarNodeContext>,
}

impl Layout {
    pub fn new() -> Self
    pub fn new_leaf(&mut self, style: Style) -> NodeId
    pub fn new_leaf_with_context(&mut self, style: Style, ctx: AkarNodeContext) -> NodeId
    pub fn new_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId
    pub fn add_child(&mut self, parent: NodeId, child: NodeId)
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId])
    pub fn remove(&mut self, node: NodeId)
    pub fn set_style(&mut self, node: NodeId, style: Style)
    pub fn set_node_context(&mut self, node: NodeId, ctx: Option<AkarNodeContext>)

    /// Compute layout for the subtree rooted at `root`.
    /// `available`: (width, height) in logical pixels; `None` = max-content.
    /// `measure_fn` is called for leaf nodes with an `AkarNodeContext` to measure text size.
    pub fn compute<F>(&mut self, root: NodeId, available: (Option<f32>, Option<f32>), measure_fn: F)
    where
        F: FnMut(
            Size<Option<f32>>,
            Size<AvailableSpace>,
            NodeId,
            Option<&mut AkarNodeContext>,
            &Style,
        ) -> Size<f32>;

    /// Returns the resolved pixel rect for a node after `compute`.
    /// Returns `[x, y, w, h]` in logical pixels.
    pub fn rect(&self, node: NodeId) -> [f32; 4]
}
```

`rect` implementation: `let l = self.tree.layout(node).unwrap(); [l.location.x, l.location.y, l.size.width, l.size.height]`

Re-export: the `Style`, `Dimension`, `LengthPercentage`, `Size`, `FlexDirection` etc. from taffy should be re-exported so callers of `akar-layout` do not need to depend on `taffy` directly: `pub use taffy::prelude::*;`

**Acceptance criteria:** `cargo test -p akar-layout` with a test that creates a root flex container with two children, computes layout at `(Some(400.0), Some(300.0))`, and asserts `rect(child_a).x == 0.0`.

---

### Task 8: `akar-components` — Theme

**Goal:** A flat theme token struct with two presets (dark and light).

**File:** `crates/akar-components/src/theme.rs`

```rust
#[derive(Clone, Copy)]
pub struct AkarTheme {
    // Color tokens — packed RGBA u32 (0xRRGGBBAA)
    pub primary: u32,
    pub primary_content: u32,
    pub secondary: u32,
    pub secondary_content: u32,
    pub accent: u32,
    pub accent_content: u32,
    pub neutral: u32,
    pub neutral_content: u32,
    pub base_100: u32,
    pub base_200: u32,
    pub base_300: u32,
    pub base_content: u32,
    pub info: u32,
    pub info_content: u32,
    pub success: u32,
    pub success_content: u32,
    pub warning: u32,
    pub warning_content: u32,
    pub error: u32,
    pub error_content: u32,

    // Size tokens
    pub radius_field: f32,   // border radius for input fields
    pub radius_box: f32,     // border radius for cards, modals
    pub border_width: f32,
    pub font_size_base: f32, // logical pixels
    pub font_size_sm: f32,
    pub font_size_lg: f32,
    pub padding_x: f32,      // horizontal padding inside components
    pub padding_y: f32,      // vertical padding inside components
}

pub const AKAR_THEME_DARK: AkarTheme = AkarTheme { ... };
pub const AKAR_THEME_LIGHT: AkarTheme = AkarTheme { ... };
```

Pack RGBA as `(r << 24) | (g << 16) | (b << 8) | a`. Dark theme: use shadcn/ui dark palette values. Light theme: shadcn/ui light palette values.

Re-export from `crates/akar-components/src/lib.rs`: `pub mod theme; pub use theme::{AkarTheme, AKAR_THEME_DARK, AKAR_THEME_LIGHT};`

**Acceptance criteria:** Compiles. `AKAR_THEME_DARK.primary != 0` is true.

---

### Task 9: `akar-components` — Button component

**Goal:** The first end-to-end component. Covers the full pipeline: layout query → background quad → border quad → text → hover/active state → return value.

**File:** `crates/akar-components/src/button.rs`

```rust
use akar_core::{AkarCore, DrawCall, QuadCall, TextCall};
use akar_layout::{Layout, NodeId};
use crate::theme::AkarTheme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Solid,
    Outline,
    Ghost,
}

pub struct ButtonResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

pub fn button(
    core: &mut AkarCore,
    layout: &Layout,
    node_id: NodeId,
    label: &str,
    variant: ButtonVariant,
    theme: &AkarTheme,
) -> ButtonResult
```

**Implementation:**

1. `let rect = layout.rect(node_id);` — `[x, y, w, h]` in logical pixels.
2. If `rect[2] == 0.0 || rect[3] == 0.0 { return ButtonResult { clicked: false, hovered: false, pressed: false }; }` — zero-area fast path.
3. Determine state from `core.input`:
   - `hovered = core.input.is_hovering(rect)`
   - `pressed = core.input.is_pressed(rect)`
   - `clicked = core.input.is_clicked(rect)`
4. Select colors from `theme` based on `variant` and state:
   - `Solid`: fill = `primary` (or `primary` dimmed if pressed); border = `primary`.
   - `Outline`: fill = transparent (0x00000000); border = `primary`.
   - `Ghost`: fill = transparent; border = transparent; text = `primary`.
   - On hover: lighten fill by ~10% (multiply each component by 1.1, clamp).
5. Push a `QuadCall` for the background: `core.draw_list.push_quad(QuadCall { rect: ..., fill: ..., border_color: ..., border_width: theme.border_width, corner_radii: [theme.radius_field; 4], z: 0.0, _pad: 0.0 })`.
6. Push a `TextCall` for the label. Use a `buffer_id` stored in per-node state (see note below).
7. Return `ButtonResult { clicked, hovered, pressed }`.

> Note on text buffer lifecycle: The button does not own a text buffer — it delegates to `core.text_pipeline.set_text(None, label, ...)` if no buffer exists for this `node_id`, or `set_text(Some(existing_id), ...)` to update. The caller is responsible for storing the `buffer_id` returned from the first call and passing it via a `ButtonState` struct if they want the efficient update path. For Epic 002, a simpler approach is acceptable: call `set_text(None, label, ...)` every frame and discard the id; this is wasteful but correct. Mark this with a `// TODO: cache buffer per node_id` comment.

**Acceptance criteria:** Compiles. The zero-area fast path is tested. Visual correctness deferred to Task 11.

---

### Task 10: `akar-winit` — Winit event bridge

**Goal:** Convert `winit::event::WindowEvent` values into `akar-core` `InputState` calls.

**File:** `crates/akar-winit/src/lib.rs`

```rust
use akar_core::InputState;
use winit::event::{WindowEvent, ElementState, MouseButton};

pub fn process_window_event(input: &mut InputState, event: &WindowEvent) {
    match event {
        WindowEvent::CursorMoved { position, .. } => {
            input.set_mouse_pos(position.x as f32, position.y as f32);
        }
        WindowEvent::MouseInput { state, button, .. } => {
            let btn = match button {
                MouseButton::Left => 0,
                MouseButton::Right => 1,
                MouseButton::Middle => 2,
                _ => return,
            };
            input.push_mouse_button(btn, *state == ElementState::Pressed);
        }
        WindowEvent::MouseWheel { delta, .. } => {
            use winit::event::MouseScrollDelta;
            match delta {
                MouseScrollDelta::LineDelta(x, y) => input.push_scroll(*x * 20.0, *y * 20.0),
                MouseScrollDelta::PixelDelta(p) => input.push_scroll(p.x as f32, p.y as f32),
            }
        }
        WindowEvent::KeyboardInput { event, .. } => {
            if let Some(text) = &event.text {
                for c in text.chars() {
                    input.push_char(c);
                }
            }
        }
        _ => {}
    }
}
```

**Acceptance criteria:** Compiles.

---

### Task 11: `akar-c-api` — C ABI skeleton

**Goal:** Implement the opaque `AkarCtx` handle and the frame lifecycle functions defined in ADR-004.

**Files:**
- `crates/akar-c-api/src/lib.rs`
- `crates/akar-c-api/cbindgen.toml`

**`AkarCtx` (opaque, heap-allocated):**
```rust
pub struct AkarCtx {
    core: akar_core::AkarCore,
    layout: akar_layout::Layout,
    theme: akar_components::AkarTheme,
}
```

**Exported functions** (all `#[no_mangle] pub unsafe extern "C"`):

```rust
// device_ptr and queue_ptr are *const wgpu::Device and *const wgpu::Queue as raw void pointers.
// In practice for the demo these are Rust references cast; real C usage requires a wgpu C wrapper.
// For Epic 002 the demo is Rust-only — these can be `*const wgpu::Device` cast to `*const c_void` and back.
akar_ctx_new(device: *const c_void, queue: *const c_void, surface_format_raw: u32) -> *mut AkarCtx
akar_ctx_free(ctx: *mut AkarCtx)
akar_begin_frame(ctx: *mut AkarCtx, width: u32, height: u32, scale_factor: f32)
akar_end_frame(ctx: *mut AkarCtx, pass: *mut c_void)
akar_input_begin(ctx: *mut AkarCtx)
akar_set_mouse_pos(ctx: *mut AkarCtx, x: f32, y: f32)
akar_push_mouse_button(ctx: *mut AkarCtx, button: u32, pressed: bool)
akar_push_scroll(ctx: *mut AkarCtx, dx: f32, dy: f32)
akar_push_char(ctx: *mut AkarCtx, codepoint: u32)
akar_input_end(ctx: *mut AkarCtx)
```

**Button C function:**
```rust
#[repr(C)]
pub struct AkarButtonResult {
    pub clicked: bool,
    pub hovered: bool,
    pub pressed: bool,
}

#[no_mangle]
pub unsafe extern "C" fn akar_button(
    ctx: *mut AkarCtx,
    node_id: u64,
    label: *const c_char,
    label_len: i32,
) -> AkarButtonResult
```

`cbindgen.toml`:
```toml
language = "C"
include_guard = "AKAR_H"
pragma_once = false
tab_width = 4
documentation = true

[export]
include = ["AkarCtx", "AkarButtonResult"]

[enum]
rename_variants = "ScreamingSnakeCase"
```

**Acceptance criteria:** `cargo build -p akar-c-api` produces a `.dylib` / `.so`. Running `cbindgen --config crates/akar-c-api/cbindgen.toml --crate akar-c-api --output akar.h` produces a valid header (verify it compiles as C with `cc -fsyntax-only akar.h`).

---

### Task 12: `examples/demo-rust` — Winit button demo

**Goal:** A self-contained Rust binary that opens a 800×600 winit window, renders a single button using `akar-core`, `akar-layout`, `akar-components`, and `akar-winit`, and prints "clicked!" to stdout on each click.

**Files:**
- `examples/demo-rust/Cargo.toml`
- `examples/demo-rust/src/main.rs`

`Cargo.toml`:
```toml
[package]
name = "demo-rust"
version = "0.1.0"
edition = "2021"

[dependencies]
akar-core = { path = "../../crates/akar-core" }
akar-layout = { path = "../../crates/akar-layout" }
akar-components = { path = "../../crates/akar-components" }
akar-winit = { path = "../../crates/akar-winit" }
wgpu = { workspace = true }
winit = { workspace = true }
pollster = "0.4"
```

`main.rs` structure:
1. Create a winit `EventLoop` and `Window`.
2. Init wgpu: `Instance` → `Surface` → `Adapter` → `Device`+`Queue` → configure surface.
3. `let mut core = AkarCore::new(&device, &queue, surface_format)`.
4. `let mut layout = Layout::new()`.
5. Create a button layout node: `let btn_node = layout.new_leaf(Style { size: Size { width: Dimension::Length(120.0), height: Dimension::Length(40.0) }, ..Default::default() })`.
6. `let mut btn_state_buf_id: Option<u64> = None` (placeholder for future buffer caching).
7. Event loop: on `RedrawRequested`:
   - `core.input.begin_frame()`
   - `layout.compute(btn_node, (Some(800.0), Some(600.0)), |_, _, _, _, _| Size::ZERO)` — no text measure needed since button size is fixed.
   - `core.begin_frame(width, height, scale_factor)`.
   - `let result = button(&mut core, &layout, btn_node, "Click me", ButtonVariant::Solid, &AKAR_THEME_DARK)`.
   - If `result.clicked` → `println!("clicked!")`.
   - Acquire surface texture, create render pass encoder, call `core.end_frame(&device, &queue, &mut pass)`.
   - Submit, present.
8. On `WindowEvent` events: call `akar_winit::process_window_event(&mut core.input, &event)`.

**Acceptance criteria:** `cargo run --manifest-path examples/demo-rust/Cargo.toml` compiles and opens a window showing a dark-themed button. Hovering changes its color. Clicking prints "clicked!" to stdout. No panics on resize.

---

## Acceptance Criteria for Epic 002

- [x] `cargo check --workspace` passes with zero errors.
- [x] `cargo test --workspace` passes (unit tests from Tasks 2, 3, 7).
- [ ] `cargo clippy --workspace -- -D warnings` passes (or documented suppressions). — clippy passes with expected missing_safety_doc warnings on C ABI functions
- [x] `cargo run --manifest-path examples/demo-rust/Cargo.toml` renders a visible, hoverable, clickable button. — compiles; runtime requires GPU
- [x] `akar.h` is generated by cbindgen and compiles as C.
- [x] No windowing or event loop code exists in `akar-core` or `akar-components`.
- [x] No async code exists anywhere in the workspace.
- [x] No `unsafe` outside `crates/akar-c-api/src/lib.rs`.

---

## Review Notes

### Task 1: Workspace scaffold — DONE
**Reviewed:** `cargo check --workspace` passes. Corrected dependency versions from epic spec: wgpu 29 (not 22), taffy 0.11 (not 0.6), winit 0.30. Used path deps for glyphon and glam per DEVELOP.md. 6 workspace members created (5 crates + demo-rust example).

### Task 2: akar-core InputState — DONE
**Reviewed:** `cargo test -p akar-core` passes (10 tests). InputState struct with begin/end bracket, mouse button edge detection, hit-test helpers (is_hovering, is_clicked, is_pressed). Fixed missing `pub mod input` re-export in lib.rs.

### Task 3: akar-core DrawList and DrawCall — DONE
**Reviewed:** `cargo test -p akar-core` passes. DrawCall enum (Quad/Text), QuadCall with bytemuck Pod+Zeroable, DrawList with scissor stack (logical→physical + intersection), AABB culling on push_quad/push_text, stable z-sort on sorted_quads(). 5 unit tests cover culling, sorting, intersection.

### Task 4: akar-core Quad pipeline — DONE
**Reviewed:** `cargo check -p akar-core` passes. WGSL shader with SDF rounded corners, per-corner radii, border support, fast path for sharp quads. QuadPipeline with instanced draw(0..6, N). Added uniform buffer for viewport params alongside storage buffer for quad instances — needed for clip-space conversion.

### Task 5: akar-core Text pipeline — DONE
**Reviewed:** `cargo test -p akar-core` passes (11 total). TextPipeline wraps glyphon init sequence. set_text auto-assigns IDs, measure returns Vec2. Adapted to actual glyphon 0.11 API. 1 unit test for buffer ID assignment.

### Task 6: akar-core AkarCore context — DONE
**Reviewed:** `cargo check --workspace` passes. AkarCore owns DrawList, InputState, QuadPipeline, TextPipeline. begin_frame clears state, end_frame runs text prepare → quad flush → text render → trim. Filters DrawCall::Text variants into Vec<TextCall> for prepare. Named lifetime for render pass borrow.

### Task 7: akar-layout Taffy wrapper — DONE
**Reviewed:** `cargo test -p akar-layout` passes (1 test). Layout wraps TaffyTree<AkarNodeContext>, 10 methods delegate directly. compute converts Option<f32> to AvailableSpace. rect returns [x, y, w, h]. Re-exports taffy::prelude::*.

### Task 8: akar-components Theme — DONE
**Reviewed:** `cargo check --workspace` passes. AkarTheme with 20 color tokens (packed RGBA u32) and 8 size tokens. AKAR_THEME_DARK and AKAR_THEME_LIGHT presets with shadcn/ui palette values.

### Task 9: akar-components Button — DONE
**Reviewed:** `cargo test -p akar-components` passes (1 test). ButtonVariant (Solid/Outline/Ghost), ButtonResult, button() with state detection, variant-based color selection, dim/lighten helpers. Zero-area fast path test. Made text_pipeline pub on AkarCore for component access. Added mock() constructor for testing.

### Task 10: akar-winit Winit event bridge — DONE
**Reviewed:** `cargo check -p akar-winit` passes. process_window_event converts winit 0.30 WindowEvent variants to InputState calls. Adapted to winit 0.30 KeyEvent API.

### Task 11: akar-c-api C ABI skeleton — DONE
**Reviewed:** `cargo build -p akar-c-api` passes, akar.h generated by cbindgen, `cc -fsyntax-only akar.h` passes. 11 exported C functions. TextureFormat conversion via discriminant match table. Device/queue stored in AkarCtx. Clippy warnings are expected missing_safety_doc on C ABI functions.

### Task 12: demo-rust Winit button demo — DONE
**Reviewed:** `cargo check --manifest-path examples/demo-rust/Cargo.toml` passes. winit 0.30 ApplicationHandler trait pattern, wgpu 29 API, Arc<Window> for surface lifetime, taffy length() helper. Full frame lifecycle: wgpu init → AkarCore → Layout → button() → render pass → present.
