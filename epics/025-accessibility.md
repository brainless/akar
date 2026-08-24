# Epic 025: Accessibility

**Status:** Deferred (2026-08-24) — in favor of Epic 026, worked on immediately. Research Complete and Verified, Implementation Not Started when resumed: Tasks 1-6 researched and re-verified 2026-08-12; Task 7 is independently ready, while Tasks 8-14 are blocked by the semantic/action contract and sequencing recorded in the readiness audit below.
**Goal:** Establish a path for akar UIs to be usable with screen readers and other assistive technology, despite akar being a pure GPU draw-list renderer with no OS-level accessibility tree today.

**Prerequisite:** Epic 021 is `Status: Done`. Independent of [[022]], [[023]], and [[024]] — this epic can be researched in parallel with those, but should not be blocked by or block them.

---

## Introduction

`AGENTS.md` currently states plainly: "Do not add accessibility scaffolding in v1. Document the punt if relevant." That punt was reasonable to get the component catalog and rendering pipeline to a working state, but it means akar UIs are currently invisible to screen readers, switch access, and any other assistive technology — the draw list produces pixels, not semantics, and nothing in `akar-core` or `akar-components` emits a parallel accessibility tree.

This is the largest and most architecturally invasive of the four topics in this round. Font support and i18n are largely "verify and extend existing capability" problems; RTL is a significant but contained layout problem. Accessibility requires a new kind of output from every interactive component — a semantic tree (role, label, state, bounds) — alongside the existing draw list, plus a platform adapter to expose that tree to Windows UIA, macOS AX, and AT-SPI (Linux). It also collides with akar's core design principle that "akar does not own the window" (`DEVELOP.md`): most accessibility toolkits assume they can hook directly into the window they're rendering into, which akar's C-ABI, developer-owns-the-window model does not straightforwardly allow.

This epic is investigation-first and should not attempt to design the full solution before understanding the constraints.

---

## Research

Initial inputs, to be expanded by the coding agent doing the investigation:

- **AccessKit is the natural reference implementation.** It's the Rust-native accessibility crate used by egui and, at least partially, by Zed/gpui — both of which are already in akar's local reference set (`~/Projects/egui`, `~/Projects/zed/crates/gpui`). AccessKit provides a semantic-tree data model and platform adapters (Windows UIA, macOS AX, AT-SPI, web). Read its integration in both of those projects before designing anything akar-specific.
- **The window-ownership problem is the key architectural risk.** AccessKit's platform adapters typically need a live handle into the OS window (HWND, NSView, etc.) to attach the accessibility tree. akar explicitly does not own the window (`DEVELOP.md` → "What akar does NOT own") — the developer does, often through `akar-winit` or their own windowing code. This means the adapter almost certainly needs to live in `akar-winit` (optional, as today) with a companion, windowing-independent semantic-tree-emission API in `akar-core`/`akar-components` that non-Rust/non-winit consumers could wire into their own platform adapter. This split needs to be validated against how egui's `egui-winit`/`eframe` split handles the same problem, since it's the closest architectural analog.
- **Semantic tree emission has to fit the existing lifecycle.** Per `DEVELOP.md`'s construct/compute/paint model, the natural place for a component to report its role/label/state is likely alongside paint (it already has resolved rect and current state there), but emitting an accessibility tree during paint would violate "paint is read-only on Layout" only if the tree is treated as part of `Layout` — it more likely belongs as a separate frame-scoped output next to the draw list, mirroring how the draw list itself is frame-scoped and flushed.
- **Keyboard focus order is a partial foundation already.** Widget IDs and focus state already exist for text editing (epic 018) and interactive components generally. This is necessary but far from sufficient for accessibility (focus order without semantic roles/labels is not usable by a screen reader), but it means this epic isn't starting from zero.
- **C ABI exposure is a second-order problem.** Non-Rust consumers of `akar.h` would need either (a) akar-owned platform adapters compiled into `libakar` for at least the common desktop platforms, or (b) a way to walk the emitted semantic tree from C and hand it to their own platform accessibility API. Option (a) is a much bigger commitment (three platform adapters shipped inside the core library) and should be weighed carefully against akar's "flat C API, no callbacks unless opted into" philosophy.
- **Theme/contrast is a smaller, more tractable adjacent concern.** Independent of the semantic-tree problem, the two shipped theme presets (`AKAR_THEME_DARK`/`AKAR_THEME_LIGHT`, per `DEVELOP.md`) should eventually be checked against WCAG contrast minimums. This is cheap to investigate and worth scoping as an early, low-risk task even before the harder semantic-tree work.

### Findings from investigation (this pass)

**AccessKit is vendored and readable locally.** Despite not being an akar dependency, AccessKit's source is present in the local cargo registry cache (populated by building egui/gpui checkouts) at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`: `accesskit-0.24.0`, `accesskit_winit-0.32.2`, `accesskit_macos-0.26.0`, `accesskit_consumer-0.35.0`. No `accesskit_windows`/`accesskit_unix` crate happened to be cached locally (not fetched by either checkout's resolved dependency graph on this machine), but their shape is inferable from `accesskit_winit`'s `platform_impl` re-export pattern and is documented on `accesskit.dev`. `accesskit-0.24.0` is a single-file crate (`src/lib.rs`, 3173 lines, plus `src/geometry.rs`, 866 lines — ~4000 lines total, not the ~4300 originally noted for `lib.rs` alone) — no separate `node.rs`/`tree.rs`, contrary to the epic's initial assumption of a `NodeBuilder` type. In this version:
  - `Role` (`lib.rs:57`) — a `#[repr(u8)]` enum, ARIA-derived, ordered by expected frequency (`Unknown`, `TextRun`, `Cell`, `Label`, `Image`, `Link`, `Row`, `ListItem`, `ListMarker`, `TreeItem`, `ListBoxOption`, `MenuItem`, `MenuListOption`, `Paragraph`, `GenericContainer`, `Button`/`CheckBox`/`RadioButton`/`Switch`/`TextInput`/`Slider`/`SpinButton`/`ComboBox`/`Window`/`Heading`/etc. further down — confirmed exists via grep, not enumerated here in full).
  - `Node` (`lib.rs:962`) — there is no separate `NodeBuilder` in 0.24; `Node` itself is mutable and built incrementally (`role`, `actions: u32` bitflags, `child_actions: u32`, `flags: u32`, and a `properties: Properties` sparse property store keyed by `PropertyId`). Property setters/getters (`set_label`, `set_value`, `push_child`, `set_bounds`, `set_numeric_value`, etc.) are generated by internal macros (`property_methods!`, `vec_type_methods!`, flag macros around `lib.rs:1014-1180`) — the public surface is effectively "call `Node::new(role)` then chain `set_*`/`push_*` setters," which is exactly what egui's and gpui's integrations do.
  - `Tree` (`lib.rs:2718`) — just `{ root: NodeId, toolkit_name: Option<String>, toolkit_version: Option<String> }`, sent once (or whenever it changes) alongside a `TreeUpdate`.
  - `TreeUpdate` (`lib.rs:2754`) — `{ nodes: Vec<(NodeId, Node)>, tree: Option<Tree>, tree_id: TreeId, focus: NodeId }`. Critically it is a **diff/patch structure, not a full snapshot**: `nodes` need only contain new-or-changed nodes; removing a child means re-sending the parent with that child removed from `Node::children` and simply not including the removed subtree. `focus` must be set on every update, defaulting to the tree root if nothing is focused. This maps directly onto akar's frame-scoped model but requires akar to diff against the previous frame's tree (or always resend the full tree and let AccessKit's adapters absorb the redundant-but-correct data — the simpler, likely-first-cut approach).

**egui's integration (the closest architectural analog) is real and traceable end-to-end:**
  - Emission happens inside immediate-mode widget code in `egui/src/context.rs`. `Context::accesskit_node_builder` (`~context.rs:587`) lazily creates the per-`Id` `accesskit::Node`, finds the nearest ancestor with an existing node via `find_accesskit_parent` (`~context.rs:595`), and calls `push_child` to link it into the tree — i.e. the tree is built incrementally, one node per `Ui`/`Response`, as the immediate-mode pass runs, not as a separate post-pass. `Response::fill_accesskit_node_common` (referenced at `context.rs:1228`) is the generic per-widget populate step (role, label, bounds, focusability) called from `Context::create_widget`.
  - Whether emission happens at all is gated per-frame by `is_accesskit_enabled: bool` (`context.rs:405`) — akar's own equivalent would be an `AkarCore` flag toggled by the platform adapter telling akar "accessibility is currently wanted" (AccessKit's platform adapters raise activation/deactivation events precisely because screen readers are usually off; building the tree every frame when nobody is listening is wasted CPU).
  - At end-of-pass (`context.rs:2593`), the accumulated per-`Id` node map is drained into an `accesskit::TreeUpdate` and stashed on `PlatformOutput::accesskit_update` (`egui/src/data/output.rs`, referenced via grep at `context.rs:2608`) — i.e. it rides out of the immutable "what happened this frame" output object alongside cursor icon, clipboard commands, and IME state. This is the shape akar should copy: a new field on whatever frame-output/`AkarCore::end_frame` returns, sibling to the draw list, not baked into `Layout`.
  - `egui-winit` (`crates/egui-winit/src/lib.rs`) is where the window handle is threaded through, confirming the epic's hypothesis: `EventLoopProxyWrapper::init_accesskit` (`lib.rs:183-197`) calls `accesskit_winit::Adapter::with_event_loop_proxy(event_loop, window, event_loop_proxy)` — it needs the live winit `Window` (for the OS handle) and an `ActiveEventLoop` (to attach at the right lifecycle point; the doc comment on `accesskit_winit::Adapter::with_event_loop_proxy`, read at `accesskit_winit-0.32.2/src/lib.rs:132-148`, says the adapter **must be constructed before the window is first shown**, using `with_visible(false)` then creating the adapter then showing — a hard sequencing constraint any akar-winit integration must respect). Every frame, `handle_platform_output_inner` (`egui-winit/src/lib.rs:1083-1178`) pulls `accesskit_update` off `PlatformOutput` and calls `accesskit.update_if_active(|| update)` (`lib.rs:1168-1174`) — a closure so the (possibly nontrivial) tree-diff work is skipped entirely when accessibility is inactive.
  - `egui`, `egui_consumer` (renamed `accesskit_consumer` upstream) and `accesskit_winit` are all **optional, feature-gated** dependencies (`egui-winit/Cargo.toml:29-30`: `accesskit = ["dep:accesskit_winit"]`), never pulled in by default. This directly supports keeping any akar accessibility support opt-in/feature-gated rather than an always-on dependency.

