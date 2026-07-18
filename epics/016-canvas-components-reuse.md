# Epic 016: Canvas Level of Detail and Component Portals

**Status:** In Progress
**Goal:** Make canvas useful for large, zoomable collections of UI-like objects without turning it into a second general-purpose UI layout system. Canvas objects render application-defined low-detail representations, including styled display text, while zoomed out; when an object has enough projected screen space, the application may render its normal akar layout and components through a canvas-attached screen-space portal.

**Prerequisite:** Epic 004 (Canvas) is `Status: Done`.

---

## Problem Statement

`CanvasPainter` currently draws only transformed world-space quads. Standard components cannot be called in a canvas scope because they expect screen-space layout rectangles, screen-space input, direct draw-list access, and (for text) mutable access to the text pipeline.

Making every component work as a uniformly transformed world-space widget would duplicate component logic, complicate text shaping and focus, and produce controls that are unusable when zoomed out. That is not the desired canvas UX.

A canvas is an overview surface. It should let applications show a large number of objects with progressively richer representations as their projected size increases:

```text
overview → silhouette / group interaction
summary  → structure and selective labels
preview  → richer application-defined representation
detail   → normal layout and fully interactive akar components
```

Only the final detail level needs full component reuse. At that level, the object is rendered as a screen-space portal positioned over its transformed world bounds. Its child layout and components use the ordinary akar APIs unchanged.

Canvas-in-Canvas remains disallowed (ADR-008 from Epic 003).

---

## Architecture Decision Records

### ADR-012: Canvas Uses Application-Defined Level of Detail

**Decision:** Canvas exposes the continuous facts needed to choose a representation—projected screen rect, projected size, visibility, and scale—and a small pure helper for threshold classification. It does not prescribe a fixed component catalog or a fixed number of LOD levels.

**Rationale:** Whether an object needs a label, a row summary, or a complete form depends on the application. A node graph, a dashboard preview, and a page thumbnail need different representations. Projected dimensions are a better decision input than zoom alone: a large object can support more detail at the same zoom than a small object.

**Consequences:**

- Applications may define any number of semantic levels (`Dot`, `Outline`, `Summary`, `Preview`, `Interactive`, etc.).
- The canvas API provides a pure `projected_rect`/`projected_size` helper and a threshold helper over a caller-supplied slice of pixel thresholds.
- Low-detail representations are drawn through `CanvasPainter` and may use group-level `CanvasInput` hit testing.
- The library provides no retained animation state. Applications may use the continuous projected size to cross-fade representations with their own state and timing.

### ADR-013: Full Components Render Through Canvas-Attached Screen-Space Portals

**Decision:** At the application-selected interactive threshold, render a normal screen-space layout subtree anchored to the canvas object's transformed bounds. The subtree uses existing akar components, `AkarCore` input, glyphon text, focus handling, and draw-list submission directly.

**Rationale:** This preserves the primary value of akar's component catalog. A text input, select, button, or checkbox is already correct and interactive once it has a screen-space rect. Re-implementing each one with world-space transforms would create duplicate APIs and behavior.

**Consequences:**

- A portal's root layout is computed with the projected screen width and height, in a dedicated local `Layout` tree.
- The portal layout carries a screen-space origin, so its ordinary `Layout::rect` calls resolve directly to the portal's screen-space child rects.
- A portal pushes the canvas screen rect as a scissor while its subtree renders, so content cannot escape the canvas unintentionally.
- Existing component functions require no behavior changes; they receive portal-resolved screen rects and use normal screen-space input.
- Widget and text-buffer IDs must be namespaced by a `Layout`-owned ID namespace so separate local layouts cannot collide. A portal layout is initialized with a stable caller-provided canvas item ID.
- Menus, tooltips, drawers, and modals remain screen-space overlay behavior. Their treatment outside the portal clip is an explicit application choice and is not required by this epic.
- akar owns no registry, pool, or eviction policy for per-object portal `Layout` instances. The application owns the lifecycle: it creates a `Layout` when an object first crosses into the interactive threshold (e.g. keyed in its own `HashMap<ItemId, Layout>`), and drops it whenever it chooses (object leaves the threshold, scrolls out of view, is deleted). Because widget/text-buffer identity is derived solely from the caller-provided namespace ID and not from any akar-internal counter, an application may freely recreate a `Layout` for the same item ID across frames — including after a drop-and-recreate cycle — without losing focus or buffer stability for that item.

### ADR-014: Low-Detail Canvas Rendering Is Deliberately Limited

