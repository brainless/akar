# Epic 017: Reusable Data Items and Lists

**Status:** In Progress
**Goal:** Provide application-agnostic data-item and list primitives that work in ordinary akar layouts and have a deliberate canvas LOD path, without importing application data models or creating a second transformed-component renderer.

**Prerequisite:** Epic 016 (Canvas Level of Detail and Component Portals) is `Status: Done`.

---

## Problem Statement

Applications commonly need to render records such as messages, tweets, commits, search results, notifications, and files. Today akar provides primitives such as `container`, `label`, `scroll_area_begin/end`, and the fixed-height `list_clip` helper, but it has no reusable item shell or list component.

As a result, applications reproduce the same work: card geometry, hover and selection response, scrolling, clipping, visible-range calculation, and the mapping from record content to title, supporting text, metadata, and trailing content. Sugacode's git-log and search-result cards demonstrate this. Their data model correctly belongs to sugacode, but its rendering code is coupled to one canvas: it directly submits quads, creates absolute overlay nodes for text, transforms world coordinates manually, and performs hover/selection inline.

The solution must preserve two existing architecture boundaries:

- akar supplies presentation, interaction responses, clipping, and layout/canvas helpers. Applications retain their data types, keys, loading, mutations, and selection policy.
- Full child-component interaction in a canvas is available only through the screen-space portal path introduced in Epic 016. Low-detail canvas items are transformed primitives and display text with group-level interaction; they are not a world-space backend for arbitrary components.

---

## Architecture Decisions

### ADR-016: Data Items Are Presentation Primitives, Not Records

**Decision:** akar will not define `Tweet`, `Message`, `Commit`, `Document`, or any generic owned record type. A data item is a composable visual shell over caller-provided layout nodes and caller-owned content.

**Rationale:** The common part of a commit and a message is their presentation and interaction behavior, not their schema. An akar-owned record type would either be too narrow for real applications or grow into an application data model.

**Consequences:**

- A `data_item` API renders item chrome and returns immediate-mode interaction state.
- Applications create the child layout and render ordinary components for title, supporting text, badges, avatars, buttons, and custom content.
- The item response does not mutate selection. Single-select, multi-select, range-select, and navigation remain application policy.
- Every item needs a stable caller-provided key wherever state or identity is required.

### ADR-016a: Virtualized Item Identity Is Keyed, Not Positional

**Decision:** Widget identity inside a virtualized list is composed from three parts: `namespace_id` (portal instance), a caller-provided stable item key (record identity), and the local structural node id (which child within the item). The existing `Layout::widget_id(node)` composes only the first and third and is not sufficient for virtualized items.

**Rationale:** Immediate-mode rebuilding means a given screen row (structural position) can hold a different record after every scroll or data mutation. If identity is derived from structural position alone, stateful child widgets — focus, text-input buffers, cursor position — silently attach to a screen position instead of the record occupying it. This is a correctness bug specific to virtualization (a static, non-scrolling layout does not have this problem, which is why it was not caught by the existing portal namespace mechanism), not a hypothetical edge case, and downstream applications depend on virtualized lists being correct under scrolling with focused/edited rows.

**Consequences:**

- `data_item` and any focusable child rendered inside it must be identified through a keyed path (e.g. `Layout::widget_id_keyed(node, key)`), not the plain `widget_id(node)` used elsewhere.
- `data_list` requires a caller-provided key per visible item (e.g. via `key_for(index)` or keys passed alongside the visible range) — `item_count` alone is not sufficient.
- Scrolling a list such that a screen row now renders a different record must not carry over focus or text-buffer state from the previous occupant of that row.
- This changes the public API surface in `data_item`/`data_list` from the earlier sketch: a key parameter is required, not optional polish.

### ADR-017: Lists Are Layout Scopes With Caller-Driven Rendering

**Decision:** A list owns viewport behavior and exposes the visible items to render. It does not own a collection, fetch data, retain item widgets, or require a callback-based renderer in the core API.

**Rationale:** akar is immediate mode and has a C ABI. A Rust closure-based list renderer is awkward for C consumers and hides important frame ordering. An explicit begin/render-visible-items/end scope matches existing scroll, canvas, dropdown, and portal APIs.

**Consequences:**

- The initial list supports fixed-height items and uses the existing O(1) `list_clip` behavior behind a component-level API.
- The caller supplies item count, row height, stable keys, and renders only the visible range.
- Variable-height virtualization is out of scope for the initial API. Applications may use an unvirtualized layout list or an application-managed measurement/index until a dedicated follow-up is designed.
- List scroll position is caller-owned, allowing applications to persist, synchronize, or reset it deliberately.

### ADR-018: Canvas Reuse Has Two Render Paths

