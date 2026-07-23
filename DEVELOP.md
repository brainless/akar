# DEVELOP.md — Development Guide

## Project Status

akar is in **pre-alpha / active development**. No stable public API exists yet. Architecture decisions and completion status are recorded in `epics/` (one file per epic) — check `epics/` for what's done and what's next; do not rely on this file or any other doc for epic status.

akar is primarily built by agents. The `demo-rust` binary ships with a complete visual debug toolchain (see "Screenshot Workflow" below) so a multi-modal agent can make a change, see its result, and iterate with no human in the loop. This is a first-class design goal, not a side benefit.

## Local Dependencies

All crate dependencies come from crates.io. The relevant dependency sources and design references below are cloned locally under `~/Projects/` for research and behavioral reference. Coding agents and contributors should read these local sources directly rather than relying on crates.io documentation or GitHub browsing: they are the local source of truth for internals and undocumented behavior. They are not Cargo path dependencies.

### Direct dependency sources

| Project | Local path | What we learn from it |
|---|---|---|
| **glyphon** | `~/Projects/glyphon` | GPU text rendering via cosmic-text + wgpu. akar's text pipeline. Read `text_render.rs`, `text_atlas.rs` first. |
| **glam** | `~/Projects/glam-rs` | Math types (Vec2, Vec4, Mat4). Reference for geometry and layout internals. |
| **wgpu** | `~/Projects/wgpu` | GPU pipeline, render passes, buffer management. Source of truth for wgpu 29 internals. |
| **taffy** | `~/Projects/taffy` | CSS Flexbox + Grid behavior and layout-engine internals. akar's layout engine. |
| **winit** | `~/Projects/winit` | Window and event integration. Its use is restricted to `akar-winit` and examples. |

### Downstream application reference

| Project | Local path | What we learn from it |
|---|---|---|
| **daftprompt** | `~/Projects/daftprompt` | The successor to sugacode and a real akar application. Reference for canvas, drawer, search, and application integration patterns. Read `src/ui/render.rs` and `src/ui/` first. |

### UI, renderer, and API design references

These are local reference-only checkouts; they are not dependencies.

| Project | Local path | What to study |
|---|---|---|
| **egui** | `~/Projects/egui` | Immediate-mode Painter API, Response type, Id/Memory system, and layout cursor. |
| **Dear ImGui** | `~/Projects/imgui` | DrawList, clipper, input model, docking, and C API surface. |
| **Nuklear** | `~/Projects/Nuklear` | Single-header C immediate-mode UI and backend-agnostic C API. |
| **sokol** | `~/Projects/sokol` | Language-neutral C API and header design, especially `sokol_gfx.h`. |
| **Zed / GPUI** | `~/Projects/zed/crates/gpui` | Production wgpu UI: scene graph, element/layout protocol, and platform abstraction. |
| **xilem** | `~/Projects/xilem` | Retained-mode architecture, widget lifecycle, and accessibility concepts that akar deliberately defers. |
| **daisyUI** | `~/Projects/daisyui` | Component catalog shape, naming, and token-based themes. |
| **shadcn/ui** | `~/Projects/shadcn_ui` | Component API ergonomics and composition patterns adapted to immediate mode. |

## Build & Run

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Run the demo application
cargo run --example demo-rust
```

## Screenshot Workflow

The `demo-rust` binary is the primary visual feedback loop for UI work, and it is tuned for agent-led (and specifically multi-modal-LLM) development. It captures exactly what akar rendered — no OS chrome, no overlapping windows — using wgpu intermediate-texture readback, so it works identically on macOS, Windows, and Linux.

```bash
# Basic: full-window screenshot after the default 5s delay, then exit
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --exit

