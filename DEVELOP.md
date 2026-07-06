# DEVELOP.md — Development Guide

## Project Status

akar is in **pre-alpha / implementation phase**. Epic 006 (Text Pipeline) is complete. No stable public API exists yet. Architecture decisions are recorded in `epics/` as they are made.

## Local Dependencies

All crate dependencies come from crates.io. The projects below are cloned locally under `~/Projects/` for design research and behavioral reference — coding agents and contributors should read these local sources directly rather than relying on crates.io docs or GitHub browsing, as they are the authoritative source for internals and undocumented behavior.

### Reference-only local checkouts

These are NOT path dependencies but are cloned locally for design research and behavioral reference.

| Project | Local path | What we learn from it |
|---|---|---|
| **glyphon** | `~/Projects/glyphon` | GPU text rendering via cosmic-text + wgpu. akar's text pipeline. Read `text_render.rs`, `text_atlas.rs` first. |
| **glam** | `~/Projects/glam-rs` | Math types (Vec2, Vec4, Mat4). Reference for geometry and layout internals. |
| **wgpu** | `~/Projects/wgpu` | GPU pipeline, render passes, buffer management. Source of truth for wgpu 29 internals. |
| **sugacode** | `~/Projects/sugacode` | Author's own wgpu + glyphon app. The direct inspiration for akar; reference for renderer setup, TextAreaData pattern, UIManager pattern, scroll containers, and drawer. Read `src/renderer.rs` and `src/ui/` first. |
| **xilem** | `~/Projects/xilem` | Linebender's Rust reactive UI (Masonry + vello). Reference for retained-mode architecture, widget lifecycle, and accessibility model — things akar deliberately defers. |
| **daisyui** | `~/Projects/daisyui` | CSS component library. Reference for component catalog shape, naming, and the token-based theme model akar mirrors. |
| **shadcn_ui** | `~/Projects/shadcn_ui` | React component library. Reference for component API ergonomics and composition patterns akar adapts to immediate mode. |

### Projects to clone for Epic 001 exploration

These are not yet present locally but are required reading for Epic 001. Clone them to `~/Projects/` before starting exploration tasks.

| Project | Repo | What to study |
|---|---|---|
| **gpui** (Zed) | `github.com/zed-industries/zed` (subtree: `crates/gpui`) | wgpu-based retained UI in production. Scene graph, element/layout protocol, platform abstraction. |
| **egui** | `github.com/emilk/egui` | The dominant Rust immediate-mode UI. Painter API, Response type, Id/Memory system, layout cursor. |
| **Dear ImGui** | `github.com/ocornut/imgui` | The canonical immediate-mode reference. DrawList, clipper, input model, docking. C API surface. |
| **Nuklear** | `github.com/Immediate-Mode-UI/Nuklear` | Single-header C immediate-mode UI. Minimal, backend-agnostic. Best C ABI reference. |
| **taffy** | `github.com/DioxusLabs/taffy` | CSS Flexbox + Grid layout in Rust. akar's layout engine candidate. |
| **sokol** | `github.com/floooh/sokol` | C headers for GPU + app. Best example of a clean, language-neutral C API design (sokol_gfx.h pattern). |

## Build & Run

```bash
# (no code yet — these are the target commands once implementation begins)
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

## Planned Project Structure

```
akar/
├── Cargo.toml                    # workspace root
├── README.md
├── DEVELOP.md
├── CLAUDE.md
├── AGENTS.md
├── epics/                        # design roadmap, one file per epic
├── crates/
│   ├── akar-core/                # quad renderer, draw list, text pipeline, input state
│   ├── akar-layout/              # taffy wrapper; resolves flex trees to pixel rects
│   ├── akar-components/          # all UI components (button, card, input, table, ...)
│   ├── akar-c-api/               # extern "C" bindings; produces libakar + akar.h
│   └── akar-winit/               # optional: winit event → akar input bridge
├── bindings/
│   ├── go/
│   ├── python/
│   └── zig/
└── examples/
    ├── demo-rust/
    ├── demo-c/
    └── demo-go/
```

## Architecture Notes

### Rendering model

akar uses a **draw list** (immediate mode, painter's algorithm): components submit draw calls into a frame-scoped list, which is sorted by Z-order, CPU-culled against the current scissor rect, then flushed to the GPU in one pass.

Two render pipelines run in the same wgpu render pass:
1. **Quad pipeline** — axis-aligned rectangles with per-corner radius, solid fill, border. Implemented in a custom WGSL shader (SDF-based for anti-aliased corners).
2. **Text pipeline** — glyphon's `TextRenderer` / `TextAtlas` / `Viewport` pipeline. Glyphs are cached in a GPU texture atlas by cosmic-text.

The developer supplies a wgpu `Device + Queue + Surface` (or the C equivalent). akar does not own the swap chain or the event loop.

### Immediate mode and large datasets

Immediate mode does not conflict with virtualizing large lists or grids. The library provides:

- **`list_clip(total_items, item_height, scroll_y)`** → visible `(first, last)` range. The developer renders only that range. O(1).
- **`is_visible(y, h)`** → bool. Fast scissor-rect intersection check the developer can use to skip expensive per-item work (text shaping, image decoding).
- **Draw-list AABB culling** (automatic). Before GPU upload, quads outside the current scissor rect are dropped. The developer never sees this.

A 1M-row × 100-col grid costs ~1,000 draw calls per frame, not 100M.

### C ABI strategy

`akar-c-api` compiles to a shared library (`libakar.dylib` / `libakar.so` / `akar.dll`) with a `cbindgen`-generated `akar.h`. All state is opaque behind an `AkarCtx*` handle. The API is flat C — no C++ templates, no Rust generics, no callbacks unless the developer opts in.

Every language binding is a thin wrapper over `akar.h`. The bindings live in `bindings/` and are maintained alongside the C API.

### Theme system

A flat `AkarTheme` struct of color tokens and size tokens. No cascade, no inheritance. Two presets ship: `AKAR_THEME_DARK` and `AKAR_THEME_LIGHT`. The developer can swap presets or mutate individual tokens.

### What akar does NOT own

- The window and swap chain — developer provides these.
- The event loop — developer drives it.
- The async runtime — none required. akar is synchronous.
- Message passing — no channels, no callbacks unless explicitly opted into.
- Accessibility — deferred beyond v1.

## Coding Conventions

*(Will be expanded once implementation begins. Preliminary:)*

- Edition 2021, MSRV TBD (will track wgpu's MSRV).
- Errors: `thiserror` for library crates, `anyhow` for examples and binaries.
- Logging: `log` facade (no tracing — akar does not impose an async subscriber).
- No `unsafe` outside `akar-c-api` FFI boundary.
- No emojis in source or docs unless explicitly requested.
- No comments unless the WHY is non-obvious. Code should be self-documenting.
- No imposed async. All public APIs are synchronous.
