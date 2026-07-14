# CLAUDE.md — Claude Code Guide

This file is auto-loaded by Claude Code as project context. The cross-agent guide (`AGENTS.md`) covers the same material for arbitrary coding agents; this file points at it and adds Claude Code-specific notes.

## What akar is

A GPU-accelerated immediate-mode UI component library with a C ABI. The Rust crates are the implementation; `akar.h` is the contract. Integrate akar from any language via the C API, not the Rust API directly.

akar is **primarily built by agents** and is designed to be used by other projects that need a cross-platform UI framework which **works and debugs well for agents** — especially multi-modal LLMs that can read the screenshots it produces. Claude Code is image-capable, so the `demo-rust` debug toolchain is tuned for this workflow: make a change, capture a PNG, read it back here, iterate — no human intermediation.

## Before starting any task

1. Confirm which epic is active (`epics/` — lowest-numbered without `Status: Done`). Pick the next epic from the roadmap or open a new one; do not rely on this or any other doc for epic status — read the epic files directly.
2. Read the full epic before touching any file.
3. Cross-reference `DEVELOP.md` for local dependency paths and architectural constraints.

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

## Debug toolchain (the primary feedback loop)

The `demo-rust` binary ships with a complete visual debug toolchain so you can see, isolate, script, and diff the UI — no external screen-capture tooling required. As Claude Code you can read the resulting PNGs directly, which is the intended loop. Full flag reference and the recommended iteration loop are in `AGENTS.md` → "Debug toolchain"; the essentials:

```bash
# Full-window screenshot, default 5s delay, then exit
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --exit

# Tight, auto-cropped capture of one component (noise-free)
cargo run --release --bin demo-rust -- --component drawer --screenshot /tmp/drawer.png --exit

# Element discovery: list isolable components, or all labeled layout rects
cargo run --release --bin demo-rust -- --list-components
cargo run --release --bin demo-rust -- --dump-layout

# Scripted interactive state (hover, click, open dropdown/modal) with @label addressing
cargo run --release --bin demo-rust -- --script /tmp/steps.txt

# Structured JSON frame dump (every draw call incl. culled, scissor, z, input snapshot)
cargo run --release --bin demo-rust -- --dump-frame /tmp/frame.json --screenshot /tmp/x.png --exit

# Diff two PNGs, or gate CI on a change threshold
akar-diff --diff /tmp/baseline.png /tmp/current.png -o /tmp/diff.png
akar-diff --compare /tmp/baseline.png /tmp/current.png --threshold 0.5
```

Tip: prefer `--component <name> --screenshot …` for a focused image of the thing you're changing; reach for `--script` when the bug is in an interactive state; `--dump-frame` when the visual alone can't explain it; `akar-diff --compare` against a baseline to confirm a fix didn't regress.

## What NOT to do

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

See `AGENTS.md` → "C ABI contract" for the per-frame call sequence (`akar_ctx_new` → `akar_begin_frame` → input/component calls → `akar_end_frame` → `akar_ctx_free`). No heap allocations are expected on the caller side beyond the context handle.

## Draw list / component / virtualization contracts

See `AGENTS.md` for the full contracts: draw list (push/cull/scissor-stack/z-sort), component function shape (query layout → submit draw calls → hit-test → return state), and virtualization (scissor before children, `list_clip` API, skip `shape_until_scroll` outside the scissor). The same rules apply here.

## Testing approach

- No live GPU in CI — component logic and layout resolution must be testable without a real wgpu device.
- A `MockDrawList` that records submitted calls is the primary unit-test tool.
- Visual verification uses the debug toolchain above. As a Claude Code user you can read the captured PNGs directly — that's the intended regression check.
- C ABI tests are written in C and compiled as integration tests under `crates/akar-c-api/tests/`.
- Run `cargo test --workspace` to execute all tests.

## Style

- Follow the conventions in `DEVELOP.md` → Coding Conventions.
- No trailing summaries at the end of responses — the diff is visible.
- No emojis in source or docs unless explicitly requested.
- No comments unless the WHY is non-obvious. Code should be self-documenting.