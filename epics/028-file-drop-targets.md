# Epic 028: File Drops on Existing Layout Nodes

**Status:** Done (2026-09-25). Tasks 1-4 are implemented. The deterministic form fixture produced hover and accepted screenshots and frame dumps. `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` pass; the workspace test run used Metal access outside the sandbox. A pre-existing data-grid test was corrected to keep its compared row visible at both scroll positions.
**Goal:** Let an application accept paths dropped from the operating system on any existing akar layout node, including a form or container, using a position supplied by its windowing host. The application owns all file loading and processing.

**Prerequisite:** Epic 027 is `Status: Done`. This epic is independent of the deferred internationalization and accessibility epics (024 and 025).

---

## Problem

akar currently forwards pointer, keyboard, and scroll input but has no file drag or drop input. An application cannot declare that its existing main-column form accepts files; it must handle a window-level drop itself and reimplement layout hit-testing. A dedicated file-drop widget would not solve this for containers that already have their own content and styling.

The desired application flow is:

```rust
let response = file_drop_target(&mut core, &layout, form_node);
for path in response.dropped_paths {
    app.handle_path(path);
}
```

`form_node` is the form's existing `NodeId`. The same helper must work on a card, list viewport, or other resolved node. akar reports paths and interaction state; it does not open files, read metadata, decode content, or choose an application action.

## Current State and Local References

- `crates/akar-core/src/input.rs` owns frame-scoped input and clears transient events in `InputState::begin_frame`. `AkarCore::end_frame` calls that method after rendering. `InputState::mouse_pos` is the most recent ordinary pointer position, not necessarily the position of an OS file drag.
- `crates/akar-layout/src/lib.rs` supplies `Layout::rect(node)` and namespaced `widget_id(node)`. Existing components use the resolved rectangle for hit-testing.
- `crates/akar-winit/src/lib.rs` translates winit events without owning a window or event loop. The workspace uses winit 0.30. Its `HoveredFile(PathBuf)`, `DroppedFile(PathBuf)`, and `HoveredFileCancelled` events have paths but no drag or drop position; a multi-file drop arrives as one `DroppedFile` event per path.
- GPUI's local `crates/gpui/src/interactive.rs` models external files as paths and gives file enter, move, and submit events a position. Its existing `div` elements can attach an `on_drop` handler for external paths.
- Tauri's local `crates/tauri-runtime/src/window.rs` models `Enter { paths, position }`, `Over { position }`, `Drop { paths, position }`, and `Leave`.
- egui's local `crates/egui-winit/src/lib.rs` forwards winit's path-only events into frame input; its file-drop example handles them at window level. This does not establish reliable per-widget targeting.
- sokol's local `sokol_app.h` exposes dropped paths and the event's pointer coordinates through a flat C API.

## Design Decisions

### 1. Host-Supplied Coordinates Are Required for Node Targeting

The host submits file drag and drop events with a position in the same window-local coordinate space used by `InputState::mouse_pos` and `Layout::rect(node)`. The event must carry its own position; `file_drop_target` must not substitute the last ordinary mouse position. The host is responsible for converting native physical/logical coordinates before submission, consistently with its ordinary pointer input.

An integration that has paths but no trustworthy position may report an unpositioned, window-level drop to the application, but it must not claim that the drop landed on a particular layout node. The core input model must represent this distinction explicitly. Do not patch akar with platform-specific window handling to hide the winit limitation; `akar-core` and `akar-components` stay window-system independent.

### 2. Any Resolved Layout Node Can Be a Target

Add a generic, non-rendering `file_drop_target(core, layout, node)` helper in `akar-components`. It reads the node's current resolved rectangle, checks a position-bearing drag/drop event, and returns a typed response containing `hovered` and dropped paths. It creates no layout nodes and draws no mandatory chrome. The caller can style its existing form or container in response to `hovered`.

Zero-area, non-finite, or absent target rectangles cannot accept a drop. Hit-testing must respect an active clipping scope so a scrolled-off part of a container does not accept a file that is visually outside the viewport. The implementation must keep coordinate conversion for the draw-list scissor explicit.

Only nodes for which the application calls the helper are targets. A parent form may receive a drop over an ordinary child control. If both parent and child opt in, each drop is delivered to one target only. The first eligible target checked in a frame claims it; applications check the more specific child before a parent fallback. Document this ordering and test it. No hover or drop event may be applied twice by repeated checks.

### 3. Paths and File Work Stay with the Application

The Rust input event owns `PathBuf`s supplied by the host, preserving native path representation. A drop may contain multiple paths; deliver all paths for the claimed drop together. An application may retain the paths beyond the frame by moving or cloning them. akar does not inspect whether a path points to a regular file, directory, or valid current filesystem entry.

The host-facing event shape follows Tauri's lifecycle: enter with paths and position, move with position, drop with paths and position, and leave/cancel. Position-bearing drop delivery is required for this epic; hover feedback is included so an existing container can highlight itself while the drag is over it. Drag state clears on leave or drop. Transient dropped paths remain available through one completed render frame and are then cleared through the existing input lifecycle.

