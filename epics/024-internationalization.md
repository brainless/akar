# Epic 024: Internationalization (i18n)

**Status:** Research Complete — Implementation Not Started
**Goal:** Verify and, where necessary, fix akar's handling of non-ASCII, multi-script, and multi-byte text throughout text editing, layout, and rendering — without akar taking ownership of translation, locale, or formatting concerns that belong to the application.

**Prerequisite:** Epic 021 is `Status: Done`. Overlaps with [[022]] (font support, for script coverage) and [[023]] (RTL, for directional scripts); this epic's scope is the remainder — correctness for scripts and text properties that are not primarily about direction or font fallback.

---

## Introduction

i18n is often treated as a large, open-ended feature area, but for a rendering/component library — as opposed to an application framework — the honest scope is much narrower. `DEVELOP.md` is explicit about what akar does not own: the event loop, async runtime, message passing, accessibility scaffolding. The same philosophy should apply here. akar should not own string resources, locale selection, pluralization rules, date/number/currency formatting, or translation management — those are application-level concerns in an immediate-mode, C-ABI library where the caller drives every frame.

What akar does need to get right, because it is genuinely in akar's layer: correct UTF-8 and grapheme-cluster handling in text editing, correct word/line-breaking for scripts that don't use spaces (CJK) or that break differently (many South/Southeast Asian scripts), and not silently mis-measuring or mis-rendering text outside the Latin-1 range. This epic is mostly a verification and gap-finding exercise against existing text-editing and layout work (epics 012, 018, 020), not a new subsystem.

---

## Research

Initial inputs (below), expanded with concrete findings from source-analysis investigation (Tasks 1-3, all done without a live GPU/display — see per-task notes):