**gpui (Zed) — contrary to this epic's original "may or may not be present" hedge, it has a substantial, already-shipped AccessKit integration**, not a stub:
  - `~/Projects/zed/crates/gpui/src/window/a11y.rs` (794 lines) is the core module; `~/Projects/zed/crates/gpui/src/_accessibility.rs` is a public user-facing guide (both are extensively doc-commented, effectively a design doc in the codebase). `~/Projects/zed/crates/gpui/examples/a11y.rs` is a full runnable example app (spin button, switch, to-do list) using gpui's a11y API.
  - Architecture (documented as ASCII diagram at `a11y.rs:7-18`): gpui core talks to `accesskit` (the tree-model crate) directly; platform-specific adapters (macOS AX, Windows UIA, Linux dbus/AT-SPI) sit **outside gpui core**, wired in per-platform. gpui's own `Cargo.toml` (`crates/gpui/Cargo.toml:50`) depends only on `accesskit` itself (workspace dep) — **no** `accesskit_macos`/`accesskit_windows`/`accesskit_unix`/`accesskit_winit` dependency in the `gpui` crate itself. Those platform adapters are pulled in by separate per-OS platform crates (referenced in `platform.rs` as callback consumers, not visible as `gpui` deps) — i.e. gpui also splits "tree emission" from "platform adapter," exactly mirroring the split the epic proposed and egui also uses (just organized as core-crate-vs-platform-crate rather than core-crate-vs-optional-feature).
  - `gpui/src/platform.rs:602-729` defines the seam precisely: `A11yCallbacks { activation: Box<dyn Fn() -> Option<accesskit::TreeUpdate>>, action: Box<dyn Fn(accesskit::ActionRequest)> }` plus three platform trait methods with **empty default bodies** — `fn a11y_init(&self, _callbacks: A11yCallbacks) {}`, `fn a11y_tree_update(&self, _tree_update: accesskit::TreeUpdate) {}`, `fn a11y_update_window_bounds(&self) {}` (`platform.rs:723,726,729`). This is the cleanest evidence available anywhere in the local reference set of "core emits tree data through a narrow trait; a platform layer that owns the window implements the trait; the default no-op body means platforms that haven't implemented accessibility yet simply do nothing" — a directly reusable pattern for akar's `akar-core`/`akar-winit` split.
  - Tree emission is driven by prepaint, not paint: the module doc (`a11y.rs:58-63`) states "This all happens in `Drawable::prepaint`. The `A11y` struct maintains a stack of nodes during prepainting... Once all `Element`s in a frame have been prepainted, we send the resulting `TreeUpdate` object to the adapter." Node identity comes from gpui's existing `GlobalElementId` (`GlobalElementId::accesskit_node_id`, `a11y.rs:52-56`) — i.e. gpui reuses its pre-existing stable element-identity system for accessibility node IDs, the same way this epic proposes reusing akar's existing `widget_id`/`widget_id_keyed` (epic 017/018) rather than inventing a parallel ID scheme.
  - The `A11y` struct (`a11y.rs:124-165`) is explicitly frame-scoped/per-window state: `active_this_frame: bool` (`a11y.rs:145`, set at `a11y.rs:182`) is loaded once at frame start and held fixed for the whole frame specifically so the node-builder stack push/pop discipline stays balanced (`a11y.rs:112-119` doc comment) — directly analogous to akar's `DrawList::begin_frame`/`end_frame` discipline and a strong argument for implementing the semantic tree as a sibling frame-scoped object (`AkarCore::a11y_tree: A11yTree` next to `AkarCore::draw_list: DrawList`) rather than folding it into `Layout`.
  - Two further gpui details worth carrying into akar's design: (1) nodes without a role are not reported at all (`_accessibility.rs:103`, "nodes with no role are not reported") — akar's emission API should treat "no role" as "opt out of this frame's tree," not an error; (2) gpui warns about *unstable* IDs producing spurious add/remove churn for screen readers (`_accessibility.rs:88-99`) — this is precisely why akar must key emission off `widget_id`/`widget_id_keyed` (already stable across frames per ADR-016a) rather than any position-derived value.

**akar's own architecture — concrete anchor points found by reading the source directly:**
  - `crates/akar-components/src/button.rs:64-166` (`button_styled`) is the paint-phase function: it reads `layout.rect(node_id)` (already-resolved pixel rect), computes `hovered`/`pressed`/`clicked` from `core.input`, mutates nothing in `Layout`, and pushes exactly one `QuadCall` and one `TextCall` into `core.draw_list`. It already computes `layout.widget_id(node_id)` (`button.rs:139`) for the text buffer ID — the same value is the natural accessibility node ID.
  - `crates/akar-core/src/draw_list.rs` is the direct structural analog for a semantic tree: `DrawList` (`draw_list.rs:47-53`) is a plain `Vec`-backed frame-scoped buffer with `begin_frame`/`push_quad`/`push_text`, cleared every frame (`draw_list.rs:66-73`, `self.calls.clear()`), with an optional `recording`/`recorded_calls` side-channel (`draw_list.rs:75-86`) used today for `--dump-frame` debug capture. A semantic tree ("`A11yTree`") could copy this exact shape: `begin_frame`/`push_node`/`nodes()`/(optionally) a debug dump mirroring `--dump-frame`.
  - `crates/akar-core/src/context.rs`: `AkarCore` (`context.rs:8-21`) already owns `draw_list: DrawList`, `input: InputState`, `text_pipeline: TextPipeline` as public frame-scoped fields; `begin_frame` (`context.rs:62-67`) and `end_frame` (`context.rs:69-120`) are the two lifecycle hooks. A semantic tree would plug in identically: a new public field, cleared in `begin_frame`, read out (and handed to a platform adapter, or left for the caller to walk) at the end of `end_frame` — after `self.input.begin_frame()` at `context.rs:117` clears single-frame input events, mirroring where AccessKit's `focus: NodeId` would be read off the *current* `InputState::focused_id`.
  - `crates/akar-core/src/input.rs:157` — `InputState::focused_id: Option<u64>` is the **only** focus-tracking primitive in akar today (see Task 4 below for how partial this is).
  - `crates/akar-layout/src/lib.rs:66-77` — `Layout::widget_id(node)` and `Layout::widget_id_keyed(node, key)` are exactly the stable per-frame IDs gpui's a11y module independently converged on reusing (`GlobalElementId::accesskit_node_id`). These should be reused verbatim as AccessKit `NodeId` sources (`accesskit::NodeId` wraps a `u64`-compatible `NodeIdContent`, `accesskit-0.24.0/src/lib.rs:640`).
  - `crates/akar-winit/src/lib.rs` (212 lines total) is the entire current scope of the winit bridge: `process_window_event(input: &mut InputState, event: &WindowEvent)` translates mouse/keyboard/scroll winit events into `akar_core::InputState` mutations. It has **zero** window-handle-holding state today — it is a pure function taking `&WindowEvent`, not a struct holding a `Window` reference. This is a meaningful gap versus egui-winit's `State` struct (which owns the winit `Window`/`accesskit_winit::Adapter` pair) — adding AccessKit support to `akar-winit` means introducing, for the first time, a stateful bridge type that outlives a single event call and holds the winit `Window`/`ActiveEventLoop` handles needed to construct `accesskit_winit::Adapter::with_event_loop_proxy`. This is a real (if modest) architectural change to `akar-winit`, not a drop-in addition.
  - `crates/akar-c-api/src/lib.rs` — grepped for `accesskit`/`a11y`/`accessib`: no hits. The C ABI has no accessibility surface today, confirming Task 1/2's framing that C-ABI exposure is a wholly separate, deferred problem (see Task 6 deferrals).
  - `epics/018-text-editing-keybindings-and-clipboard.md` (Status: Done) confirms `Key::Tab` exists as a named key (`akar-winit/src/lib.rs:26`, mapped from `winit::keyboard::NamedKey::Tab`) but grepping all of `akar-components` and `akar-core` for `focus_next`/`focus_prev`/`tab_order`/`TabOrder`/`tab_index` returns **zero matches**. `Tab` is decoded as a `Key` value but nothing currently consumes it to move focus between widgets.

---

## Review — 2026-08-12 verification pass

Independent fact-check of every finding and citation above against the current source. No source file was modified; only this epic file was edited. Method is labelled per item: **[exec]** = verified by running a command, **[src]** = verified by reading the cited source.

### Verified as written

- **`cargo check --workspace` is clean on `main`** at commit `7413111` (warnings only, all pre-existing; `demo-rust` emits 119 warnings). **[exec]**
- **accesskit 0.24.0 API shape claims are all correct.** `accesskit-0.24.0/src/lib.rs` is 3173 lines and `src/geometry.rs` is 866 (4039 total). `Role` at `lib.rs:57`, `NodeId(pub NodeIdContent)` at `lib.rs:640`, `Node { role, actions, child_actions, flags, properties }` (all fields private) at `lib.rs:962`, `Tree { root, toolkit_name, toolkit_version }` at `lib.rs:2718`, `TreeUpdate { nodes: Vec<(NodeId, Node)>, tree: Option<Tree>, tree_id: TreeId, focus: NodeId }` at `lib.rs:2754`. A repo-wide grep for `NodeBuilder` in that crate returns zero hits — the epic's claim that 0.24 has no `NodeBuilder` is correct. **[src]**
- **New detail worth carrying forward:** `TreeId` (`lib.rs:670`) is `TreeId(pub Uuid)`, not an integer, and `TreeId::ROOT` (nil UUID, `lib.rs:675`) is the value a single-window akar app should use. `TreeUpdate`'s own doc comment (`lib.rs:2740-2780`) also states the diff rule more strictly than the epic did: an updated node must re-send *all* of its fields (unchanged ones included), and removing a subtree means re-sending the parent without that child and including neither the child nor any descendant. The epic's "always resend everything" fallback remains valid and correct under this rule.
- **accesskit_winit sequencing constraint is real.** `accesskit_winit-0.32.2/src/lib.rs:132-148`: `Adapter::with_event_loop_proxy` doc says the adapter "must be done before the window is shown for the first time... use `WindowAttributes::with_visible` to make the window initially invisible, then create the adapter, then show the window", and the function **panics if the window is already visible**. (Epic cited `lib.rs:131-152`; the exact doc block is `132-146` with the `pub fn` at `148`.) **[src]**
- **egui citations all resolve.** `egui/crates/egui/src/context.rs:405` (`is_accesskit_enabled`), `:587` (`accesskit_node_builder`), `:595` (`find_accesskit_parent`), `:1228` (`fill_accesskit_node_common`), `:2608` (`platform_output.accesskit_update = Some(accesskit::TreeUpdate {`); `egui/crates/egui-winit/src/lib.rs:184` (`init_accesskit`), `:192` (`Adapter::with_event_loop_proxy`), `:1083` (`handle_platform_output_inner`), `:1173` (`accesskit.update_if_active(|| update)`); `egui/crates/egui-winit/Cargo.toml:30` (`accesskit = ["dep:accesskit_winit"]`) and `:72` (`accesskit_winit = { workspace = true, optional = true }`). **[src]**
- **gpui citations all resolve.** `zed/crates/gpui/src/window/a11y.rs` is 794 lines, `src/_accessibility.rs` 295, `examples/a11y.rs` 266. `A11yCallbacks` at `platform.rs:602`; `a11y_init` / `a11y_tree_update` / `a11y_update_window_bounds` empty default bodies at `platform.rs:723,726,729`. `crates/gpui/Cargo.toml:50` is `accesskit.workspace = true` and is gpui's only accesskit dependency. **[src]**
- **akar anchor points** `akar-core/src/context.rs:8` (`pub struct AkarCore`), `:48` (`mock`), `:62` (`begin_frame`), `:69` (`end_frame`), `:117` (`self.input.begin_frame()`); `akar-core/src/draw_list.rs:47` (`pub struct DrawList`), `:66` (`begin_frame`), `:75` (`start_recording`), `:84` (`recorded_calls`); `akar-core/src/input.rs:157` (`pub focused_id: Option<u64>`); `akar-layout/src/lib.rs:66` (`widget_id`), `:74` (`widget_id_keyed`); `akar-components/src/button.rs:64` (`button_styled`), `:83-85` (hover/press/click), `:139` (`layout.widget_id(node_id)`); `akar-winit/src/lib.rs` is 212 lines with `process_window_event` at `:89` and the `NamedKey::Tab` mapping at `:26`. **[src]**
- **Focus-order gap confirmed.** A workspace-wide grep for `focus_next`, `focus_prev`, `tab_order`, `TabOrder`, `tab_index` returns zero hits. The only `focused_id` writers in `akar-components` are `text_input.rs:64,71,150`, `textarea.rs:83,90,174`, and `data_list.rs:260`; `canvas.rs:743-744` is the never-focus assertion. **[exec]**
- **`switch` really does lack any label parameter.** `crates/akar-components/src/switch.rs:7-13` is `switch(core, layout, node_id, on: &mut bool, theme)` — no text argument of any kind. `checkbox.rs:12` (`label: &str`) and `radio.rs:11` (`labels: &[&str]`) do have caller-supplied labels. The epic's corrected claim stands. **[src]**
- **The Task 5 contrast table was independently recomputed from the current hex values in `crates/akar-components/src/theme.rs:41-115` using the sRGB-linearization / relative-luminance formula. Every one of the 32 ratios printed in Task 5 matches to two decimal places. No numbers were changed.** **[exec]**