**Decision:** The canvas path supplies transformed primitives, bounded display text, projected geometry, visibility, and world-space group hit testing. It does not promise full component rendering or child-widget interaction below the interactive threshold.

**Rationale:** Low-detail rendering must remain cheap and legible at overview scale. A silhouette, card background, list rows, badges, and selective labels cover the intended use cases without trying to make tiny controls usable.

**Consequences:**

- Canvas primitive work includes correcting all world-space quad dimensions: border width, corner radii, shadow offset, blur, and spread scale with zoom.
- `CanvasPainter` provides a bounded text primitive for overview, summary, and preview labels. The caller supplies content and style; akar projects it, applies the canvas clip, culls invisible work, shapes it, and submits it to the renderer.
- Canvas text is display-only, not a small `Label` component tree: it has no focus, selection, text editing, child layout, or child hit targets. Appearance initially covers text metrics, color, optional background and padding, alignment, and overflow behavior supported by the existing text pipeline; arbitrary font loading is not introduced by this epic.
- `CanvasInput` is used for object/group hover, press, and click; portal children use regular `InputState` through existing components.
- A general transformed-world component backend and a trait-based rewrite of the whole component catalog are deferred.

### ADR-015: Canvas Text Is a Library-Managed Display Primitive

**Decision:** Low-detail labels use a `CanvasPainter` text primitive rather than an application-managed screen-space layout path.

**Rationale:** Applications should be able to request a label in world coordinates without recreating projection, temporary layout, clipping, culling, shaping, and text-buffer management. A separate screen-space layout subtree for every summary label would be portal-like boilerplate without offering normal widget interaction.

**Consequences:**

- Applications choose the content and visual style of every low-detail label, including font metrics, size, color, optional background, padding, alignment, and supported overflow behavior.
- akar owns world-to-screen conversion, canvas scissor clipping, visibility culling, text shaping, and draw-list submission end-to-end.
- Text size and background geometry are specified in world units and scale with zoom. Applications may still alter style or omit text at each LOD using projected dimensions.
- The primitive does not create a layout node or widget identity and cannot receive focus or input. Use a portal when a normal label/component subtree or child interaction is required.

---

## Public API Direction

Exact names remain subject to implementation review. The intended shape is:

```rust
pub struct CanvasProjectedRect {
    pub screen_rect: [f32; 4],
    pub pixels_per_world_unit: f32,
    pub visible: bool,
}

pub struct CanvasInput<'a> { /* wraps InputState with a cached world mouse position */ }

pub struct CanvasTextStyle {
    pub font_size: f32,
    pub color: u32,
    pub background: Option<u32>,
    pub padding: [f32; 4],
    // Alignment and overflow fields use the capabilities of the text pipeline.
}

impl CanvasResponse {
    pub fn project(&self, world_rect: WorldRect) -> CanvasProjectedRect;
    pub fn lod_index(&self, world_rect: WorldRect, thresholds_px: &[f32]) -> usize;
}

impl CanvasPainter {
    pub fn push_text(&mut self, world_rect: WorldRect, text: &str, style: &CanvasTextStyle);
}

pub fn canvas_portal_begin(
    core: &mut AkarCore,
    screen_rect: [f32; 4],
) -> CanvasPortalResponse;

pub fn canvas_portal_end(core: &mut AkarCore);
```

The caller owns a dedicated `Layout` for each portal-capable object. Before rendering, it sets that layout's screen origin from the projected portal rect and its stable namespace from the application's canvas item ID, then computes its root to the portal's screen size. Between `canvas_portal_begin` and `canvas_portal_end`, the caller invokes existing components normally with `&mut AkarCore` and `&Layout`. Explicit begin/end functions match the existing scroll, dropdown, drawer, and canvas scope patterns without holding a borrow of `AkarCore` across component calls.

An application can use arbitrary LOD levels:

```rust
match canvas.lod_index(card_bounds, &[48.0, 120.0, 220.0]) {
    0 => draw_dot(&mut painter, card_bounds),
    1 => {
        draw_card_outline(&mut painter, card_bounds, &canvas_input);
        painter.push_text(card_bounds, "Server A", &summary_label_style);
    }
    2 => draw_card_preview(&mut painter, card_bounds),
    _ => render_card_portal(/* normal layout + components */),
}
```

For smooth transitions, applications can use projected dimensions directly to calculate alpha or other visual interpolation; interactive behavior is enabled only at the selected interactive level.

---

## Implementation Tasks

