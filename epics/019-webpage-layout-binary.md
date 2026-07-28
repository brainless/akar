# Epic 019: Webpage Layout Binary

**Status:** Done
**Goal:** Create a standalone `webpage-rust` example binary that demonstrates webpage-like layouts built with akar, reproducing the Xiaomi MiMo landing page as a real-world reference implementation.

**Prerequisite:** Epic 018 is `Status: Done`.

---

## Context

akar's existing examples (`demo-rust`, `canvas-basic-rust`) showcase individual components and canvas features. None demonstrate a full webpage-style layout — stacked sections, card grids, scrollable content, and a fixed header — which is the most common real-world UI pattern downstream applications need.

The goal is a reference binary that proves akar can render a multi-section, scrollable webpage layout with:
- A fixed top navigation bar
- A hero section with background pattern and title
- A row of product cards with consistent spacing
- "Build with MiMo" and "Paper" sections with numbered list items
- Vertical scrolling with proper content-to-viewport clipping
- Full-screen window height for complete page screenshots

This binary also serves as a testbed for layout edge cases: flex gaps with background fills, auto-height leaf nodes in taffy, z-value clipping behavior in the quad shader, and font family selection via glyphon/cosmic-text.

---

## Design Decisions

### Manual Rendering over Component APIs

The binary renders each section with direct `push_quad` / `push_text` calls rather than using `akar-components` widget functions. This is deliberate: the goal is to demonstrate raw layout + rendering for webpage patterns, not to exercise the component catalog. It also avoids coupling the example to component API churn.

### Layout via Taffy Flex Column

The page is a single Column flex tree: `root > [header, scroll_container > [hero, cards_row, build_section, paper_section]]`. The `scroll_container` fills remaining viewport height via `flex_grow: 1.0` and hosts the scroll area. Leaf nodes that need visible area use explicit pixel heights — taffy resolves `Dimension::auto()` to 0 for leaf nodes with no children, which caused invisible sections.

### Scroll via `scroll_area_begin`/`scroll_area_end`

The scroll mechanism uses `akar-components`' `scroll_area_begin`/`scroll_area_end`, which push/pop a scissor rect and track `scroll_y` with hover-based wheel input. The caller computes `content_height` from layout rects and owns the scroll state.

### Z-Value Clipping Discovery

The quad WGSL shader passes `q.z` directly as clip-space z:
```wgsl
out.position = vec4<f32>(clip_pos.x, -clip_pos.y, q.z, 1.0);
```
Negative z values are clipped by the GPU rasterizer. All background quads originally at z:-1.0, z:-0.5, z:-0.3 were silently discarded. The fix: all z-values must be >= 0.0. Layering is achieved by sorting quads by ascending z before GPU upload.

### Font Family via `glyphon::Attrs`

`TextPipeline::set_text` previously hardcoded `glyphon::Attrs::new()` (defaulting to `Family::Serif` is wrong — it defaults to `Family::SansSerif`). To support serif headings matching the MiMo reference design, an `attrs: Option<glyphon::Attrs>` parameter was added. Callers pass `None` for default or `Some(Attrs::new().family(Family::Serif))` for serif text.

### Window Height from Monitor

The window size is set from `event_loop.primary_monitor().size()`, subtracting 40px for the OS taskbar. This ensures the full page is visible in screenshots without manual window resizing.

### Screenshot Without Input

The original redraw loop only requested redraws on non-redraw events. After the screenshot delay elapsed with no mouse/keyboard input, no redraw fired and the screenshot never triggered. Fix: unconditionally call `state.window.request_redraw()` at the end of the `RedrawRequested` handler.

---

## Tasks

### Task 1 — Initial Binary and MiMo Layout

**Status:** Done

- Create `examples/webpage-rust/` with `Cargo.toml` and `src/main.rs`.
- Implement the MiMo page layout: root Column, header Row (logo + nav items), hero section with "M I M O" pattern and "HELLO, I'M MiMo" title, three product cards in a Row, "Build with MiMo" section with numbered items, "Paper" section with paper listing.
- Render each section with direct `push_quad`/`push_text` calls using theme color constants.
- Add `--site mimo`, `--screenshot`, `--delay`, `--exit` CLI flags.
- Add wgpu surface setup, `AkarCore` initialization, and frame render loop.

### Task 2 — Scroll, Window Height, and Screenshot Fix

**Status:** Done

- Add `scroll_area_begin`/`scroll_area_end` wrapping content sections in a scrollable container.
- Create a `scroll_container` flex node between root and content with `flex_grow: 1.0`.
- Add `scroll_y: f32` and `scroll_container: NodeId` to `AppState`.
- Set window to full monitor height via `event_loop.primary_monitor().size()` minus taskbar offset.
- Fix screenshot-not-firing: add `state.window.request_redraw()` at end of `RedrawRequested` handler so continuous redraws occur after the delay.
- Compute `content_height` from paper section bottom relative to scroll container top.

### Task 3 — Fix Z-Value Clipping and Background Rendering

**Status:** Done

- **Root cause:** The quad shader passes `q.z` as clip-space z. Negative values are clipped by the GPU.
- Change all background quad z-values from negative (-1.0, -0.5, -0.3) to 0.0.
- Add a cards-row bounding-box background quad at z:0.0 to fill the 24px flex gaps between cards.
- Verify that hero, build, and paper section backgrounds render correctly at z:0.0.

### Task 4 — Fix Layout and Section Visibility

**Status:** Done

- Give `build_section` an explicit height of 304px (120px header + 2 rows x 80px).
- Give `paper_section` an explicit height of 210px (110px header + 1 row x 100px).
- Remove section margins that exposed the black viewport clear color.
- Remove `border_width: 1.0` from all section and card quads — spacing is handled by layout gaps, not borders.

### Task 5 — Font Family Support in akar-core

**Status:** Done

- Add `attrs: Option<glyphon::Attrs>` parameter to `TextPipeline::set_text()`.
- Use `attrs.as_ref().unwrap_or(&glyphon::Attrs::new())` internally.
- Update all 26 call sites across the workspace with `None` to preserve existing behavior.
- Apply `Family::Serif` to the three main headings: "HELLO, I'M MiMo", "Build with MiMo", "Paper".

### Task 6 — Code Quality and Formatting

**Status:** Done

- Run `cargo fmt` to fix formatting across the workspace.
- Verify `cargo test --workspace` passes.
- Verify `cargo check --workspace` compiles clean (pre-existing clippy warnings in screenshot.rs are unrelated).

---

## Acceptance Criteria

- [x] `cargo run --release --bin webpage-rust -- --site mimo --screenshot /tmp/page.png --exit` produces a clean screenshot of the full MiMo page.
- [x] No black gaps between cards, sections, or below content.
- [x] Vertical scrolling works with mouse wheel inside the content area.
- [x] Window opens at full monitor height.
- [x] Screenshot fires automatically after the delay without requiring mouse movement.
- [x] Headings render in serif font; body text renders in sans-serif.
- [x] All existing tests and workspace code remain unbroken.

---

## Notes for Future Work

- Support loading custom font files (TTF/OTF) via `glyphon::FontSystem` — currently limited to system-installed fonts.
- Add a visible scrollbar track/thumb (currently the draw list scrollbar is functional but the track is not styled).
- Support additional page sections and responsive layout breakpoints.
- Add `--dump-layout` support to the webpage-rust binary for element discovery.
- Explore negative z-value support in the quad shader for proper back-to-front layering without sorting artifacts.
- Add hover/press/click states to cards and list items for interactive webpage demos.
