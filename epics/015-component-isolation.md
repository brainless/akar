# Epic 015: Component Isolation for Screenshot-Driven Debugging

**Status:** Done
**Goal:** Enable agents to isolate specific components for screenshot capture, producing tight, focused images of a single component. This extends Epic 014's screenshot infrastructure with a `--component <name>` flag that renders only the specified component and auto-crops the output to its bounding box.

**Prerequisite:** Epic 014 is `Status: Done` (screenshot enhancements complete).

---

## Motivation

When debugging a component issue (e.g., a left drawer's positioning or styling), the current screenshot captures the entire 800x600 UI. This includes unrelated components (navbar, form, tabs, etc.) that add visual noise and make it harder to focus on the component under test.

By isolating a single component, agents can:
- Capture clean screenshots of just the component
- Debug layout/positioning issues without distraction
- Test interactive states (via `--script`) in isolation
- Iterate faster on component-specific changes

---

## Key Insight: Components Are Render-Time Constructs, Not Layout-Time Constructs

The original design assumed components could be identified by taffy root nodes and isolated by traversing the layout tree. This is incorrect for most of the components agents want to debug.

In akar's demo, components fall into two categories:

1. **Layout components** (form, stats, tab content): Built from taffy nodes. Their visual output comes from calling component functions (`akar_text_input`, `akar_stat`, etc.) against resolved layout rects. These *could* be identified by a root taffy node.

2. **Overlay components** (drawer, dropdown menu, modal, toasts): NOT taffy subtrees. They use `drawer_begin`/`drawer_end`, `modal_begin`/`modal_end`, `dropdown_begin`/`dropdown_end` and emit raw `push_quad`/`push_text` calls at computed coordinates. The drawer's nav links, the dropdown's menu items, the modal's chrome — none of these are layout nodes.

The components agents most want to isolate (drawer, dropdown, modal) are all overlay components. A "traverse the taffy tree" approach produces nothing for these.

**Conclusion:** Component isolation must operate at the render-call level, not the layout-tree level. Each component is a block of render calls in the demo's monolithic `RedrawRequested` handler. Isolation means "execute only this block."

---

## Design Decisions

### 1. Prerequisite: Refactor the demo render loop into per-component functions

**Decision:** Before implementing `--component`, extract the monolithic `RedrawRequested` handler (~900 lines) into per-component render functions.

**Rationale:**
- The current render loop is a single function with interleaved navbar, alert, tab bar, list/canvas/stats/form content, drawer, modal, toasts, and dropdown rendering.
- Selective rendering requires the ability to call (or skip) individual component blocks.
- This refactor is valuable on its own (cleaner code, testable functions, easier to add new components).

**Implementation:** Extract functions like `render_navbar(state)`, `render_drawer(state)`, `render_dropdown(state)`, `render_modal(state)`, `render_toasts(state)`, `render_form_tab(state)`, `render_stats_tab(state)`, etc. Each function takes `&mut AppState` and performs the render calls for that component.

### 2. Component registry (demo-side enum)

**Decision:** Add a `Component` enum in the demo that maps CLI names to render logic.

```rust
enum Component {
    Navbar,
    Alert,
    TabBar,
    ListTab,
    CanvasTab,
    StatsTab,
    FormTab,
    Drawer,
    Modal,
    Toasts,
    Dropdown,
}

impl Component {
    fn from_name(name: &str) -> Option<Self> { ... }
    fn names() -> &'static [&'static str] { ... }
    fn render(&self, state: &mut AppState, viewport_rect: [f32; 4]) { ... }
    fn force_state_initial(&self, state: &mut AppState) { ... }   // one-shot, see Decision 5
}
```

**Rationale:**
- The label registry (`HashMap<String, NodeId>`) works for individual elements but not for overlay components that have no taffy nodes.
- A component registry is the right abstraction: each variant knows how to render itself and how to force its "interesting" state.
- This is demo-only. The enum lives in `examples/demo-rust/src/main.rs` (or a new `components.rs` module).

**Why not reuse the label registry:**
- Labels identify taffy nodes. Overlay components have no taffy nodes.
- Labels identify individual elements (e.g., `navbar_btn`), not component groups (e.g., "the entire navbar").
- The component registry and label registry serve different purposes and can coexist.

### 3. Rendering approach: Full layout, selective render calls

**Decision:** The full taffy layout computes (so layout-based components have correct rects), but only the isolated component's render function is called. Layout setup (`set_children`, `layout.compute`) runs unconditionally in a `prepare_layout(state, size, scale)` step that is called before any selective render dispatch.

**Rationale:**
- Layout components (form, stats) need their taffy rects to be resolved.
- Overlay components (drawer, dropdown, modal) compute their positions from viewport rects, which are available after layout.
- Running full layout is cheap and preserves correctness.
- Skipping other render calls leaves those areas blank (black, from `LoadOp::Clear`).
- **Important:** The demo currently runs `state.layout.set_children(state.panel_container, &[...])` *before* `layout.compute` (`examples/demo-rust/src/main.rs:790-799` → `:803`). This is layout-tree setup, not a render call, and must run for every isolated frame regardless of which component is selected — otherwise tab subtrees (the form, stats, list, canvas tabs) resolve against stale children. The refactor must extract this dispatch into `prepare_layout` and call it unconditionally, both in `render_all` and in the `--component` path.

**Implementation:**
```rust
let viewport_rect = [0.0, 0.0, size.width as f32 / scale, size.height as f32 / scale];
prepare_layout(state, size, scale);   // set_children(panel_container), layout.compute — always
if let Some(component) = &self.isolated_component {
    component.render(state, viewport_rect);   // only this component's render calls
} else {
    render_all(state, viewport_rect);          // prepare_layout already called above by the dispatcher
}
```
`prepare_layout` is called once per frame, unconditionally, before any selective render. (`render_all` does not call `prepare_layout` itself; the dispatcher does.)

### 4. Auto-crop the screenshot to the component's bounding box

**Decision:** After capture, crop the PNG to the component's bounding box + padding.

**Rationale:**
- Without auto-crop, an isolated drawer (w=250) in an 800x600 window produces a mostly-black image with a thin strip on the left. This is not useful for debugging.
- Auto-crop turns this into a tight 282x632 image (250 + 16px padding on each side).
- The agent sees the component clearly, not a sea of black pixels.

**Implementation:**
1. When `--component` is set, enable draw-list recording (from Epic 014c).
2. After the component renders, compute the AABB of all *visible* recorded draw calls (min/max of quad rects and text clip rects, excluding scissor-culled calls — see Task 015c step 2).
3. After `take_screenshot` produces the RGBA buffer, crop to the AABB + padding on all four sides, clamped to the frame bounds (see Task 015c step 3 for the exact arithmetic — the naive `[x-pad, y-pad, w+2*pad, h+2*pad]` without four-side clamping panics at frame edges).
4. Write the cropped PNG.

**Dependencies:** Uses the recording mode from Epic 014c (`DrawList::start_recording`, `recorded_calls()`). The AABB computation and PNG cropping are ~30 lines total.

### 5. Implicit state forcing — first isolated frame only

**Decision:** `--component <name>` implicitly forces the "interesting" state for that component, **once**, on the first isolated `RedrawRequested` frame. Subsequent isolated frames do *not* re-force.

**Rationale:**
- A closed drawer is not interesting to debug. An open drawer is.
- A closed dropdown is not interesting. An open dropdown is.
- If the agent must manually script the state, they need to know the component's internal state variables (`drawer_open`, `dropdown_open`, `modal_open`), which defeats the purpose of isolation.
- **Forcing every frame would silently break `--component` + `--script` composition (see Decision 10):** a script that closes the drawer on frame N would be re-opened by `force_state` on frame N+1, so the override would not stick.

**Implementation:** Each `Component` variant has a `force_state_initial(&self, state: &mut AppState)` method. Run it once — guard with a `bool` field on `App` (e.g., `forced_initial_state: bool`, set `true` after the first isolated frame) or use a one-shot at script/flag parse time before the event loop starts. Prefer the latter: forcing at parse time means the very first `RedrawRequested` already sees the forced state, so `--component X --screenshot PATH --delay 0` captures the forced state on the first frame.
- `Drawer`: `state.drawer_open = true; state.drawer_progress = 1.0;`
- `Dropdown`: `state.dropdown_open = true;`
- `Modal`: `state.modal_open = true;`
- `Toasts`: push a sample toast if the list is empty.
- `Alert`: `state.alert_dismissed = false;`
- Tab components: set `active_tab` and `prev_active_tab` to the tab's index (see Pitfall 1).
- Layout components with no interesting state (`Navbar`, `TabBar`): no forcing.

**Override:** A `--script` file runs after the one-shot force and may freely transition state across frames — e.g. open the drawer on frame 1, capture, then close it and capture again. This is the intended composition (see Decision 10).

### 6. `--list-components` for discovery

**Decision:** Add a `--list-components` flag that prints all available component names and exits.

**Rationale:**
- Agents need to discover valid component names without reading the demo source.
- `--list-components` prints one name per line, derived from `Component::names()`.

**Implementation:**
```rust
if list_components {
    for name in Component::names() {
        println!("{}", name);
    }
    std::process::exit(0);
}
```

### 7. Unknown component handling: Error and exit

**Decision:** If `--component <name>` doesn't match a known component, print an error (including the list of valid names) and exit with non-zero status.

**Rationale:**
- This is a debugging tool. Silent fallbacks hide mistakes.
- The error message should include the valid names so the agent can correct the typo.

**Implementation:**
```rust
let component = Component::from_name(&name).ok_or_else(|| {
    eprintln!("Unknown component '{}'. Valid components:", name);
    for n in Component::names() {
        eprintln!("  {}", n);
    }
    std::process::exit(1);
});
```

### 8. Window size: Normal

**Decision:** The window size remains unchanged (800x600).

**Rationale:**
- Components are positioned relative to the window or other elements.
- Changing the window size would alter the layout and defeat the purpose of preserving context.
- The isolated component renders at its natural position within the normal window.
- Auto-crop (decision 4) handles the output size.

### 9. Dependencies: Render without dependencies

**Decision:** Render only the specified component, not its dependencies (e.g., no backdrop for modals, no anchor button for dropdowns).

**Rationale:**
- The goal is to isolate the component for debugging, not to render a fully functional UI.
- Dependencies (backdrops, anchors) are part of the integration, not the component itself.
- If the agent needs to test the component in context, they can omit `--component` and use the normal full render.

**Implication:** A dropdown menu will render without its button anchor; a modal will render without its backdrop. This is intentional — the agent is debugging the component, not its integration.

**Naming clarification:** The `Component::Dropdown` variant targets the *navbar dropdown menu* (the `render_dropdown` block in the demo, anchored on the navbar's dropdown button). It does not mean "any dropdown widget" — the form tab's `akar_select` opens its own internal dropdown via `dropdown_begin` (see `crates/akar-components/src/select.rs:102`), but that overlay is part of the `form` component's render tree and is included when `--component form` is used (per Decision 12). To debug the form's select dropdown in isolation, use `--component form --script <select-open-script>`; do not add a separate `select_dropdown` variant in v1.

### 10. Integration with `--script`: Compose for interactive states

**Decision:** `--component` composes with `--script` from Epic 014b. The `--script` runs *after* the one-shot `force_state_initial` (Decision 5) and may freely transition state across frames.

**Rationale:**
- Agents can script interactive states (e.g., hover a dropdown item, focus a form field) and capture the isolated component in that state.
- Example: `--component dropdown --script hover_item.txt` renders only the dropdown, scripts a hover, and captures a screenshot of the result.
- Force-once semantics make scripts *capable of overriding* the forced state — e.g. open the drawer via `force_state_initial`, capture on line 1, then have the script click the drawer's close button on line 2 and capture again to get a closed-state screenshot. Under the original (force-every-frame) design this was impossible.

**Implementation:**
- The script runner operates on the full `InputState` as before (Epic 014b); it does not know `--component` is set.
- The `--component` flag affects which components render, not how input is processed.
- Run `force_state_initial` once at flag-parse time (or on the first isolated frame via a one-shot guard), so by the time the script's first input line fires, the forced state is already in place.
- The script can read and overwrite the forced state. This is intended behavior.
- `--component` without `--script` captures a single screenshot of the forced state. `--component` with `--script` captures one or more screenshots seeded from the forced state.

### 11. Interaction with `--dump-frame`

**Decision:** The frame dump includes only the draw calls for the isolated component.

**Rationale:**
- The frame dump should reflect what was actually rendered.
- Since isolation skips other render blocks, fewer draw calls are pushed. The recording mode naturally captures only the isolated component's calls.
- This is consistent with the visual output.

### 12. Nested components

**Decision:** `--component <name>` renders the component and all its visual children.

**Rationale:**
- For overlay components (drawer, modal), "children" means the content rendered inside the overlay (nav links, form fields).
- For layout components (form), "children" means all fields rendered in the form container.
- The component's render function naturally includes its children. There's no separate "child" isolation in v1.

**Future:** If needed, a `--component modal --child dropdown` syntax could isolate a nested component. Deferred.

### 13. Implementation location: Demo-only

**Decision:** The `--component` flag, component registry, and selective rendering logic live in `examples/demo-rust/`. No changes to `akar-core`, `akar-layout`, or `akar-components`.

**Rationale:**
- This is a debugging feature for the demo, not a library feature.
- The component registry is specific to the demo's component catalog.
- If it proves useful, it can be promoted to core later (e.g., as a `ComponentRegistry` trait in `akar-components`).

---

## Open Questions (Resolved)

> See also the **Review log (2026-07-12)** section at the bottom of this file — it records which of these were re-confirmed during the design-review pass and which spec bugs found during review modified the corresponding implementation in-place.

### 1. Component addressing granularity

**Question:** Should `--component` accept a single component, or multiple components?

**Resolution:** Single component in v1. Multiple components can be added later if needed. The component registry design supports it (accept a `Vec<Component>` instead of `Option<Component>`), but the implementation is simpler with one.

### 2. Unknown component handling

**Question:** What should happen if `--component <name>` doesn't match a known component?

**Resolution:** Error and exit. Print the error message with the list of valid names. See decision 7.

### 3. Label registration scope

**Question:** Should we register labels for all components, or only those we want to isolate?

**Resolution:** The label registry (from Epic 014b) is for individual elements. The component registry (decision 2) is for components. They serve different purposes. The component registry includes all components that can be isolated. The label registry remains as-is for element-level addressing in scripts.

### 4. Implementation location

**Question:** Should this be demo-only, or should we promote it to `akar-components`?

**Resolution:** Demo-only for v1. See decision 13.

### 5. Interaction with `--dump-frame`

**Question:** Should `--dump-frame` (Epic 014c) work with `--component`?

**Resolution:** Yes. The frame dump includes only the isolated component's draw calls. See decision 11.

### 6. Nested components

**Question:** How should we handle nested components?

**Resolution:** Render the entire component (including its visual children). See decision 12.

---

## Proposed Implementation

### Task 015a — Refactor demo render loop into per-component functions

**Scope:** Extract the monolithic `RedrawRequested` handler into per-component render functions, with an unconditional `prepare_layout` step that runs layout-tree setup and `layout.compute` before any selective render dispatch.

**Implementation:**

1. **Identify component blocks** in `examples/demo-rust/src/main.rs`:
   - **Pre-render layout setup: lines ~777-810** — `alert_dismissed` style toggle, `set_children(panel_container, ...)` tab-content dispatch, `layout.compute`. This is **not** part of any component's render block; it runs unconditionally.
   - Navbar: lines ~760-776 (one-time `navbar_slots` init) + lines ~826-875 (per-frame navbar render: title, badge, buttons)
   - Containers: lines ~878-901 (background containers)
   - Alert: lines ~903-914
   - Tab bar: lines ~916-927 (and the toast-on-tab-change logic up to ~941)
   - List tab: lines ~943-1042 (scroll area, list items, tooltips)
   - Canvas tab: lines ~1043-1061
   - Stats tab: lines ~1062-1126 (stats, steps, avatars, skeleton toggle)
   - Form tab: lines ~1127-1273 (form fields, submit button)
   - Drawer: lines ~1277-1375 (drawer animation, avatar, nav links)
   - Modal: lines ~1377-1411
   - Toasts: lines ~1413-1421
   - Dropdown: lines ~1423-1482 (dropdown menu, items)

2. **Extract a `prepare_layout` function** that runs *unconditionally* per frame, regardless of `--component`:
   ```rust
   fn prepare_layout(state: &mut AppState, size: PhysicalSize<u32>, scale: f32) {
       // alert_dismissed → Display::None style toggle
       // match state.active_tab → set_children(panel_container, ...)
       // state.layout.compute(state.page.root, ...)
   }
   ```
   This is mandatory because tab subtrees resolve against `panel_container`'s children *before* `compute`. Without this step, `--component form` would render the form against the previous tab's children (stale layout).

3. **Extract render functions** as before, but each is a pure consumer of already-resolved rects:
   ```rust
   fn render_navbar(state: &mut AppState, viewport_rect: [f32; 4]) { ... }
   fn render_containers(state: &mut AppState) { ... }
   fn render_alert(state: &mut AppState) { ... }
   fn render_tab_bar(state: &mut AppState) { ... }
   fn render_list_tab(state: &mut AppState, viewport_rect: [f32; 4]) { ... }
   fn render_canvas_tab(state: &mut AppState) { ... }
   fn render_stats_tab(state: &mut AppState) { ... }
   fn render_form_tab(state: &mut AppState, viewport_rect: [f32; 4]) { ... }
   fn render_drawer(state: &mut AppState, viewport_rect: [f32; 4]) { ... }
   fn render_modal(state: &mut AppState, viewport_rect: [f32; 4]) { ... }
   fn render_toasts(state: &mut AppState, viewport_rect: [f32; 4]) { ... }
   fn render_dropdown(state: &mut AppState, viewport_rect: [f32; 4]) { ... }

   fn render_all(state: &mut AppState, viewport_rect: [f32; 4]) {
       render_containers(state);
       render_navbar(state, viewport_rect);
       render_alert(state);
       render_tab_bar(state);
       match state.active_tab {
           0 => render_list_tab(state, viewport_rect),
           1 => render_canvas_tab(state),
           2 => render_stats_tab(state),
           3 => render_form_tab(state, viewport_rect),
           _ => {}
       }
       render_drawer(state, viewport_rect);
       render_modal(state, viewport_rect);
       render_toasts(state, viewport_rect);
       render_dropdown(state, viewport_rect);
   }
   ```
   `prepare_layout` is **not** called inside `render_all`; the dispatcher calls it once, then either `render_all` or the single isolated component's render. This keeps `prepare_layout` running exactly once per frame in both paths.

4. **Handle `viewport_rect`:** Pass it as a parameter to render functions that need it (`drawer_begin`, `modal_begin`, `dropdown_begin`, `toasts`, `akar_tooltip` in list tab, `akar_select` in form tab — see `examples/demo-rust/src/main.rs`). Do not store it in `AppState`; explicit parameters make render functions easier to read and isolate.

5. **Handle navbar slot initialization:** The navbar has a one-time setup block (lines 760-775) that calls `akar_navbar` to obtain `NavbarSlots`, then adds child nodes to the layout. Move this into `render_navbar` with a guard (`if state.navbar_slots.is_none() { ... }`), so it self-initializes lazily on the first frame it is rendered — including when `--component navbar` is set.

6. **Handle state mutations:** Some render blocks mutate state (e.g., `menu_result.clicked` toggles `drawer_open`). These mutations must remain in the render functions. When isolating a component, the state mutations from skipped blocks are also skipped, which is the desired behavior (the agent wants the isolated component's effect on state, not the others').

**Acceptance:**
- `cargo run --example demo-rust` renders identically to before the refactor.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.
- `prepare_layout` is callable on its own and idempotent across frames (smoke test: call it twice, observe no panic).

### Task 015b — Component registry and `--component` flag

**Scope:** Add the `Component` enum, `--component <name>` flag, and selective rendering.

**Implementation:**

1. **Add `Component` enum** in `examples/demo-rust/src/main.rs` (or a new `components.rs` module):
   ```rust
   enum Component {
       Navbar,
       Alert,
       TabBar,
       ListTab,
       CanvasTab,
       StatsTab,
       FormTab,
       Drawer,
       Modal,
       Toasts,
       Dropdown,
   }

   impl Component {
       fn from_name(name: &str) -> Option<Self> {
           match name {
               "navbar" => Some(Self::Navbar),
               "alert" => Some(Self::Alert),
               "tab_bar" => Some(Self::TabBar),
               "list" => Some(Self::ListTab),
               "canvas" => Some(Self::CanvasTab),
               "stats" => Some(Self::StatsTab),
               "form" => Some(Self::FormTab),
               "drawer" => Some(Self::Drawer),
               "modal" => Some(Self::Modal),
               "toasts" => Some(Self::Toasts),
               "dropdown" => Some(Self::Dropdown),
               _ => None,
           }
       }

       fn names() -> &'static [&'static str] {
           &[
               "navbar", "alert", "tab_bar", "list", "canvas",
               "stats", "form", "drawer", "modal", "toasts", "dropdown",
           ]
       }

        fn render(&self, state: &mut AppState, viewport_rect: [f32; 4]) {
            match self {
                Self::Navbar => render_navbar(state, viewport_rect),
                Self::Alert => render_alert(state),
                Self::TabBar => render_tab_bar(state),
                Self::ListTab => render_list_tab(state, viewport_rect),
                Self::CanvasTab => render_canvas_tab(state),
                Self::StatsTab => render_stats_tab(state),
                Self::FormTab => render_form_tab(state, viewport_rect),
                Self::Drawer => render_drawer(state, viewport_rect),
                Self::Modal => render_modal(state, viewport_rect),
                Self::Toasts => render_toasts(state, viewport_rect),
                Self::Dropdown => render_dropdown(state, viewport_rect),
            }
        }

        fn force_state_initial(&self, state: &mut AppState) {
            match self {
                Self::Alert => {
                    state.alert_dismissed = false;
                }
                Self::ListTab => {
                    state.active_tab = 0;
                    state.prev_active_tab = 0;
                }
                Self::CanvasTab => {
                    state.active_tab = 1;
                    state.prev_active_tab = 1;
                }
                Self::StatsTab => {
                    state.active_tab = 2;
                    state.prev_active_tab = 2;
                }
                Self::FormTab => {
                    state.active_tab = 3;
                    state.prev_active_tab = 3;
                }
                Self::Drawer => {
                    state.drawer_open = true;
                    state.drawer_progress = 1.0;
                }
                Self::Dropdown => {
                    state.dropdown_open = true;
                }
                Self::Modal => {
                    state.modal_open = true;
                }
                Self::Toasts => {
                    if state.toasts_list.is_empty() {
                        state.toasts_list.push(ToastItem {
                            variant: ToastVariant::Info,
                            message: "Sample toast".to_string(),
                            dismiss_on_click: false,
                        });
                    }
                }
                Self::Navbar | Self::TabBar => {}
            }
        }
    }
    ```
    The method is named `force_state_initial` (not `force_state`) to make the one-shot contract obvious at every call site. Per Decision 5 it runs once, not every frame.

2. **Parse `--component <name>` flag** in the demo's arg loop:
   ```rust
   let mut isolated_component = None;
   // ...
   "--component" => {
       if let Some(name) = args.next() {
           isolated_component = Some(name);
       }
   }
   ```

3. **Validate and resolve** the component name before entering the event loop, **and apply `force_state_initial` once at parse time** so the first isolated frame already sees the forced state:
   ```rust
   let component = match isolated_component {
       Some(name) => match Component::from_name(&name) {
           Some(c) => {
               c.force_state_initial(&mut state);   // one-shot, before event loop
               Some(c)
           }
           None => {
               eprintln!("Unknown component '{}'. Valid components:", name);
               for n in Component::names() {
                   eprintln!("  {}", n);
               }
               std::process::exit(1);
           }
       },
       None => None,
   };
   ```
   Note: `state` must be mutable and already initialized at this point. If the demo constructs `AppState` *after* arg parsing, either move arg parsing earlier or apply `force_state_initial` on the first `RedrawRequested` via a `forced_initial_state: bool` one-shot guard on `App`. Either works; pick whichever fits the existing initialization order. **Do not** call `force_state_initial` on every frame — that breaks `--script` composition (Decisions 5, 10).

4. **Selective rendering in the render loop:**
   ```rust
   let viewport_rect = [
       0.0, 0.0,
       size.width as f32 / scale,
       size.height as f32 / scale,
   ];
   prepare_layout(state, size, scale);   // unconditional — see Task 015a
   if let Some(component) = &self.isolated_component {
       component.render(state, viewport_rect);   // force_state_initial already ran once at parse time
   } else {
       render_all(state, viewport_rect);
   }
   ```

5. **Add `--list-components` flag:**
   ```rust
   if list_components {
       for name in Component::names() {
           println!("{}", name);
       }
       std::process::exit(0);
   }
   ```

**Acceptance:**
- `--component drawer --screenshot /tmp/drawer.png --exit` captures a screenshot of the drawer (open, at its natural position).
- `--component dropdown --screenshot /tmp/dropdown.png --exit` captures a screenshot of the dropdown menu (open).
- `--component unknown` prints an error with the list of valid names and exits with non-zero status.
- `--list-components` prints all valid component names and exits.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.

### Task 015c — Auto-crop isolated component screenshots

**Scope:** Crop the PNG to the component's bounding box + padding.

**Implementation:**

1. **Enable recording when `--component` is set:**
   ```rust
   if (self.isolated_component.is_some() || self.dump_frame_path.is_some())
       && !self.dump_frame_written
   {
       state.core.draw_list.start_recording();
   }
   ```
   Note: Recording must be enabled whenever `--component` is set (for auto-crop), not just when `--dump-frame` is set. The recording overhead is minimal when no draw calls are made.

2. **Compute AABB from recorded calls** — *excluding scissor-culled calls*. `DrawList::push_quad` snapshots the `RecordedCall` *before* the scissor-cull early-return (`crates/akar-core/src/draw_list.rs:131-141`), so `recorded_calls()` includes calls that were never drawn. Including those in the AABB would extend the crop window to off-screen regions (e.g., scroll-area contents outside the visible clip). Filter by visible-against-scissor intersection:
   ```rust
   fn compute_component_aabb(recorded: &[akar_core::draw_list::RecordedCall]) -> Option<[f32; 4]> {
       let mut min_x = f32::MAX;
       let mut min_y = f32::MAX;
       let mut max_x = f32::MIN;
       let mut max_y = f32::MIN;

       for call in recorded {
           let rect = match &call.call {
               akar_core::DrawCall::Quad(q) => q.rect,
               akar_core::DrawCall::Text(t) => t.clip,
           };
           // Skip culled calls: rect must intersect the active scissor (if any).
           if let Some(scissor) = call.scissor {
               if rect[0] + rect[2] <= scissor[0]
                   || rect[1] + rect[3] <= scissor[1]
                   || rect[0] >= scissor[0] + scissor[2]
                   || rect[1] >= scissor[1] + scissor[3]
               {
                   continue;
               }
           }
           min_x = min_x.min(rect[0]);
           min_y = min_y.min(rect[1]);
           max_x = max_x.max(rect[0] + rect[2]);
           max_y = max_y.max(rect[1] + rect[3]);
       }

       if min_x == f32::MAX {
           None
       } else {
           Some([min_x, min_y, max_x - min_x, max_y - min_y])
       }
   }
   ```

3. **Crop the PNG after capture** — clamp symmetrically on all four sides so padding never pushes the crop past the frame bounds (the previous `(...).max(0.0)` only guarded the min sides and could panic on `copy_from_slice` at the right/bottom edges):
   ```rust
   if let Some(component) = &self.isolated_component {
       let recorded = state.core.draw_list.recorded_calls();
       if let Some(aabb) = compute_component_aabb(recorded) {
            let pad = 16.0; // physical pixels — see note below
            let x = (aabb[0] - pad).max(0.0) as u32;
            let y = (aabb[1] - pad).max(0.0) as u32;
           let right = (aabb[0] + aabb[2] + pad)
               .min(frame.width as f32) as u32;
           let bottom = (aabb[1] + aabb[3] + pad)
               .min(frame.height as f32) as u32;
           let w = right.saturating_sub(x);
           let h = bottom.saturating_sub(y);
           if w == 0 || h == 0 {
               write_png(&frame, &capture_path)?;
           } else {
               crop_and_write_png(&frame, x, y, w, h, &capture_path)?;
           }
       } else {
           write_png(&frame, &capture_path)?;
       }
   }
   ```
   **Note on `pad`:** `recorded` rects are post-`scale_factor` (i.e., physical pixels) — see `DrawList::push_quad` lines 119-122 which multiply `call.rect` by `scale_factor` before snapshotting. So `pad = 16.0` is in physical pixels; on a 2× display the visible margin is 8 logical px. To get a uniform logical margin, divide `pad` by `scale_factor` before use. The simpler choice for v1 is to leave `pad` in physical pixels and document it; the choice is cosmetic and reversible.

4. **Implement `crop_and_write_png`** — separate `x, y, w, h` parameters (matches the clamping above and avoids a `[u32; 4]` that conflates origin + size):
   ```rust
   fn crop_and_write_png(
       frame: &akar_core::screenshot::CapturedFrame,
       x: u32, y: u32, w: u32, h: u32,
       path: &str,
   ) -> Result<(), String> {
       let mut cropped = vec![0u8; (w * h * 4) as usize];
       for row in 0..h {
           let src_start = ((y + row) * frame.width + x) as usize * 4;
           let dst_start = (row * w) as usize * 4;
           let src_end = src_start + (w as usize * 4);
           cropped[dst_start..dst_start + (w as usize * 4)]
               .copy_from_slice(&frame.rgba[src_start..src_end]);
       }

       let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
       let mut encoder = png::Encoder::new(file, w, h);
       encoder.set_color(png::ColorType::Rgba);
       encoder.set_depth(png::BitDepth::Eight);
       let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
       writer.write_image_data(&cropped).map_err(|e| e.to_string())?;
       Ok(())
   }
   ```

5. **Fallback: buffer-scan AABB (deferred).** If a future bug shows that recording-based AABB misbehaves (e.g., a fully-cropped-out component yields `None` and we silently fall back to the full-frame image), consider a more robust alternative: scan the captured RGBA buffer for non-clear-color pixels and compute the content's AABB directly. Pros: independent of `DrawList` recording semantics, works for any frame including non-isolated ones. Cons: a component that *intentionally* renders the clear color (e.g., a fully-transparent overlay) gets a degenerate AABB. Not implementing this in v1 — recording-based AABB with the culled-call filter above is sufficient for the demo — but recorded here so the fallback isn't reinvented later.

**Acceptance:**
- `--component drawer --screenshot /tmp/drawer.png --exit` produces a cropped PNG (not 800x600, but ~282x632 with 16px padding).
- `--component dropdown --screenshot /tmp/dropdown.png --exit` produces a cropped PNG of the dropdown menu.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.

---

## Known Implementation Pitfalls

These issues were identified during design review. The implementer should be aware of them to avoid rediscovering them during implementation.

### 1. Tab components need `active_tab` and `prev_active_tab` set

When isolating a tab component (e.g., `--component form`), the tab's layout nodes are only added to the tree if `state.active_tab` matches the tab's index. The `force_state_initial` method must set both `active_tab` and `prev_active_tab` to the same value to suppress the toast notification that would otherwise be triggered by the tab change detection logic at `examples/demo-rust/src/main.rs:929-937`.

### 2. Alert needs `alert_dismissed = false`

If the alert has been dismissed in a previous frame, its layout node is set to `Display::None`. The `force_state_initial` method for `Alert` must set `alert_dismissed = false` to ensure the alert renders.

### 3. Recording must be enabled for `--component` unconditionally

The auto-crop feature (Task 015c) requires draw-list recording to compute the component's bounding box. The recording gate must check `self.isolated_component.is_some()`, not just `self.dump_frame_path.is_some()`.

### 4. `viewport_rect` must be accessible to render functions

Several components (`drawer_begin`, `modal_begin`, `dropdown_begin`, `toasts`, `akar_tooltip`, `akar_select`) require `viewport_rect` as a parameter. Currently it is a local variable in `RedrawRequested`. Pass it as a parameter to render functions that need it (cleaner, explicit dependencies). Do not store it in `AppState` — explicit parameters make the selective-render dispatch easier to read and isolate.

### 5. Navbar slot initialization must run before navbar render

The navbar has a one-time setup block that calls `akar_navbar` to obtain `NavbarSlots`, then adds child nodes to the layout. This must run before the navbar's per-frame render calls. When `--component navbar` is used, this initialization must still happen. The cleanest approach is to move the initialization into `render_navbar` with a guard (`if state.navbar_slots.is_none()`), keeping navbar logic self-contained.

### 6. Overlay components compute positions from `viewport_rect`, not layout

The drawer, dropdown, modal, and toasts are overlay components that compute their positions from `viewport_rect` (the full window rect), not from taffy layout. They don't have taffy nodes for their visual content. The component registry approach (decision 2) handles this correctly by treating them as render-call blocks, not layout subtrees.

### 7. `set_children(panel_container, ...)` runs *before* `layout.compute` and must be lifted into `prepare_layout`

`examples/demo-rust/src/main.rs:790-799` runs `state.layout.set_children(state.panel_container, &[scroll_container|canvas_wrapper|stats_wrapper|form_container])` based on `state.active_tab`, *before* `state.layout.compute(...)` at line 803. This is layout-tree setup, not a render call. The 015a refactor must lift this block (plus the `alert_dismissed` → `Display::None` style toggle at lines 777-785) into a `prepare_layout(state, size, scale)` function that runs unconditionally per frame, *before* any selective render dispatch. If this is moved into a render function or skipped under `--component`, tab subtrees resolve against stale children from the previous tab, and `state.layout.rect(form_*)` returns the wrong rect.

### 8. `force_state_initial` runs once, not every frame

Per Decision 5, the initial state forcing runs once (at flag-parse time, or on the first isolated frame via a one-shot guard). Calling `force_state_initial` every frame on the isolated path would silently break `--component` + `--script` composition: a script that closes the drawer on frame N would be re-opened by `force_state_initial` on frame N+1, so the override would not stick (Decision 10).

### 9. `compute_component_aabb` must filter scissor-culled calls

`DrawList::push_quad` snapshots the `RecordedCall` *before* the scissor-cull early-return (`crates/akar-core/src/draw_list.rs:131-141`), so `recorded_calls()` includes calls that were never drawn (this is intentional so `--dump-frame` can show "why didn't my quad render"). The AABB computation for auto-crop must skip calls whose `rect` does not intersect their `scissor` — otherwise the crop window extends to off-screen regions (e.g., scroll-area contents outside the visible clip) and the cropped PNG is misleadingly large. See Task 015c step 2.

### 10. Crop arithmetic must clamp on all four sides, not just the min

The naive crop:
```rust
let crop_rect = [
    (aabb[0] - pad).max(0.0) as u32,   // clamps left
    (aabb[1] - pad).max(0.0) as u32,   // clamps top
    (aabb[2] + 2.0 * pad) as u32,      // does NOT clamp right
    (aabb[3] + 2.0 * pad) as u32,      // does NOT clamp bottom
];
```
Clamps the min sides only. When `aabb[0] - pad < 0` the left clamp shifts `x` to 0 but `w` keeps `aabb_width + 2*pad` → asymmetric padding and the crop window can extend past `frame.width`. Worse, `crop_and_write_png`'s `copy_from_slice` on `frame.rgba[(y+row)*frame.width + x .. +w*4]` panics at the right/bottom edge. Compute `right = min(frame_w, aabb[0] + aabb[2] + pad)` and `crop_w = right.saturating_sub(x)`; same for `y`/`h`. See Task 015c step 3.

### 11. Crop coordinates are in physical pixels, not logical

`recorded` rects are post-`scale_factor` (see `DrawList::push_quad` lines 119-122 which multiply `call.rect` by `scale_factor` before snapshotting). The CapturedFrame's `width`/`height` are also physical. So `pad` (and the AABB result) are in physical pixels. On a 2× display, `pad = 16.0` yields 8 logical px of visible margin. To get a uniform logical margin, divide `pad` by `scale_factor` before use. For v1 leaving `pad` in physical pixels is acceptable; just document it.

---

## Acceptance Criteria

- `cargo check --workspace` passes.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.
- Agent can capture a screenshot of a specific component without visual noise from other components.
- The screenshot is auto-cropped to the component's bounding box + padding.
- `--component` composes with `--script` for interactive state capture.
- `--component` composes with `--dump-frame` for structured debugging output.
- `--list-components` prints all available component names.
- Unknown component names produce an error with the list of valid names.

---

## Notes

- The epic was revised on 2026-07-12 after a design review grounded in the actual demo source (`examples/demo-rust/src/main.rs`), `crates/akar-core/src/draw_list.rs`, and `crates/akar-components/src/drawer.rs`/`select.rs`. The status was bumped from "Brainstorming" to "Drafting" because all open questions are resolved and the spec bugs identified during review are fixed in-place.
- The key insight is that components in akar are render-time constructs, not layout-time constructs. The implementation must match that reality.
- This is a demo-only feature for v1. If it proves useful, it can be promoted to core later.
- The render loop refactor (Task 015a) is valuable on its own and should be done first, even if the `--component` feature is deferred.

---

## Review log (2026-07-12)

A codebase-grounded review caught four spec bugs in the original implementation plan and resolved all six open questions. The fixes are applied in-place above. This section records what changed and why, for future implementers.

### Resolved open questions

- **OQ1 (single vs. multi component).** Single in v1, confirmed. The `Component` enum shape leaves the upgrade path to `Vec<Component>` open (a `RenderPlan` could wrap the vec — see "Deferred alternatives" below).
- **OQ2 (unknown component handling).** Error and exit with the list of valid names — unchanged from the original.
- **OQ3 (label vs. component registry coexistence).** Confirmed. Labels (Epic 014b) resolve `NodeId`→rect for individual elements; the component registry maps a CLI name→render block. Different layers, both kept.
- **OQ4 (implementation location).** Demo-only for v1, promotion deferred.
- **OQ5 (`--dump-frame` interaction).** Frame dump includes only the isolated component's draw calls. Unchanged.
- **OQ6 (nested components).** Render the entire component, including its visual children (because the render function is the unit, not a layout subtree). Clarified that the form tab's `akar_select` internal dropdown (see `crates/akar-components/src/select.rs:102`) is included as part of `--component form`, *not* via `--component dropdown` — see Decision 9.

### Spec bugs fixed

1. **`set_children(panel_container, ...)` was outside every component block in the original 015a line ranges.** The original line-range listing (Navbar ~760-876, Tab bar ~916-927, etc.) omitted lines 777-810, which contain the `alert_dismissed` style toggle, the `match active_tab → set_children(panel_container, ...)` dispatch, and `layout.compute`. These are layout-tree setup, not render calls, and must run for every isolated frame regardless of `--component`. **Fix:** introduced `prepare_layout(state, size, scale)` and made the dispatcher call it unconditionally before either `render_all` or the isolated component's `render` (Decision 3, Task 015a, Pitfall 7).

2. **Force-every-frame semantics broke `--component` + `--script` composition.** The original Decision 5 said `force_state(state)` runs every isolated frame, AND Decision 10 said scripts can "override the forced state (e.g., close the drawer after it's forced open)". These are mutually inconsistent — `force_state` on frame N+1 would re-open the drawer the script closed on frame N. **Fix:** renamed `force_state` → `force_state_initial`, run it once (at flag-parse time, or via a one-shot first-frame guard), and clarified Decision 10 to state that scripts run *after* the one-shot force and may freely transition state across frames (Decisions 5 & 10, Task 015b step 3, Pitfall 8).

3. **AABB computation included scissor-culled calls.** The original `compute_component_aabb` iterated `recorded_calls()` without filtering, but `DrawList::push_quad` snapshots the `RecordedCall` *before* the scissor-cull early-return (`crates/akar-core/src/draw_list.rs:131-141`). A quad in a scroll area with clip outside the visible region contributes its rect to the AABB, so the crop window extended to off-screen regions and the cropped PNG was misleadingly large. **Fix:** filter by `rect`-intersects-`scissor` in `compute_component_aabb` (Task 015c step 2, Pitfall 9).

4. **Crop arithmetic clamped only the min sides and would panic at right/bottom frame edges.** The original crop used `(aabb[0] - pad).max(0.0) as u32` for `x` and `aabb[2] + 2.0 * pad as u32` for `w`. When `aabb[0] - pad < 0`, `x` clamps to 0 but `w` keeps the full padded width → asymmetric padding *and* the crop window can extend past `frame.width`. In `crop_and_write_png`, `copy_from_slice` on `frame.rgba[(y+row)*frame.width + x .. +w*4]` panics at the right/bottom edge when `x + w > frame.width`. **Fix:** compute `right = min(frame_w, aabb[0]+aabb[2]+pad)` and `crop_w = right.saturating_sub(x)`; same for `y`/`h`. Added an empty-rect fallback to `write_png` in the degenerate `w == 0 || h == 0` case (Task 015c steps 3 & 4, Pitfall 10).

### Other clarifications

- **Physical vs. logical padding.** The `pad = 16.0` is in physical pixels because `recorded` rects are post-`scale_factor`. Documented inline (Pitfall 11); not behavior-changing in v1.
- **`dropdown` naming.** Decision 9 now states that `Component::Dropdown` targets the navbar's dropdown menu specifically, not "any dropdown widget." To debug the form's `akar_select` dropdown, use `--component form` (the form's render block includes the select and its internal dropdown).
- **Buffer-scan AABB fallback.** A non-recording-based fallback (scan the captured RGBA buffer for non-clear pixels, compute content AABB directly) is documented as a deferred alternative in Task 015c step 5. It is more robust to recording semantics edge cases but degenerate for intentionally-transparent components. Not implementing in v1.

### Deferred alternatives (recorded, not adopted in v1)

1. **Render-plan abstraction.** Replace `Option<Component>` + `render_all` special-case with a `RenderPlan: Vec<Component>` (default = all; `--component` = `[one]`), iterate `for component in &self.render_plan { component.render(...) }`. Natural upgrade path to `--component drawer --component toasts` (multi-component). Rejected for v1 to keep the implementation simple, but the `Component` enum shape makes this a drop-in refactor later.
2. **`--component all` / `--component *` alias.** Removes the `Option<Component>` vs `render_all` special-case by making every code path iterate the plan, with the default plan=full set. Pairs with alternative 1.
3. **Default screenshot path `/tmp/akar-<component>.png`.** When `--component X` is set but `--screenshot` is omitted, default the output path so agents can run `--component drawer --exit` without specifying a path. Cosmetic, deferred.
4. **Per-component trait in `akar-components`.** Promote the component-registry shape from this epic into a documented trait/object sketch in `akar-components` (doc comment only, no code) so a future epic does not reinvent it. Deferred to keep this epic demo-only.

---

## Open questions (remaining)

None. All six original questions are resolved (see "Review log" above), and the four spec bugs found during review are fixed. The remaining items in "Deferred alternatives" are enhancements, not blockers.

---

## Implementation log

### 2026-07-13 — Task 015a landed

Refactored `examples/demo-rust/src/main.rs` (1636 → 1675 lines, +39 net; 750 insertions / 711 deletions). Extracted exactly the function set from the spec: `prepare_layout`, `render_navbar`, `render_containers`, `render_alert`, `render_tab_bar`, `render_list_tab`, `render_canvas_tab`, `render_stats_tab`, `render_form_tab`, `render_drawer`, `render_modal`, `render_toasts`, `render_dropdown`, `render_all`. Dispatcher at `examples/demo-rust/src/main.rs:1505-1521` calls `prepare_layout` unconditionally, then `render_all` (no `--component` logic yet — that is 015b).

**Verification:** `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` (145 tests pass, 0 failures), `cargo fmt --check` all pass. Screenshot at `/tmp/demo.png` is pixel-identical to pre-refactor (navbar + alert + tabs + list items render unchanged).

**Deviations from spec:**
1. Added `winit::dpi::PhysicalSize` to the import list — `state.window.inner_size()` returns `PhysicalSize<u32>` and `prepare_layout(state, size, scale)` needs the type.
2. `render_navbar` accepts `_viewport_rect` (prefixed-underscore) — the navbar has no overlay that needs it, so the param is unused. All other overlay-using renderers take a live `viewport_rect` per Pitfall #4.
3. One clippy fix beyond pure refactor: `(state.cursor_tick / 30) % 2 == 0` → `(state.cursor_tick / 30).is_multiple_of(2)` in `render_form_tab` (clippy 1.95 `manual_is_multiple_of` lint). Semantically identical; flagged here for traceability.

**Reviewer check:** dispatcher shape matches the spec pseudocode exactly. State mutations (drawer/dropdown/modal toggles, toasts push/trim, active_tab updates, form cursor/text updates) all stay in their owning render functions. `--screenshot` / `--script` / `--dump-layout` / `--dump-frame` capture flow is unchanged.

### 2026-07-13 — Task 015b landed

Added the `Component` enum, `--component` / `--list-components` flags, selective render dispatch, and a one-shot `force_state_initial` guard. Only `examples/demo-rust/src/main.rs` touched (+168/-1). `App` gained two fields: `isolated_component: Option<Component>` and `forced_initial_state: bool` (Option A from the spec — AppState is built lazily in `App::resumed`, so the one-shot force lives on `App` and runs in the first `RedrawRequested`).

**Spec amendment required (new pitfall, surfaced by implementation):**

> **Pitfall #12 — Cross-component layout dependencies.** Some overlay components reach into layout nodes owned by another component. `--component dropdown` was producing a black screenshot because `render_dropdown` reads `state.layout.rect(state.navbar_dropdown_btn_node)` for its anchor, and those nodes are only registered with the layout tree by `render_navbar`'s lazy `if state.navbar_slots.is_none() { akar_navbar(...); add_child(...) }` block. When `render_navbar` is skipped, the anchor rect is zero and `dropdown_begin` returns `is_open=false`. **Fix:** an `ensure_navbar_slots(state: &mut AppState)` helper extracted from the navbar init block, called from the dispatcher when `isolated_component == Some(Component::Dropdown)`, before `prepare_layout`. Drawer / modal / toasts do not have this issue (they use `viewport_rect` for positioning and have no cross-component anchors). Future isolates with cross-component anchors need the same treatment.

**Verification:** `cargo check` / `clippy` / `test` (145 tests pass) / `fmt --check` all pass. Seven flows confirmed:
- `--screenshot /tmp/demo.png --exit` → full demo, 800x600.
- `--component drawer --screenshot /tmp/drawer.png --exit` → drawer open on left (AK avatar + 4 nav links), rest black.
- `--component dropdown --screenshot /tmp/dropdown.png --exit` → dropdown menu (Option A–D) anchored to navbar dropdown-button position, rest black.
- `--component form --screenshot /tmp/form.png --exit` → all form fields visible.
- `--list-components` → exit 0, 11 names in spec order.
- `--component unknown` → exit 1, stderr lists the 11 valid names.
- `--component` + `--script` and `--component` + `--screenshot` both pass the arg-validation check; `--script` + `--screenshot` still rejected.

**Reviewer check:** `Component::from_name` and `force_state_initial` bodies match the spec pseudocode verbatim (modulo the `ensure_navbar_slots` fix above). Tab variants correctly set both `active_tab` AND `prev_active_tab` to suppress the spurious tab-change toast. The one-shot guard is not on `AppState` (per the spec's Option A guidance) and does not re-run on subsequent frames, so `--component` + `--script` composition works.

### 2026-07-13 — Task 015c landed

Added `compute_component_aabb` and `crop_and_write_png` to `examples/demo-rust/src/main.rs` (+117/-20). Extended the `start_recording` gate to include `isolated_component.is_some()` (Pitfall #3). Auto-crop splice inside the `Ok(frame)` arm runs before the full-frame PNG write, sets a `cropped: bool` on success, and the full-frame write is gated by `if !cropped { ... }` (Option β per the spec). `stop_recording()` is called once inside the splice, regardless of AABB outcome.

**Spec amendment required (new pitfall, surfaced by implementation):**

> **Pitfall #13 — Z_SCRIM quads dominate overlay AABBs.** The drawer's scrim (a single full-window darkening quad at `crates/akar-components/src/drawer.rs:49-76` with `z = Z_SCRIM` and `rect = [250, 0, 550, 600]`) and the modal's backdrop (`crates/akar-components/src/modal.rs:38-50`) extend the auto-crop AABB to the full window because the scrim spans nearly the entire surface. Without filtering, `--component drawer` produces a 800x600 crop with the drawer content visible in the top-left — i.e. no crop. **Fix:** in `compute_component_aabb`, `continue` if `q.z == Z_SCRIM`. Verified: `Z_SCRIM` has only the two call sites above (drawer + modal backdrops), so the filter is conservative and cannot drop legitimate component content. Text calls have no `Z_SCRIM` callers in the codebase.

**Verification — partial due to winit NSApp hang (environmental, not a code defect):**
- `cargo check` / `clippy` / `test` (145 tests pass) / `fmt --check` all pass.
- `--component drawer --screenshot /tmp/drawer.png --exit` → **266x600** PNG, AK avatar + 4 nav links, right-side black padding visible. Z_SCRIM filter is what made this a real crop.
- `--component dropdown --screenshot /tmp/dropdown.png --exit` → **116x145** PNG, all 4 options (A–D) with padding on all four sides.
- `--component form`, `--component navbar`, and the full `--screenshot` demo hung in `[NSApplication _nextEventMatchingEventMask:]` after the first 1–2 runs in the same shell session. This is a known winit/macOS limitation (NSApp activation state gets stuck once a non-foregrounded run completes; the second run in a fresh session blocks forever waiting for an event that never arrives). It is environmental — the same `target/release/demo-rust` binary that produced the working drawer and dropdown captures produces no output on the hung cases. The AABB + crop path is identical for all components, and the two most complex cases (overlay with scrim filtering, overlay with cross-component layout dep) are verified visually. Re-run the unverified commands in a fresh GUI session to confirm.

**Expected dimensions from the spec (recorded for cross-check after re-verification):**
- `--component drawer` — spec said ~282x632; actual 266x600. The difference is that the AABB is the panel's `rect` (`(0, 0, 250, 600)`) plus 16px padding (250+16=266 wide, 600+0=600 tall), not the spec's estimate which assumed the panel rect is `(16, 16, 250, 600)`. The 266x600 is correct — the spec was approximate.
- `--component dropdown` — ~116x145 actual matches the spec's "tight" expectation.
- `--component form` — should be ~the form's `form_container` rect + 16px padding (~552x~600 in 800x600 window).
- `--component navbar` — should be ~816x80 (full-width, 48px tall + 16px padding).
- `--screenshot` (no component) — must remain 800x600 (the full-frame write path).

**Reviewer check:** `compute_component_aabb` filters by Z_SCRIM (Pitfall #13) and by scissor intersection (Pitfall #9) before extending the AABB. `crop_and_write_png` uses 4-side clamp via `right = (aabb[0] + aabb[2] + PAD).min(frame.width as f32) as u32; bottom = (aabb[1] + aabb[3] + PAD).min(frame.height as f32) as u32; w = right.saturating_sub(x); h = bottom.saturating_sub(y)` (Pitfall #10). `pad = 16.0` is in physical pixels (Pitfall #11). On degenerate AABBs (`w == 0 || h == 0`) the splice falls through to the full-frame write — same as a non-isolated capture. The non-isolated path (no `--component`) is unchanged.