### Task 1: Canvas Geometry, Projection, and Low-Detail Primitives

**Status:** Done

**Files:**

- `crates/akar-components/src/canvas.rs`
- `crates/akar-layout/src/canvas_transform.rs`
- `crates/akar-components/src/lib.rs`

**Work:**

1. Add a public projected-geometry result type and `CanvasResponse::project(WorldRect)`.
2. Add a pure LOD threshold helper. It accepts ordered caller thresholds in logical screen pixels and classifies using the projected minimum dimension. Define and test empty, boundary, and unordered-threshold behavior.
3. Add `CanvasInput`, created once from `&InputState` plus `screen_to_world`. It exposes world-space hover, press, and click checks for `WorldRect`.
4. Correct `CanvasPainter::push_quad` so every world-space geometric field is scaled consistently: border width, corner radii, shadow offset, shadow blur, and shadow spread.
5. Add a buffered `CanvasPainter::push_text` display primitive. It accepts world bounds, text, and caller-provided style; `canvas_end` projects, clips, culls, shapes, and submits the text through the existing text pipeline.
6. Define `CanvasTextStyle` for font metrics/size, color, optional background, padding, alignment, and supported overflow behavior. Specify that its geometric values are world units and scale with zoom; use only font-family capabilities already available through akar's text pipeline.
7. Preserve the existing canvas scissor and world visibility behavior. Canvas text must never create focusable widgets, text-buffer IDs, or child hit targets.

**Acceptance criteria:**

- Unit tests cover projected rects, LOD boundaries, transformed world input, all scaled quad fields, canvas-text projection and clipping, invisible-text culling, and the absence of widget/input state from canvas text.
- `cargo test -p akar-components -p akar-layout` passes.

### Task 2: Portal-Local Layout Resolution

**Status:** Done

**Files:**

- `crates/akar-layout/src/lib.rs`
- `crates/akar-layout/src/portal.rs` (new)
- `crates/akar-layout/src/tests` or module tests as appropriate

**Work:**

1. Add a screen-space origin and stable ID namespace to `Layout`. Both default to zero so existing layouts retain their exact behavior.
2. Add explicit setters used only by caller-owned portal layouts; `Layout::rect` resolves local Taffy results plus the configured origin.
3. Keep current `Layout::rect` semantics unchanged for existing callers.
4. Provide `Layout::widget_id(NodeId) -> u64`, composed from the layout namespace and local node ID. Refactor component-internal text/focus buffer identifiers to use it instead of converting `NodeId` directly.
5. Test nested local child offsets, two portal layouts with the same local node IDs, and zero-area portal roots.

**Acceptance criteria:**

- Portal child rects equal local Taffy results translated by the portal origin, while a default layout produces its existing rects unchanged.
- Portal layouts with identical local node IDs produce distinct widget IDs.
- A `Layout` dropped and later recreated with the same namespace ID produces the same `widget_id` for the same local node ID, confirming the application can freely manage portal `Layout` lifecycle (create on demand, drop on eviction) without an akar-owned registry.
- Existing layout tests and APIs remain backward compatible.
- `cargo test -p akar-layout` passes.

### Task 3: Canvas Portal Scope

**Status:** Done

**Files:**

- `crates/akar-components/src/canvas.rs`
- `crates/akar-components/src/lib.rs`

**Work:**

1. Implement explicit `canvas_portal_begin`/`canvas_portal_end` helpers that accept a projected screen rect and push/pop the portal clip reliably.
2. Ensure it composes with `canvas_begin`/`canvas_end` without losing the canvas scissor or changing existing draw-list behavior.
3. Define the required ordering rule: render the buffered low-detail canvas content before the portal subtree, then render the portal while clipped to its projected bounds.
4. Document the existing renderer limitation that glyphon text is rendered after quads globally; do not claim strict quad/text submission ordering from this epic.
5. Add unit tests for scissor nesting and portal layout rect translation using `AkarCore::mock`.

**Acceptance criteria:**

- A portal subtree can draw through ordinary component functions while clipped to the projected canvas rect.
- Existing screen-space component APIs do not change.
- `cargo test -p akar-components` passes.

### Task 4: Interactive Portal Demonstration

**Status:** Done

**Files:**

- `examples/canvas-basic-rust/src/main.rs`
- `examples/canvas-basic-rust/Cargo.toml` if needed

**Work:**

