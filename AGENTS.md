# AGENTS.md — Guide for Coding Agents

This document defines how coding agents interact with the akar codebase. Read it before touching anything.

## What akar is

A GPU-accelerated immediate-mode UI component library with a C ABI. The Rust crates are the implementation; `akar.h` is the contract. An agent that needs to use akar from any language targets the C API, not the Rust API directly.

akar is **primarily built by agents** and is designed to be used by other projects that need a cross-platform UI framework which **works and debugs well for agents** — especially multi-modal LLMs that can read the screenshots it produces. The `demo-rust` binary ships with a full visual debug toolchain (see "Debug toolchain" below) so an agent can make a change, see the result, and iterate with no human in the loop.

## Before starting any task

1. Confirm which epic is active (`epics/` — lowest-numbered without `Status: Done`). Pick the next epic from the roadmap or open a new one; do not rely on this or any other doc for epic status — read the epic files directly.
2. Read the full epic before touching any file.
3. Cross-reference `DEVELOP.md` for local dependency paths and architectural constraints.

## Local source access

All reference projects are cloned locally. Prefer reading these over web searches — they are the authoritative source for internals and undocumented behavior.

| Project | Local path | Read first |
|---|---|---|
| **sugacode** | `~/Projects/sugacode/src/` | `renderer.rs`, `src/ui/` — predecessor app, rendering pattern reference |
| **glyphon** | `~/Projects/glyphon/src/` | `text_render.rs`, `text_atlas.rs` — text renderer akar wraps |
| **wgpu** | `~/Projects/wgpu/` | GPU pipeline, render passes, buffer management (wgpu 29 internals) |
| **glam** | `~/Projects/glam-rs/src/` | Math types (Vec2, Vec4, Mat4) |
| **xilem** | `~/Projects/xilem/` | Mature Rust retained-mode UI (reference only) |
| **daisyui** | `~/Projects/daisyui/` | CSS component library (naming/token reference) |
| **shadcn_ui** | `~/Projects/shadcn_ui/` | React component library (API ergonomics reference) |

Do not fetch URLs for these projects. Read files locally.

## What NOT to do

- Do not impose an event loop — akar is driven by the developer's loop.
- Do not impose an async runtime — all akar APIs are synchronous.
- Do not add windowing (winit, SDL, GLFW) to `akar-core` or `akar-components`. Windowing belongs in `akar-winit` and is always optional.
- Do not add accessibility scaffolding in v1. Document the punt if relevant.
- Do not edit `akar.h` directly — it is always `cbindgen`-generated from `akar-c-api`.

## Debug toolchain (visual feedback loop)

This is the primary feedback loop for UI work, and it is built specifically for agents. akar's `demo-rust` binary ships with a complete capture/inspect toolchain so an agent can see, isolate, script, and diff its UI — no human intermediation and no external screen-capture tooling required.

The screenshot captures exactly what akar rendered (no OS chrome) using wgpu intermediate-texture readback. It works identically on macOS, Windows, and Linux. Design and history are in `epics/013-screenshot-utility.md`, `epics/014-screenshot-enhancements.md`, and `epics/015-component-isolation.md`.

### Capture

```bash
# Basic: full-window screenshot after default 5s delay, then exit
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --exit

# Configurable delay (float seconds; 0 = first frame). Lets agents iterate fast.
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --delay 0.5 --exit
```

### Scripted input (non-idle states)

`--script <FILE>` drives the demo into a non-idle state (hover, press, focus, open dropdown/modal) and captures the result without manual interaction. `--script` and `--screenshot` are mutually exclusive — the script issues its own `screenshot` lines (and may issue several). Line-based, one step advanced per frame for precise frame alignment:

```
# comment
hover @submit_button          # @label OR bare X Y
delay 0.1
click @submit_button          # same-frame press+release → is_clicked fires this frame
press left                    # hold a button across frames (active-state shots)
release left
scroll 0 -120
key Enter
type "hello"
screenshot /tmp/pressed.png   # capture on this frame; can repeat for before/after
```

