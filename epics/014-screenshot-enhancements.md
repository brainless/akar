# Epic 014: Screenshot Utility Enhancements for Agent-Led Development

**Status:** In Progress
**Goal:** Extend the screenshot utility (Epic 013) with features that enable fully autonomous coding-agent development cycles. The current implementation has three limitations that block agent-led UI work:

1. Fixed 5-second delay before capture — not configurable.
2. No programmatic input injection — cannot trigger hover/press/focus states.
3. No structured logging — debugging relies solely on visual inspection.

This epic is a **brainstorming and planning** phase. The features listed below are starting points; the final design should be refined through discussion before implementation begins.

**Prerequisite:** Epic 013 is `Status: Done`.

---

## Final Design Decisions

Added 2026-07-09 after review by GLM5.2 and Qwen3.7 (see their notes below). This section supersedes the "Brainstorming: Feature Ideas" open questions wherever it addresses them directly and defines the implementation path. The brainstorming section is retained as history.

### Scope and structure

The epic is implemented as five tasks sequenced by impact/effort. 014a + 014b together deliver the epic's core goal — autonomous agent capture of non-idle UI states. 014c and 014d are incremental. 014e is documented as deferred (no code). Each task is independently shippable.

```
014a (delay + robustness) ──► 014b (script + labels) ──► 014c (frame dump) ──► 014d (diff)
014e (headless) — deferred, no dependency on 014a–014d
```

### Decisions

1. **Input injection: script format, not CLI flags.** `InputState` is a plain injectable struct (`crates/akar-core/src/input.rs:18`) and `akar-winit` is just one caller of its methods — so injection itself is free. The hard problem is **frame alignment**: hover/press states are frame-scoped because `InputState::begin_frame` clears per-frame events at the end of `AkarCore::end_frame` (`context.rs:96`). CLI flags (`--hover X,Y --click X,Y`) cannot express "press on frame N, release on frame N+1, capture on frame N" — they fire all events up front and race the capture timer. A line-based script format solves this by making `screenshot` a command in the sequence and advancing one input command per frame. A same-frame `click` (press+release) fires `is_clicked` in one frame per `input.rs:58-91`.

2. **Element addressing: labels-first on top of coordinates.** A `HashMap<String, NodeId>` on `Layout` + ~20 `register_label` calls in the demo for nodes already held as named `AppState` fields (~30 lines total, no component signature changes, no C ABI changes). `--dump-layout` prints `name x y w h` per registered label. Labels cover the ~60% of elements the demo owns as nodes; coordinates remain the fallback for inline-computed rects (dropdown items at `main.rs:1306`, list rows at `main.rs:829`, individual tabs) where no `NodeId` exists to register. The expensive version of labels (per-component `Option<&str>` params, self-registration) is explicitly rejected.

3. **Structured logging: gated recording mode in `DrawList`.** `DrawList::push_quad` (`draw_list.rs:88`) discards the active scissor at push time. To record scissor state per call without touching the 112-byte `#[repr(C)]` `QuadCall` Pod struct and its size assert (`draw_list.rs:25`), add `recording: bool` and `recorded: Vec<RecordedCall { call, scissor }>` to `DrawList`. Snapshot at push time *before* the cull early-return so culled calls are included (the point is debugging "why didn't my quad render"). Zero overhead when recording is off.

4. **`--wait-for-idle` deferred.** `DrawList::begin_frame` clears every frame (`draw_list.rs:52`), so cross-frame comparison requires new state, and looping animations never settle. Not worth the complexity for v1.

5. **Multi-capture: solved by the script format, not a separate flag.** Multiple `screenshot PATH` lines in one script produce before/after captures in one run. `--screenshot-before`/`--screenshot-after` as separate flags would conflate two runs awkwardly and is rejected.

6. **Diff: a separate `akar-diff` binary, kept in this epic.** ~150 lines, no GPU, no akar deps. `--diff` produces a visual diff PNG; `--compare` exits non-zero if the changed-pixel ratio exceeds a threshold (for CI regression gates). Pixel-exact for v1; perceptual diff is deferred. Baselines are caller-managed, not checked into the repo.

7. **Headless rendering (014e): deferred, punt documented.** The audience is CI, not local agent workflows. `AkarCore::mock` (`context.rs:37`) already creates a headless device, and the capture path renders to an intermediate texture (`screenshot.rs:106`), not a surface — so headless capture is architecturally feasible by skipping the blit pass. The real blocker is adapter availability on CI runners (no software Metal on macOS; `lavapipe` Linux-only; WARP Windows-only), not the rendering path. A future epic will address this when CI visual regression is prioritized.

8. **Screenshot robustness: fold the panic fixes into 014a.** `screenshot.rs:218` (`device.poll(...).unwrap()`) and `screenshot.rs:222` (`receiver.recv().unwrap().unwrap()`) panic on GPU failure instead of returning `ScreenshotError::BufferMapFailed`. These exercise the same code path as the delay task and are fixed there.

### Task summary

