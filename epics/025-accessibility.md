# Epic 025: Accessibility

**Status:** Not Started
**Goal:** Establish a path for akar UIs to be usable with screen readers and other assistive technology, despite akar being a pure GPU draw-list renderer with no OS-level accessibility tree today.

**Prerequisite:** Epic 021 is `Status: Done`. Independent of [[022]], [[023]], and [[024]] — this epic can be researched in parallel with those, but should not be blocked by or block them.

---

## Introduction

`AGENTS.md` currently states plainly: "Do not add accessibility scaffolding in v1. Document the punt if relevant." That punt was reasonable to get the component catalog and rendering pipeline to a working state, but it means akar UIs are currently invisible to screen readers, switch access, and any other assistive technology — the draw list produces pixels, not semantics, and nothing in `akar-core` or `akar-components` emits a parallel accessibility tree.

This is the largest and most architecturally invasive of the four topics in this round. Font support and i18n are largely "verify and extend existing capability" problems; RTL is a significant but contained layout problem. Accessibility requires a new kind of output from every interactive component — a semantic tree (role, label, state, bounds) — alongside the existing draw list, plus a platform adapter to expose that tree to Windows UIA, macOS AX, and AT-SPI (Linux). It also collides with akar's core design principle that "akar does not own the window" (`DEVELOP.md`): most accessibility toolkits assume they can hook directly into the window they're rendering into, which akar's C-ABI, developer-owns-the-window model does not straightforwardly allow.

This epic is investigation-first and should not attempt to design the full solution before understanding the constraints.

---

## Research

Initial inputs, to be expanded by the coding agent doing the investigation:

- **AccessKit is the natural reference implementation.** It's the Rust-native accessibility crate used by egui and, at least partially, by Zed/gpui — both of which are already in akar's local reference set (`~/Projects/egui`, `~/Projects/zed/crates/gpui`). AccessKit provides a semantic-tree data model and platform adapters (Windows UIA, macOS AX, AT-SPI, web). Read its integration in both of those projects before designing anything akar-specific.
- **The window-ownership problem is the key architectural risk.** AccessKit's platform adapters typically need a live handle into the OS window (HWND, NSView, etc.) to attach the accessibility tree. akar explicitly does not own the window (`DEVELOP.md` → "What akar does NOT own") — the developer does, often through `akar-winit` or their own windowing code. This means the adapter almost certainly needs to live in `akar-winit` (optional, as today) with a companion, windowing-independent semantic-tree-emission API in `akar-core`/`akar-components` that non-Rust/non-winit consumers could wire into their own platform adapter. This split needs to be validated against how egui's `egui-winit`/`eframe` split handles the same problem, since it's the closest architectural analog.
- **Semantic tree emission has to fit the existing lifecycle.** Per `DEVELOP.md`'s construct/compute/paint model, the natural place for a component to report its role/label/state is likely alongside paint (it already has resolved rect and current state there), but emitting an accessibility tree during paint would violate "paint is read-only on Layout" only if the tree is treated as part of `Layout` — it more likely belongs as a separate frame-scoped output next to the draw list, mirroring how the draw list itself is frame-scoped and flushed.
- **Keyboard focus order is a partial foundation already.** Widget IDs and focus state already exist for text editing (epic 018) and interactive components generally. This is necessary but far from sufficient for accessibility (focus order without semantic roles/labels is not usable by a screen reader), but it means this epic isn't starting from zero.
- **C ABI exposure is a second-order problem.** Non-Rust consumers of `akar.h` would need either (a) akar-owned platform adapters compiled into `libakar` for at least the common desktop platforms, or (b) a way to walk the emitted semantic tree from C and hand it to their own platform accessibility API. Option (a) is a much bigger commitment (three platform adapters shipped inside the core library) and should be weighed carefully against akar's "flat C API, no callbacks unless opted into" philosophy.
- **Theme/contrast is a smaller, more tractable adjacent concern.** Independent of the semantic-tree problem, the two shipped theme presets (`AKAR_THEME_DARK`/`AKAR_THEME_LIGHT`, per `DEVELOP.md`) should eventually be checked against WCAG contrast minimums. This is cheap to investigate and worth scoping as an early, low-risk task even before the harder semantic-tree work.

