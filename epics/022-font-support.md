# Epic 022: Font Support

**Status:** Not Started
**Goal:** Give applications control over which fonts akar loads and falls back to, so text renders correctly for scripts beyond the current default font's coverage (CJK, Arabic, Devanagari, emoji, etc.).

**Prerequisite:** Epic 021 is `Status: Done`.

---

## Introduction

akar's text pipeline wraps glyphon, which is backed by cosmic-text and, beneath that, `fontdb` for font matching and `swash`/`rustybuzz` for shaping. Font matching and fallback are not novel problems — they are largely solved inside cosmic-text already. What is unclear is how much of that capability akar currently exposes to applications versus hard-codes internally.

Epic 020 introduced akar-owned typography types (`AkarTypography` and friends) so that component styling APIs do not leak glyphon types across the C ABI. Font support needs to extend that same boundary: applications must be able to load custom font files/bytes and configure fallback without ever touching `fontdb` or `cosmic_text` directly, including from non-Rust languages through `akar.h`.

This epic is investigation-first. The tasks below are for a coding agent to establish the current state of font handling in `akar-core`'s `TextPipeline` before any API is designed.

---

## Research

Initial inputs, to be expanded by the coding agent doing the investigation:

- **cosmic-text/fontdb already do the hard part.** Font matching, family fallback chains, and system font discovery are `fontdb` features. akar likely does not need to reimplement matching logic — it needs an API surface that configures `fontdb`'s font source (bundled bytes vs. system scan) and exposes family/fallback selection to callers.
- **Local source of truth**: `~/Projects/glyphon` for how akar's `TextPipeline` currently constructs its `FontSystem`, and whether system font scanning is enabled today (this has portability and cold-start-time implications — system font scanning is slow and non-deterministic across machines, which cuts against akar's "identical screenshot on macOS/Windows/Linux" design goal in `DEVELOP.md`).
- **Bundled vs. system fonts is a real tradeoff.** Bundled fonts guarantee reproducible screenshots (important for the agent-driven visual feedback loop and `akar-diff` regression gating) but grow binary size and require akar to ship/redistribute font files with a license story. System fonts are free but make `demo-rust` screenshots non-reproducible across machines, which directly conflicts with the screenshot-based debug toolchain this project depends on.
- **C ABI shape matters early.** Loading a font from bytes (`akar_load_font_bytes(ctx, bytes, len)`) is straightforward to bind from any language. Anything that leans on filesystem paths or platform font APIs is harder to keep portable and should be treated with caution.
- **Fallback chains are string-directed today in most cosmic-text-based apps** — an ordered list of family names with a terminal fallback. Whatever akar exposes should probably mirror that rather than inventing a new configuration model.
- **This epic is a soft prerequisite for [[023]] and [[024]]** — RTL and CJK/complex-script rendering are meaningless without the right fonts loaded and falling back correctly.

---

## Tasks

### Task 1 — Inventory Current Font Handling

**Status:** Not Started

- Read `TextPipeline` and `FontSystem` construction in `akar-core` end to end.
- Determine: is `fontdb` scanning system fonts today, or is akar bundling a fixed font? Which font(s), and where do the bytes come from?
- Determine whether any family/fallback configuration is currently exposed to applications, in Rust or through `akar.h`.
- Read `~/Projects/glyphon` (`text_render.rs`, `text_atlas.rs`, and its `FontSystem`/`fontdb` usage) to confirm what glyphon expects from a host application versus what it does internally.
- Document findings in this epic's Research section.

### Task 2 — Reproducibility Constraint Check

**Status:** Not Started

- Run `demo-rust --screenshot` on the current default font setup and confirm whether output depends on any system-installed font (test by temporarily renaming/hiding a likely system font, if feasible, or by inspecting `fontdb` source for scan-order guarantees).
- Assess the impact on `akar-diff --compare` CI regression gating (`epics/014`) if font resolution is not fully deterministic across machines.
- Document a recommendation: bundle a default font, allow opt-in system scanning, or both.

### Task 3 — Prototype Non-Latin Rendering

**Status:** Not Started

- Load a CJK font (e.g., a Noto CJK subset) and an Arabic font by bytes into the current `TextPipeline`, bypassing the public API if necessary (direct `cosmic_text`/`fontdb` calls) to establish a spike.
- Render a CJK string and an Arabic string through `push_text` or the equivalent low-level path, and capture screenshots via the debug toolchain.
- Note any glyph, fallback, or shaping failures encountered — this establishes the actual gap between "glyphon can do this" and "akar exposes this."

### Task 4 — API and Scope Proposal

**Status:** Not Started

- Based on Tasks 1-3, propose a minimal font-loading and fallback-configuration API (Rust + C ABI shape) as a design section in this epic.
- Propose how the API interacts with `AkarTypography` (epic 020) — e.g., does `AkarTypography` gain a font-family field that resolves through the new fallback chain?
- Identify what is explicitly out of scope for a first implementation (e.g., variable fonts, font subsetting, per-glyph color fonts/emoji presentation selection) and record it as an Explicit Deferral.
- Once this proposal is reviewed, convert it into implementation Tasks and update this epic's Status.

---

## Notes for Future Work

- Variable font axis control (weight/width interpolation) is out of scope until core loading/fallback lands.
- Font subsetting for binary-size-sensitive embedders is a later optimization, not a blocker.
- Color/emoji font presentation is closely related to Unicode emoji-sequence handling in text editing (epic 018) and should be scoped carefully to avoid scope creep into a full emoji-support epic.
