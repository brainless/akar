# AGENTS.md — Guide for Coding Agents

This document defines how coding agents interact with the akar codebase. Read it before touching anything.

## What akar is

A GPU-accelerated immediate-mode UI component library with a C ABI. The Rust crates are the implementation; `akar.h` is the contract. An agent that needs to use akar from any language targets the C API, not the Rust API directly.

## Before starting any task

1. Confirm which epic is active (`epics/` — lowest-numbered without `Status: Done`).
2. Read the full epic before touching any file.
3. Cross-reference `DEVELOP.md` for local dependency paths and architectural constraints.
4. Do not begin implementation tasks until Epic 001 is `Status: Done`. The architecture is not yet stable.

## Local source access

All reference projects are cloned locally. Prefer reading these over web searches:

```
~/Projects/wgpu/          — wgpu 29 source
~/Projects/glyphon/src/   — text renderer (TextRenderer, TextAtlas, Viewport)
~/Projects/glam-rs/src/   — math types
~/Projects/sugacode/src/  — predecessor app; rendering pattern reference
~/Projects/xilem/         — mature Rust retained-mode UI (reference only)
~/Projects/daisyui/       — CSS component library (naming/token reference)
~/Projects/shadcn_ui/     — React component library (API ergonomics reference)
```

Do not fetch URLs for these projects. Read files locally.

## C ABI contract

Once `akar-c-api` exists, agents integrating akar from non-Rust languages must:

1. Link against the compiled shared library (`libakar.dylib` / `libakar.so` / `akar.dll`).
2. Include the generated `akar.h` (do not write it manually — it is `cbindgen` output).
3. Call `akar_ctx_new(device_ptr, queue_ptr)` to create an `AkarCtx*`. All subsequent calls take this pointer as the first argument.
4. Each frame: call `akar_begin_frame(ctx, width, height, dpi)`, submit input via `akar_set_mouse*` / `akar_push_char` etc., call component functions, call `akar_end_frame(ctx)`.
5. Call `akar_ctx_free(ctx)` on shutdown.

No heap allocations are expected on the caller side beyond the context handle. All internal buffers are owned by the context.

## Constraints agents must respect

**Never impose on the developer:**
- An event loop — akar is driven by the developer's loop
- An async runtime — all akar APIs are synchronous
- A message-passing model — no channels, no callbacks unless explicitly opted into
- A windowing library — the developer supplies device/queue/surface

**Never add to `akar-core` or `akar-components`:**
- winit, SDL, or any windowing dependency
- tokio, async-std, or any async runtime
- Accessibility scaffolding (deferred, document the punt)

**Never edit `akar.h` directly.** Regenerate it with `cbindgen`.

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
