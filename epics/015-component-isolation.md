# Epic 015: Component Isolation for Screenshot-Driven Debugging

**Status:** Brainstorming
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
    fn render(&self, state: &mut AppState) { ... }
    fn force_state(&self, state: &mut AppState) { ... }
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

**Decision:** The full taffy layout computes (so layout-based components have correct rects), but only the isolated component's render function is called.

**Rationale:**
- Layout components (form, stats) need their taffy rects to be resolved.
- Overlay components (drawer, dropdown, modal) compute their positions from viewport rects, which are available after layout.
- Running full layout is cheap and preserves correctness.
- Skipping other render calls leaves those areas blank (black, from `LoadOp::Clear`).

**Implementation:**
```rust
if let Some(component) = &self.isolated_component {
    component.force_state(state);  // e.g., force drawer_open = true
    component.render(state);       // only this component's render calls
} else {
    render_all(state);
}
```

### 4. Auto-crop the screenshot to the component's bounding box

**Decision:** After capture, crop the PNG to the component's bounding box + padding.

**Rationale:**
- Without auto-crop, an isolated drawer (w=250) in an 800x600 window produces a mostly-black image with a thin strip on the left. This is not useful for debugging.
- Auto-crop turns this into a tight 282x632 image (250 + 16px padding on each side).
- The agent sees the component clearly, not a sea of black pixels.

**Implementation:**
1. When `--component` is set, enable draw-list recording (from Epic 014c).
2. After the component renders, compute the AABB of all recorded draw calls (min/max of quad rects and text clip rects).
3. After `take_screenshot` produces the RGBA buffer, crop to `[aabb.x - pad, aabb.y - pad, aabb.w + 2*pad, aabb.h + 2*pad]`.
4. Write the cropped PNG.

**Dependencies:** Uses the recording mode from Epic 014c (`DrawList::start_recording`, `recorded_calls()`). The AABB computation and PNG cropping are ~30 lines total.

### 5. Implicit state forcing

**Decision:** `--component <name>` implicitly forces the "interesting" state for that component.

**Rationale:**
- A closed drawer is not interesting to debug. An open drawer is.
- A closed dropdown is not interesting. An open dropdown is.
- If the agent must manually script the state, they need to know the component's internal state variables (`drawer_open`, `dropdown_open`, `modal_open`), which defeats the purpose of isolation.

**Implementation:** Each `Component` variant has a `force_state(&self, state: &mut AppState)` method:
- `Drawer`: `state.drawer_open = true; state.drawer_progress = 1.0;`
- `Dropdown`: `state.dropdown_open = true;`
- `Modal`: `state.modal_open = true;`
- `Toasts`: push a sample toast if the list is empty.
- Layout components (form, stats, navbar): no forcing needed; they render in their default state.

**Override:** The script can override the forced state. If the agent wants a closed drawer, they can set `state.drawer_open = false` in a script (though this requires script support for direct state mutation, which is deferred).

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

### 10. Integration with `--script`: Compose for interactive states

**Decision:** `--component` composes with `--script` from Epic 014b.

**Rationale:**
- Agents can script interactive states (e.g., hover a dropdown item, focus a form field) and capture the isolated component in that state.
- Example: `--component dropdown --script hover_item.txt` renders only the dropdown, scripts a hover, and captures a screenshot.