---

## Tasks

### Task 1 — AccessKit Integration Survey

**Status:** Not Started

- Read AccessKit's own documentation/source structure (crate boundaries: core tree model vs. platform adapters) to understand what it expects a host to provide.
- Read `~/Projects/egui`'s AccessKit integration (`egui-winit`/`eframe` or equivalent) in detail: what triggers tree emission, how it maps egui's immediate-mode widget calls to AccessKit nodes, and how the window handle is threaded through.
- Read `~/Projects/zed/crates/gpui`'s accessibility approach, if present, as a second production data point on a wgpu-based UI.
- Document the minimal integration contract (what akar would need to produce per frame) as a design note in this epic.

### Task 2 — Window-Ownership Architecture Options

**Status:** Not Started

- Given akar does not own the window, evaluate at least two architectures: (a) semantic-tree emission lives in `akar-core`/`akar-components`, platform adapter lives in `akar-winit` (optional, same pattern as today); (b) platform adapters ship inside `libakar` itself, gated by build features.
- Assess each option against the C ABI contract (`AGENTS.md` → C ABI contract) and against non-winit consumers (anyone driving their own window/event loop through raw `akar.h`).
- Recommend one direction with tradeoffs documented, not both pursued in parallel.

### Task 3 — Semantic Tree Emission Prototype (Single Component)

**Status:** Not Started

- Prototype minimal semantic-tree emission for one component (`button` is the obvious choice — simple role, label, clickable state) without wiring a real platform adapter yet. Emit the equivalent of an AccessKit `NodeBuilder` into a `Vec` or similar, alongside the existing draw list, and print/inspect it.
- Verify this fits the construct/compute/paint lifecycle without violating "paint is read-only on Layout" (`DEVELOP.md`) — confirm the tree is a separate frame-scoped output, not a mutation of `Layout` itself.
- Note what's needed from each component (role taxonomy, label source — likely caller-supplied string, not inferred from visual text) to generalize this beyond `button`.

### Task 4 — Keyboard Focus Order Audit

**Status:** Not Started

- Review current focus/widget-ID handling (epic 018 and any prior focus work) to determine how close the existing model is to a usable accessibility focus order.
- Identify gaps: is focus order currently visual/DOM-order equivalent, or does it need an explicit tab-order concept?

### Task 5 — Theme Contrast Check

**Status:** Not Started

- Check `AKAR_THEME_DARK` and `AKAR_THEME_LIGHT` color token pairs (text-on-background, interactive-element-on-background) against WCAG 2.1 AA contrast minimums.
- Document any failing pairs as a quick, low-risk fix candidate independent of the larger semantic-tree effort.

### Task 6 — Scope Proposal for First Implementation Pass

**Status:** Not Started

- Based on Tasks 1-5, propose a minimal first implementation scope — likely: semantic-tree emission API in `akar-core`, a small role taxonomy covering the most common interactive components (button, text input, checkbox/switch/radio, link), and a single-platform adapter (recommend starting with whichever platform is the primary dev target) wired through `akar-winit`.
- Explicitly list deferrals: full 30+ component coverage, non-winit platform adapters, C-ABI-exposed adapters, live-region/dynamic-announcement support.
- Once reviewed, convert this into implementation Tasks and update this epic's Status.

---

## Notes for Future Work

- Full component-catalog semantic coverage (all 30+ components) is a large, incremental effort that should follow the initial small-scope proof, not precede it.
- Non-Rust / non-winit consumers of `akar.h` wanting accessibility support may need a documented pattern for wiring their own platform adapter against an exposed semantic-tree walk API — this is explicitly deferred until the Rust/winit path is proven.
- Reduced-motion and high-contrast-mode preferences (OS-level user settings) are related but distinct from screen-reader support and are out of scope for this epic's initial research.