# Configurable delay (float seconds; 0 = first frame)
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --delay 0.5 --exit
```

Beyond the basic capture, the binary exposes a full debug toolchain (see `AGENTS.md` → "Debug toolchain" for the recommended loop and full flag reference):

- `--script <FILE>` — line-based input injection (`hover`, `press`, `release`, `click`, `scroll`, `key`, `type`, `delay`, `screenshot`) with `@label` element addressing, for capturing non-idle/interactive states frame-precisely.
- `--dump-layout` — prints `name x y w h` for every labeled layout node and exits (element discovery).
- `--dump-frame <PATH>` — structured JSON dump for the captured frame: every draw call (including culled ones, with z-order and scissor), labeled layout rects, and an input snapshot.
- `--component <name>` / `--list-components` — isolate a single component, force its interesting state once (open drawer/dropdown/modal), and **auto-crop** the PNG to its bounding box, removing unrelated UI as visual noise.
- `akar-diff` — standalone binary, no GPU/akar deps. `--diff BASE CUR -o OUT.png` draws a visual diff (changed pixels red, unchanged dimmed); `--compare BASE CUR --threshold PCT` exits non-zero when the changed-pixel ratio exceeds a threshold (CI regression gate).

Design rationale and history for the toolchain live in `epics/013-screenshot-utility.md`, `epics/014-screenshot-enhancements.md`, and `epics/015-component-isolation.md`.

**Remaining limitations** (deferred, not blocking agent workflows):
- No perceptual diff — `akar-diff` is pixel-exact for v1.
- No headless/offscreen rendering — `AkarCore::mock` and the intermediate-texture capture path make it architecturally feasible, but adapter availability on CI runners (no software Metal on macOS; `lavapipe` Linux-only; WARP Windows-only) is the real blocker. Punt documented in `epics/014` (Task 014e); a future epic will address it when CI visual regression is prioritized.

## Project Structure

```
akar/
├── Cargo.toml                    # workspace root
├── README.md
├── DEVELOP.md
├── CLAUDE.md
├── AGENTS.md
├── akar.h                        # cbindgen-generated C header
├── epics/                        # design roadmap, one file per epic
├── crates/
│   ├── akar-core/                # quad renderer, draw list, text pipeline, input state, screenshot
│   ├── akar-layout/              # taffy wrapper; resolves flex trees to pixel rects
│   ├── akar-components/          # all UI components (30+ components implemented)
│   ├── akar-c-api/               # extern "C" bindings; produces libakar + akar.h
│   └── akar-winit/               # optional: winit event → akar input bridge
└── examples/
    ├── demo-rust/                # comprehensive demo of all components; CLI debug toolchain
    ├── canvas-basic-rust/        # canvas component example
    └── akar-diff/                # standalone PNG diff/compare binary (no GPU/akar deps)
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

### Canvas LOD and portals

Canvas provides continuous level of detail via projected screen dimensions. `CanvasResponse::project()` returns a `CanvasProjectedRect` with the object's screen rect, pixels-per-world-unit, and visibility. `lod_index()` classifies an object against caller-supplied pixel thresholds using the projected minimum dimension.

Applications define arbitrary LOD thresholds — the library does not prescribe a fixed set of levels. Common patterns: dot (smallest), outline + summary label, preview, and interactive portal.

Low-detail levels use `CanvasPainter` primitives: transformed quads (with zoom-scaled border width, corner radii, and shadow fields) and display text (`push_text` with `CanvasTextStyle`). Group-level world-space hover/press/click detection is available through `CanvasInput`.

At the interactive threshold, applications render a normal screen-space layout subtree through `canvas_portal_begin/end`. Portal layouts have their own `screen_origin` and `namespace_id` to avoid widget ID collisions. Applications own portal layout lifecycle — create on threshold entry, drop on exit. No akar-owned registry or eviction policy.

Canvas text is display-only: no focus, no widget identity, no input. It does not create layout nodes or text-buffer IDs. Use portal mode when interactive text or child components are needed.

The `examples/canvas-basic-rust/` example demonstrates the full LOD + portal pattern with dot, outline+summary, preview, and interactive portal levels.

Renderer limitation: glyphon text renders after quads globally (not per-draw-call ordered). Strict quad/text interleaving within a frame requires a separate renderer architecture change.

### Data items and lists

Data items are presentation primitives, not records (ADR-016). akar does not define `Tweet`, `Message`, `Commit`, or any generic owned record type. A data item is a composable visual shell over caller-provided layout nodes and caller-owned content. The item response reports hover, press, and click state without retaining or mutating caller selection.

Lists are fixed-height virtualized scopes with caller-owned scroll state (ADR-017). The caller supplies item count, row height, and stable per-item keys. `data_list_begin` returns the visible range and content origin; the caller renders only that range. Scroll position is caller-owned, allowing applications to persist, synchronize, or reset it deliberately. Variable-height virtualization is out of scope.

Widget identity inside a virtualized list is keyed, not positional (ADR-016a). Identity is composed from `namespace_id`, a caller-provided stable item key (record identity), and the local structural node id. Using plain `widget_id(node)` ties identity to the screen row and corrupts focus/text-buffer state on scroll. Every focusable child rendered inside a data item must use `widget_id_keyed(node, key)`.

Canvas summary items (`canvas_data_item`) are display-only. They render world-space backgrounds and textual fields through `CanvasPainter` with group-level hover/press/click via `CanvasInput`. They create no layout nodes, focusable widgets, text-buffer IDs, or child hit targets. Use portal mode (`canvas_portal_begin/end`) for interactive components inside a canvas.

### What akar does NOT own

- The window and swap chain — developer provides these.
- The event loop — developer drives it.
- The async runtime — none required. akar is synchronous.
- Message passing — no channels, no callbacks unless explicitly opted into.
- Accessibility — deferred beyond v1.

## Coding Conventions

- Edition 2021, MSRV TBD (will track wgpu's MSRV).
- Errors: `thiserror` for library crates, `anyhow` for examples and binaries.
- Logging: `log` facade (no tracing — akar does not impose an async subscriber).
- No `unsafe` outside `akar-c-api` FFI boundary.
- No emojis in source or docs unless explicitly requested.
- No comments unless the WHY is non-obvious. Code should be self-documenting.
- No imposed async. All public APIs are synchronous.