**Decision:** A low-detail canvas data item will use a constrained `CanvasPainter` helper and display-only text. At the caller-selected interactive LOD, the normal layout `data_item` and `data_list` APIs render through `canvas_portal_begin/end`.

**Rationale:** This reuses visual semantics and data mapping across canvas and screen layouts without duplicating the full component catalog in world space or making tiny controls interactive.

**Consequences:**

- The canvas helper supports an item background and bounded textual fields appropriate for summary/preview levels.
- Low-detail interaction is one response for the item bounds via `CanvasInput`; title, metadata, and trailing content are not individual hit targets.
- Portals retain standard components, text input, focus, scrolling, and clipping.
- The same item styling vocabulary should be usable by both paths where meaningful, while geometric values on canvas continue to be world units and scale with zoom.

### ADR-019: The C ABI Is Designed Alongside the Rust API

**Decision:** The implementation starts in Rust but reserves a flat, begin/end and descriptor-oriented C ABI. It must not make Rust closures or generic item records part of the required behavior.

**Rationale:** akar's public contract is `akar.h`. Designing only a Rust closure API would force a later incompatible redesign for every non-Rust consumer.

---

## Public API Direction

Exact names are subject to implementation review. The intended layering is:

```rust
pub struct DataItemStyle { /* surface, padding, spacing, state colors */ }
pub struct DataItemResponse {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
}

pub fn data_item(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    key: u64,
    style: &DataItemStyle,
) -> DataItemResponse;

pub struct DataListState {
    pub scroll_y: f32,
}
pub struct DataListResponse { /* viewport rect, content origin, visible range */ }

pub fn data_list_begin(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    state: &mut DataListState,
    item_count: usize,
    item_height: f32,
) -> DataListResponse;
pub fn data_list_end(core: &mut AkarCore);
```

`key` is the caller's stable record identity (e.g. a hash of a database id or content key), not the row's screen position. The caller creates the list root and each visible item subtree, then invokes ordinary components inside each item, passing the same `key` (or a value derived from it) to any focusable child so it can be identified via `Layout::widget_id_keyed` rather than plain `widget_id`. See ADR-016a: this key is required, not optional — without it, focus and text-buffer state attach to the row's structural position instead of its record and corrupt on scroll.

For canvas summary levels, an item descriptor should carry only display-oriented fields, for example title, supporting text, metadata, and a style. It is not a generic nested component tree. At interactive detail, callers create or retrieve a portal-local `Layout`, set its screen origin and stable namespace, and use the normal item/list APIs inside the portal.

---

## Implementation Tasks

### Task 1: API Design and Identity Contract ✅

**Files:**

- `crates/akar-components/src/data_item.rs` (new)
- `crates/akar-components/src/data_list.rs` (new)
- `crates/akar-components/src/lib.rs`
- `crates/akar-layout/src/lib.rs` — added `widget_id_keyed`

**Work:**

1. Define the minimal compositional data-item style and response types.
2. Establish stable item-key rules for virtualized items, including how keys compose with portal layout namespaces and child widget IDs.
3. Define a fixed-height list scope API that is usable from Rust and translatable to flat C begin/end calls.
4. Document the intentional omission of application records, selection ownership, loading, sorting, and variable-height virtualization.

**Acceptance criteria:**

- [x] The API can express a commit, message, tweet, and search-result item without an akar-owned record enum.
- [x] Two visible items with identical local layout node IDs cannot collide in focus or text-buffer identity.
- [x] Scrolling a virtualized list such that a screen row now renders a different record does not carry over focus or text-buffer state from the previous record at that position (see ADR-016a). This must be covered by a test that scrolls a list with a focused/edited row and asserts the new occupant is not focused/pre-filled.
- [x] The API does not require a Rust closure to render list items.

### Task 2: Layout Data Item ✅

**Files:**

- `crates/akar-components/src/data_item.rs` — fully rendered item shell
- `crates/akar-components/src/theme.rs`
- `crates/akar-components/src/lib.rs`

**Work:**

1. Implement the item shell using the resolved layout rect and existing draw-list primitives.
2. Provide theme-derived default styling and explicit style overrides for normal, hover, pressed, and selected presentation.
3. Return interaction state without retaining or mutating caller selection.
4. Ensure zero-area and transparent styles submit no invalid draw calls.

**Acceptance criteria:**

- [x] The shell composes with labels, badges, avatars, buttons, and text inputs supplied by the caller.
- [x] Hover, press, and click behavior is unit-tested with `AkarCore::mock`.
- [x] Existing components and theme behavior remain backward compatible.

### Task 3: Fixed-Height Layout Data List ✅

**Files:**