| Task | Scope | Key files |
|---|---|---|
| 014a | `--delay <SECS>` CLI flag + screenshot panic fixes | `examples/demo-rust/src/main.rs`, `crates/akar-core/src/screenshot.rs` |
| 014b | Input script runner + label registry + `--dump-layout` + `--script <FILE>` | `crates/akar-layout/src/lib.rs`, `examples/demo-rust/src/main.rs` (or new `script.rs`) |
| 014c | `--dump-frame <PATH>` JSON output via gated `DrawList` recording | `crates/akar-core/src/draw_list.rs`, `examples/demo-rust/src/main.rs` |
| 014d | `akar-diff` binary (`--diff`, `--compare`) | new `examples/akar-diff/` or `tools/akar-diff/` |
| 014e | Headless rendering — deferred, no code | none |

Detailed implementation plans for each task are in "Notes from GLM5.2" below. "Notes from Qwen3.7" records the independent review that converged on the same conclusions.

### Revised acceptance criteria

- `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` all pass.
- **014a:** `--delay <SECS>` controls capture timing; `--delay 0` captures the first frame; default 5s preserved; screenshot failures return `Err` instead of panicking.
- **014b:** `--dump-layout` prints labeled rects; `--script <FILE>` drives the demo into a non-idle state (e.g. dropdown open, button pressed, form focused) and captures a screenshot of that state without manual interaction.
- **014c:** `--dump-frame <PATH>` emits a valid JSON file with all draw calls (including culled), z-orders, scissor rects, labeled layout rects, and input state for the captured frame.
- **014d:** `akar-diff --diff` and `--compare` work on two PNGs.
- **014e:** no code; punt documented.

---

## Brainstorming: Feature Ideas

### 1. Configurable Capture Delay

**Problem:** The hardcoded 5-second delay is too long for quick iterations and too short for apps that need time to load data or animate into a target state.

**Ideas to consider:**
- `--delay <SECONDS>` CLI flag (float, e.g., `--delay 0.5` for 500ms).
- `--delay 0` for immediate capture (useful for static UIs).
- `--wait-for-idle` flag: capture after N frames with no draw-list changes (heuristic for "animation complete").
- Should the delay be a minimum, or an exact wait? What if the app isn't ready?

**Open questions:**
- Should we support a "capture on next frame" mode (zero delay, capture the very next frame after the flag is parsed)?
- How does this interact with `--exit`? If delay is 0 and exit is set, the app captures and exits immediately.

### 2. Programmatic Input Injection

**Problem:** Agents cannot screenshot hover states, pressed states, focused inputs, open dropdowns, or scrolled positions. They can only capture the default idle state.

**Ideas to consider:**
- CLI-driven input events: `--click X,Y`, `--hover X,Y`, `--scroll X,Y,DX,DY`, `--key Enter`, `--focus NODE_ID`.
- A small input script format: a text file with a sequence of events and delays, executed before capture.
  ```
  hover 720 30
  delay 0.1
  click 720 30
  delay 0.2
  screenshot
  ```
- Expose input injection via the C ABI so downstream apps can script their own capture sequences.
- Should input events bypass the normal event loop, or feed into it? (Bypassing is simpler but less realistic.)

**Open questions:**
- How do agents know the coordinates of UI elements? Options:
  - Expose layout rects via a `--dump-layout` flag that prints all node IDs and their pixel rects.
  - Use a naming/labeling system so agents can reference elements by name rather than coordinates.
  - Combine: `--click @dropdown_button` where `@dropdown_button` is a registered label.
- Should we support "capture all states" mode? E.g., automatically iterate hover/pressed/focused for every interactive element and produce a sprite sheet.

### 3. Structured Logging and Debug Output

**Problem:** When a UI looks wrong, agents need more than a screenshot to diagnose the issue. They need to know what draw calls were submitted, what z-ordering was used, what scissor rects were active, etc.

**Ideas to consider:**
- `--log-draw-calls` flag: print every draw call (quad + text) with rect, z, fill color, scissor state.
- `--log-layout` flag: print the resolved pixel rect for every layout node.
- `--log-input` flag: print all input events processed this frame.
- A `--debug-overlay` mode: render draw call bounding boxes, z-order numbers, and scissor rects as an overlay on the screenshot.
- JSON output mode: `--dump-frame /tmp/frame.json` produces a structured file with all draw calls, layout rects, and input state for programmatic analysis.

**Open questions:**
- Should logging be per-frame or cumulative? Per-frame is more useful for animation debugging.
- How much detail? A full dump could be thousands of lines. Should there be filter levels (e.g., `--log-level quads` vs `--log-level all`)?
- Should the debug overlay be rendered by akar itself (using its own draw pipeline), or as a post-processing step on the screenshot PNG?

### 4. Multi-Capture and Comparison

**Problem:** Agents need to compare before/after states to verify their changes had the intended effect.