Element addressing is **labels-first on top of coordinates**. Labels are a `HashMap<String, NodeId>` in `akar-layout`; the demo registers ~20 of its named interaction nodes (e.g. `@navbar_dropdown`, `@form_submit`). Coordinates remain the fallback for inline-computed rects (dropdown items, list rows) that have no `NodeId` to register.

### Layout & frame inspection

```bash
# Print "name x y w h" for every labeled layout node, then exit (element discovery)
cargo run --release --bin demo-rust -- --dump-layout

# Structured JSON dump for the captured frame: every draw call (incl. culled, with
# z-order and scissor), labeled layout rects, and an input snapshot.
cargo run --release --bin demo-rust -- --dump-frame /tmp/frame.json --screenshot /tmp/x.png --exit
```

`--dump-frame` uses a gated recording mode in `DrawList` that snapshots `{call, scissor}` *before* the scissor-cull early-return, so culled calls are included — useful for "why didn't my quad render?" debugging.

### Component isolation

`--component <name>` renders a single component, forces its interesting state once (open drawer, open dropdown, etc.), and **auto-crops** the PNG to that component's bounding box + padding — removing unrelated UI as visual noise.

```bash
# Discovery: list isolable component names and exit
cargo run --release --bin demo-rust -- --list-components

# Isolate just the drawer (forced open), auto-cropped
cargo run --release --bin demo-rust -- --component drawer --screenshot /tmp/drawer.png --exit

# Composes with --script (force runs once, script may then transition state)
cargo run --release --bin demo-rust -- --component dropdown --script /tmp/hover_item.txt
```