- **Scope boundary.** Reaffirmed. Nothing in the investigation below argues for akar owning translation, locale formatting, or pluralization. See "Explicitly Out of Scope" under Task 4.
- **Grapheme clusters vs. UTF-8 byte boundaries — confirmed bug, not assumption.** Epic 018 established `TextEditState { cursor, anchor }` (`crates/akar-components/src/text_edit.rs:1-5`) as UTF-8 byte offsets, normalized to *char* boundaries via `normalize_position` (`text_edit.rs:37-43`, uses `str::is_char_boundary`). `previous_boundary`/`next_boundary` (`text_edit.rs:117-131`) step by exactly one `char` (one Unicode scalar value) using `char_indices()`/`chars()` — this is codepoint granularity, not grapheme-cluster granularity. Both `text_input.rs` (Backspace: `text_input.rs:110-117`; Delete: `text_input.rs:118-125`; Left/Right: `text_input.rs:127-137`) and `textarea.rs` (Backspace: `textarea.rs:136-143`; Delete: `textarea.rs:144-151`; Left/Right: `textarea.rs:154-157`) call these same helpers directly. Confirmed by source reading, not by a live script run (no GPU/display in this environment) — see Task 1 for the concrete reproduction cases this predicts.
- **Caret geometry compounds the same codepoint-vs-cluster gap.** `crates/akar-core/src/text_pipeline.rs:368-379` (`glyph_boundary_x`) interpolates the caret's x-position *within* a shaped glyph cluster by linear fraction of **codepoint count** (`cluster.chars().count()`, line 371) rather than grapheme-cluster count. When cosmic-text shapes multiple codepoints (combining marks, some ligatures) into one glyph cluster, and the editing engine's codepoint-granular cursor stops mid-cluster, the caret renders at a plausible-looking but not necessarily correct fractional x inside that glyph. This is a distinct, smaller finding from the navigation bug above — it's a geometry approximation, not a crash/mis-edit — but shares the same root cause (codepoint granularity standing in for grapheme granularity) and should be fixed by the same underlying change.
- **No grapheme-segmentation crate in akar's direct dependency graph.** `grep -rn unicode-segmentation` across `Cargo.toml`/`Cargo.lock` shows `unicode-segmentation` (1.13.3) and `unicode-width` (0.2.2) present in `Cargo.lock` only as *transitive* dependencies — pulled in by `cosmic-text` 0.18.2 (via glyphon) and by `winit`. Neither `akar-core/Cargo.toml` nor `akar-components/Cargo.toml` lists them directly. Since `unicode-segmentation` is already resolved in the workspace's dependency tree at a compatible version, adding it as a direct dependency of `akar-components` (where `text_edit.rs` lives) costs no new transitive dependency and no version-resolution risk.
- **Line/word breaking for CJK — confirmed correct by default, no akar action needed.** `crates/akar-core/src/text_pipeline.rs:61-97` (`TextPipeline::set_text`) never calls `glyphon::Buffer::set_wrap` / never touches `cosmic_text::Wrap` — it relies entirely on cosmic-text's buffer default. Read directly from the vendored cosmic-text source at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.18.2/src/buffer.rs:262`: the default is `Wrap::WordOrGlyph` ("wraps at the word level, or fallback to glyph level if a word can't fit on a line by itself" — `cosmic-text-0.18.2/src/layout.rs:111-120`). Critically, cosmic-text's line-breaking is not naive space-splitting: `cosmic-text-0.18.2/src/shape.rs:870` calls `unicode_linebreak::linebreaks(span)`, i.e. the UAX #14 Unicode Line Breaking Algorithm, to find break opportunities before shaping into "words." UAX #14 treats CJK ideographs as break-opportunities between (almost) every character, independent of whitespace. Conclusion (source-analysis based, no rendered screenshot available in this environment — see Task 2): CJK paragraph text wrapping already works correctly today with zero akar-side configuration, because akar never overrides the correct cosmic-text default. This is a "verify existing correctness," not a gap.
- **Codebase assumption audit — clean outside the text-editing engine.** `grep -rn '\.len()\|chars().count()\|chars().nth(' crates/akar-core/src crates/akar-layout/src crates/akar-components/src` turns up no character-count-assumes-byte-count bugs in `akar-layout` (it has no text-measuring logic of its own — all width/height comes from `akar_core::TextPipeline` via the `default_measure_fn` callback described in `DEVELOP.md`'s "Component lifecycle" section) or in any `akar-components` widget other than `text_edit.rs`/`text_input.rs`/`textarea.rs`. The `.len()` calls elsewhere in those three files are legitimate byte-offset arithmetic consistent with `TextEditState`'s documented byte-offset contract (e.g. `text_edit.rs:23,38,130`; `text_input.rs:121,143-144`; `textarea.rs:25,52-53`) — not bugs. Two narrower findings inside the text-editing files: `text_input.rs:189` masks passwords via `"*".repeat(value.chars().count())`, which uses codepoint count rather than grapheme count — for a multi-codepoint grapheme (e.g. a ZWJ emoji sequence typed into a password field) this reveals the internal codepoint count as extra asterisks, a minor information leak/cosmetic bug, listed as a Task below. `textarea.rs:28-30` (`character_column`) and `textarea.rs:32-37` (`position_at_character_column`) compute Up/Down column position by codepoint count for the same reason as the cursor-boundary functions — same root cause, same fix.
- **Downstream reference (`~/Projects/daftprompt`).** `grep -rniE "TODO|FIXME|unicode|grapheme|CJK|non-ascii|multibyte|utf-?8"` over `~/Projects/daftprompt/src/ui/` returns no hits — no existing workaround or bug comment. `src/ui/render.rs` uses `akar_text_input` (aliased from `akar-components`) for its search box (see `render.rs:48`, `render.rs:1138`) but has no bespoke text-width or character-counting logic of its own; it delegates entirely to akar's widget. Nothing in daftprompt surfaces a new i18n concern beyond what's found directly in akar's own text-editing code.
- **Overlap discipline confirmed.** Checked `epics/022-font-support.md` and `epics/023-rtl-text-rendering.md` for scope collision: 022 explicitly owns font/script coverage (CJK/Arabic/Devanagari fallback) and calls itself "a soft prerequisite for 023 and 024"; 023 explicitly excludes CJK vertical writing modes as unrelated to RTL. Neither epic currently touches grapheme-cluster navigation or line-breaking configuration, so this epic's scope (below) does not duplicate either.

---

## Tasks

### Task 1 — Grapheme Cluster Audit in Text Editing

**Status:** Done (source-analysis based — no live GPU/display in this research environment; conclusions are derived from reading `text_edit.rs`/`text_input.rs`/`textarea.rs` and are not yet confirmed against `--script` screenshots. A future implementation pass should still capture the `--script` reproduction before/after fixing, per the debug toolchain convention.)

- Simple multi-byte-but-single-codepoint characters (CJK, e.g. `汉`): **not affected**. `previous_boundary`/`next_boundary` step by one `char`, and a CJK ideograph is one `char` (one Unicode scalar value, 3 UTF-8 bytes). Backspace/Delete/Left/Right will behave correctly for plain CJK text today.
- Combining diacritics (e.g. Vietnamese `ệ` = `e` + combining circumflex + combining dot-below, or Devanagari conjuncts): **predicted failure**. `previous_boundary`/`next_boundary` (`crates/akar-components/src/text_edit.rs:117-131`) advance by exactly one `char`. A base character followed by one or more combining marks is multiple `char`s but one grapheme cluster. Backspace at the end of such a cluster will delete only the last combining mark, not the whole visual character — the cursor appears to not move / the character appears unchanged after one Backspace press, requiring N presses to remove what the user perceives as one character.
- Multi-codepoint emoji / ZWJ sequences (e.g. family emoji `👨‍👩‍👧‍👦`, several codepoints joined by U+200D ZERO WIDTH JOINER): **predicted failure**, same mechanism as above but more visually severe — Backspace/Delete will step through and can leave a "broken" emoji rendering (dangling ZWJ or an isolated component emoji) rather than deleting the whole visual glyph in one keystroke. Left/Right arrow navigation will likewise stop the caret mid-sequence.
- Root cause is single and shared across `text_input.rs` and `textarea.rs`: both files import and call the same `previous_boundary`/`next_boundary` from `text_edit.rs` (confirmed via `grep -n previous_boundary\|next_boundary crates/akar-components/src/{text_input,textarea}.rs`), so a fix in the shared engine (epic 018's design intent) fixes both widgets at once.
- No `unicode-segmentation`-equivalent grapheme logic exists anywhere in `akar-components` today (confirmed by grep — only present transitively via `cosmic-text`/`winit` in `Cargo.lock`).

### Task 2 — Line-Breaking Verification for CJK Text

**Status:** Done (source-analysis based — no live GPU/display in this research environment; conclusion is derived from reading `akar-core`'s `TextPipeline` and the vendored `cosmic-text` 0.18.2 source, not from a rendered screenshot. A future pass should still capture a `paragraph`/`textarea` CJK screenshot via `--component`/`--script` to visually confirm, since this is a "should already work" claim, not a tested one.)

- `crates/akar-core/src/text_pipeline.rs:61-97` (`TextPipeline::set_text`) constructs the `glyphon::Buffer`, calls `buffer.set_metrics`, `buffer.set_size`, and `buffer.set_text(..., glyphon::Shaping::Advanced, None)`, then `buffer.shape_until_scroll`. It never calls `buffer.set_wrap` — wrap mode is left at cosmic-text's own default.
- Read directly from `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.18.2/src/buffer.rs:262`: the default is `Wrap::WordOrGlyph`. Per `cosmic-text-0.18.2/src/layout.rs:111-120`, this wraps at the word level and falls back to glyph-level breaking when a "word" doesn't fit — this is already the more permissive of the two non-`None` options, not the strict `Wrap::Word` that would risk failing to wrap unbroken CJK runs.
- More importantly, cosmic-text's definition of "word" for line-breaking purposes is not whitespace-delimited. `cosmic-text-0.18.2/src/shape.rs:870` calls `unicode_linebreak::linebreaks(span)` — the UAX #14 Unicode Line Breaking Algorithm — to compute break opportunities before grouping into shaping "words." UAX #14 assigns break-opportunity classes to CJK ideographs such that (with limited exceptions for punctuation-pairing rules) a break opportunity exists between nearly every pair of adjacent CJK characters, independent of whitespace.
- Conclusion: CJK paragraph wrapping in `paragraph`/`textarea` already works correctly today with zero configuration from akar's `TextPipeline`, because akar does not override cosmic-text's already-correct default. No akar-side fix is required for Task 2's concern. This should be confirmed visually once a GPU/display environment is available (a CJK screenshot through `--component paragraph` or `--component textarea` is a low-cost follow-up, listed as a Task below, primarily to catch any font-fallback interaction with epic 022 rather than because the wrap logic itself is in doubt).

### Task 3 — Codebase Assumption Audit

**Status:** Done

Punch list (see Research above for the full grep methodology and line citations):

1. `crates/akar-components/src/text_edit.rs:117-131` — `previous_boundary`/`next_boundary` operate on codepoints, not grapheme clusters. **Root cause of Task 1's findings.**
2. `crates/akar-core/src/text_pipeline.rs:368-379` (`glyph_boundary_x`) — interpolates caret x-position within a shaped glyph cluster using codepoint count (`cluster.chars().count()`), not grapheme-cluster count. Same root cause as #1, surfaces as a caret-geometry approximation rather than a broken edit.
3. `crates/akar-components/src/text_input.rs:189` — password masking via `"*".repeat(value.chars().count())` renders one asterisk per codepoint, not per grapheme cluster; a multi-codepoint grapheme in a password field leaks its codepoint count as extra asterisks. Minor; independent of #1/#2's navigation fix but shares the same "what counts as one visible character" question.
4. `crates/akar-components/src/textarea.rs:28-30` (`character_column`) and `textarea.rs:32-37` (`position_at_character_column`) — Up/Down vertical navigation computes column position by codepoint count. Same root cause as #1; a grapheme-aware column function is needed for Up/Down to land the caret in the visually-equivalent column on multi-codepoint-grapheme lines.
5. `akar-layout` — clean. It owns only taffy tree resolution; all text measurement is delegated to `akar_core::TextPipeline` via the measure callback (`DEVELOP.md`'s "Component lifecycle: construct, compute, paint"), so there is no separate character-counting logic to audit there.
6. All other `akar-components` widgets (`alert.rs`, `avatar.rs`, `badge.rs`, `card.rs`, `data_item.rs`, `data_list.rs`, `label.rs`, `paragraph.rs`, `select.rs`, `tabs.rs`, etc.) — `.len()`/`.chars()` usages found by grep are on `Vec`s (draw-list lengths, option lists, container/card counts), not on user-facing text content. No latent i18n bugs found outside the text-editing engine.
7. `~/Projects/daftprompt/src/ui/` — no TODO/FIXME/unicode/grapheme/CJK/non-ASCII comments found (`grep -rniE` returned no matches). The one akar text widget it uses (`akar_text_input`, aliased in `render.rs:48`, called at `render.rs:1138`) is used as-is with no bespoke width/character-counting workaround, so daftprompt has not (yet) surfaced or worked around any of the issues in #1-#4 above — consistent with them being real but not yet hit by daftprompt's current usage (a single-line search box, unlikely to have exercised combining-mark or ZWJ input).

### Task 4 — Scope Proposal for First Implementation Pass

**Status:** Done — converted into Tasks 5-9 below.

**Proposed fix, concretely.** Adopt `unicode-segmentation` (already resolved in the workspace's `Cargo.lock` transitively via `cosmic-text`/`winit`, so no new dependency-tree risk) as a **direct** dependency of `akar-components` (where `text_edit.rs` lives). Replace the codepoint-stepping logic in the shared text-editing engine with grapheme-cluster-stepping logic, in one place, so both `text_input.rs` and `textarea.rs` inherit the fix without per-widget changes:

- `crates/akar-components/src/text_edit.rs:117-123` (`previous_boundary`) and `text_edit.rs:125-131` (`next_boundary`) — reimplement using `unicode_segmentation::UnicodeSegmentation::grapheme_indices(true)` (extended grapheme clusters, the `true` argument) instead of `char_indices()`/`chars()`. These two functions are the single chokepoint used by both widgets' Backspace/Delete/Left/Right, so this one change fixes Task 1's findings in both widgets at once.
- `crates/akar-components/src/textarea.rs:28-30` (`character_column`) and `textarea.rs:32-37` (`position_at_character_column`) — reimplement the "column" unit as grapheme-cluster count instead of `char` count, using the same `unicode-segmentation` iterator, so Up/Down lands in the visually-equivalent column.
- `crates/akar-core/src/text_pipeline.rs:368-379` (`glyph_boundary_x`) — change the interpolation fraction from `cluster.chars().count()`/codepoint-position to a grapheme-cluster-based fraction (or, if a glyph cluster boundary never actually needs to split a grapheme cluster once `previous_boundary`/`next_boundary` are fixed above, this may reduce to an assertion/simplification rather than a behavior change — needs to be re-verified once the cursor-boundary fix lands, since a grapheme-correct cursor should no longer produce indices that fall strictly inside a grapheme cluster).
- `crates/akar-components/src/text_input.rs:189` — change `value.chars().count()` to a grapheme-cluster count (same `unicode-segmentation` iterator) for password-mask asterisk rendering.
- `normalize_position` (`text_edit.rs:37-43`) currently only guarantees a `char`-boundary-valid position (via `str::is_char_boundary`), which is necessary but not sufficient for grapheme-safety; it should additionally be snapped to a grapheme-cluster boundary once externally-supplied `TextEditState` (from a caller, e.g. after an app-level mutation of the string) is normalized, so a caller can never hand back a cursor/anchor that splits a grapheme cluster.
- Existing tests in `text_edit.rs`'s `#[cfg(test)] mod tests` (e.g. `backspace_and_delete_ranges_are_unicode_safe`, `select_all_covers_unicode_value`) already cover codepoint-level Unicode correctness (`aé🙂` style fixtures) and should remain green; new tests must add ZWJ-emoji and combining-diacritic fixtures that the current codepoint-only tests do not exercise.

**Explicitly out of scope (restated, so future contributors don't misread grapheme/line-breaking fixes as an opening to add more).** Unchanged from the epic's original framing and this epic's Notes section:
- Locale-aware number, date, currency, or unit formatting. Not akar's concern at any layer; if ever needed, it is a separate, optional, application-facing companion crate — never `akar-core`/`akar-components`.
- Translation and string-resource management (`.po`/`.mo`/ICU MessageFormat/etc.). akar takes `&str`/`String` and renders exactly what it is given; it does not own application copy.
- Pluralization rules (CLDR plural categories or otherwise). An application-layer concern that sits on top of translation, which akar does not own either.
- Input Method Editor (IME) composition (needed for live CJK/Korean input, not just rendering already-committed CJK text). Already flagged as future work in epic 018's Notes; large enough for its own epic, not folded into this one.
- Font fallback / script coverage (which font renders which script) — owned by [[022]].
- Bidirectional/RTL layout and directional text properties — owned by [[023]].
- Locale/region selection UI or any "current locale" concept in akar's state — akar has no notion of a current locale today and this epic does not introduce one.

---

### Task 5 — Grapheme-Cluster-Aware Cursor Navigation in the Shared Text-Editing Engine

**Status:** Not Started

- Add `unicode-segmentation` to `[workspace.dependencies]` in the root `Cargo.toml` (pinned to `1.13.3`, the version already resolved in `Cargo.lock`, to avoid an unrelated dependency bump), following the existing convention for every other shared dependency (`wgpu`, `glyphon`, `glam`, `taffy`, `winit`, `thiserror`, `log`, `bytemuck`); then depend on it via `unicode-segmentation.workspace = true` in `crates/akar-components/Cargo.toml`. Do not add a bespoke per-crate version pin outside `[workspace.dependencies]` — that would be inconsistent with how every other cross-cutting dependency in this workspace is declared. Note also that Task 7 may require the same crate inside `akar-core` (for `glyph_boundary_x`) if its fix turns out to need grapheme-cluster counting; if so, add `unicode-segmentation.workspace = true` to `crates/akar-core/Cargo.toml` too rather than duplicating the version pin.
- Reimplement `previous_boundary`/`next_boundary` in `crates/akar-components/src/text_edit.rs:117-131` using `UnicodeSegmentation::grapheme_indices(true)` (extended grapheme clusters) instead of `char_indices()`/`chars()`.
- Update `normalize_position` (`text_edit.rs:37-43`) so externally-supplied `TextEditState` positions are snapped to a grapheme-cluster boundary, not merely a `char` boundary.
- Add test fixtures the current suite lacks: a ZWJ family emoji (e.g. `"👨‍👩‍👧‍👦"`), a Vietnamese or Devanagari combining-mark string, and mixed ASCII/CJK/combining-mark content, covering Backspace, Delete, Left, Right, and select-all.
- Verify no regression against the existing codepoint-level fixtures (`aé🙂` style) already in `text_edit.rs`'s test module.
- No `text_input.rs`/`textarea.rs` changes should be required for Backspace/Delete/Left/Right beyond this shared-engine change, since both already call through `previous_boundary`/`next_boundary`; confirm this by re-running (not editing) their existing tests.

### Task 6 — Grapheme-Aware Vertical Navigation and Password Masking

**Status:** Not Started

- Reimplement `character_column` and `position_at_character_column` in `crates/akar-components/src/textarea.rs:28-37` using grapheme-cluster counts (via the same `unicode-segmentation` dependency added in Task 5) so `move_vertical`/Up/Down land in the visually-equivalent column on lines containing multi-codepoint grapheme clusters.
- Change `crates/akar-components/src/text_input.rs:189` from `value.chars().count()` to a grapheme-cluster count for password-mask asterisk rendering.
- Add tests: Up/Down navigation across textarea lines containing combining-mark/emoji sequences of differing codepoint-vs-grapheme length; masked `text_input` asterisk count for a value containing a ZWJ sequence (expect one asterisk per grapheme cluster, not per codepoint).

### Task 7 — Grapheme-Aware Caret Geometry

**Status:** Not Started

- Re-derive `glyph_boundary_x` in `crates/akar-core/src/text_pipeline.rs:368-379` once Task 5 lands: confirm whether a grapheme-corrected cursor can still produce an `index` that falls strictly inside a shaped glyph cluster's byte range (it may not, if grapheme boundaries and cosmic-text's glyph-cluster boundaries turn out to always coincide for the fonts akar ships/tests with — this needs to be checked empirically with `MockDrawList`/unit tests, not assumed).
- If mid-cluster indices remain possible (e.g. ligatures that a font maps to one glyph across a full word, not just a grapheme cluster), change the interpolation fraction from codepoint count to grapheme-cluster count.
- Add/extend `text_pipeline.rs`'s existing `#[cfg(test)] mod tests` (which already builds `glyphon::Buffer`s directly, see `text_pipeline.rs:385-397`) with a combining-mark or CJK+combining case exercising `text_geometry`'s caret output.

### Task 8 — Visual Confirmation of CJK Wrapping and Grapheme Navigation

**Status:** Not Started

- This epic's Tasks 1 and 2 conclusions were reached by source analysis only, with no live GPU/display available in that research session. Before closing this epic, capture the visual evidence the debug toolchain is built for:
  - A `--component paragraph` or `--component textarea` screenshot of a long CJK (Chinese or Japanese) string at a fixed width, confirming character-level wrapping with no overflow.
  - A `--script` reproduction (per `examples/demo-rust/scripts/text_edit_*.txt` conventions) driving `text_input`/`textarea` through Backspace/Delete/Left/Right over a ZWJ emoji and a combining-diacritic string, both before Task 5 (to document the bug) and after (to prove the fix), saved as before/after screenshots per `AGENTS.md`'s scripted-input conventions.
- Run `akar-diff --compare` between the before/after captures as evidence of the behavior change (not a regression gate, since the change is intentional).

### Task 9 — C ABI, Documentation, and Verification

**Status:** Not Started

- No C ABI struct shape changes are expected — `AkarTextEditState { cursor, anchor }` remains UTF-8 byte offsets; only the internal stepping logic changes granularity. Confirm this holds once Task 5-7 land (i.e. no `akar.h` regeneration should be needed), and note explicitly in the PR/commit if it turns out otherwise.
- Add a C integration test under `crates/akar-c-api/tests/` covering Backspace/Delete over a multi-codepoint grapheme cluster passed across the C boundary as UTF-8 bytes, matching the existing C ABI test conventions from epic 018 Task 7.
- Run `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
- Update this epic's `Status:` to `Done` (with commit reference, per this repo's convention) once Tasks 5-9 are complete and verified.

---

## Acceptance Criteria

- [ ] Backspace, Delete, Left, and Right in both `text_input` and `textarea` operate on extended grapheme clusters, not codepoints — a single keystroke deletes/crosses one visually-perceived character for ZWJ emoji sequences and combining-mark scripts (Vietnamese, Devanagari), verified by unit tests in `text_edit.rs`.
- [ ] `normalize_position` snaps externally-supplied `TextEditState` positions to a grapheme-cluster boundary, not merely a `char` boundary.
- [ ] `textarea`'s Up/Down vertical navigation (`character_column`/`position_at_character_column`) lands the caret in the visually-equivalent column on lines containing multi-codepoint grapheme clusters.
- [ ] Password masking (`text_input.rs`'s masked mode) renders one asterisk per grapheme cluster, not per codepoint.
- [ ] Caret geometry (`glyph_boundary_x`) is verified correct (or simplified, if empirically no longer reachable mid-cluster) once grapheme-aware cursor positions are the only positions the caret renderer ever sees.
- [ ] Existing codepoint-level Unicode test fixtures (`aé🙂` style) remain green — this is a granularity fix, not a behavior regression for simple multi-byte text.
- [ ] CJK line-breaking in `paragraph`/`textarea` is confirmed visually correct via a captured screenshot (Task 8), matching the source-analysis conclusion that no akar-side wrap configuration is needed.
- [ ] No `AkarTextEditState` C ABI shape change; `akar.h` regenerates identically. If this assumption turns out false, it is called out explicitly rather than silently absorbed.
- [ ] A C integration test exercises Backspace/Delete over a multi-codepoint grapheme cluster across the C ABI boundary.
- [ ] `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` all pass.
- [ ] This epic still does not introduce locale formatting, translation, pluralization, IME composition, or any "current locale" concept — scope stays limited to grapheme/codepoint correctness and the CJK line-breaking verification.

---

## Notes for Future Work

- Locale-aware number/date/currency formatting is explicitly out of scope for akar itself; if there is ever a companion crate, it would be a separate, optional, application-facing library, not part of `akar-core`/`akar-components`.
- Input Method Editor (IME) composition (needed for Chinese/Japanese/Korean text input) is already flagged as future work in epic 018's Notes and is closely related to this epic's concerns but large enough to warrant its own epic when prioritized.
- Translation/string-resource management is explicitly not akar's concern — it belongs to the application layer, consistent with akar not owning any application state.
