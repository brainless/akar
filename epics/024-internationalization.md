# Epic 024: Internationalization (i18n)

**Status:** Not Started
**Goal:** Verify and, where necessary, fix akar's handling of non-ASCII, multi-script, and multi-byte text throughout text editing, layout, and rendering — without akar taking ownership of translation, locale, or formatting concerns that belong to the application.

**Prerequisite:** Epic 021 is `Status: Done`. Overlaps with [[022]] (font support, for script coverage) and [[023]] (RTL, for directional scripts); this epic's scope is the remainder — correctness for scripts and text properties that are not primarily about direction or font fallback.

---

## Introduction

i18n is often treated as a large, open-ended feature area, but for a rendering/component library — as opposed to an application framework — the honest scope is much narrower. `DEVELOP.md` is explicit about what akar does not own: the event loop, async runtime, message passing, accessibility scaffolding. The same philosophy should apply here. akar should not own string resources, locale selection, pluralization rules, date/number/currency formatting, or translation management — those are application-level concerns in an immediate-mode, C-ABI library where the caller drives every frame.

What akar does need to get right, because it is genuinely in akar's layer: correct UTF-8 and grapheme-cluster handling in text editing, correct word/line-breaking for scripts that don't use spaces (CJK) or that break differently (many South/Southeast Asian scripts), and not silently mis-measuring or mis-rendering text outside the Latin-1 range. This epic is mostly a verification and gap-finding exercise against existing text-editing and layout work (epics 012, 018, 020), not a new subsystem.

---

## Research

Initial inputs, to be expanded by the coding agent doing the investigation:

- **Scope boundary.** Reaffirm and apply the same "what akar does NOT own" principle from `DEVELOP.md` to i18n explicitly: no bundled translation strings, no locale-aware formatting APIs, no pluralization. If this epic's investigation finds a compelling reason to cross that line, it should be called out explicitly rather than assumed.
- **Grapheme clusters vs. UTF-8 byte boundaries.** Epic 018 established `TextEditState { cursor, anchor }` as UTF-8 byte offsets, with edits "normalized to valid character boundaries." Character boundary (Unicode scalar value) is not the same as grapheme cluster boundary — emoji with modifiers/ZWJ sequences, combining diacritics, and many Indic scripts form multi-codepoint grapheme clusters that must move/delete as one unit under Backspace/Delete/arrow-key navigation. This needs to be tested directly against the current implementation, not assumed correct.
- **Line/word breaking for non-space-delimited scripts.** cosmic-text/glyphon handle line-breaking internally, but the investigation should confirm whether CJK text wraps correctly (character-level breaking, no spaces required) with the current `TextPipeline` configuration versus falling back to space-delimited breaking, which would fail silently for Chinese/Japanese paragraph text.
- **Downstream reference.** `~/Projects/daftprompt` is a real akar application (`src/ui/`) — check whether it has already surfaced any i18n-adjacent issues (non-ASCII input handling, unexpected wrapping) worth reading into this epic's findings.
- **Overlap discipline.** Font fallback (script coverage) belongs to [[022]]; directional layout belongs to [[023]]. This epic should stay focused on correctness of text processing/editing/wrapping for scripts and text properties that are direction-neutral (CJK, combining marks, emoji sequences) to avoid the three epics duplicating work.

---

## Tasks

### Task 1 — Grapheme Cluster Audit in Text Editing

**Status:** Not Started

- Test current `text_input`/`textarea` Backspace, Delete, and arrow-key navigation against strings containing: multi-codepoint emoji (e.g., family emoji with ZWJ), combining diacritics (e.g., Vietnamese or Devanagari text), and simple multi-byte-but-single-codepoint characters (e.g., CJK) — using the scripted-input tool (`--script`) from the debug toolchain.
- Identify where behavior currently operates on UTF-8 scalar boundaries versus what a grapheme-cluster-correct implementation would do (e.g., using a crate like `unicode-segmentation` for `is_char_boundary`-equivalent grapheme logic).
- Document specific failure cases with reproduction scripts and screenshots.

### Task 2 — Line-Breaking Verification for CJK Text

**Status:** Not Started

- Render a long CJK (Chinese or Japanese) paragraph in `paragraph`/`textarea` at a fixed width and verify wrapping behavior via screenshot — does it break at character boundaries (correct default for CJK) or fail to wrap because it's looking for spaces?
- Check whether this is configurable in cosmic-text's shaping/wrap settings and whether akar's `TextPipeline` currently sets it explicitly or relies on a default.
- Document findings, including whether the current default already happens to be correct (cosmic-text may handle this out of the box).

### Task 3 — Codebase Assumption Audit

**Status:** Not Started

- Grep `akar-core`, `akar-layout`, and `akar-components` for width/length calculations that assume 1 byte or 1 codepoint per rendered character (a common latent bug source distinct from the epic 018 selection-geometry work, which already moved caret/selection geometry to shaped-layout-derived values).
- Check `~/Projects/daftprompt/src/ui/` for any existing workarounds or bug reports related to non-ASCII text handling in a real akar application.
- Document findings as a punch list.

### Task 4 — Scope Proposal for First Implementation Pass

**Status:** Not Started

- Based on Tasks 1-3, propose concrete fixes where gaps were found (e.g., adopt grapheme-cluster-aware navigation in the shared text-editing engine from epic 018).
- Explicitly restate what remains out of scope (locale formatting, translation, pluralization) so future contributors don't misread this epic as an invitation to add those.
- Once reviewed, convert this into implementation Tasks and update this epic's Status.

---

## Notes for Future Work

- Locale-aware number/date/currency formatting is explicitly out of scope for akar itself; if there is ever a companion crate, it would be a separate, optional, application-facing library, not part of `akar-core`/`akar-components`.
- Input Method Editor (IME) composition (needed for Chinese/Japanese/Korean text input) is already flagged as future work in epic 018's Notes and is closely related to this epic's concerns but large enough to warrant its own epic when prioritized.
- Translation/string-resource management is explicitly not akar's concern — it belongs to the application layer, consistent with akar not owning any application state.
