# Epic 014: Screenshot Utility Enhancements for Agent-Led Development

**Status:** In Progress
**Goal:** Extend the screenshot utility (Epic 013) with features that enable fully autonomous coding-agent development cycles. The current implementation has three limitations that block agent-led UI work:

1. Fixed 5-second delay before capture — not configurable.
2. No programmatic input injection — cannot trigger hover/press/focus states.
3. No structured logging — debugging relies solely on visual inspection.

This epic is a **brainstorming and planning** phase. The features listed below are starting points; the final design should be refined through discussion before implementation begins.

**Prerequisite:** Epic 013 is `Status: Done`.

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