- `crates/akar-components/src/data_list.rs` — scissor, scroll, clamp, nested scissor, ADR-016a test
- `crates/akar-components/src/lib.rs`
- `crates/akar-core/src/lib.rs` — `list_clip` unchanged

**Work:**

1. Implement list clipping, wheel scrolling, clamping, and explicit scope cleanup using the existing scissor stack.
2. Expose a padded visible range and content origin suitable for callers to construct only visible item layouts.
3. Confirm nesting with scroll areas, portals, and canvas scissors.
4. Document the fixed-height constraint and the supported fallback for variable-height data.
5. Require and thread a per-item key (see ADR-016a) alongside the visible range — the caller supplies `key_for(index)` or an equivalent keyed source, and `data_list` must make it straightforward to pass that key into each visible item's `data_item` call. `item_count` alone must not be treated as sufficient identity.

**Acceptance criteria:**

- [x] No off-screen item is required to submit quads or shape text.
- [x] Scroll offsets clamp correctly for empty, short, and long lists.
- [x] The list scissor is restored after end, including when nested in a portal.
- [x] Unit tests cover range boundaries, scroll input, and nested scissors.
- [x] A unit test scrolls the visible range by one item, renders a focused/edited item at the boundary row, and confirms focus/text-buffer state does not transfer to the newly visible record at that position.

### Task 4: Canvas Data-Item Summary Helper ✅

**Files:**

- `crates/akar-components/src/canvas.rs`
- `crates/akar-components/src/data_item.rs` — added `CanvasDataItemDescriptor`, `canvas_data_item()`
- `crates/akar-components/src/color.rs` — added `f32_to_color`
- `crates/akar-components/src/lib.rs`

**Work:**

1. Define a constrained display descriptor and style shared with the layout item where applicable.
2. Render world-space background and display-only textual fields through `CanvasPainter`.
3. Return group-level world-space hover/press/click using `CanvasInput`.
4. Preserve canvas culling, clipping, and world-unit scaling.

**Acceptance criteria:**

- [x] Summary items create no layout nodes, focusable widgets, text-buffer IDs, or child hit targets.
- [x] Invisible items are culled before text shaping.
- [x] Canvas item geometry and text scale correctly with zoom.

### Task 5: Portal Composition Example and Visual Verification

**Files:**

- `examples/canvas-basic-rust/src/main.rs` or a focused data-list example
- relevant component tests and scripted capture assets

**Work:**

1. Demonstrate summary canvas items at low detail and normal data-item/list rendering in a portal at interactive detail.
2. Include at least one focusable child in the portal to verify stable namespaced identity.
3. Capture an overview and an interactive portal state; use frame dumps when validating scissor or identity behavior.

**Acceptance criteria:**

- The example makes the transition boundary clear without implying world-space child interaction.
- Overview and interactive screenshots are visually verified.

### Task 6: C ABI and Documentation

**Files:**

- `crates/akar-c-api/`
- generated `akar.h` through the existing cbindgen workflow
- `DEVELOP.md`
- `README.md`
- `AGENTS.md`

**Work:**

1. Expose the agreed descriptor and begin/end surface through `akar-c-api` where the component API is mature enough.
2. Add C integration coverage without editing `akar.h` manually.
3. Document data ownership, fixed-height virtualization, canvas summary limits, and portal reuse guidance.

**Acceptance criteria:**

- A C caller can render a list through the generated header without application data ownership moving into akar.
- `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` pass.

---

## Scope

### Included

- Composable item shells and responses.
- Fixed-height, virtualized layout lists with caller-owned scroll state.
- Low-detail canvas item summaries and group-level interaction.
- Portal reuse of normal data-item and list components.
- Stable item identity rules and C ABI-aware API design.

### Deferred

- Application data models, fetching, sorting, filtering, and selection policy.
- Variable-height virtualized lists and measurement caches.
- Arbitrary child-widget interaction below the canvas interactive LOD.
- A generic transformed-world backend for the component catalog.
- Retained list/item trees or an akar-owned portal lifecycle registry.

---

## Acceptance Criteria

- [ ] An application can render its own messages, tweets, commits, and search results with the same item/list primitives.
- [ ] Item interaction is reported without akar owning selection or record state.
- [ ] Fixed-height lists clip and virtualize correctly in a normal layout and in a portal.
- [ ] Canvas summaries use display-only text and group interaction only.
- [ ] Full interactive items in a canvas use a clipped portal with ordinary components.
- [ ] Item and child-widget identity remains stable and collision-free across virtualized rows and portal layouts, keyed by caller-provided record identity rather than screen position (ADR-016a) — verified by a scroll-and-refocus test, not just a same-frame collision test.
- [ ] The C ABI follows the generated-header contract.
- [ ] Formatting, clippy, tests, and representative visual captures pass.