### 4. The C ABI Mirrors the Same Capability

`akar-c-api` exposes flat functions to submit the host's drag lifecycle and query a target by its existing node ID. Submission copies path data into the context; the caller's pointer may be released after the call. The target response provides an explicit count and a caller-buffer accessor for each path, with documented lifetime and error behavior. No Rust-owned pointer is returned to C, and `akar.h` is regenerated by cbindgen rather than edited directly.

Path encoding is explicit and length-delimited. The implementation must preserve native Rust `PathBuf`s, including paths that cannot be represented as UTF-8. Define and test a portable UTF-8 input form plus native Unix byte and Windows UTF-16 forms for lossless interop; do not silently use `to_string_lossy` for a path the application will open later. The C output accessor identifies the encoding and required byte length before copying.

## Public API Direction

Names may be adjusted during implementation, but the ownership and position rules above are fixed. A representative Rust shape is:

```rust
pub enum FileDragInput {
    Enter { position: [f32; 2], paths: Vec<PathBuf> },
    Move { position: [f32; 2] },
    Drop { position: [f32; 2], paths: Vec<PathBuf> },
    Leave,
}

pub struct FileDropResponse {
    pub hovered: bool,
    pub dropped_paths: Vec<PathBuf>,
}

pub fn file_drop_target(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
) -> FileDropResponse;
```

The exact storage and borrowing model may differ, but querying one target must not discard paths intended for a later eligible target. Multiple drop operations queued before a rendered frame must retain their separate positions and paths. The C API must expose equivalent behavior without a callback or caller-side heap allocation requirement.

## Tasks

### Task 1: Input Events and Lifetime

- Add native-path file drag/drop events to `InputState`, with explicit per-event position and a separate representation for unpositioned window-level drops.
- Preserve multiple paths in one drop and multiple drops before a render frame without losing their positions or delivering a path twice.
- Define when hover state changes, when drop events are consumed, and when frame cleanup occurs. Verify that `akar_input_begin` and `AkarCore::end_frame` cannot accidentally erase a submitted drop before a target observes it when used in the documented order.
- Add CPU-only tests for enter/move/leave/drop, multiple files, multiple drops, invalid coordinates, and frame cleanup.

### Task 2: Generic Target on an Existing Node

- Implement `file_drop_target` without adding a new widget or changing the layout tree.
- Hit-test the current resolved node rectangle in the input coordinate space, intersecting any active visible clipping region.
- Return hover state and paths, with deterministic first-eligible-target claim behavior. Test parent/child target ordering, ordinary non-target children, repeated checks, zero-area nodes, and clipped targets.
- Include an example where a main-column form accepts a path and hands it to application code without akar reading the file.

### Task 3: Host Adapters and Position Limitation

- Expose a direct Rust host submission path, and implement the matching C submission functions.
- In `akar-winit`, translate only what winit 0.30 actually provides. Preserve its paths as window-level unpositioned input, or expose them to the application for submission through a position-aware host hook. Do not infer a drop position from `CursorMoved` or `InputState::mouse_pos` without platform verification.
- Document how a host with native drag coordinates converts and passes them to akar. Keep winit and all other windowing APIs outside `akar-core` and `akar-components`.
- If precise node targeting through stock winit remains unavailable, state that limitation in the demo/API documentation and verify the position-aware path with deterministic injected events.

### Task 4: C ABI and Verification

- Add length-delimited path submission and caller-buffer path retrieval, with explicit encoding and required-size queries. Regenerate `akar.h` through cbindgen.
- Add C integration coverage for one form target, multiple paths, native encoding round trips, invalid pointers/lengths, undersized output buffers, and one-time delivery.
- Add a scripted or fixture-driven demo path that injects a positioned file drop at a labeled form node and records a screenshot/frame dump of its hover and accepted state. Keep the fixture independent of a live OS file drag.
- Run `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.

## Acceptance Criteria

- [x] An application can make an existing form or container node accept dropped paths with one generic target call and no new widget type.
- [x] The result contains native `PathBuf`s in Rust; akar does no file I/O, metadata lookup, or decoding.
- [x] Multi-file drops and multiple drops between frames retain all paths and correct positions; each drop is delivered at most once.
- [x] Hover, leave, and drop work with an explicit host-supplied position, and the target follows current layout after resize or recomputation.
- [x] Parent/child precedence, clipping, zero-area nodes, and event lifetime are covered by CPU-only tests.
- [x] The C API preserves paths without lossy conversion and is covered by a compiled C integration test; `akar.h` is generated.
- [x] Stock winit's path-only events never masquerade as accurately positioned node drops. The exact limitation and host-provided position route are documented.
- [x] A deterministic demo fixture shows an existing form accepting a path; the full workspace formatting, Clippy, and test gates pass.

## Out of Scope

- Opening, validating, reading, uploading, or parsing a dropped path.
- Dragging files out of akar into the operating system.
- A mandatory drop-zone widget or automatic style for every target.
- Replacing the application's event loop or adding an async runtime.
- OS-specific drag APIs in `akar-core` or `akar-components`.
