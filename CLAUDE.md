# CLAUDE.md — Claude Code Guide

## What akar is

A GPU-accelerated immediate-mode UI component library with a C ABI. The Rust crates are the implementation; `akar.h` is the contract. Integrate akar from any language via the C API, not the Rust API directly.

## Before starting any task

1. Confirm which epic is active (`epics/` — lowest-numbered without `Status: Done`).
2. Read the full epic before touching any file.
3. Cross-reference `DEVELOP.md` for local dependency paths and architectural constraints.
4. Do not begin implementation tasks until Epic 001 is `Status: Done`. The architecture is not yet stable.

## Local source access

All reference projects are cloned locally under `~/Projects/`. Read these directly — do not fetch URLs or guess from crates.io docs.

| Project | Local path | Read first |
|---|---|---|
| **sugacode** | `~/Projects/sugacode/src/` | `renderer.rs`, `src/ui/` — predecessor app, rendering pattern reference |
| **glyphon** | `~/Projects/glyphon/src/` | `text_render.rs`, `text_atlas.rs` — text renderer akar wraps |
| **wgpu** | `~/Projects/wgpu/` | GPU pipeline, render passes, buffer management (wgpu 29 internals) |
| **glam** | `~/Projects/glam-rs/src/` | Math types (Vec2, Vec4, Mat4) |
| **xilem** | `~/Projects/xilem/` | Mature Rust retained-mode UI (reference only) |
| **daisyui** | `~/Projects/daisyui/` | CSS component library (naming/token reference) |
| **shadcn_ui** | `~/Projects/shadcn_ui/` | React component library (API ergonomics reference) |

## What NOT to do

- Do not write implementation code before Epic 001 is `Status: Done`. Epic 001 is research and design — its output is Epic 002, where implementation begins.
- Do not impose an event loop — akar is driven by the developer's loop.
- Do not impose an async runtime — all akar APIs are synchronous.
- Do not add windowing (winit, SDL, GLFW) to `akar-core` or `akar-components`. Windowing belongs in `akar-winit` and is always optional.
- Do not add accessibility scaffolding in v1. Document the punt if relevant.
- Do not edit `akar.h` directly — it is always `cbindgen`-generated from `akar-c-api`.

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
2. Submits background rect + border rect + text (if any) to the draw list via `akar-core` primitives.
3. Checks hit-test from the input state to determine hover/active/focus.
4. Returns a state enum (`Idle | Hovered | Pressed | Focused`) or a typed result (`Clicked: bool`, `value: f32`, etc.).
5. Must work correctly with a zero-area rect (when the layout system gives it no space).

## Virtualization contract

For scroll containers and list components:
- Always push a scissor rect before rendering children; pop it after.
- Expose `list_clip(total, item_height, scroll_y)` so developers can avoid submitting off-screen items entirely.
- Do not call `glyphon::Buffer::shape_until_scroll` for items outside the scissor rect.

## Testing approach

*(Will be defined in Epic 002. Preliminary expectations:)*

- No live GPU in CI — component logic and layout resolution must be testable without a real wgpu device.
- A `MockDrawList` that records submitted calls is the primary unit-test tool.
- Visual regression tests (screenshot comparison) are manual for now.
- C ABI tests are written in C and compiled as integration tests under `crates/akar-c-api/tests/`.

## Style

- Follow the conventions in `DEVELOP.md` → Coding Conventions.
- No trailing summaries at the end of responses — the diff is visible.
- No emojis in source or docs unless explicitly requested.
- No comments unless the WHY is non-obvious. Code should be self-documenting.