### Corrections applied to the text above

1. **`Layout::parents` is at `akar-layout/src/lib.rs:37`, not `:35`** (three places said `:35`). More importantly, **`parents` is a private field with no public accessor**: `grep "pub fn " crates/akar-layout/src/lib.rs` shows no `parent`/`ancestors` function, and the only read is the internal ancestor walk in `Layout::rect` at `lib.rs:207`. Task 10's proposed `layout.widget_id(parents[&node_id])` therefore **does not compile today** and requires adding a small public accessor to `akar-layout` first. This is now spelled out in Task 10. **[src]**
2. **Test-module line ranges drifted**: `button.rs` tests are `169-258` (epic said `174-257`); `draw_list.rs` tests are `197-329` (epic said `196-329`); gpui's `A11y` struct is at `a11y.rs:124` with `active_this_frame` at `:145` and its initializer at `:164` (epic said `120-140`). Corrected inline.
3. **The status-color contrast failures are actively shipping, not latent.** The epic hedged ("very likely below AA today"). Verified: `crates/akar-components/src/badge.rs:65-68` maps `BadgeVariant::Success/Warning/Error/Info` to `(theme.success, theme.success_content)` etc. and paints that pair as fill + text; `crates/akar-components/src/toast.rs:66-69,93` fills the toast with the raw status color and then draws **hardcoded `0xFFFFFFFF` text** on it. So a success badge/toast ships at **2.28:1** and a warning badge/toast at **2.15:1** right now — below even the 3:1 non-text floor. `alert.rs:44-49` is not affected: it fills with `base_200` and only uses the status color for a `dim_color(_, 0.5)` border. **[src]**
4. **The `primary`/`secondary`-on-`base_100` finding remains correctly described as latent** — `button.rs` draws no focus ring, and no component currently paints a `primary`/`secondary` hairline against `base_100`. Unchanged.

### Blockers resolved during this pass (so tasks can proceed)

- **Version pinning and winit compatibility — verified, not a blocker.** akar pins `winit = "0.30"` (`Cargo.toml:20`, workspace dep; `akar-winit/Cargo.toml:8` uses `winit.workspace = true`) and `Cargo.lock:3014-3016` resolves it to **winit 0.30.13**. `accesskit_winit-0.32.2/Cargo.toml:82-84` requires `winit = "0.30.5"` with `default-features = false` — a caret requirement satisfied by 0.30.13, resolving to the *same* winit crate instance with no duplicate-version split. `accesskit_winit-0.32.2` in turn depends on `accesskit = "0.24.0"` (`Cargo.toml:67-68`). **A coding agent should pin `accesskit_winit = "0.32"` and, if it needs the core types directly, `accesskit = "0.24"` — both already vendored locally, both compatible with akar's winit.** **[src]**
- **Caveat on `accesskit_winit` default features**: its default feature set (`Cargo.toml:36-44`) is `["accesskit_unix", "async-io", "rwh_06", "winit/x11", "winit/wayland"]`. Enabling it therefore turns on `winit/x11` + `winit/wayland` for the whole workspace via cargo feature unification on Linux, and pulls a dbus/async-io stack in via `accesskit_unix` (`Cargo.toml:106-109`). Since the dependency is feature-gated off by default in `akar-winit` (Task 11), this only affects builds that opt in — but it should be stated in the feature's doc comment rather than discovered later.
- **`Layout` parent map shape (for `A11yNode::parent_id`)**: `parents: HashMap<NodeId, NodeId>` at `akar-layout/src/lib.rs:37`, child-keyed, populated in `new_with_children` (`:105`), `add_child` (`:112`), `set_children` (`:118`), and cleared per node in `remove` (`:123`). It is a complete child→parent map over every node added through `Layout`'s public API. Task 10 needs one new one-line public accessor; the data is already there and correct.
- **Task 9 vs epics 017/018 — no conflict found.** Epic 018 filters tab out of committed text before `InputState::push_char` (`akar-winit/src/lib.rs:107-113`, `is_committed_text_char`), so a Tab press never reaches `text_input`/`textarea` as a character and no component matches `Key::Tab` anywhere (`grep Key::Tab` hits only `akar-winit/src/lib.rs:26,166-167` and `akar-c-api/src/lib.rs:1342`). Tab arrives only as a `KeyEvent` in `InputState::key_events` (`akar-winit/src/lib.rs:114-122`) and is currently consumed by nothing. Epic 017's ADR-016a constraint (identity must come from `widget_id_keyed` with a caller record key, never screen position) applies to the focus sequence exactly as it applies to `A11yNode::id`: the sequence must store `widget_id`/`widget_id_keyed` values, never indices into a per-frame node list. **[exec/src]**
- **One real ordering constraint Task 9 must design around** (not previously noted): focusable components register themselves during *paint*, but a Tab keypress is available to `akar-core` at the *start* of the frame. A Tab pressed in frame N can therefore only be resolved against the focus sequence built in frame N-1. This is exactly how egui handles it and is acceptable, but it must be an explicit design decision (keep the previous frame's sequence alive across `begin_frame` rather than clearing it, or defer the Tab resolution by one frame) rather than an accident.
- **Testing caveat for Tasks 8/10/13**: `AkarCore::mock()` (`akar-core/src/context.rs:48-60`) is not GPU-free — it calls `instance.request_adapter(...)` and `adapter.request_device(...)` and panics with "no suitable adapter" if none exists, despite `AGENTS.md`'s "No live GPU in CI" testing rule. Any `A11yTree` unit test written against `AkarCore::mock()` inherits that requirement. Prefer testing `A11yTree` directly (it needs no GPU, being a plain `Vec` type) and reserve `mock()` for the component-level assertions in Task 10 that genuinely need a whole `AkarCore`. **[src]**

### Still genuinely blocked

- **Task 12's screen-reader announcement verification.** Nothing in akar's toolchain, and no local reference, makes a real VoiceOver/NVDA/Orca announcement machine-readable. This requires either a human-in-the-loop pass or an `accesskit_consumer`-based harness (the `egui_kittest` approach). The *wiring* half of Task 12 is unblocked; the *verification* half is not.
- **Task 14** is blocked only by sequencing: it documents what Tasks 7-12 actually shipped, so it cannot be written accurately before they land.

---

## Tasks

### Task 1 — AccessKit Integration Survey

**Status:** Done — re-verified 2026-08-12 (all accesskit/egui/gpui citations resolve; see Review section)
**Readiness:** N/A — research task, complete. Its output feeds Tasks 8 and 11.

- AccessKit's crate boundary is: `accesskit` (core, `#![no_std]`-capable tree/node/role data model, zero platform code, vendored locally at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/accesskit-0.24.0/src/lib.rs`) vs. per-platform adapter crates (`accesskit_macos`, `accesskit_windows` — not locally cached but same API shape documented on accesskit.dev, `accesskit_unix` for AT-SPI on Linux) vs. `accesskit_winit` (a thin windowing-specific wrapper that constructs the right platform adapter and threads winit lifecycle events through it, `~/.cargo/registry/.../accesskit_winit-0.32.2`). A host is expected to provide: (1) a `TreeUpdate` (diff of nodes + a `focus: NodeId`) whenever the UI changes, produced by a closure/callback so it's only computed when accessibility is actually active, and (2) an `ActionHandler` callback that receives `ActionRequest`s (e.g. click-via-screen-reader, increment/decrement, set-value) and translates them back into host-side state mutations.
- egui's integration is bottom-to-top real, not aspirational: node accumulation happens inline during the immediate-mode pass (`egui/src/context.rs:587` `accesskit_node_builder`, `~context.rs:1228` `fill_accesskit_node_common`), the finished `TreeUpdate` rides out on `PlatformOutput::accesskit_update` at end-of-pass (`context.rs:2593-2611`), and `egui-winit` is exactly where the window handle enters: `EventLoopProxyWrapper::init_accesskit` (`egui-winit/src/lib.rs:183-197`) calls `accesskit_winit::Adapter::with_event_loop_proxy(event_loop, window, event_loop_proxy)`, and `handle_platform_output_inner` (`egui-winit/src/lib.rs:1168-1174`) delivers the update every frame via `accesskit.update_if_active(|| update)`. The `accesskit_winit::Adapter` doc comment (`accesskit_winit-0.32.2/src/lib.rs:132-148`) imposes a real sequencing constraint: the adapter must be constructed **before the window is first shown** (create window hidden, construct adapter, then show).
- gpui's integration is also real and shipped (not absent, contrary to the epic's original hedge): `~/Projects/zed/crates/gpui/src/window/a11y.rs` (794 lines, heavily doc-commented) plus a runnable example at `~/Projects/zed/crates/gpui/examples/a11y.rs`. Emission happens during `Drawable::prepaint` (not paint) using a node-builder stack, node IDs are derived from gpui's pre-existing `GlobalElementId` (i.e. reusing existing stable widget identity, not inventing a new ID scheme), and the platform seam is a narrow trait (`gpui/src/platform.rs:602-729`: `A11yCallbacks` + `a11y_init`/`a11y_tree_update`/`a11y_update_window_bounds` with empty default bodies). `gpui`'s own `Cargo.toml` depends only on `accesskit` core (`crates/gpui/Cargo.toml:50`) — platform adapters live outside the `gpui` crate entirely.
- Minimal integration contract akar would need to produce per frame, synthesized from both data points: (1) a frame-scoped, diffable `TreeUpdate`-shaped output (nodes changed-or-new this frame, not a full snapshot every time, though a naive "always resend everything" first cut is acceptable and correct, just wasteful), keyed by akar's existing `widget_id`/`widget_id_keyed`; (2) a `focus: NodeId`-equivalent read from `InputState::focused_id` at end of frame; (3) an activation flag so the tree is only built when a screen reader is actually attached (both egui and gpui gate on this to avoid per-frame overhead when accessibility is inactive); (4) a callback/return channel for `ActionRequest`s coming back from the platform adapter into component-owned state, which does not fit akar's current "components mutate caller-owned state synchronously within the same call" model (see Task 6 deferrals — akar has no callback/action-dispatch mechanism today, this would be new).

