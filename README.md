# akar

A GPU-accelerated, language-neutral UI component library built on [wgpu](https://github.com/gfx-rs/wgpu) and [glyphon](https://github.com/grovesNL/glyphon). Provides ready-to-use components — buttons, cards, inputs, tables, overlays — styled out of the box and arranged with a flexbox layout engine.

## Why

Building a desktop or embedded UI with wgpu today means writing your own rect renderer, text shaping pipeline, layout engine, hover/focus state machine, and component primitives from scratch — every time. akar collapses that boilerplate into a single library with a stable C ABI, so you focus on your application, not the rendering plumbing.

The component catalog is inspired by shadcn/ui and daisyUI: a small set of well-styled, composable primitives that cover 90% of real UIs without fighting a framework.

## Design philosophy

- **Immediate mode.** You call draw functions; the library draws. No retained widget tree, no diffing, no ownership puzzles. State lives where you put it.
- **Language neutral.** The public API is a C ABI (a `.dylib`/`.so`/`.dll` + `akar.h`). Any language that can call C can use akar. Rust is the implementation detail.
- **Zero framework opinions.** No event loop, no async runtime, no message-passing model imposed. You feed input; you drive the frame. Wire it to winit, SDL2, GLFW, or a test harness — your choice.
- **Batteries-included components.** Buttons, badges, labels, cards, inputs, checkboxes, toggles, selects, sliders, modals, drawers, tables, progress bars, toasts — pre-styled, themeable via a flat token struct.
- **Layout via Flexbox.** Built on [taffy](https://github.com/DioxusLabs/taffy): the same CSS Flexbox model you already know, resolved to pixel coordinates before draw calls.
- **Virtualization first.** Infinite scroll and large data grids are first-class via a list-clipper API. The library never renders what is off-screen.
- **Built by agents, debuggable by agents.** akar is primarily written by coding agents, and is designed to be used by other projects that need a cross-platform UI framework which works and debugs well for agents — especially multi-modal LLMs. The `demo-rust` binary ships with a complete visual debug toolchain (screenshot capture, scripted input injection, layout/frame inspection, component isolation, and a diff tool) so an agent can see, isolate, and iterate on its UI with no human in the loop.

## For whom

- **Rust developers** who want an ImGui-class productivity boost without giving up wgpu's rendering power.
- **Non-Rust developers** (Go, Python, Zig, Swift, C#, Odin...) who want a native GPU UI without a Rust toolchain in their build.
- **Game and simulation developers** who need UI panels that coexist with a wgpu render pass.
- **Tool authors** — CLI tools with a GUI escape hatch, data viewers, dev-tool overlays.

## Stack

- **Renderer:** wgpu 29 (quad + text pipeline)
- **Text:** glyphon (cosmic-text backed, GPU atlas)
- **Layout:** taffy (CSS Flexbox / Grid)
- **Math:** glam
- **C ABI:** Rust `extern "C"` + `cbindgen`-generated `akar.h`
- **Optional windowing integration:** winit (in a separate `akar-winit` crate)

## Status

**Pre-alpha.** The API is functional but may change as development continues. See `epics/` for the design roadmap and completion status.

## Screenshot workflow

akar's `demo-rust` binary ships with a visual debug toolchain purpose-built for agent-led development (especially multi-modal LLMs). It captures exactly what akar rendered — no OS chrome — via wgpu intermediate-texture readback, identically on macOS, Windows, and Linux.

```bash
# Basic capture after default 5s delay, then exit
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --exit

# Configurable delay (float seconds; 0 = first frame)
cargo run --release --bin demo-rust -- --screenshot /tmp/demo.png --delay 0.5 --exit
```

Beyond the basic capture, the toolchain includes:

- `--script <FILE>` — line-based input injection (`hover`, `press`, `release`, `click`, `scroll`, `key`, `type`, `delay`, `screenshot`) with `@label` element addressing, for capturing non-idle and interactive states frame-precisely.
- `--dump-layout` — prints `name x y w h` for every labeled layout node and exits (element discovery for `@label` addressing and coordinate fallback).
- `--dump-frame <PATH>` — structured JSON dump for the captured frame: every draw call (including culled ones, with z-order and scissor), labeled layout rects, and an input snapshot.
- `--component <name>` / `--list-components` — isolate a single component, force its interesting state once (open drawer/dropdown/modal), and **auto-crop** the PNG to its bounding box, removing unrelated UI as visual noise.
- `akar-diff` — standalone binary (no GPU/akar deps). `--diff BASE CUR -o OUT.png` draws a visual diff (changed pixels red); `--compare BASE CUR --threshold PCT` exits non-zero when the changed-pixel ratio exceeds a threshold for CI regression gates.

See `AGENTS.md` → "Debug toolchain" for the recommended iteration loop and full flag reference. Design and history: `epics/013-screenshot-utility.md`, `epics/014-screenshot-enhancements.md`, `epics/015-component-isolation.md`.

## License

MIT
