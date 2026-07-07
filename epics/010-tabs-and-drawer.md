# Epic 010: Tabs and Drawer

**Status:** Planned
**Goal:** Two structural components that organize content into switchable regions (Tabs) and a collapsible side panel (Drawer). This epic introduces Z-layer rendering — the ability to render content above the normal page layer — required for Drawer and foundational for Epic 011's overlay stack.

**Prerequisite:** Epic 009 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean.

---

## Task Notes

### Task 1 — Z-Layer Constants (Done)

`crates/akar-core/src/lib.rs`:
- `Z_BASE = 0.0`, `Z_SCRIM = 0.5`, `Z_FLOAT = 1.0`, `Z_OVERLAY = 2.0` added as public constants.
- Exported from `akar-core` crate root. Clippy and tests pass.

### Task 2 — Tabs Component (Done)

`crates/akar-components/src/tabs.rs`:
- `TabVariant` enum: `Boxed`, `Lifted`, `Pills`, `Underline`
- `tab_bar()` renders evenly-spaced tabs across node width; uses manual rect math (no sub-layout)
- Each variant renders: Boxed (border + distinct fill), Lifted (rounded top, flat bottom on active), Pills (full pill), Underline (3px accent bar on active)
- Text via `text_pipeline.set_text()` with per-variant active/inactive colors
- Hit-testing via `core.input.is_clicked()` per tab rect
- Tests: zero labels (0 calls, no click), all tabs rendered (3 quads, active fill check), click detection (simulated click returns Some(0)), all variants no panic (4 variants > 0 calls)
- 4 tests pass, 43 total component tests, clippy clean

### Task 3 — Drawer Component (Done)

`crates/akar-components/src/drawer.rs`:
- `DrawerEdge` enum: `Left`, `Right`
- `drawer_begin()` renders scrim at `Z_SCRIM` (0.5) and panel background at `Z_FLOAT` (1.0), pushes scissor for panel rect, returns `DrawerResponse { close_requested }`
- `drawer_end()` pops the scissor
- Panel radii: outer edge flat, inner edge rounded via `theme.radius_box`
- Panel shadow direction depends on edge
- Tests: zero width (no draw calls), left/right edge (2 quads at correct positions), scrim click (close_requested=true), panel click (close_requested=false), scissor push/pop
- 6 tests pass, 49 total component tests, clippy clean

### Task 4 — C ABI Exposure (Done)

`crates/akar-c-api/src/lib.rs`:
- `AkarTabBarResponse { clicked_index: i32 }` repr(C) struct
- `akar_tab_bar(ctx, node_id, labels, label_count, label_lengths, active_index, variant)` — wraps `akar_components::akar_tab_bar`, uses same string array pattern as `akar_steps`
- `AkarDrawerResponse { close_requested: bool }` repr(C) struct
- `akar_drawer_begin(ctx, edge, panel_width, viewport_rect)` — wraps `drawer_begin`, rect passed as `*const f32` (same pattern as `akar_scroll_area_begin`)
- `akar_drawer_end(ctx)` — wraps `drawer_end`
- `akar.h` regenerated — all 5 new symbols confirmed present (AkarDrawerResponse, AkarTabBarResponse, akar_drawer_begin, akar_drawer_end, akar_tab_bar)

### Task 5 — Demo Update (Done)

`examples/demo-rust/src/main.rs` — reworked layout and rendering:
- Navbar "Notifications" button → "Menu" button; toggles `drawer_open`
- Tab bar (Underline variant) with 3 tabs: List, Canvas, Stats; switching reparents `panel_container` children
- Tab "List": existing 50-item scrollable list with progress bars
- Tab "Canvas": centered placeholder text
- Tab "Stats": stat cards + steps + avatar row with skeleton toggle
- Drawer: animated via `ease_out_cubic` (speed 0.08, max 250px), renders avatar circle + 4 nav links, scrim click closes
- Clippy and test pass clean



---

## Scope

### Components

#### Tabs
A tab bar plus controlled panel switching. The `tab_bar` component renders a horizontal row of tab buttons. The caller tracks the active tab index and renders the appropriate panel content below. No retained panel tree — the caller wraps the panel content in a container and calls only the active panel's components per frame.

Returns a `TabBarResponse { clicked: Option<usize> }` so the caller knows when to switch the active index.