### Task 2 — Window-Ownership Architecture Options

**Status:** Done — re-verified 2026-08-12 (both precedents confirmed: `egui-winit/Cargo.toml:30`, `gpui/Cargo.toml:50`, `gpui/src/platform.rs:723,726,729`)
**Readiness:** N/A — decision task, complete. Option (a) is the accepted architecture for Tasks 8-11.

- **Option (a): tree emission in `akar-core`/`akar-components`, platform adapter in `akar-winit`.** This is the pattern both reference projects independently converged on (egui: core-crate + optional-feature `egui-winit`; gpui: core-crate + separate per-platform crates behind a narrow trait). It requires no `accesskit_macos`/`accesskit_windows`/`accesskit_unix`/`accesskit_winit` dependency in `akar-core` or `akar-components` at all — those crates only need to produce plain data (role/label/state/bounds per node), with zero knowledge of `accesskit` types if akar defines its own minimal intermediate struct, or a direct dependency on the `accesskit` core crate (not the platform adapters) if akar adopts its types verbatim to avoid a translation layer. `akar-winit` becomes the only crate that depends on `accesskit_winit` (feature-gated, matching `egui-winit`'s `accesskit = ["dep:accesskit_winit"]` pattern at `egui-winit/Cargo.toml:29-30`), and is the only crate that ever touches a live window handle for accessibility purposes — consistent with `AGENTS.md`'s existing rule "Do not add windowing... to `akar-core` or `akar-components`." Cost: non-winit consumers (raw `akar.h`, or a Rust app driving its own window) get a tree but no adapter, and must either (i) wire their own adapter against an exposed tree-walk API, or (ii) go without. This is an *explicit, acceptable* gap per Task 6 below, not a blocker for the winit path.
- **Option (b): platform adapters shipped inside `libakar` itself, gated by build features.** This would mean `akar-c-api` (or a new crate) statically links `accesskit_macos`/`accesskit_windows`/`accesskit_unix` and internally owns/attaches to whatever window handle the C caller supplies (raw `HWND`/`NSView`/`xcb_window_t` — akar's C ABI would need a new "give me your platform window handle" entry point, something it currently has zero of; `akar-c-api/src/lib.rs` has no window-handle-accepting function today, confirmed by grep). This directly contradicts the C ABI contract's "flat C API, no callbacks unless the developer opts in" philosophy (`AGENTS.md` → C ABI contract) — AccessKit's `ActionHandler` is fundamentally callback-shaped, and three vendored platform adapters compiled into every `libakar` build is a much larger permanent dependency-graph and binary-size commitment (`accesskit_macos` alone pulls in Objective-C/AppKit bridging; `accesskit_windows` pulls in `windows-rs` UIA bindings; `accesskit_unix` pulls in a dbus stack) than akar has taken on for any feature to date. It also does not remove the window-ownership problem, it just relocates it into C-ABI surface area (the caller must still hand akar a real platform window handle, which is exactly the kind of window-lifecycle entanglement `DEVELOP.md`'s "What akar does NOT own" section exists to avoid).
- **Recommendation: Option (a).** It is the only option consistent with `DEVELOP.md`'s "What akar does NOT own" (window, event loop) and with the "flat C API, no callbacks unless opted into" rule in `AGENTS.md`. It also has two independent production precedents (egui, gpui) doing the exact same split, which de-risks the design. Option (b) is not ruled out forever — it is explicitly listed as a Task 6 deferral, revisitable once the winit-only path is proven and there is real non-Rust-consumer demand.

### Task 3 — Semantic Tree Emission Prototype (Single Component)

**Status:** Done (design sketch only — the code below is illustrative, not integrated into any crate; it was written directly into this epic and never compiled)
**Readiness:** N/A — design task, complete. Treat the sketch as a starting point for Task 8, not a frozen spec; the one known defect is `parent_id`, addressed in Task 10.

The sketch below mirrors `DrawList`'s existing shape (`crates/akar-core/src/draw_list.rs:47-86`) — a plain frame-scoped `Vec`-backed struct with `begin_frame`/push/read, cleared every frame — rather than anything AccessKit-specific, so it stays a `akar-core` concept with no `accesskit` crate dependency. A translation layer to `accesskit::TreeUpdate` would live in `akar-winit` (per Task 2's Option (a)), consuming `A11yTree` and producing real `accesskit::Node`/`accesskit::TreeUpdate` values.

```rust
// crates/akar-core/src/a11y.rs — DESIGN SKETCH, NOT IMPLEMENTED.
// Mirrors DrawList's frame-scoped Vec + begin_frame/clear discipline.

/// Minimal role taxonomy for a first implementation pass (see Task 6).
/// Deliberately a strict subset of accesskit::Role, expanded incrementally
/// as components gain emission — not a 1:1 mirror of AccessKit's full enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yRole {
    Button,
    TextInput,
    CheckBox,
    RadioButton,
    Switch,
    Link,
    Label,
    GenericContainer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct A11yState {
    pub focused: bool,
    pub disabled: bool,
    pub checked: Option<bool>, // None = not applicable (e.g. Button); Some for CheckBox/Switch/RadioButton
    pub pressed: Option<bool>, // transient press state, mirrors hover/press already computed for paint
}

/// One semantic node. `id` is akar's existing `widget_id`/`widget_id_keyed` —
/// reused verbatim, not a new identity scheme (see Research: gpui converged
/// on the same reuse-existing-stable-id approach via GlobalElementId).
pub struct A11yNode {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub role: A11yRole,
    pub label: String, // caller-supplied or component-internal text; never inferred from pixels
    pub bounds: [f32; 4], // same [x, y, w, h] convention as QuadCall::rect / Layout::rect
    pub state: A11yState,
}

/// Frame-scoped semantic tree, sibling to `DrawList` on `AkarCore`.
/// Lifecycle mirrors DrawList exactly: begin_frame clears it, components
/// push_node during paint, end_frame hands the finished Vec to whatever
/// consumes it (a11y-enabled akar-winit bridge, or nothing if inactive).
pub struct A11yTree {
    nodes: Vec<A11yNode>,
    active: bool, // set once per frame by the platform layer; false = don't bother building
}

impl A11yTree {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), active: false }
    }

    pub fn begin_frame(&mut self, active: bool) {
        self.nodes.clear();
        self.active = active;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// No-op when inactive, so components can call this unconditionally
    /// without an `if core.a11y.is_active()` guard at every call site.
    pub fn push_node(&mut self, node: A11yNode) {
        if self.active {
            self.nodes.push(node);
        }
    }

    pub fn nodes(&self) -> &[A11yNode] {
        &self.nodes
    }
}

// --- Hypothetical button.rs integration (NOT a real edit to button.rs) ---
//
// Inserted into button_styled, after the existing hover/pressed/clicked
// computation (button.rs:83-85) and after the draw_list pushes (button.rs:124-159),
// using values already computed for paint — no new state, no extra rect query:
//
//     core.a11y.push_node(A11yNode {
//         id: layout.widget_id(node_id),      // same id already used for the text buffer (button.rs:139)
//         parent_id: None,                     // parent linkage: deferred, see Task 6/8
//         role: A11yRole::Button,
//         label: label.to_string(),            // the same `label: &str` already passed in
//         bounds: rect,                        // the same `rect` already resolved from layout.rect(node_id)
//         state: A11yState {
//             focused: core.input.focused_id == Some(layout.widget_id(node_id)),
//             disabled: false,                  // button has no disabled concept yet — see Task 6 deferrals
//             checked: None,
//             pressed: Some(pressed),           // reuses the `pressed` bool already computed at button.rs:84
//         },
//     });
```

Lifecycle fit, confirmed against `DEVELOP.md`'s construct/compute/paint model: the sketch above plugs in at the same point `button_styled` already submits draw calls — inside paint, after `layout.rect(node_id)` has been read but with no writes to `Layout` at all. It satisfies "paint is read-only on `Layout`" for the same reason the draw list itself does: `A11yTree`, like `DrawList`, is a field on `AkarCore`, not on `Layout`, so pushing to it during paint is exactly as legal as pushing a `QuadCall` is today. No change to the construct/compute/paint contract is needed.

