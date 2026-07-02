# CLAUDE.md — Claude Code Guide

## Before you start any task

1. Read `DEVELOP.md` in full. It has the local dependency table, the planned project structure, and the architectural decisions made so far.
2. Read the active epic in `epics/` (the lowest-numbered file without `Status: Done`). Each epic defines the current work scope and its acceptance criteria.
3. If a task requires understanding a local reference project (glyphon, sugacode, xilem, etc.), read the source files directly from `~/Projects/<name>/` — do not guess from memory or crates.io docs.

## Reference projects

All open source dependencies and reference projects are cloned locally under `~/Projects/`. Always read source from there. Key ones for akar:

- `~/Projects/sugacode/src/renderer.rs` and `~/Projects/sugacode/src/ui/` — the direct predecessor. Understand this before touching any rendering code.
- `~/Projects/glyphon/src/` — the text renderer akar wraps. Read `text_render.rs` and `text_atlas.rs` for the prepare/render contract.
- `~/Projects/wgpu/` — wgpu 29 source. Check here when API behavior is unclear.
- `~/Projects/xilem/` — reference for what a mature retained-mode Rust UI looks like.
- `~/Projects/daisyui/` and `~/Projects/shadcn_ui/` — component naming, token structure, and API surface that akar mirrors in immediate mode.

## What NOT to do

- Do not write code before Epic 001 is marked `Status: Done`. Epic 001 is a research and design epic — its output is Epic 002, which is where implementation begins.
- Do not impose async, tokio, channels, or any concurrency model on akar's public API.
- Do not add windowing (winit, SDL, GLFW) to `akar-core` or `akar-components`. Windowing belongs in `akar-winit` and is always optional.
- Do not add accessibility scaffolding in v1. Document the punt in code comments if relevant.
- Do not generate or modify `akar.h` by hand — it is always `cbindgen`-generated from `akar-c-api`.

## Crate responsibility boundaries

| Crate | Owns | Must NOT touch |
|---|---|---|
| `akar-core` | wgpu pipelines, draw list, scissor, input state struct | Layout, components, windowing, C API |
| `akar-layout` | taffy wrapper, flex tree → pixel rect resolution | Rendering, components |
| `akar-components` | All UI components; calls core + layout | wgpu directly, windowing |
| `akar-c-api` | `extern "C"` surface, `AkarCtx` opaque handle | Business logic (delegates to components) |
| `akar-winit` | winit event → akar input bridge | Everything else |

## Style

- Follow the conventions in `DEVELOP.md` → Coding Conventions.
- No trailing summaries at the end of responses — the diff is visible.
- Prefer reading existing files before editing. Use local source paths, not URLs.
