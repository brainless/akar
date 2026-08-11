# Epic 023: Right-to-Left (RTL) Text and Layout

**Status:** Not Started
**Goal:** Establish correct rendering and interaction for right-to-left scripts (Arabic, Hebrew, etc.), covering both glyph shaping and direction-aware layout.

**Prerequisite:** Epic 021 is `Status: Done`. Findings from [[022]] (font support) inform this epic — RTL scripts need correctly loaded, fallback-capable fonts before layout direction is even worth testing.

---

## Introduction

RTL is commonly treated as a text-shaping problem, but the shaping half is largely already solved: cosmic-text's shaping backend (rustybuzz) and glyphon already implement the Unicode Bidirectional Algorithm and correct glyph shaping for Arabic/Hebrew scripts at the glyph level. What akar does not yet have is **direction-aware layout** — the property that an entire subtree (icon positions, scroll direction, text alignment, caret movement, selection anchors, component internal padding) mirrors when the surrounding context is RTL.

This is a layout-and-interaction problem more than a rendering problem, and it touches nearly every component in `akar-components` to some degree (icon placement in buttons/badges, scrollbar side, tab bar order, navbar item order, text input caret/selection math). Given the size of that surface, this epic should not try to convert the whole component catalog in one pass — the initial scope should prove the direction model end-to-end on a small number of components before it is rolled out further.

---

## Research

Initial inputs, to be expanded by the coding agent doing the investigation:

- **Shaping is not the gap; layout is.** Confirm via `~/Projects/glyphon` and cosmic-text's shaping path that bidi reordering and per-run direction are already handled at the glyph level. The investigation should focus on what's missing above that layer, not attempt to reimplement bidi.
- **taffy's RTL support is partial and needs verification.** `~/Projects/taffy` supports `flex-direction: row-reverse`, which can approximate RTL for simple row layouts, but this is not the same as full logical-properties layout (`start`/`end` instead of `left`/`right`, mirrored margins/padding, mirrored `justify-content`). Read taffy's source and its own test suite for what direction/writing-mode support exists versus what would need to live in akar's layout wrapper (`akar-layout`) as a pre/post-processing step.
- **Reference prior art before designing from scratch.** `~/Projects/egui` — check if egui has any RTL support and how it structures direction as a layout concept, since akar's immediate-mode model is close to egui's. `~/Projects/zed/crates/gpui` is a secondary reference for a production wgpu UI's approach, if it has one.
- **A global direction context is the likely shape.** Something like `AkarDirection { Ltr, Rtl }` set on the theme/context (mirroring how the flat `AkarTheme` already works — see `DEVELOP.md` → Theme system) rather than a per-node property, at least for a first version. Per-node direction (mixed-direction documents) is a much larger undertaking and should be an explicit deferral initially.
- **Text editing is a distinct sub-problem.** Epic 018 established `TextEditState { cursor, anchor }` as byte offsets and caret/selection geometry derived from shaped glyphon layout (epics/018, Task 5). For RTL text, "moving right" and "moving forward in the string" are not the same operation — Left/Right arrow key handling, caret geometry, and selection-drag math all need direction awareness. This is likely the highest-value, most self-contained place to prove the direction model, since it's already isolated behind `TextEditState` and shared editing helpers.
- **Component audit needed.** Icon-bearing components (buttons with leading/trailing icons, navbar, tab bar) and scrollable containers (scroll area, virtualized lists) have hard-coded left/right or start-position assumptions that need to be found and cataloged before any mirroring work starts.

---

## Tasks

### Task 1 — Confirm Shaping-Layer Bidi Behavior

**Status:** Not Started

- Read cosmic-text's shaping path (via `~/Projects/glyphon`) to confirm bidi reordering and RTL glyph shaping already work without akar-side intervention.
- Render a mixed-direction string (Latin + Arabic) through the existing `push_text` path and screenshot it via the debug toolchain to verify current behavior with no akar-side direction handling.
- Document what already works correctly today versus what looks wrong (e.g., paragraph alignment, caret position, layout box direction).

### Task 2 — Survey taffy Direction Support

**Status:** Not Started

- Read `~/Projects/taffy` for existing direction/writing-mode support (`flex-direction`, any logical-property handling).
- Determine whether direction-aware layout should be implemented as a taffy-level style property akar passes through, or as a pre/post-processing mirror step in `akar-layout` that taffy is unaware of.
- Document the tradeoffs of each approach (correctness, taffy upgrade risk, complexity in `akar-layout`).

### Task 3 — Prior Art Survey

**Status:** Not Started

- Check `~/Projects/egui` for RTL/direction handling and how it's modeled at the API level.
- Check `~/Projects/zed/crates/gpui` for the same, if present.
- Summarize applicable patterns and how they would map onto akar's construct/compute/paint lifecycle (`DEVELOP.md` → Component lifecycle).

### Task 4 — Component Direction Audit

**Status:** Not Started

- Grep `akar-components` for hard-coded left/right, start-position, or margin/padding asymmetries (icon slots, navbar item order, tab bar, scrollbar side, badge/button icon placement).
- Produce a list of components ranked by how directly they'd need to change for basic RTL correctness, versus ones that are already direction-neutral (e.g., centered text, symmetric containers).

### Task 5 — Text Input Caret/Selection Direction Prototype

**Status:** Not Started

- Using a hardcoded RTL string in `text_input`, verify current Left/Right arrow and selection-drag behavior against expected RTL semantics (visual-left vs. logical-previous).
- Identify exactly what in epic 018's shared editing engine (`TextEditState`, selection/caret geometry) needs a direction parameter.
- Capture before/after screenshots once a minimal fix is prototyped, even if not yet wired to a public API.

### Task 6 — Scope Proposal for First Implementation Pass

**Status:** Not Started

- Based on Tasks 1-5, propose a minimal first implementation scope: likely a context-level `AkarDirection`, taffy row-reverse wiring, and direction-correct text input caret/selection — deliberately excluding full component-catalog mirroring.
- List explicit deferrals (per-node/mixed-direction documents, full icon/scrollbar mirroring across all 30+ components, vertical writing modes).
- Once reviewed, convert this into implementation Tasks and update this epic's Status.

---

## Notes for Future Work

- Mixed-direction (bidi) documents with per-node direction overrides are a substantially larger effort than a single global direction flag and should stay out of scope until the global-direction model is proven.
- Vertical writing modes (CJK vertical text) are unrelated to RTL and are explicitly out of scope for this epic.
- Canvas world-space text (`CanvasPainter`, display-only per `DEVELOP.md`) may need its own direction handling pass separate from layout-based components, since it bypasses `akar-layout` entirely.