Generalizing beyond `button`, per-component needs identified by reading `button.rs`, `text_input.rs`, and `theme.rs` together:
- **Role taxonomy**: components map roughly 1:1 to AccessKit roles already (`Button`, `TextInput` for `text_input`/`textarea`, `CheckBox`, `RadioButton`, `Switch`, `Link`) — akar does not need to invent new semantics, just wire the mapping.
- **Label source**: must be caller-supplied, never inferred from visual/rendered text. `button` already takes `label: &str` as a parameter distinct from any glyph data — trivial to reuse. Corrected during review of this pass: `checkbox` (`checkbox.rs:12`, `label: &str`) and `radio_group` (`radio.rs:11`, `labels: &[&str]`) **already** take a caller-supplied rendered-text label today — these can be reused verbatim as the accessible name, the same way `button`'s label is reused, no new parameter needed. Only `switch` (`switch.rs:7-13`) has no text parameter of any kind and will need a new caller-supplied `label`/`aria_label`-equivalent argument or builder field that does not exist today — this is real new component-API surface for `switch` specifically, not free, but it is a single-component gap, not a three-component one.
- **State source**: `hovered`/`pressed`/`clicked`/`focused` are already computed identically for paint in every interactive component (per `AGENTS.md`'s Component contract, step 2) — zero new computation needed, just forwarding into `A11yState`.
- **Parent/child linkage**: not sketched above (`parent_id: None`) because akar's `Layout` tree already has real parent pointers (`Layout` stores `parents: HashMap<NodeId, NodeId>`, `akar-layout/src/lib.rs:37`) that could supply this via `layout.widget_id(parents[&node_id])`, but wiring that up for every component call site is real work deferred to Task 8/9, not resolved by this prototype.

### Task 4 — Keyboard Focus Order Audit

**Status:** Done — re-verified 2026-08-12 (zero hits for `focus_next`/`focus_prev`/`tab_order`/`TabOrder`/`tab_index` workspace-wide; `focused_id` writers limited to `text_input.rs:64,71,150`, `textarea.rs:83,90,174`, `data_list.rs:260`)
**Readiness:** N/A — audit task, complete. Its conclusions are implemented by Task 9.

- Current focus model, confirmed by reading `crates/akar-core/src/input.rs:157` and every call site: `InputState::focused_id: Option<u64>` is a single global "currently focused widget" value, set exclusively by **mouse click** — `text_input.rs:64`, `textarea.rs:83`, and `data_list.rs:260` all set it inside a click-handling branch (e.g. `text_input.rs:64` sets `core.input.focused_id = Some(id_u64)` when the component detects a click on itself). `canvas.rs:743-744` has a dedicated test (`"canvas text must not set focused_id"`) confirming canvas display text deliberately never participates in focus, consistent with `DEVELOP.md`'s canvas-text-is-display-only rule.
- Only `text_input`, `textarea`, and (implicitly, for row selection) `data_list` currently read or write `focused_id` at all. `button`, `checkbox`, `switch`, `radio`, `link`, `select`, `dropdown`, `slider`, and every other interactive component compute `hovered`/`pressed`/`clicked` from mouse hit-testing (`core.input.is_hovering`/`is_pressed`/`is_clicked`, e.g. `button.rs:83-85`) but never touch `focused_id` — meaning most of akar's interactive catalog has **no keyboard-focus concept at all** today, only mouse interaction.
- `Key::Tab` is decoded as a named key by the winit bridge (`akar-winit/src/lib.rs:26`, mapping `winit::keyboard::NamedKey::Tab`) and exists in `akar_core::Key`, but grepping all of `akar-components` and `akar-core` for `focus_next`, `focus_prev`, `tab_order`, `TabOrder`, and `tab_index` returns zero matches anywhere in the codebase. Nothing currently consumes a `Tab` keypress to move focus between widgets.
- **Gap assessment: focus order is neither visual/DOM-order equivalent nor an explicit tab-order concept — it effectively does not exist as a keyboard-navigable sequence.** The only "order" today is "whichever single widget was last clicked," which is necessary-but-nowhere-near-sufficient scaffolding for accessibility (a screen reader / switch-access user who cannot click needs Tab/Shift+Tab to move a live cursor through interactive elements in a predictable order, independent of mouse activity). This confirms and sharpens the epic's original framing ("focus order without semantic roles/labels is not usable by a screen reader") — the reverse is also true here: akar currently has semantic-adjacent state (`focused_id`) but no keyboard-driven order to walk it with, and most components don't participate in focus at all. A real implementation pass needs: (1) an ordered focus sequence (construction-order-of-widget-registration is the cheapest starting definition, matching how `Layout`'s tree is built depth-first during construct), (2) `Tab`/`Shift+Tab` handling that advances/retreats `focused_id` along that sequence, threaded through every interactive component uniformly rather than the current per-component ad hoc click-only assignment, and (3) `checkbox`/`switch`/`radio`/`button`/`link` gaining the same `focused_id` participation `text_input`/`textarea` already have. This is scoped as new Task 9 below.

### Task 5 — Theme Contrast Check

**Status:** Done — **all 32 ratios independently recomputed from the current `theme.rs` hex values on 2026-08-12 and confirmed correct to two decimal places; no numbers changed**
**Readiness:** N/A — measurement task, complete. Its failures are fixed by Task 7, which now carries concrete verified replacement hex values.

Token values read from `crates/akar-components/src/theme.rs:41-115` (`AKAR_THEME_DARK` at lines 41-77, `AKAR_THEME_LIGHT` at lines 79-115). Contrast ratios computed directly from the RGBA hex values using the standard WCAG relative-luminance formula (`L = 0.2126*R + 0.7152*G + 0.0722*B` on gamma-linearized sRGB channels, `contrast = (L_lighter + 0.05) / (L_darker + 0.05)`) — not estimated. WCAG 2.1 AA minimums: 4.5:1 for normal text, 3:1 for large text (≥18pt/24px, or ≥14pt/18.66px bold) and for non-text UI-component boundaries.

**`AKAR_THEME_DARK`:**

| Pair | Ratio | Normal text (≥4.5) | Large text / UI (≥3.0) |
|---|---|---|---|
| `base_content` on `base_100` | 19.06:1 | PASS | PASS |
| `base_content` on `base_200` | 16.97:1 | PASS | PASS |
| `base_content` on `base_300` | 14.27:1 | PASS | PASS |
| `muted_content` on `base_100` | 7.76:1 | PASS | PASS |
| `muted_content` on `secondary` | 5.71:1 | PASS | PASS |
| `primary_content` on `primary` (button text) | 17.85:1 | PASS | PASS |
| `secondary_content` on `secondary` | 13.98:1 | PASS | PASS |
| `accent_content` on `accent` | 13.98:1 | PASS | PASS |
| `neutral_content` on `neutral` | 14.27:1 | PASS | PASS |
| `info_content` on `info` | 3.68:1 | **FAIL** | PASS |
| `success_content` on `success` | 2.28:1 | **FAIL** | **FAIL** |
| `warning_content` on `warning` | 2.15:1 | **FAIL** | **FAIL** |
| `error_content` on `error` | 3.76:1 | **FAIL** | PASS |
| `primary` on `base_100` (as a UI-component boundary, e.g. a focus ring or unfilled outline button border) | 1.11:1 | **FAIL** | **FAIL** |
| `secondary` on `base_100` | 1.36:1 | **FAIL** | **FAIL** |

**`AKAR_THEME_LIGHT`:**

| Pair | Ratio | Normal text (≥4.5) | Large text / UI (≥3.0) |
|---|---|---|---|
| `base_content` on `base_100` | 19.90:1 | PASS | PASS |
| `base_content` on `base_200` | 18.10:1 | PASS | PASS |
| `base_content` on `base_300` | 15.68:1 | PASS | PASS |
| `muted_content` on `base_100` | 4.83:1 | PASS | PASS |
| `muted_content` on `base_200` | 4.40:1 | **FAIL** | PASS |
| `muted_content` on `secondary` | 4.41:1 | **FAIL** | PASS |
| `primary_content` on `primary` | 17.85:1 | PASS | PASS |
| `secondary_content` on `secondary` | 16.30:1 | PASS | PASS |
| `accent_content` on `accent` | 16.30:1 | PASS | PASS |
| `neutral_content` on `neutral` | 17.18:1 | PASS | PASS |
| `info_content` on `info` | 3.68:1 | **FAIL** | PASS |
| `success_content` on `success` | 2.28:1 | **FAIL** | **FAIL** |
| `warning_content` on `warning` | 2.15:1 | **FAIL** | **FAIL** |
| `error_content` on `error` | 3.76:1 | **FAIL** | PASS |
| `secondary` on `base_100` (UI boundary) | 1.10:1 | **FAIL** | **FAIL** |
| `error` on `base_100` (e.g. error-state border) | 3.76:1 | n/a | PASS |
| `info` on `base_100` | 3.68:1 | n/a | PASS |

**Findings:**
- `info`, `success`, `warning`, and `error` are the semantic "status" colors and their paired `*_content` tokens (used as text/icon color on top of a status-colored fill, e.g. a solid status badge or a toast) **fail normal-text AA in both themes, identically** — the status colors themselves are unchanged between `AKAR_THEME_DARK` and `AKAR_THEME_LIGHT` (`info: 0x3b82f6ff`, `success: 0x22c55eff`, `warning: 0xf59e0bff`, `error: 0xef4444ff` are byte-identical in both presets, confirmed by direct comparison of `theme.rs:55-62` and `theme.rs:93-100`), so this is a single fix applicable to both themes at once, not two separate fixes. `success_content`/`warning_content` fail even the *large-text/UI* 3:1 minimum (2.28:1 and 2.15:1) — these are the most severe failures found. **Verified 2026-08-12: this is actively shipping, not hypothetical.** `crates/akar-components/src/badge.rs:65-68` paints `(theme.success, theme.success_content)` (and the `warning`/`error`/`info` equivalents) as fill + text for the solid badge variants, and `crates/akar-components/src/toast.rs:66-69` fills the toast with the raw status color while `toast.rs:93` draws hardcoded `0xFFFFFFFF` text on top of it. A success badge or toast is therefore at 2.28:1 and a warning one at 2.15:1 in the shipped build. `alert.rs:44-49` is **not** affected — it fills with `base_200` and uses the status color only for a `dim_color(_, 0.5)` border.
- `primary`/`secondary` used as a **UI-component boundary color** (e.g. an outline-variant button border, or a focus ring drawn in `primary`) against `base_100` fails even the relaxed 3:1 non-text minimum in the dark theme (`primary` on `base_100` = 1.11:1) and light theme (`secondary` on `base_100` = 1.10:1) — both are near-invisible-contrast pairs, because in both themes one of `primary`/`secondary` is deliberately close in luminance to `base_100` (dark theme: `primary = 0x0f172a` is a near-black navy close to `base_100 = 0x09090b`; light theme: `secondary = 0xf1f5f9` is a near-white slate close to `base_100 = 0xffffff`). This is a real usability concern for low-vision users trying to see an outline-variant button's border or an unfocused-but-visible interactive-element boundary, though `button.rs` does not currently draw a dedicated focus ring at all (only border/fill color changes on hover/press), so this specific failure mode is latent rather than actively shipping a broken visual today.
- Everything text-on-`base_*` (the overwhelming majority of actual body text, `base_content`/`muted_content` on `base_100`/`base_200`/`base_300`) passes comfortably in both themes, several times over the 4.5:1 minimum — the core reading experience is not at risk.
- **Fix candidates (independent of the semantic-tree effort, cheap, no architecture change — just token value edits in `theme.rs`):** darken/lighten `success` and `warning` (and their fixed white `*_content` values, or vice versa) enough to clear at least the 3:1 non-text minimum for large/UI use and ideally 4.5:1 for any direct text-on-status-fill usage; consider a dedicated darker/lighter shade for `info_content`/`error_content` use rather than pure white; and, if outline-variant button borders or focus rings are meant to be visible against `base_100`, `primary`/`secondary` need a genuinely distinct luminance step from `base_100` in whichever theme currently pairs them near-identically, or a new dedicated `focus_ring`/`border_visible` token distinct from `primary`/`secondary` entirely. These are token-value-only changes with no crate/architecture impact, and are separable from — and could ship well before — the semantic-tree work.

### Task 6 — Scope Proposal for First Implementation Pass

**Status:** Done — proposal below; converted into implementation Tasks 7-14
**Readiness:** N/A — scoping task, complete. Version pinning for the winit path is resolved in the Review section and restated in Task 11.

**Architecture (from Task 2):** Option (a) — semantic-tree emission (`A11yTree`, sketched in Task 3) lives in `akar-core`, populated by `akar-components` during paint, with zero `accesskit`/`accesskit_winit` dependency in either crate. The only new dependency (`accesskit_winit`, feature-gated) is added to `akar-winit`, matching the existing "windowing is optional and isolated to `akar-winit`" rule in `AGENTS.md`'s crate responsibility table.

**Platform adapter to start with: `accesskit_winit`, which covers macOS/Windows/Linux uniformly through one dependency.** `accesskit_winit` (vendored locally at `~/.cargo/registry/.../accesskit_winit-0.32.2`) is not itself platform-specific — it internally selects the right platform adapter (macOS AX / Windows UIA / Linux AT-SPI) based on target OS and exposes one `Adapter` type to the winit-hosted caller, which is exactly the pattern `egui-winit` already uses in production (`egui-winit/Cargo.toml:29-30`, `egui-winit/src/lib.rs:183-197`). Since `akar-winit` already targets exactly the same three desktop platforms via `winit` itself (`DEVELOP.md`'s dependency table lists `winit` restricted to `akar-winit` and examples, no OS-specific carve-outs), there is no reason to hand-pick a single OS first — `accesskit_winit` gets all three for the same integration cost that egui already paid. This differs slightly from the epic's original phrasing ("recommend starting with whichever platform is the primary dev target") because the reference implementation shows that granularity isn't actually where the complexity lives; the complexity is in `akar-winit` gaining a stateful bridge type at all (see below), not in per-OS adapter selection.

**Role taxonomy for the first component set** (mirrors the `A11yRole` sketch in Task 3): `Button`, `TextInput` (covers both `text_input` and `textarea`), `CheckBox`, `RadioButton`, `Switch`, `Link`. These six were chosen because: (1) they are the components epic 018 and the existing `focused_id` mechanism already touch at least partially (`text_input`, `textarea`), or that have unambiguous, uncontroversial AccessKit role mappings with no design judgment calls (`button`, `checkbox`, `radio`, `switch`, `link`); (2) they cover the interaction patterns (click-to-activate, click-to-toggle, text-entry, navigate) that the large majority of a real desktop UI's interactive surface is built from, per `README.md`'s own framing of the catalog ("buttons, cards, inputs, tables, modals, drawers, sliders, toggles"); (3) they exclude components whose accessible semantics are legitimately harder to get right on a first pass (`slider` — needs numeric-value/range semantics; `select`/`dropdown` — needs `ComboBox`/`ListBox` parent-child semantics and open/closed state; `data_list`/`table` — needs virtualization-aware, only-report-visible-rows semantics; `modal`/`drawer` — needs focus-trap and `role="dialog"` live-region behavior). Those are explicitly deferred, not silently dropped — see new Task 10/13 below for a follow-on pass.

**Explicit deferrals** (per this epic's existing "Notes for Future Work" and `AGENTS.md`'s "What NOT to do" — accessibility scaffolding beyond a proven minimal slice stays out of `akar-core`/`akar-components` until the six-role slice above ships and is validated with a real screen reader):
- Full 30+ component coverage — only the six roles above ship first; `slider`, `select`/`dropdown`, `data_list`/`data_item`, `modal`/`drawer`, `tabs`, `steps`, `toast`, `tooltip`, canvas/portal content, and all purely-decorative components (`avatar`, `badge`, `skeleton`, `separator`, `progress` as a display-only bar) are deferred.
- Non-winit platform adapters (a Rust app driving its own window without `akar-winit`, or any non-Rust consumer) — only the `akar-winit` + `accesskit_winit` path ships. `A11yTree`'s data remains readable in principle by anyone (it's a plain akar-core struct), but no second adapter integration is built or documented in this pass.
- C-ABI-exposed adapters (Option (b) from Task 2, or any `akar.h` surface for walking the tree from C) — fully deferred; no new `extern "C"` functions for accessibility in this pass. `akar-c-api/src/lib.rs` gets no changes.
- Live-region/dynamic-announcement support (e.g. announcing toast notifications or async content changes to a screen reader proactively, independent of focus) — AccessKit supports this via node updates outside the focus path, but it requires akar to model "this changed and should be announced even though nothing is focused there," which is new design work not covered by the button-shaped prototype in Task 3.
- Full action coverage (increment/decrement, set-value, scroll, and rich text-edit actions) is deferred. The first pass must still queue and deliver the minimum `Focus` and activation/default actions for its six advertised roles; otherwise assistive technology can discover controls but cannot operate them. A frame-scoped queue lets the platform callback enqueue requests while components continue to mutate caller-owned state synchronously during the next developer-driven frame (Tasks 9 and 11).
- Keyboard focus-order infrastructure (Tab/Shift+Tab traversal, an explicit focus sequence) is a **prerequisite**, not a deferral, per Task 4's findings — it is scoped as Task 9 below and should land before or alongside the semantic-tree emission work, since a tree with no way to move `focused_id` between nodes via keyboard is not meaningfully usable by the assistive-tech users this epic is for.
- Theme contrast fixes (Task 5) are independent and lower-risk; they can ship on their own timeline, not gated on any of the above (scoped as Task 7 below, since it requires no architecture and could go out immediately).
- Reduced-motion / high-contrast OS-level preference detection remains out of scope for this epic entirely, as already noted in "Notes for Future Work."

---

## Implementation Tasks (for a future implementation agent)

These are concrete, file/crate-level tasks derived from Tasks 1-6 above. None of them have been implemented — this research pass made no changes outside this epic file. Suggested order: Task 7 first (independent, zero architecture risk), then Task 8 and Task 9 in parallel (both are prerequisites for Task 11), then Tasks 10-14 in roughly listed order.

### Task 7 — Fix Failing Theme Contrast Pairs

**Status:** Not Started
**Readiness:** Ready for implementation — concrete replacement hex values are supplied below and were computed and verified on 2026-08-12; a coding agent does not need to guess or search for values.

- Edit `crates/akar-components/src/theme.rs`. Both `AKAR_THEME_DARK` (lines 41-77) and `AKAR_THEME_LIGHT` (lines 79-115) currently share byte-identical `info`/`success`/`warning`/`error` values (`0x3b82f6ff`/`0x22c55eff`/`0xf59e0bff`/`0xef4444ff`), so fixing the failing `*_content`-on-status-color pairs found in Task 5 (`success_content` on `success` = 2.28:1, `warning_content` on `warning` = 2.15:1, `info_content` on `info` = 3.68:1, `error_content` on `error` = 3.76:1 — all below the 4.5:1 normal-text minimum, with `success`/`warning` also below the 3:1 large-text/UI minimum) is a single edit applied to both presets.
- **Recommended token values (computed and verified 2026-08-12; apply identically to both presets, since the status tokens are byte-identical across them).** This set brings all four `*_content`-on-status pairs above the 4.5:1 normal-text AA minimum while keeping every status color above the 3:1 non-text minimum against *both* themes' `base_100`, so solid badges, toasts and any status-colored fill or border pass at any font size:

  | Token | Current | Proposed | white `*_content` on it | vs dark `base_100` (`0x09090b`) | vs light `base_100` (`0xffffff`) |
  |---|---|---|---|---|---|
  | `info` | `0x3b82f6ff` | `0x2563ebff` | 3.68 → **5.17** | 5.41 → 3.85 | 3.68 → **5.17** |
  | `success` | `0x22c55eff` | `0x15803dff` | 2.28 → **5.02** | 8.73 → 3.97 | 2.28 → **5.02** |
  | `warning` | `0xf59e0bff` | `0xb45309ff` | 2.15 → **5.02** | 9.26 → 3.96 | 2.15 → **5.02** |
  | `error` | `0xef4444ff` | `0xdc2626ff` | 3.76 → **4.83** | 5.29 → 4.12 | 3.76 → **4.83** |

  All four `*_content` values stay `0xffffffff`; only the four fill colors change. Note the deliberate trade: the fills get darker, so their contrast *against the dark theme's* `base_100` drops (8.73 → 3.97 for `success`, worst case 3.85 for `info`) — every one of those still clears the 3:1 non-text minimum, so a status-colored border or bar remains visible in the dark theme, but the margin is thinner than today and a further darkening step would break it. Do not darken past these values without recomputing.
- **Alternative if the brighter fills are considered visually load-bearing:** keep the current fill hexes and change the four `*_content` tokens from `0xffffffff` to a dark ink such as `0x09090bff` (the light theme's `base_content`). That yields `info` 5.41, `success` 8.73, `warning` 9.26, `error` 5.29 — all comfortably AA — at the cost of a visual style change (dark text on saturated fills rather than white). Pick one approach; do not mix.
- Whichever approach is taken, re-run the same luminance/contrast computation used in Task 5 (formula documented there) against the final values before committing — do not eyeball hex values.
- Separately: `primary` on `base_100` in the dark theme (1.11:1) and `secondary` on `base_100` in the light theme (1.10:1) fail even the 3:1 UI-component-boundary minimum. Decide whether this needs a fix (it matters only if/when a focus ring or outline-variant border is drawn in `primary`/`secondary` directly against `base_100` — check `button.rs`'s `ButtonVariant::Outline` path and any future focus-ring implementation from Task 9) or a new dedicated token (e.g. `focus_ring`) that doesn't reuse `primary`/`secondary`, sidestepping the conflict with those tokens' other roles. If a `focus_ring` token is added, note that no single value works for both presets (dark `base_100` is near-black, light `base_100` is white), so it must be a per-preset value. Verified candidates: dark preset `0x60a5faff` (7.83:1 against `0x09090b`) and light preset `0x2563ebff` (5.17:1 against `0xffffff`) — both far above the 3:1 UI-boundary minimum, and both blue so the two presets read as the same affordance.
- Add or extend unit tests in `theme.rs` (or a new `#[cfg(test)]` module there) asserting computed contrast ratios for the token pairs fixed here stay at or above their WCAG target, so a future token-value change can't silently regress contrast. `akar-components` has no existing contrast-checking test utility — this task should add one (a small standalone `fn contrast_ratio(u32, u32) -> f32` following the sRGB-linearization formula, not a new crate dependency).
- No architecture impact; independent of every other task in this section; safe to ship alone.

### Task 8 — `A11yTree` and `A11yNode` in `akar-core`

**Status:** Not Started
**Readiness:** Blocked — the frame-buffer mechanics are ready, but the proposed node contract is not sufficient for Tasks 10-12. Resolve the semantic/action contract below before freezing public akar-owned types.

- Add `crates/akar-core/src/a11y.rs` implementing `A11yRole`, `A11yState`, `A11yNode`, and `A11yTree` per the Task 3 design sketch above (treat that sketch as a starting point, not a frozen spec — it has not been reviewed against real AccessKit round-tripping). `A11yTree` should mirror `DrawList`'s existing lifecycle exactly (`crates/akar-core/src/draw_list.rs:47-86`): `begin_frame`/clear, push-during-paint, read-after-`end_frame`.
- Wire `A11yTree` onto `AkarCore` (`crates/akar-core/src/context.rs:8-21`) as a new public field, cleared in `AkarCore::begin_frame` (`context.rs:62-67`) alongside `self.draw_list.begin_frame(...)`, and left in its finished state after `AkarCore::end_frame` (`context.rs:69-120`) for a consumer (initially: nothing consumes it yet — that's Task 11) to read.
- `A11yTree::begin_frame` should take an `active: bool` so the tree is only populated when accessibility is actually wanted (mirrors both egui's `is_accesskit_enabled` gate, `egui/src/context.rs:405`, and gpui's `active_this_frame`, `zed/crates/gpui/src/window/a11y.rs`). For this task alone (no platform adapter wired yet), default `active` to `true` unconditionally or expose a manual toggle on `AkarCore` — the real activation signal (screen reader presence) only becomes available once Task 11 wires `accesskit_winit`.
- No dependency on the `accesskit` crate itself in this task — keep `A11yNode`/`A11yRole` as plain akar-owned types (per Task 2's Option (a) rationale: `akar-core`/`akar-components` should not need to know about `accesskit` types at all). The translation to real `accesskit::Node`/`TreeUpdate` values happens entirely in `akar-winit` (Task 11).
- Expand the Task 3 sketch before coding. `role`/`label`/`bounds` plus three booleans cannot faithfully translate the selected six roles: text inputs need at least current value, text selection, multiline/read-only state, and supported actions; interactive controls need supported actions (`Focus`, `Default`, and role-specific actions as applicable). AccessKit only exposes an operation to assistive technology when the node advertises the corresponding action. Decide which properties/actions are first-pass requirements and represent them in the windowing-independent node type rather than trying to reconstruct them in `akar-winit`.
- Define an explicit frame root. Every `TreeUpdate.tree.root` must identify a node present in the update, and that root must own the top-level accessible children. The current sketch only emits component nodes with optional layout parents; Task 11 cannot create a valid hierarchy from that unless `A11yTree` either owns a synthetic root ID/node or exposes enough top-level information for the adapter to synthesize one deterministically. Xilem/Masonry demonstrates the required shape in `masonry_core/src/passes/accessibility.rs`: a `Role::Window` node is included and points at the content root.
- Add unit tests in `a11y.rs` following the existing `draw_list.rs` test style (`crates/akar-core/src/draw_list.rs:197-329`): frame-clearing behavior, inactive-frame no-op behavior, and a basic push/read round trip. Test `A11yTree` **directly**, not through `AkarCore::mock()`: verified 2026-08-12, `AkarCore::mock()` (`context.rs:48-60`) calls `instance.request_adapter(...)` / `adapter.request_device(...)` and panics with "no suitable adapter" without a real GPU, which contradicts `AGENTS.md`'s "No live GPU in CI" rule. `A11yTree` is a plain `Vec`-backed type and needs no `AkarCore` at all — the `draw_list.rs` tests already construct a bare `DrawList` this way. Reserve `mock()` for the component-level assertions in Task 10 that genuinely need a whole `AkarCore`.

### Task 9 — Keyboard Focus Traversal (Tab / Shift+Tab)

**Status:** Not Started
**Readiness:** Blocked — traversal mechanics are specified, but keyboard operation and assistive-action delivery must be designed with them before this can serve as the accessibility focus foundation.

- Prerequisite work identified by Task 4: akar currently has no way to move `InputState::focused_id` (`crates/akar-core/src/input.rs:157`) via keyboard at all — only mouse clicks set it, and only in `text_input.rs`, `textarea.rs`, and `data_list.rs`.
- Design an ordered focus sequence. The cheapest starting definition is construction order: since `Layout`'s tree is built depth-first during the construct phase (`DEVELOP.md` → Component lifecycle), a registration-order list of focusable `widget_id`s built up as components declare themselves focusable is a reasonable v1 (matches visual/DOM tab order in the common case, which is what most other UI toolkits do by default too). This likely needs a new `Vec<u64>` (or similar) living on `InputState` or a new sibling struct, populated during paint the same way `A11yTree` is (each focusable component appends itself), and reset every `begin_frame` like everything else frame-scoped in `akar-core`.
- Add `Key::Tab`/`Shift+Tab` handling (physically decoded already at `akar-winit/src/lib.rs:26`, but nothing currently consumes it for focus movement) that advances/retreats `focused_id` along that sequence. This is new logic in `akar-core` (likely `InputState` or a new `FocusState`), not `akar-winit` — `akar-winit`'s job stays limited to translating the raw key event, per its existing narrow scope (`process_window_event`, `akar-winit/src/lib.rs:89-130`).
- Extend `button`, `checkbox`, `switch`, `radio`, `link` (the six roles chosen in Task 6, minus `text_input`/`textarea` which already participate) to read/write `focused_id` the same way `text_input.rs`/`textarea.rs` already do (`text_input.rs:64,67,71,74`), and to register themselves in the new focus sequence during paint.
- Focus without operation is insufficient. Specify and test keyboard activation in the same focus design: Enter/Space for buttons as appropriate, Space for checkbox/switch/radio, and Enter for links. Also specify a frame-scoped queue for AccessKit `ActionRequest`s (`Focus`, `Default`, text-selection actions, and later role-specific value actions) so requests delivered by `akar-winit` can be consumed synchronously by components on the next developer-driven frame. This preserves akar's synchronous component contract without requiring AccessKit's callback thread to mutate caller-owned component state.
- **Verified non-conflict with epics 017/018 (2026-08-12).** Epic 018 filters tab out of committed text before `InputState::push_char` (`akar-winit/src/lib.rs:107-113`, via `is_committed_text_char`), so a Tab press never reaches `text_input`/`textarea` as a character; a workspace grep for `Key::Tab` hits only `akar-winit/src/lib.rs:26,166-167` and `akar-c-api/src/lib.rs:1342`, i.e. nothing consumes it. Tab arrives solely as a `KeyEvent` pushed at `akar-winit/src/lib.rs:114-122`. Claiming Tab for focus traversal therefore takes it from nobody. Epic 017's ADR-016a constraint does apply: the focus sequence must store `widget_id`/`widget_id_keyed` values, **never** an index into a per-frame node list, or virtualized-list scrolling will move focus to the wrong record exactly as ADR-016a describes.
- **Frame-ordering decision that must be made explicitly.** Focusable components register themselves during *paint*, but the Tab keypress is available to `akar-core` at the *start* of the frame — so a Tab pressed in frame N can only be resolved against the sequence built during frame N-1. Either (i) keep the previous frame's sequence alive across `begin_frame` (double-buffer it) and resolve Tab immediately against that, or (ii) defer the resolution by one frame. Option (i) matches egui and is preferred. Whichever is chosen, do not clear the sequence in `begin_frame` the way `DrawList` clears its calls without first preserving the previous frame's copy.
- This task is a prerequisite for Task 11 (semantic-tree consumers need a working `focus: NodeId`-equivalent to report), and should land before or alongside Task 8/10, not after — per Task 6's explicit note that this is a prerequisite, not a deferral.

### Task 10 — Wire the Six-Role Component Set into `A11yTree`

**Status:** Not Started
**Readiness:** Blocked — depends on Tasks 8 and 9 resolving the node property/action and action-queue contracts. The layout-parent prerequisite is otherwise fully specified.

- Once Task 8 (`A11yTree`) and Task 9 (focus participation) exist, extend `button.rs`, `text_input.rs`, `textarea.rs`, `checkbox.rs`, `switch.rs`, `radio.rs`, `link.rs` to call `core.a11y.push_node(...)` during paint, following the `button.rs` sketch in Task 3 exactly (reusing already-computed `hovered`/`pressed`/`focused` state, `layout.rect(node_id)`, and `layout.widget_id(node_id)` — no new per-frame computation beyond what paint already does).
- Corrected during review of this pass: `checkbox` (`checkbox.rs:12`) and `radio_group` (`radio.rs:11`) already take a caller-supplied rendered-text `label`/`labels` parameter — reuse that value directly as the accessible name, no new component-API surface needed for either. `switch.rs:7-13` is the only one of the three with no text parameter at all; this task must add one for `switch` only (e.g. a new `label: &str` argument) since AccessKit labels must never be inferred from rendered pixels (per Task 1's integration contract and Task 3's finding). This is a small but real, back-compat-breaking-or-additive component API change for `switch`; decide the exact shape (new parameter vs. new `_labeled` variant function, following the `button`/`button_styled` split precedent already in `button.rs:52-62`) as part of this task, not assumed.
- Populate `A11yNode::parent_id` from `Layout`'s existing child-to-parent map rather than leaving it `None` as the Task 3 sketch did — real parent/child linkage is needed for AccessKit's `Node::children` field on the eventual `accesskit::Node`, which is built in Task 11 from this data. **Verified 2026-08-12: `parents: HashMap<NodeId, NodeId>` (`crates/akar-layout/src/lib.rs:37`) is a private field with no public accessor** — `grep "pub fn " crates/akar-layout/src/lib.rs` shows no `parent`/`ancestors` function, and the only read is the internal ancestor walk inside `Layout::rect` at `lib.rs:207`. The Task 3 sketch's `layout.widget_id(parents[&node_id])` does not compile from outside the crate. Add a one-line accessor to `akar-layout` first:

  ```rust
  pub fn parent(&self, node: NodeId) -> Option<NodeId> {
      self.parents.get(&node).copied()
  }
  ```

  The map itself is complete and correct: it is populated in `new_with_children` (`lib.rs:105`), `add_child` (`lib.rs:112`) and `set_children` (`lib.rs:118`), and cleaned up in `remove` (`lib.rs:123`), covering every node added through `Layout`'s public API. Note that not every layout node is an accessibility node, so `parent_id` should walk up until it finds an ancestor that emitted a node this frame (as egui's `find_accesskit_parent`, `egui/src/context.rs:595`, does), rather than blindly taking the immediate layout parent.
- Add `MockDrawList`-style tests (per `AGENTS.md` → Testing approach) asserting the right `A11yNode` is pushed with the right role/label/state for each of the six components, using `AkarCore::mock()` the same way `button.rs`'s existing tests do (`button.rs:169-258`).
- For `text_input` and `textarea`, assertions must cover value, selection/caret, multiline state, focus, and supported actions, not only role/label/state. For buttons/toggles/links, assert advertised actions agree with both keyboard handling and the ActionRequest path; otherwise a screen reader may announce a control that cannot be invoked.

### Task 11 — `accesskit_winit` Bridge in `akar-winit`

**Status:** Not Started
**Readiness:** Blocked — dependency compatibility is resolved, but the bridge lifecycle and action path are not. Depends on the contracts from Tasks 8 and 9 and must include the requirements below.

- `akar-winit` currently has zero window-handle-holding state (`crates/akar-winit/src/lib.rs` is 212 lines, entirely the stateless `process_window_event(input: &mut InputState, event: &WindowEvent)` function plus small pure key-mapping helpers). This task introduces a new stateful bridge type (e.g. `AkarWinitA11yBridge` or folded into a new `AkarWinitState` if one doesn't already exist elsewhere in the demo/example code) that owns an `accesskit_winit::Adapter`, gated behind a new Cargo feature (e.g. `accesskit`, matching `egui-winit`'s `accesskit = ["dep:accesskit_winit"]` pattern at `egui-winit/Cargo.toml:30`, with the optional dep declared as at `egui-winit/Cargo.toml:72`) so `akar-winit` has no new dependency by default.
- **Versions to pin, and winit compatibility — verified 2026-08-12, not a blocker.** Pin `accesskit_winit = "0.32"` (and `accesskit = "0.24"` if the bridge names the core types directly). akar pins `winit = "0.30"` at `Cargo.toml:20` and `Cargo.lock:3014-3016` resolves it to **winit 0.30.13**; `accesskit_winit-0.32.2/Cargo.toml:82-84` requires `winit = "0.30.5"` (`default-features = false`), which 0.30.13 satisfies under caret semantics, resolving to the same winit crate instance with no duplicate-version split. `accesskit_winit-0.32.2` itself depends on `accesskit = "0.24.0"` (`Cargo.toml:67-68`). Both crates are already vendored at `~/.cargo/registry/src/index.crates.io-*/`.
- **Feature-unification caveat to document in the feature's doc comment.** `accesskit_winit`'s default feature set (`accesskit_winit-0.32.2/Cargo.toml:36-44`) is `["accesskit_unix", "async-io", "rwh_06", "winit/x11", "winit/wayland"]`. Enabling the akar feature therefore turns on `winit/x11` and `winit/wayland` workspace-wide on Linux via cargo feature unification, and pulls in a dbus/async-io stack through `accesskit_unix` (`Cargo.toml:106-109`). This only affects builds that opt in, but it should be stated up front rather than discovered by a downstream consumer.
- Respect `accesskit_winit::Adapter::with_event_loop_proxy`'s documented sequencing constraint (`accesskit_winit-0.32.2/src/lib.rs:132-148`): the adapter must be constructed before the window is first shown (create window with `with_visible(false)`, construct the adapter, then show the window) — this is an ordering requirement on whatever example/demo code exercises this bridge (likely `examples/demo-rust`), not just the bridge itself.
- Call `Adapter::process_event(window, event)` for every incoming winit window event, before application handling, as required by `accesskit_winit-0.32.2/src/lib.rs:250-257`. Extend the bridge's event-loop API to handle `accesskit_winit::Event::{InitialTreeRequested, ActionRequested, AccessibilityDeactivated}` when using `with_event_loop_proxy`; `InitialTreeRequested` must trigger a full-tree update by the next display refresh even if akar would otherwise remain idle. The current task text's per-frame `update_if_active` call alone does not satisfy activation.
- Each frame, after `AkarCore::end_frame` (Task 8's `A11yTree` is now populated), translate `A11yTree::nodes()` into a real `accesskit::TreeUpdate` (per Task 1's findings: `nodes: Vec<(NodeId, Node)>`, `tree: Option<Tree>`, `tree_id: TreeId`, `focus: NodeId` — the `focus` field should be sourced from Task 9's focus-sequence state; use `TreeId::ROOT`, the nil UUID at `accesskit-0.24.0/src/lib.rs:670-676`, for a single-window app, and note that `accesskit::Node`'s fields are private so it must be built via `Node::new(role)` plus `set_*`/`push_child` setters) and call the adapter's update path (mirroring `egui-winit`'s `accesskit.update_if_active(|| update)`, `egui-winit/src/lib.rs:1168-1174`, so the translation closure only runs when accessibility is actually active).
- This is the only crate in this whole feature that should ever depend on `accesskit`/`accesskit_winit` directly, per Task 2's Option (a) recommendation — do not add either dependency to `akar-core` or `akar-components`.
- Do not defer action-request delivery. `with_event_loop_proxy` always produces `ActionRequested` events, and AccessKit's contract permits queued asynchronous handling. Translate those events into Task 9's frame-scoped action queue; components then consume them synchronously on the next frame. Full coverage of every AccessKit action remains deferred, but `Focus` and activation/default actions for the first-pass roles are required for the advertised semantics to be truthful and for switch-access/screen-reader users to operate controls.

### Task 12 — Demo/Example Wiring and Manual Verification

**Status:** Not Started
**Readiness:** Blocked — by Tasks 8-11. Once they land, demo wiring is ready; real platform announcement/action verification still needs a human-in-the-loop pass. `accesskit_consumer` can validate tree semantics but cannot prove VoiceOver/NVDA/Orca platform behavior.

- Wire Task 11's bridge into `examples/demo-rust` behind the same feature flag, so there is a real, runnable end-to-end path — the existing screenshot/script debug toolchain (`AGENTS.md` → Debug toolchain) cannot verify screen-reader behavior directly (it's pixel-based), so this task needs a different verification method: manually run `demo-rust` with a real screen reader active (VoiceOver on macOS, NVDA on Windows, Orca on Linux) and confirm the six wired components (Task 10) are announced with correct role/label/value/state, can be focused and invoked through assistive-technology actions, and that Tab/Shift+Tab (Task 9) moves keyboard focus through them in a sane order. Do not require a visually rendered "screen-reader focus indicator": assistive technologies may maintain a separate virtual cursor; verify announcements/actions and akar's own keyboard-focus presentation separately.
- Document the manual verification steps and expected announcements in this epic or a new doc, since this is the one part of the whole feature that cannot be automated by akar's existing agent-driven screenshot loop — a future agent picking this up should know upfront that this task requires either a human-in-the-loop pass or a specialized accessibility-testing tool (e.g. `accesskit_consumer`-based test harness, which egui uses for its own automated a11y tests — see `egui_kittest/tests/accesskit.rs`, `egui_kittest/src/node.rs` as a reference for how egui automates this without a real screen reader).

### Task 13 — Full `MockDrawList`-Equivalent Test Harness for `A11yTree`

**Status:** Not Started
**Readiness:** Blocked — depends on Task 8's final node contract. The basic recorder is then ready; the `accesskit_consumer` cross-check additionally needs Task 11's translation. It reduces, but does not eliminate, Task 12's manual platform-verification requirement.

- akar's existing testing approach (`AGENTS.md` → Testing approach) relies on `MockDrawList` for GPU-free component-logic testing. This task adds the `A11yTree` equivalent: a way to assert "component X pushed exactly this `A11yNode`" without needing a real `accesskit` platform adapter or a GPU device, generalizing the ad hoc assertions sketched in Task 10 into a reusable test utility (following how `DrawList::start_recording`/`recorded_calls` — `draw_list.rs:75-86` — already gives tests structured visibility into what was pushed, independent of scissor culling).
- Optionally cross-check against `accesskit_consumer` (vendored locally at `~/.cargo/registry/.../accesskit_consumer-0.35.0`, used by `egui_kittest` for exactly this purpose — see `egui_kittest/tests/accesskit.rs`) if a translation to real `accesskit::TreeUpdate` values (Task 11) is far enough along to validate round-tripping without a live platform adapter.

### Task 14 — C ABI / Non-Winit Documentation Punt

**Status:** Not Started
**Readiness:** Blocked — by sequencing only. It documents what Tasks 7-12 actually shipped, so it cannot be written accurately before they land. No research is missing.

- Not an implementation task — a documentation task. Once Tasks 7-12 ship, update `AGENTS.md`'s "Do not add accessibility scaffolding in v1" line (it will be stale/inaccurate at that point) and add an explicit note to `DEVELOP.md`'s "What akar does NOT own" section and/or a new section describing the current accessibility surface: what ships (winit + six roles), and what's still punted (C ABI, non-winit consumers, the other ~24 components, live regions, action dispatch) — matching this epic's Task 6 deferrals list so the punt stays documented and intentional rather than silently missing, per the existing project convention of always documenting punts (`AGENTS.md`: "Do not add accessibility scaffolding in v1. Document the punt if relevant.").
- In that documentation, distinguish the minimum focus/activation ActionRequest support that ships in the first pass from the richer role-specific action coverage that remains deferred; do not describe all action dispatch as punted.

---

## Implementation Readiness Audit — 2026-08-12

Reviewed against current akar source/history, `accesskit` 0.24.0, `accesskit_winit` 0.32.2, egui, gpui, and Xilem/Masonry from the local checkouts/cache only.

| Task | Gate after review |
|---|---|
| 1-6 | Done; research/scoping outputs are sufficient with Task 6's action-dispatch correction above. |
| 7 | **Ready** and independent. |
| 8 | **Blocked by design:** finalize semantic properties/actions and the synthetic root contract. |
| 9 | **Blocked by design:** specify keyboard activation and the frame-scoped assistive-action queue together with traversal. |
| 10 | **Blocked by 8-9;** layout parent accessor and component list are otherwise specified. |
| 11 | **Blocked by 8-9;** dependency versions are verified, but activation/deactivation events, `process_event`, full initial tree, and minimum action delivery are required. |
| 12 | **Blocked by 8-11** and ultimately requires manual platform verification; consumer tests are not a substitute. |
| 13 | **Blocked by 8** (and by 11 for the optional AccessKit translation cross-check). |
| 14 | **Blocked by sequencing only;** write after the shipped surface is known. |

Recommended implementation order after the two small design decisions are recorded: **7 independently; then 8 → 9 → 10 → 11 → 13 → 12 → 14**. Tasks 8 and 9 may be implemented together because their action contracts meet at `AkarCore`/`InputState`.

---

## Notes for Future Work

- Full component-catalog semantic coverage (all 30+ components) is a large, incremental effort that should follow the initial small-scope proof, not precede it.
- Non-Rust / non-winit consumers of `akar.h` wanting accessibility support may need a documented pattern for wiring their own platform adapter against an exposed semantic-tree walk API — this is explicitly deferred until the Rust/winit path is proven.
- Reduced-motion and high-contrast-mode preferences (OS-level user settings) are related but distinct from screen-reader support and are out of scope for this epic's initial research.