**Variants from daisyUI/shadcn:**
- `Boxed` — tabs have a border, active tab has a distinct fill.
- `Lifted` — tabs sit on top of the panel border, active tab merges into the panel.
- `Pills` — rounded pill buttons, no panel border.
- `Underline` — minimal, just an underline on the active tab.

Default variant: `Boxed`.

#### Drawer (Side Panel / Sheet)
A panel that slides in from the left or right edge, rendering above the main content. The caller controls open/closed state and (if desired) animation progress as a `f32` in `[0.0, 1.0]`. akar renders the panel at the interpolated width; animation is caller-managed (immediate mode — no retained animation state).

Requires Z-layer rendering: the drawer quad and its content must render above the main page content. The draw list's Z value is used — drawer components render at `z = 1.0`, main content at `z = 0.0`.

**Structure:**
- Scrim (semi-transparent overlay covering the main area) at `z = 0.5`.
- Drawer panel (card-style background) at `z = 1.0`.
- Drawer content (caller-rendered at z = 1.0 as well).

Returns a `DrawerResponse { close_requested: bool }` — true when the user clicks the scrim.

---

## Key Design Decisions

### Z-Layer Rendering

The draw list already sorts by `z: f32` and has a z field on every `QuadCall`. What is missing is a convention:

| Layer | z value | Used for |
|---|---|---|
| Base | 0.0 | Normal page content, containers, cards |
| Scrim | 0.5 | Drawer scrim, modal backdrop |
| Float | 1.0 | Drawer panel, modal dialog (Epic 011) |
| Overlay | 2.0 | Tooltip, toast (Epic 011) |

These are defined as public constants in `akar-core`:

```rust
pub const Z_BASE: f32    = 0.0;
pub const Z_SCRIM: f32   = 0.5;
pub const Z_FLOAT: f32   = 1.0;
pub const Z_OVERLAY: f32 = 2.0;
```

All existing components use `z: 0.0` implicitly — no change needed. New components that render above the base layer pass `Z_FLOAT` or `Z_OVERLAY` to their `QuadCall`s.

**Hit testing and z-order.** The draw list sorts for rendering but `InputState` has no z-aware hit testing — `is_hovering` is a pure rect check. When a drawer is open, the caller should only call components inside the drawer; the main page components should be skipped. This is app logic, not akar logic. The scrim's `close_requested` flag gives the caller the signal to close.

### Tabs Panel Ownership

Tabs do not own or manage the panel content node. The caller creates a panel node in their layout tree and renders content into it each frame based on the active tab index. This is correct for immediate mode — no retained widget tree.

### Drawer Width and Animation

The caller passes `panel_width: f32` — the current animated width (computed externally, e.g., `eased_progress * max_width`). akar renders the drawer at exactly that width, positioned against the specified edge. No easing functions in akar — the caller applies whatever curve they want.

### Drawer Clip

The drawer panel content is scissor-clipped to the panel rect so that content rendering during open/close animation does not overflow. The scrim always covers the full viewport minus the drawer panel.

---

## C ABI

- `akar_tab_bar(ctx, bar_node_id, labels, label_count, active_index, variant) -> AkarTabBarResponse`
  where `AkarTabBarResponse { clicked_index: i32 }` (-1 = no click).
- `akar_drawer_begin(ctx, edge, panel_width, viewport_rect) -> AkarDrawerResponse`
  where `edge` is `0=Left, 1=Right` and `AkarDrawerResponse { close_requested: bool }`.
- `akar_drawer_end(ctx)` — pops scissor.

---

## Demo

The demo gains:
- A tab bar below the navbar with three tabs ("List", "Canvas", "Stats"). Switching tabs swaps the visible content panel.
- A drawer triggered by a "Menu" button in the navbar. When open, the drawer shows the avatar and a list of navigation links. Clicking outside closes it.

---

## Acceptance Criteria

- [ ] `Z_BASE`, `Z_SCRIM`, `Z_FLOAT`, `Z_OVERLAY` constants exported from `akar-core`.
- [ ] `tab_bar` renders all tab labels; active tab is visually distinct; returns clicked index correctly.
- [ ] All four tab variants (Boxed, Lifted, Pills, Underline) render without panic.
- [ ] `drawer_begin` / `drawer_end` render the scrim and panel at the correct z-levels.
- [ ] Drawer scissor clips content to the panel rect.
- [ ] `close_requested` is true when the scrim is clicked.
- [ ] All components exposed in `akar.h`.
- [ ] `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass clean.
- [ ] No retained animation state — width is caller-provided.