1. Turn the example objects into a multi-level canvas representation: silhouette, summary with styled canvas text, and interactive portal.
2. Make the threshold configuration visible in source and easy for downstream developers to adapt.
3. Render a normal layout subtree in interactive mode containing at least a container, label, button, and text input.
4. Demonstrate group-level hover and display-only canvas text at low detail, and normal child interaction at interactive detail.
5. Verify pan/zoom still works and that portal content stays clipped to the canvas.

**Acceptance criteria:**

- The example visually demonstrates every LOD level.
- At interactive detail, normal button and text-input behavior works without canvas-specific variants.
- Capture representative overview and interactive screenshots with the project debug workflow.

### Task 5: Tests and Visual Regression Assets

**Status:** Done

**Files:**

- `crates/akar-components/src/canvas.rs`
- `crates/akar-layout/src/portal.rs`
- `examples/canvas-basic-rust/` test assets or scripts as appropriate

**Work:**

1. Add unit tests for all pure LOD, projection, input, canvas-text, and portal-layout behavior.
2. Add scripted visual captures for overview, summary, and interactive modes where the example tooling supports them.
3. Use `--dump-frame` to confirm canvas and portal scissors, transformed geometry, and expected input state when visual output is ambiguous.
4. Run workspace formatting, clippy, and tests.

**Acceptance criteria:**

- `cargo fmt --check` passes.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.

### Task 6: Update `DEVELOP.md`

**File:** `DEVELOP.md`

**Work:**

1. Extend the architecture notes to distinguish low-detail canvas rendering from interactive screen-space portals.
2. Document the projected-size/LOD model, library-managed display text, and that applications own threshold and transition policy.
3. Add the canvas-basic example as the reference for portal composition and visual verification.
4. State the intended overlay behavior and the renderer-wide quad/text ordering limitation.

**Acceptance criteria:** The development guide accurately describes the shipped API and does not promise world-space reuse of every component.

### Task 7: Update `README.md`

**File:** `README.md`

**Work:**

1. Add a concise canvas capability note under design philosophy or stack/status.
2. Explain that canvas supports overview-to-detail LOD and can promote an object to normal interactive components at sufficient detail.
3. Keep the explanation user-focused; link detailed behavior to `DEVELOP.md` and the canvas example rather than duplicating implementation details.

**Acceptance criteria:** README accurately communicates the feature without implying a retained scene graph or a mandatory zoom policy.

### Task 8: Update `AGENTS.md`

**File:** `AGENTS.md`

**Work:**

1. Add canvas LOD/portal guidance to the component and debug-toolchain sections.
2. Require visual verification at at least one overview level and one interactive-portal level for canvas changes.
3. Clarify that low-detail canvas interaction is group-level, canvas text is display-only, and that full inputs/selects belong in portal mode.
4. Document the scissor and global text-after-quads ordering constraint relevant to portal implementation.

**Acceptance criteria:** Agent guidance explains how to choose the canvas path versus the portal path and how to validate each.

---

## Scope

### Included

- Projected geometry and configurable LOD helpers.
- World-space group interaction for low-detail canvas objects.
- Correct transformed quad styling.
- Styled, clipped, culled canvas display text for low-detail labels.
- Screen-space, clipped portals that reuse existing layout and component APIs.
- A canvas example with overview, summary, and interactive states.

### Deferred

- A generic transformed-world backend for every component.
- Canvas-native text input, textarea, select, dropdown, modal, drawer, tooltip, toast, and scroll-area variants.
- Arbitrary font loading or a new font-registration API for canvas text.
- Uniformly scaling every component's text and metrics with canvas zoom.
- Canvas-in-canvas.
- A retained scene graph, animation system, or library-owned LOD state.
- Strict painter ordering between quads and glyphon text; this requires a separate renderer architecture change.

---

## Acceptance Criteria

- [x] A downstream application can choose arbitrary LOD thresholds using projected screen dimensions.
- [x] Low-detail canvas objects support world-space group hover/press/click without child-widget interaction.
- [x] Canvas quad borders, radii, and shadows scale correctly with zoom.
- [x] Low-detail canvas objects can render caller-styled, display-only text that is projected, clipped, and culled by akar without creating widget or input state.
- [x] At interactive detail, a caller can render an ordinary local layout subtree with existing components, including button and text input, inside a clipped portal.
- [x] Existing screen-space component APIs remain backward compatible.
- [x] The canvas-basic example demonstrates all levels and captures overview plus interactive verification states.
- [x] `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` pass.
- [ ] `DEVELOP.md`, `README.md`, and `AGENTS.md` are updated when the implementation lands.