**Implementation:**
- The script runner operates on the full `InputState` as before.
- The `--component` flag affects which components render, not how input is processed.
- The script can override the forced state (e.g., close the drawer after it's forced open) if needed.

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

**Scope:** Extract the monolithic `RedrawRequested` handler into per-component render functions.

**Implementation:**

1. **Identify component blocks** in `examples/demo-rust/src/main.rs`:
   - Navbar: lines ~760-876 (navbar setup, button renders, dropdown button)
   - Containers: lines ~878-901 (background containers)
   - Alert: lines ~903-914
   - Tab bar: lines ~916-927
   - List tab: lines ~943-1042 (scroll area, list items, tooltips)
   - Canvas tab: lines ~1043-1061
   - Stats tab: lines ~1062-1126 (stats, steps, avatars, skeleton toggle)
   - Form tab: lines ~1127-1273 (form fields, submit button)
   - Drawer: lines ~1277-1375 (drawer animation, avatar, nav links)
   - Modal: lines ~1377-1411
   - Toasts: lines ~1413-1421
   - Dropdown: lines ~1423-1482 (dropdown menu, items)

2. **Extract render functions:**
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

3. **Handle `viewport_rect`:** Several components need the viewport rect — `drawer_begin`, `modal_begin`, `dropdown_begin`, `toasts`, `akar_tooltip` (in list tab), and `akar_select` (in form tab) all take `viewport_rect` as a parameter. Currently it is a local variable computed at the top of `RedrawRequested`. Pass it as a parameter to render functions that need it, or store it in `AppState` during `RedrawRequested` before calling render functions.

4. **Handle navbar slot initialization:** The navbar has a one-time setup block (lines 760-775) that calls `akar_navbar` to obtain `NavbarSlots`, then adds child nodes to the layout. This must run before the navbar's per-frame render calls. When `--component navbar` is used, this initialization must still happen. Options:
   - Keep the initialization outside the render functions (run it unconditionally before selective rendering).
   - Move it into `render_navbar` with a guard (`if state.navbar_slots.is_none()`).
   The second option is cleaner — it keeps navbar logic self-contained.

5. **Handle state mutations:** Some render blocks mutate state (e.g., `menu_result.clicked` toggles `drawer_open`). These mutations must remain in the render functions. When isolating a component, the state mutations from skipped blocks are also skipped, which is the desired behavior (see decision 5).

**Acceptance:**
- `cargo run --example demo-rust` renders identically to before the refactor.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.

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

        fn force_state(&self, state: &mut AppState) {
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

3. **Validate and resolve** the component name before entering the event loop:
   ```rust
   let component = match isolated_component {
       Some(name) => match Component::from_name(&name) {
           Some(c) => Some(c),
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

4. **Selective rendering in the render loop:**
   ```rust
   let viewport_rect = [
       0.0, 0.0,
       size.width as f32 / scale,
       size.height as f32 / scale,
   ];
   if let Some(component) = &self.isolated_component {
       component.force_state(state);
       component.render(state, viewport_rect);
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

2. **Compute AABB from recorded calls:**
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

3. **Crop the PNG after capture:**
   ```rust
   if let Some(component) = &self.isolated_component {
       let recorded = state.core.draw_list.recorded_calls();
       if let Some(aabb) = compute_component_aabb(recorded) {
           let pad = 16.0;
           let crop_rect = [
               (aabb[0] - pad).max(0.0) as u32,
               (aabb[1] - pad).max(0.0) as u32,
               (aabb[2] + 2.0 * pad) as u32,
               (aabb[3] + 2.0 * pad) as u32,
           ];
           // Crop the RGBA buffer and write the cropped PNG
           crop_and_write_png(&frame, &crop_rect, &capture_path)?;
       } else {
           // No draw calls; write the full image
           write_png(&frame, &capture_path)?;
       }
   }
   ```

4. **Implement `crop_and_write_png`:**
   ```rust
   fn crop_and_write_png(
       frame: &akar_core::screenshot::CapturedFrame,
       crop: &[u32; 4],
       path: &str,
   ) -> Result<(), String> {
       let [x, y, w, h] = *crop;
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

**Acceptance:**
- `--component drawer --screenshot /tmp/drawer.png --exit` produces a cropped PNG (not 800x600, but ~282x632 with 16px padding).
- `--component dropdown --screenshot /tmp/dropdown.png --exit` produces a cropped PNG of the dropdown menu.
- `cargo clippy --workspace -- -D warnings` passes.
- `cargo test --workspace` passes.

---

## Known Implementation Pitfalls

These issues were identified during design review. The implementer should be aware of them to avoid rediscovering them during implementation.

### 1. Tab components need `active_tab` and `prev_active_tab` set

When isolating a tab component (e.g., `--component form`), the tab's layout nodes are only added to the tree if `state.active_tab` matches the tab's index. The `force_state` method must set both `active_tab` and `prev_active_tab` to the same value to suppress the toast notification that would otherwise be triggered by the tab change detection logic.

### 2. Alert needs `alert_dismissed = false`

If the alert has been dismissed in a previous frame, its layout node is set to `Display::None`. The `force_state` method for `Alert` must set `alert_dismissed = false` to ensure the alert renders.

### 3. Recording must be enabled for `--component` unconditionally

The auto-crop feature (Task 015c) requires draw-list recording to compute the component's bounding box. The recording gate must check `self.isolated_component.is_some()`, not just `self.dump_frame_path.is_some()`.

### 4. `viewport_rect` must be accessible to render functions

Several components (`drawer_begin`, `modal_begin`, `dropdown_begin`, `toasts`, `akar_tooltip`, `akar_select`) require `viewport_rect` as a parameter. Currently it is a local variable in `RedrawRequested`. Either:
- Pass it as a parameter to render functions that need it (cleaner, explicit dependencies), or
- Store it in `AppState` during `RedrawRequested` (less explicit, but avoids parameter passing).

The first option is preferred for clarity.

### 5. Navbar slot initialization must run before navbar render

The navbar has a one-time setup block that calls `akar_navbar` to obtain `NavbarSlots`, then adds child nodes to the layout. This must run before the navbar's per-frame render calls. When `--component navbar` is used, this initialization must still happen. The cleanest approach is to move the initialization into `render_navbar` with a guard (`if state.navbar_slots.is_none()`), keeping navbar logic self-contained.

### 6. Overlay components compute positions from `viewport_rect`, not layout

The drawer, dropdown, modal, and toasts are overlay components that compute their positions from `viewport_rect` (the full window rect), not from taffy layout. They don't have taffy nodes for their visual content. The component registry approach (decision 2) handles this correctly by treating them as render-call blocks, not layout subtrees.

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

- This epic is intentionally a brainstorming document. The final feature set and implementation plan should be refined through discussion before any code is written.
- The key insight is that components in akar are render-time constructs, not layout-time constructs. The implementation must match that reality.
- This is a demo-only feature for v1. If it proves useful, it can be promoted to core later.
- The render loop refactor (Task 015a) is valuable on its own and should be done first, even if the `--component` feature is deferred.