**Ideas to consider:**
- `--screenshot-before /tmp/before.png --screenshot-after /tmp/after.png`: capture two states in one run (e.g., before and after an interaction).
- `--diff /tmp/baseline.png /tmp/current.png`: produce a pixel-diff image highlighting changes.
- A "regression mode": compare against a stored baseline and exit with non-zero status if the diff exceeds a threshold.

**Open questions:**
- Should diff be pixel-exact or perceptual (accounting for anti-aliasing differences)?
- Where should baselines be stored? In the repo? Per-component?

### 5. Headless / Offscreen Rendering

**Problem:** The current screenshot tool requires a real window (winit + surface). CI environments and some agent workflows may not have a display available.

**Ideas to consider:**
- An offscreen rendering mode that creates a wgpu device with a software backend (e.g., lavapipe on Linux, or wgpu's own null backend if available).
- Render to an intermediate texture at a fixed resolution without ever creating a surface.
- This would enable screenshot capture in CI for visual regression testing.

**Open questions:**
- Is a software GPU backend fast enough for practical use?
- Does wgpu support headless rendering on all target platforms (macOS, Windows, Linux)?
- Should this be a separate binary (`demo-headless`) or a flag on `demo-rust`?

---

## Prioritization (Proposed)

The features above are listed roughly in order of impact for agent-led development:

1. **Programmatic input injection** — highest impact. Without this, agents cannot verify hover/pressed/focus states or test interactive components.
2. **Configurable delay** — medium impact. Removes the fixed 5-second wait and enables faster iteration.
3. **Structured logging** — medium impact. Helps agents diagnose why a UI looks wrong.
4. **Multi-capture and comparison** — lower impact but valuable for regression testing.
5. **Headless rendering** — lowest priority for now; useful for CI but not required for local agent workflows.

---

## Implementation Notes

- All new CLI flags should be additive; existing `--screenshot` and `--exit` flags must continue to work unchanged.
- Input injection should feed into akar's existing `InputState` struct, not bypass it. This ensures components see realistic input events.
- The debug overlay (if implemented) should use akar's own draw pipeline, demonstrating that the library can render diagnostic information about itself.
- Any new C ABI surface (e.g., for input injection) should be added to `akar-c-api` alongside the existing input functions.

---

## Acceptance Criteria (Draft — to be refined after brainstorming)

- `cargo check --workspace` passes
- `cargo clippy --workspace -- -D warnings` passes
- `cargo test --workspace` passes
- Agent can capture a screenshot of a specific UI state (e.g., dropdown open, button pressed) without manual interaction
- Agent can configure the capture delay
- Agent can request structured output about the frame's draw calls

---

## Notes

- This epic is intentionally a brainstorming document. The final feature set and implementation plan should be refined through discussion before any code is written.
- The goal is to enable coding agents to develop and debug akar (and downstream apps using akar) with minimal human intervention — the agent makes a change, captures a screenshot, inspects the result, and iterates autonomously.

---

## Notes from GLM5.2

Added 2026-07-09 after a codebase review grounded in the Epic 013 implementation (`crates/akar-core/src/screenshot.rs`, `context.rs`, `input.rs`, `draw_list.rs`, `lib.rs`), the demo render loop (`examples/demo-rust/src/main.rs`), `akar-layout/src/lib.rs`, and `akar-winit/src/lib.rs`. This section proposes splitting the epic into Tasks 014a–014e sequenced by impact/effort, with implementation plans. It supersedes the open questions in the "Brainstorming" section where it addresses them directly.

### Summary

The epic as written bundles five features whose effort ranges from ~15 minutes (configurable delay) to a multi-week platform gamble (headless rendering). The cheap, high-impact wins risk getting blocked behind the expensive, low-impact ones. The proposed restructure keeps everything inside Epic 014 as Tasks 014a–014e (per maintainer preference) but sequences them so that 014a + 014b together deliver ~80% of the epic's stated goal — "fully autonomous coding-agent development cycles" — and the rest is incremental.

Two decisions were confirmed with the maintainer before this section was written:

1. **Scope:** keep one epic, split into Tasks 014a–014e internally (not separate epics).
2. **Element addressing:** labels-first (`@submit_button`) built on top of a coordinate dump (`--dump-layout`), not one or the other. The incremental cost of labels on top of coordinates is small (a `HashMap` on `Layout` + ~15 registration calls in the demo); the agent workflow of "guess which rect is the submit button from a raw dump" is error-prone enough to make that cost worth paying. Labels cover the ~60% of elements the demo explicitly owns as named `NodeId`s; coordinates remain the fallback for sub-elements created inline inside component functions (dropdown items, list rows, individual tabs) where no `NodeId` exists to register.

### Findings from codebase review

These ground the task plans below and correct a few assumptions in the "Brainstorming" section.

1. **Input injection is mostly already free — `InputState` is a plain injectable struct.** `crates/akar-core/src/input.rs:18` exposes `set_mouse_pos`, `push_mouse_button`, `push_scroll`, `push_char`, `push_key`. `akar-winit/src/lib.rs:4` is just *one* caller of these methods; nothing ties input to winit or to an event loop. The hard part of input injection is not injection — it is element addressing and frame alignment, both of which the epic under-specifies.

2. **Hover/press states are frame-scoped, which breaks the existing capture model.** `InputState::begin_frame` (`input.rs:45`) clears per-frame events (pressed/released/scroll/chars/keys) and is called at the *end* of `AkarCore::end_frame` (`context.rs:96`), after components have read them. To screenshot a hover state, the injected mouse position must be live on the **same frame** that gets captured. The current `request_screenshot()` + fixed 5s-delay flow (`main.rs:1353-1361`) does not model "inject input on frame N, capture on frame N". The script runner in Task 014b must align injection and capture to specific frames, not to wall-clock delay alone.

3. **A synthetic click can fire `is_clicked` in a single frame.** `push_mouse_button` (`input.rs:58`) sets `mouse_buttons_pressed` on a false→true transition and `mouse_buttons_released` on a true→false transition, reading `was_down` from the current `mouse_buttons` state. Calling `push_mouse_button(0, true)` then `push_mouse_button(0, false)` in the same frame produces both events, and `is_clicked` (`input.rs:90`, `released[0] && is_hovering`) fires. This lets the script runner implement `click` as a same-frame press+release without a two-frame dance. A held/active-state screenshot is the separate `press`/`release` split.

4. **Element addressing has a real ceiling: many interactable rects are not `NodeId`s.** Dropdown menu items (`main.rs:1306`), list rows (`main.rs:829`), and individual tabs inside `akar_tab_bar` are computed inline as `[f32;4]` rects, never stored as layout nodes. `Layout::rect` (`akar-layout/src/lib.rs:128`) only resolves registered taffy nodes. Labels can only ever cover the former; coordinates are the only option for the latter. This is why both are built, with labels as sugar over coordinates.

5. **Structured logging has a hidden dependency: scissor state is not recorded per call.** `DrawList::push_quad` (`draw_list.rs:88`) culls against the active scissor at push time but discards it — kept calls do not remember which scissor was active. The "Brainstorming" section lists scissor state as a `--log-draw-calls` field, but producing it requires either (a) adding a `scissor` field to `QuadCall`, which ripples into the 112-byte `#[repr(C)]` Pod struct and its `const _: () = assert!(size_of == 112)` (`draw_list.rs:25`) and the quad pipeline's vertex layout, or (b) a gated recording mode that snapshots `{call, scissor}` at push time into a side vector. Task 014c uses (b) to avoid the layout ripple.

6. **Configurable delay is a demo-only one-liner.** `Duration::from_secs(5)` lives at `main.rs:1357`. Zero `akar-core` involvement. It is listed as a "feature" in the brainstorming but is trivially small; sequencing it first unblocks faster iteration for all subsequent tasks.

### Issues to fix regardless of task scope

These exist in the Epic 013 implementation and should be fixed early (folded into Task 014a) since the new tasks exercise the same paths:

- `screenshot.rs:218` calls `device.poll(PollType::wait_indefinitely()).unwrap()` — panics on GPU failure instead of returning `ScreenshotError::BufferMapFailed`.
- `screenshot.rs:222` calls `receiver.recv().unwrap().unwrap()` — the inner `unwrap` is mapped to `BufferMapFailed`, the outer `recv().unwrap()` panics on a dropped sender.
- `ScreenshotError` (`screenshot.rs:11`) is a single-variant enum. If PNG encoding ever moves from the demo into core, it will need an `EncodingError` variant; for now the demo owns encoding so this is deferred.

### Task breakdown

Tasks are sequenced by impact/effort. Each is independently shippable. 014a + 014b together meet the epic's core acceptance criteria; 014c–014e are incremental.

#### Task 014a — Configurable capture delay + screenshot robustness

**Scope:** demo-only CLI change plus the two panic fixes above. No `akar-core` API change beyond error mapping.

**Implementation:**

`examples/demo-rust/src/main.rs`:
- Parse `--delay <SECONDS>` (float) alongside `--screenshot` and `--exit` in the existing arg loop (`main.rs:94-105`).
- Add `delay_secs: f64` to `App` (default `5.0` when `--screenshot` is set, ignored otherwise). `--delay 0` means capture on the first `RedrawRequested` after `resumed()`.
- Replace the `Duration::from_secs(5)` check at `main.rs:1357` with `Duration::from_secs_f64(self.delay_secs)`.
- Backward compatible: `--screenshot PATH` without `--delay` keeps the 5s default.

`crates/akar-core/src/screenshot.rs`:
- `take_screenshot`: replace `device.poll(...).unwrap()` (`screenshot.rs:218`) with a mapped `ScreenshotError::BufferMapFailed`. Replace `receiver.recv().unwrap()` (`screenshot.rs:222`) the same way (the channel never legitimately closes since the closure owns the sender, but the panic path should still be guarded).

**Acceptance:** `--delay 0.5 --screenshot /tmp/x.png --exit` captures after 500ms; `--delay 0` captures the first frame; no `--delay` keeps 5s; `cargo clippy --workspace -- -D warnings` passes; a forced buffer-map failure returns `Err` instead of panicking.

#### Task 014b — Input script runner + label registry

**Scope:** the core of the epic. A small script format executed by the demo against `InputState`, plus a label registry in `akar-layout` and `--dump-layout` as the coordinate fallback. No component signature changes, no C ABI changes — injection goes through the already-public `InputState` methods.

**Implementation:**

1. **Label registry in `akar-layout`** (`crates/akar-layout/src/lib.rs`):
   - Add `labels: HashMap<String, NodeId>` to the `Layout` struct (`lib.rs:22`).
   - `pub fn register_label(&mut self, name: &str, node: NodeId)` — inserts into the map. Cheap, idempotent, last-write-wins.
   - `pub fn resolve_label(&self, name: &str) -> Option<NodeId>` — lookup.
   - `pub fn labeled_rects(&self) -> Vec<(String, [f32; 4])>` — resolves every registered label to a pixel rect via the existing `rect` method (`lib.rs:128`); returns `(name, [x,y,w,h])` pairs.
   - ~20 lines total. No taffy tree iteration, no component changes, no ABI surface.

2. **`--dump-layout` in the demo** (`examples/demo-rust/src/main.rs`):
   - Parse `--dump-layout` flag. When set, after the first `layout.compute(...)` (`main.rs:686`), print each `labeled_rects()` entry as `name x y w h` (one per line) to stdout, then exit. This is the coordinate fallback for agents and the discovery tool for label names.
   - Register labels in `resumed()` for the nodes the demo already holds as named `AppState` fields: `navbar_btn_node`, `navbar_new_btn_node`, `navbar_dropdown_btn_node`, `alert_node`, `tab_bar_node`, `form_name_node`, `form_notes_node`, `form_agreement_node`, `form_radio_dark`, `form_radio_light`, `form_notifications_node`, `form_font_size_node`, `form_language_node`, `form_submit_node`, `skeleton_toggle_node`, `stat_nodes[0..3]`, `avatar_nodes[0..3]`, `steps_node`. ~20 `register_label` calls.
   - Sub-elements without `NodeId`s (dropdown items, list rows, individual tabs) are not labelable; agents use `--dump-layout` coordinates for those.

3. **Input script format** — a line-based text file:
   ```
   hover @form_submit_node
   delay 0.1
   click @form_submit_node
   delay 0.2
   screenshot /tmp/pressed.png
   ```
   Lines:
   - `hover <X> <Y>` or `hover @label` — set mouse pos (label resolved to rect center).
   - `press <button>` / `release <button>` — hold/release a mouse button (for active-state screenshots). `button` is `left`/`right`/`middle`, default `left`.
   - `click <X> <Y>` / `click @label` — same-frame press+release (see finding 3 above).
   - `scroll <DX> <DY>` — `push_scroll`.
   - `key <Enter|Escape|Tab|Backspace|...>` — `push_key` (the `Key` enum, `input.rs:4`).
   - `type "text"` — `push_char` for each char.
   - `delay <SECONDS>` — wall-clock wait before advancing.
   - `screenshot <PATH>` — request capture on this frame.
   - `# comment`, blank lines — ignored.

4. **Script runner** (`examples/demo-rust/src/main.rs` or a new `examples/demo-rust/src/script.rs`):
   - Parse `--script <FILE>` flag. If set, load and parse the file at startup into a `Vec<ScriptStep>`.
   - The runner holds a cursor. On each `RedrawRequested` frame, *before* components run (i.e. right after `state.core.begin_frame(...)`, `main.rs:633`), it advances the script:
     - For `delay X`: start/continue a timer; do not advance past this line until X seconds have elapsed.
     - For input lines (`hover`/`press`/`release`/`click`/`scroll`/`key`/`type`): inject into `state.core.input` immediately, advance to the next line. These take effect this frame because `input.begin_frame()` (the per-frame clear) already ran at the end of the previous frame's `end_frame` (`context.rs:96`).
     - For `screenshot PATH`: set `self.screenshot_path = Some(PATH)` for this frame, set `is_capture_frame`, advance. The existing capture path (`main.rs:1359-1448`) handles the rest. The `--delay` from 014a is bypassed on script-driven frames (the script's own `delay` lines control timing).
   - The app continues running after the script exhausts so the final state remains inspectable, unless `--exit` is also set (then exit after the last `screenshot` line completes).
   - `--script` and `--screenshot` are mutually exclusive at the CLI level (the script drives its own `screenshot` lines).

5. **Frame alignment correctness note:** because `InputState::begin_frame` clears per-frame events at the *end* of `end_frame`, an input line injected at the top of frame N is visible to all components during frame N and is cleared before frame N+1. A `click` (same-frame press+release) therefore fires `is_clicked` on frame N. A `press` on frame N followed by `release` on frame N+1 keeps `mouse_buttons[0]` down across both frames, so `is_pressed` is true on frame N (good for an active-state screenshot captured on frame N). The runner advances one non-`delay` line per frame so the agent has precise frame-level control.

6. **C ABI exposure:** explicitly deferred. The script runner is demo-side; injection uses existing public `InputState` methods. No new `akar-c-api` surface in this task. Non-Rust users can already call `akar_set_mouse*`/`akar_push_char` per the C ABI contract in `AGENTS.md`; a script format for C consumers is a follow-up if requested.

**Acceptance:** `--dump-layout` prints labeled rects and exits; `--script /tmp/open_dropdown.txt` can open the dropdown (click `@navbar_dropdown_btn_node`), hover an item, and capture a screenshot of the open state; `cargo test --workspace` passes (add unit tests for the label registry and the script parser); `cargo clippy --workspace -- -D warnings` passes.

#### Task 014c — Structured frame dump (`--dump-frame`)

**Scope:** a gated recording mode in `DrawList` that snapshots `{call, active_scissor}` at push time, plus a demo flag that serializes the recorded calls, labeled rects, and input snapshot to JSON. The on-screen debug overlay (rendering bounding boxes/z-numbers onto the screenshot) is deferred to a follow-up — it needs its own draw pipeline usage and is a separate sub-feature.

**Implementation:**

1. **Recording mode in `DrawList`** (`crates/akar-core/src/draw_list.rs`):
   - Add `recording: bool` and `recorded: Vec<RecordedCall>` to `DrawList`.
   - `pub struct RecordedCall { pub call: DrawCall, pub scissor: Option<[f32;4]> }` — a debug-only snapshot. `DrawCall` is already `Clone`/`Debug` (`draw_list.rs:3`).
   - `pub fn start_recording(&mut self)` / `pub fn stop_recording(&mut self)` — toggle. `begin_frame` clears `recorded` if recording is on.
   - In `push_quad` and `push_text`: if `self.recording`, push a `RecordedCall { call: ..., scissor: self.active_scissor() }` *before* the scissor-cull early-return — so the dump includes culled calls (marked by their scissor not intersecting), which is useful for debugging "why didn't my quad render". This means recording must snapshot *before* the cull check, not after.
   - `pub fn recorded_calls(&self) -> &[RecordedCall]` — accessor.
   - `QuadCall` and `TextCall` are unchanged; the 112-byte assert and Pod layout are untouched. The recording vector is only populated when `start_recording` was called — zero overhead otherwise.

2. **JSON dump in the demo** (`examples/demo-rust/src/main.rs`):
   - Parse `--dump-frame <PATH>`. When set, call `state.core.draw_list.start_recording()` right after `begin_frame` (`main.rs:633`), run the frame, then after `end_frame` serialize `recorded_calls()`, `state.layout.labeled_rects()`, and a snapshot of `state.core.input` (mouse pos, button states, scroll delta, chars, keys, focused_id) to JSON via `serde_json`.
   - `examples/demo-rust/Cargo.toml`: add `serde = "1"`, `serde_json = "1"` (and `serde` derives on a small `FrameDump` struct). `DrawCall`/`QuadCall`/`TextCall` already derive `Debug`; add `serde::Serialize` derives in `akar-core` (behind a `serde` feature flag on the crate to keep it optional for non-Rust consumers).
   - The dump is per-frame (the frame on which `--dump-frame` is set). For multi-frame dumps, combine with `--script` and emit one JSON file per `screenshot` line (e.g. `screenshot /tmp/x.png` also writes `/tmp/x.json` if `--dump-frame` is set). Simpler v1: one dump on the capture frame only.

3. **Filtering:** defer `--log-level` granularity. v1 dumps everything; the JSON is machine-parseable so the consumer filters. A full dump for the demo is on the order of a few hundred calls — manageable.

**Acceptance:** `--dump-frame /tmp/frame.json --screenshot /tmp/x.png --exit` produces a valid JSON file with every draw call (including culled ones), its z, its scissor, all labeled rects, and the input state for the captured frame; `cargo test --workspace` passes (add a test that `start_recording` + a culled quad produces a `recorded_calls` entry); `cargo clippy --workspace -- -D warnings` passes.

#### Task 014d — Multi-capture and comparison

**Scope:** tooling around existing PNG output. No `akar-core` changes. A small diff/compare utility, not a demo flag.

**Implementation:**

1. **New binary** `examples/akar-diff/` (or `tools/akar-diff/`):
   - `akar-diff --diff /tmp/baseline.png /tmp/current.png -o /tmp/diff.png` — reads two PNGs, writes a diff PNG: changed pixels in red, unchanged pixels dimmed to ~30% brightness. Pixel-exact for v1; perceptual diff (accounting for AA differences) is explicitly deferred as a research item.
   - `akar-diff --compare /tmp/baseline.png /tmp/current.png --threshold <PCT>` — captures no images; diffs and exits non-zero if the changed-pixel ratio exceeds `PCT`. For CI regression gates.
   - Dependencies: `png` (already used by the demo), `clap` or manual arg parsing (match the demo's manual style for consistency).
   - ~150 lines, no GPU, no akar deps.

2. **Multi-capture in one run:** not a separate flag. Task 014b's script format already supports multiple `screenshot PATH` lines, so `--screenshot-before /tmp/before.png --screenshot-after /tmp/after.png` is expressed as a script:
   ```
   screenshot /tmp/before.png
   click @navbar_dropdown_btn_node
   delay 0.2
   screenshot /tmp/after.png
   ```
   No new CLI surface needed.

3. **Baselines:** stored wherever the caller points `--compare` at. Not checked into the repo by default; a per-component baseline directory is a project-policy decision, not an akar decision.

**Acceptance:** `akar-diff --diff` produces a visually correct diff PNG; `--compare` exits 0 for identical images and non-zero for images differing above threshold; `cargo clippy --workspace -- -D warnings` passes.

#### Task 014e — Headless / offscreen rendering (deferred)

**Scope:** explicitly deferred out of Epic 014 per the maintainer discussion. Documented here as the future shape and risks so the punt is recorded (per `AGENTS.md` "document the punt if relevant").

**Why deferred:** the audience is CI, not local agent workflows. The platform risk is real and asymmetric:
- **macOS:** no software Metal backend. Headless rendering requires a real GPU, which requires a display (or a virtual display via `xvfb`-equivalent, which macOS does not provide cleanly). This makes headless CI on macOS impractical without a remote GPU.
- **Linux:** `lavapipe` (software Vulkan) works but is slow and not always installed in CI images. `xvfb` + a real adapter is the reliable path.
- **Windows:** WARP (software D3D) is available and reliable but slow.

**What is already feasible:** `AkarCore::mock` (`context.rs:37`) already creates a headless wgpu device via `InstanceDescriptor::new_without_display_handle()`. The screenshot path renders to an intermediate `wgpu::Texture` with `RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_SRC` (`screenshot.rs:106`), not to a surface. So a headless *capture* is architecturally possible — skip the surface entirely, render directly to the capture texture, read it back. The blocker is adapter availability on CI runners, not akar's rendering path.

**Future epic shape (when prioritized):**
- A `demo-headless` binary (not a `demo-rust` flag — different lifecycle, no winit) that creates a headless device, renders a fixed scene to an intermediate texture at a configurable resolution, and writes a PNG.
- Documented adapter requirements per platform (lavapipe on Linux, WARP on Windows, "not supported on macOS without a real GPU" on macOS).
- Combined with Task 014d's `--compare` for CI visual regression.

**Acceptance for 014e within this epic:** none — the task is documented as deferred. No code is written. A future epic will pick it up when CI visual regression is prioritized.

### Sequencing and dependencies

```
014a (delay + robustness) ──┐
                            ├─► 014b (script + labels) ──► 014c (frame dump) ──► 014d (diff)
                            │                                         │
                            └─────────────────────────────────────────┴─► 014d uses 014c JSON for structured diffs (optional)
014e (headless) — deferred, no dependency on 014a–014d
```

014a is a prerequisite for 014b because the script runner reuses the configurable-delay capture path. 014c builds on 014b's label registry for the `labeled_rects()` portion of the JSON dump. 014d is independent but benefits from 014c's structured output for richer diffs. 014e is cut entirely.

### Revised acceptance criteria for Epic 014

Replacing the draft criteria in the "Acceptance Criteria (Draft)" section:

- `cargo check --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` all pass.
- **014a:** `--delay <SECS>` controls capture timing; `--delay 0` captures the first frame; default 5s preserved; screenshot failures return `Err` instead of panicking.
- **014b:** `--dump-layout` prints labeled rects; `--script <FILE>` can drive the demo into a non-idle state (e.g. dropdown open, button pressed, form focused) and capture a screenshot of that state without manual interaction.
- **014c:** `--dump-frame <PATH>` emits a valid JSON file with all draw calls (including culled), z-orders, scissor rects, labeled layout rects, and input state for the captured frame.
- **014d:** `akar-diff --diff` and `--compare` work on two PNGs.
- **014e:** no code; punt documented.

---

## Notes from Qwen3.7

Initial analysis proposed four tasks: configurable delay, CLI-based input injection, `--dump-layout`, and `--dump-frame`. After reviewing GLM5.2's analysis, several positions have been revised.

### Revisions based on GLM5.2's findings

**Task 1 (delay) — updated:** GLM5.2 identified two panic paths in `screenshot.rs:218,222` (`device.poll(...).unwrap()` and `receiver.recv().unwrap().unwrap()`) that should return `ScreenshotError::BufferMapFailed` instead. These should be folded into the delay task since they exercise the same code path.

**Task 2 (input injection) — revised:** My original proposal used CLI flags (`--hover X,Y`, `--click X,Y`). GLM5.2 identified a critical issue I missed: **hover/press states are frame-scoped**. `InputState::begin_frame` clears per-frame events at the end of `AkarCore::end_frame` (`context.rs:96`). CLI flags cannot express "hold press across frame N, release on frame N+1" for active-state screenshots. The input script format (line-based text file) solves this cleanly by advancing one command per frame, giving precise frame-level control. The script format also gives us multi-capture for free (multiple `screenshot PATH` lines in one script). The parsing is simple (~8 commands, line-based) and the frame-alignment benefit is essential.

**Task 3 (dump-layout) — revised:** I originally said label-based targeting "requires a component labeling/registration system that does not exist today" and should be a separate epic. GLM5.2 showed this is ~20 lines: a `HashMap<String, NodeId>` on `Layout` plus ~20 `register_label` calls in the demo for the named `NodeId` fields already in `AppState`. The agent workflow of guessing which rect is the submit button from a raw coordinate dump is error-prone enough to justify the trivial cost. Labels cover the ~60% of elements the demo explicitly owns as named nodes; coordinates remain the fallback for inline-computed rects (dropdown items, list rows, individual tabs).

**Task 4 (dump-frame) — updated:** GLM5.2 found that `DrawList::push_quad` discards the active scissor at push time — kept calls do not remember which scissor was active. The frame dump needs scissor state for debugging. GLM5.2's solution is a gated recording mode: add `recording: bool` and `recorded: Vec<RecordedCall>` to `DrawList`, where `RecordedCall { call: DrawCall, scissor: Option<[f32;4]> }` snapshots the call and active scissor at push time (before the cull check, so culled calls are also recorded). This avoids touching the 112-byte `QuadCall` `#[repr(C)]` Pod struct.

**Multi-capture/diff — revised:** I originally said defer entirely. GLM5.2 showed that multi-capture is already solved by the script format (multiple `screenshot PATH` lines). The `akar-diff` binary (~150 lines, no GPU, no akar deps) is small enough to include as a separate task rather than deferring. It provides `--diff` (visual diff PNG) and `--compare` (exit non-zero if diff exceeds threshold) for CI regression gates.

### Revised task list

The tasks below align with GLM5.2's Task 014a–014d breakdown. Task 014e (headless rendering) remains deferred as a separate epic.

1. **Task 014a — Configurable delay + screenshot robustness:** `--delay <SECS>` CLI flag (float, default 5.0), fix panic paths in `screenshot.rs` to return `Err`.
2. **Task 014b — Input script runner + label registry:** Line-based script format (`hover`, `press`, `release`, `click`, `scroll`, `key`, `type`, `delay`, `screenshot`), `HashMap<String, NodeId>` label registry in `akar-layout`, `--dump-layout` flag, `--script <FILE>` flag.
3. **Task 014c — Structured frame dump:** Gated recording mode in `DrawList` that snapshots `{call, scissor}` at push time, `--dump-frame <PATH>` flag that serializes draw calls, labeled rects, and input state to JSON.
4. **Task 014d — Diff utility:** Separate `akar-diff` binary with `--diff` (visual diff PNG) and `--compare` (threshold-based exit code) modes.
5. **Task 014e — Headless rendering:** Deferred to a separate epic.

---

## Review Log

### 014a — Configurable delay + screenshot robustness (merged)
- **Files:** `crates/akar-core/src/screenshot.rs`, `examples/demo-rust/src/main.rs`.
- **Verdict:** Implemented as specified. `device.poll(...).unwrap()` and `receiver.recv().unwrap().unwrap()` now return `ScreenshotError::BufferMapFailed`. `--delay <SECS>` (f64, default 5.0) drives capture timing; `--delay 0` captures the first frame. `cargo clippy --workspace -- -D warnings` and `cargo fmt --check` pass.
- **Note:** `delay_secs` defaults to 5.0 unconditionally (only consulted inside the `is_capture_frame` gate, so it is effectively ignored when no `--screenshot` is set) — matches the intended backward-compatible behavior.

### 014b — Input script runner + label registry + --dump-layout + --script (merged)
- **Files:** `crates/akar-layout/src/lib.rs` (label registry), `examples/demo-rust/src/script.rs` (new parser+runner), `examples/demo-rust/src/main.rs` (flags, label registration, capture-gate bypass), `examples/demo-rust/scripts/open_dropdown.txt` (fixture).
- **Verdict:** Implemented as specified. Label registry is a `HashMap<String,NodeId>` with `register_label`/`resolve_label`/`labeled_rects`. `--dump-layout` prints `name x y w h` and exits. `--script <FILE>` parser supports `hover`/`press`/`release`/`click`/`scroll`/`key`/`type`/`delay`/`screenshot` with `@label` addressing. Capture gate bypassed (`normal_capture || script_capture_path.is_some()`) so script `screenshot` captures that exact frame; multiple captures allowed; `--exit` honored after exhaustion. 13 unit tests added (parser + runner + registry). `clippy -D warnings`, `fmt`, `test --workspace` all pass.
- **Note:** The runner injects input after `layout.compute` and before components read it, and relies on `InputState::begin_frame` clearing per-frame events at `end_frame` for frame alignment. The demo cannot be exercised headlessly here (needs a GPU), so visual confirmation of a scripted capture is left to the screenshot tool on a GPU host.