Unknown component names print an error with the valid list and exit non-zero. Composes with `--dump-frame` (the dump reflects only the isolated component's calls). Demo-only feature; no `akar-core`/`akar-layout`/`akar-components` involvement.

### Diff & regression

The standalone `akar-diff` binary compares two PNGs — no GPU, no akar deps:

```bash
# Visual diff: changed pixels in red, unchanged dimmed to 30%
akar-diff --diff /tmp/baseline.png /tmp/current.png -o /tmp/diff.png

# CI gate: exit non-zero when changed-pixel ratio exceeds threshold
akar-diff --compare /tmp/baseline.png /tmp/current.png --threshold 0.5
```

Multi-capture in one run is expressed via multiple `screenshot` lines in a `--script` (before/after), not a separate flag. Baselines are caller-managed file paths; perceptual diff is deferred (pixel-exact for v1).

### Recommended loop

1. Make your change.
2. `--list-components` / `--dump-layout` to find what to capture and where it is.
3. `--component <name> --screenshot …` for a tight, noise-free image (or `--screenshot` for the full window).
4. `--script` when the issue is in an interactive state.
5. `--dump-frame` when the visual alone isn't enough.
6. `akar-diff --compare` against a baseline to verify the fix didn't regress.

## Crate responsibility boundaries

| Crate | Owns | Must NOT touch |
|---|---|---|
| `akar-core` | wgpu pipelines, draw list, scissor, input state struct | Layout, components, windowing, C API |
| `akar-layout` | taffy wrapper, flex tree → pixel rect resolution | Rendering, components |
| `akar-components` | All UI components; calls core + layout | wgpu directly, windowing |
| `akar-c-api` | `extern "C"` surface, `AkarCtx` opaque handle | Business logic (delegates to components) |
| `akar-winit` | winit event → akar input bridge | Everything else |

## C ABI contract

Once `akar-c-api` exists, agents integrating akar from non-Rust languages must:

1. Link against the compiled shared library (`libakar.dylib` / `libakar.so` / `akar.dll`).
2. Include the generated `akar.h` (do not write it manually — it is `cbindgen` output).
3. Call `akar_ctx_new(device_ptr, queue_ptr)` to create an `AkarCtx*`. All subsequent calls take this pointer as the first argument.
4. Each frame: call `akar_begin_frame(ctx, width, height, dpi)`, submit input via `akar_set_mouse*` / `akar_push_char` etc., call component functions, call `akar_end_frame(ctx)`.
5. Call `akar_ctx_free(ctx)` on shutdown.

No heap allocations are expected on the caller side beyond the context handle. All internal buffers are owned by the context.

## Draw list contract (for `akar-core` contributors)

The draw list is the internal rendering queue. Agents extending the renderer must follow:

1. All draw calls are submitted via `DrawList::push(DrawCall)` during `begin_frame` → `end_frame`.
2. Before GPU upload, `DrawList::flush()` culls calls whose AABB does not intersect the active scissor rect. This is automatic and invisible to component authors.
3. Scissor rects are pushed/popped in a stack (`DrawList::push_scissor` / `pop_scissor`). Scroll areas push a scissor before rendering children.
4. Z-order is explicit: each `DrawCall` carries a `z: f32`. The draw list sorts ascending before flush.

## Component contract (for `akar-components` contributors)

Each component function:
1. Calls `akar-layout` to query resolved pixel bounds for its node ID.
2. Checks hit-test from the input state to determine hover/active/focus/click.
3. If clicked, mutates any caller-owned state (`*checked`, `*selected`, etc.) immediately — before submitting draw calls. Drawing must always reflect the post-click value, never the pre-click one. A component that draws first and mutates after looks correct in isolation but silently lags one frame behind every click, and needs an unrelated event (e.g. a mouse move) to ever catch up. See checkbox/switch/radio/tab_bar history.
4. Submits background rect + border rect + text (if any) to the draw list via `akar-core` primitives, using the already-updated state.
5. Returns a state enum (`Idle | Hovered | Pressed | Focused`) or a typed result (`Clicked: bool`, `value: f32`, etc.).
6. Must work correctly with a zero-area rect (when the layout system gives it no space).

## Virtualization contract

For scroll containers and list components:
- Always push a scissor rect before rendering children; pop it after.
- Expose `list_clip(total, item_height, scroll_y)` so developers can avoid submitting off-screen items entirely.
- Do not call `glyphon::Buffer::shape_until_scroll` for items outside the scissor rect.

## Canvas and portal guidance

When working with canvas changes, verify at one overview level and one interactive-portal level using screenshots. Use `--component` or `--screenshot` with `examples/canvas-basic-rust/`.

Low-detail canvas interaction is group-level only — hover/press/click on the whole object, not child widgets. `CanvasInput` operates on `WorldRect` bounds.

Canvas text is display-only. It never creates focus, widget state, or text-buffer IDs. Use portal mode for interactive text inputs, selects, buttons, or any component requiring child interaction.

`canvas_portal_begin/end` push/pop scissors. The portal subtree renders through normal component APIs — no canvas-specific component variants needed. Portal layouts must use unique `namespace_id` values to avoid widget ID collisions.

The scissor stack intersects automatically. A portal inside a canvas is clipped to both the portal bounds and the canvas bounds.

glyphon text renders after quads globally. Do not expect strict quad/text ordering within a frame — this is a known renderer limitation.

Reference: `examples/canvas-basic-rust/` for the canonical LOD + portal pattern.

## Testing approach

- No live GPU in CI — component logic and layout resolution must be testable without a real wgpu device.
- A `MockDrawList` that records submitted calls is the primary unit-test tool.
- Visual verification uses the debug toolchain (see "Debug toolchain" above). The fastest loop is usually `--component <name> --screenshot …` for a tight, noise-free image; `--script` for interactive states; `--dump-layout`/`--dump-frame` when the visual alone is not enough; `akar-diff --compare` against a baseline to detect regressions.
- C ABI tests are written in C and compiled as integration tests under `crates/akar-c-api/tests/`.
- Run `cargo test --workspace` to execute all tests.

## Style

- Follow the conventions in `DEVELOP.md` → Coding Conventions.
- No emojis in source or docs unless explicitly requested.
- No comments unless the WHY is non-obvious. Code should be self-documenting.
