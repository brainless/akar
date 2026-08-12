# Epic 024: Internationalization (i18n)

**Status:** Research Complete and Verified — Ready for Implementation. Tasks 1-4 (research) are Done and were re-verified on 2026-08-12 against current source, with execution-based evidence on a machine with a real GPU and display. Implementation Tasks 5-9 are ready for a coding agent. Task 8 has a required ordering constraint: capture the current broken behavior before Task 5 changes it, then add/fix the durable demo fixtures and capture the corrected behavior after Tasks 5-7. No implementation code has been written.
**Goal:** Verify and, where necessary, fix akar's handling of non-ASCII, multi-script, and multi-byte text throughout text editing, layout, and rendering — without akar taking ownership of translation, locale, or formatting concerns that belong to the application.

**Prerequisite:** Epic 021 is `Status: Done`. Overlaps with [[022]] (font support, for script coverage) and [[023]] (RTL, for directional scripts); this epic's scope is the remainder — correctness for scripts and text properties that are not primarily about direction or font fallback.

---

## Introduction

i18n is often treated as a large, open-ended feature area, but for a rendering/component library — as opposed to an application framework — the honest scope is much narrower. `DEVELOP.md` is explicit about what akar does not own: the event loop, async runtime, message passing, accessibility scaffolding. The same philosophy should apply here. akar should not own string resources, locale selection, pluralization rules, date/number/currency formatting, or translation management — those are application-level concerns in an immediate-mode, C-ABI library where the caller drives every frame.

What akar does need to get right, because it is genuinely in akar's layer: correct UTF-8 and grapheme-cluster handling in text editing, correct word/line-breaking for scripts that don't use spaces (CJK) or that break differently (many South/Southeast Asian scripts), and not silently mis-measuring or mis-rendering text outside the Latin-1 range. This epic is mostly a verification and gap-finding exercise against existing text-editing and layout work (epics 012, 018, 020), not a new subsystem.

---

## Research

Initial inputs (below), expanded with concrete findings from source-analysis investigation (Tasks 1-3), and re-verified on 2026-08-12 with execution-based evidence on a machine with a real GPU and display — see "Review — 2026-08-12 verification pass" and the per-task notes:

- **Scope boundary.** Reaffirmed. Nothing in the investigation below argues for akar owning translation, locale formatting, or pluralization. See "Explicitly Out of Scope" under Task 4.
- **Grapheme clusters vs. UTF-8 byte boundaries — confirmed bug, not assumption.** Epic 018 established `TextEditState { cursor, anchor }` (`crates/akar-components/src/text_edit.rs:1-5`) as UTF-8 byte offsets, normalized to *char* boundaries via `normalize_position` (`text_edit.rs:37-43`, uses `str::is_char_boundary`). `previous_boundary`/`next_boundary` (`text_edit.rs:117-131`) step by exactly one `char` (one Unicode scalar value) using `char_indices()`/`chars()` — this is codepoint granularity, not grapheme-cluster granularity. Both `text_input.rs` (Backspace: `text_input.rs:110-117`; Delete: `text_input.rs:118-125`; Left/Right: `text_input.rs:127-137`) and `textarea.rs` (Backspace: `textarea.rs:136-143`; Delete: `textarea.rs:144-151`; Left/Right: `textarea.rs:154-157`) call these same helpers directly. Originally confirmed by source reading; the combining-mark case was subsequently reproduced live through `demo-rust` on 2026-08-12 (three Backspace presses to remove one visually-perceived character) — see Task 1 and the review section.
- **Caret geometry compounds the same codepoint-vs-cluster gap.** `crates/akar-core/src/text_pipeline.rs:368-379` (`glyph_boundary_x`) interpolates the caret's x-position *within* a shaped glyph cluster by linear fraction of **codepoint count** (`cluster.chars().count()`, line 371) rather than grapheme-cluster count. When cosmic-text shapes multiple codepoints (combining marks, some ligatures) into one glyph cluster, and the editing engine's codepoint-granular cursor stops mid-cluster, the caret renders at a plausible-looking but not necessarily correct fractional x inside that glyph. This is a distinct, smaller finding from the navigation bug above — it's a geometry approximation, not a crash/mis-edit — but shares the same root cause (codepoint granularity standing in for grapheme granularity) and should be fixed by the same underlying change.
- **No grapheme-segmentation crate in akar's direct dependency graph.** `unicode-segmentation` (`Cargo.lock:2392-2394`) and `unicode-width` (`Cargo.lock:2398-2400`) are present in `Cargo.lock` only as *transitive* dependencies. Provenance corrected on 2026-08-12 by `cargo tree -i`: `unicode-segmentation` 1.13.3 has exactly one reverse dependency, `cosmic-text` 0.18.2 (via `glyphon` 0.11.0) — **not** `winit`; `unicode-width` 0.2.2 arrives via `codespan-reporting` → `naga` → `wgpu` — also not `winit`. Exactly one version of `unicode-segmentation` is resolved (1.13.3); the 1.12.0 copy present in the local cargo registry belongs to another workspace and is not in akar's lockfile. Neither `akar-core/Cargo.toml` nor `akar-components/Cargo.toml` lists either crate directly. Since `unicode-segmentation` is already resolved in the workspace's dependency tree at a compatible version, adding it as a direct dependency of `akar-components` (where `text_edit.rs` lives) costs no new transitive dependency and no version-resolution risk.
- **Line/word breaking for CJK — confirmed correct by default, no akar action needed.** `crates/akar-core/src/text_pipeline.rs:61-97` (`TextPipeline::set_text`) never calls `glyphon::Buffer::set_wrap` / never touches `cosmic_text::Wrap` — it relies entirely on cosmic-text's buffer default. Read directly from the vendored cosmic-text source at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.18.2/src/buffer.rs:262`: the default is `Wrap::WordOrGlyph` ("wraps at the word level, or fallback to glyph level if a word can't fit on a line by itself" — `cosmic-text-0.18.2/src/layout.rs:111-120`). Critically, cosmic-text's line-breaking is not naive space-splitting: `cosmic-text-0.18.2/src/shape.rs:870` calls `unicode_linebreak::linebreaks(span)`, i.e. the UAX #14 Unicode Line Breaking Algorithm, to find break opportunities before shaping into "words." UAX #14 treats CJK ideographs as break-opportunities between (almost) every character, independent of whitespace. Conclusion (source-analysis based, no rendered screenshot available in this environment — see Task 2): CJK paragraph text wrapping already works correctly today with zero akar-side configuration, because akar never overrides the correct cosmic-text default. This is a "verify existing correctness," not a gap.
- **Codebase assumption audit — clean outside the text-editing engine.** `grep -rn '\.len()\|chars().count()\|chars().nth(' crates/akar-core/src crates/akar-layout/src crates/akar-components/src` turns up no character-count-assumes-byte-count bugs in `akar-layout` (it has no text-measuring logic of its own — all width/height comes from `akar_core::TextPipeline` via the `default_measure_fn` callback described in `DEVELOP.md`'s "Component lifecycle" section) or in any `akar-components` widget other than `text_edit.rs`/`text_input.rs`/`textarea.rs`. The `.len()` calls elsewhere in those three files are legitimate byte-offset arithmetic consistent with `TextEditState`'s documented byte-offset contract (e.g. `text_edit.rs:23,38,130`; `text_input.rs:121,143-144`; `textarea.rs:25,52-53`) — not bugs. Two narrower findings inside the text-editing files: `text_input.rs:189` masks passwords via `"*".repeat(value.chars().count())`, which uses codepoint count rather than grapheme count — for a multi-codepoint grapheme (e.g. a ZWJ emoji sequence typed into a password field) this reveals the internal codepoint count as extra asterisks, a minor information leak/cosmetic bug, listed as a Task below. `textarea.rs:28-30` (`character_column`) and `textarea.rs:32-37` (`position_at_character_column`) compute Up/Down column position by codepoint count for the same reason as the cursor-boundary functions — same root cause, same fix.
- **Downstream reference (`~/Projects/daftprompt`).** `grep -rniE "TODO|FIXME|unicode|grapheme|CJK|non-ascii|multibyte|utf-?8"` over `~/Projects/daftprompt/src/ui/` returns no hits — no existing workaround or bug comment. `src/ui/render.rs` uses `akar_text_input` (aliased from `akar-components`) for its search box (see `render.rs:48`, `render.rs:1138`) but has no bespoke text-width or character-counting logic of its own; it delegates entirely to akar's widget. Nothing in daftprompt surfaces a new i18n concern beyond what's found directly in akar's own text-editing code.
- **Overlap discipline confirmed.** Checked `epics/022-font-support.md` and `epics/023-rtl-text-rendering.md` for scope collision: 022 explicitly owns font/script coverage (CJK/Arabic/Devanagari fallback) and calls itself "a soft prerequisite for 023 and 024"; 023 explicitly excludes CJK vertical writing modes as unrelated to RTL. Neither epic currently touches grapheme-cluster navigation or line-breaking configuration, so this epic's scope (below) does not duplicate either.

---

## Tasks

### Task 1 — Grapheme Cluster Audit in Text Editing

**Status:** Done — and, as of 2026-08-12, **verified by execution**, not source analysis alone. The combining-diacritic prediction below was reproduced end-to-end through the real `demo-rust` binary and confirmed by screenshot (see "Review — 2026-08-12 verification pass" → "Task 1 reproduced live"). All `file:line` citations in this task resolve correctly against current source.

**Readiness:** Ready for implementation (research task; no code owed)

- Simple multi-byte-but-single-codepoint characters (CJK, e.g. `汉`): **not affected**. `previous_boundary`/`next_boundary` step by one `char`, and a CJK ideograph is one `char` (one Unicode scalar value, 3 UTF-8 bytes). Backspace/Delete/Left/Right will behave correctly for plain CJK text today.
- Combining diacritics (e.g. Vietnamese `ệ` = `e` + combining circumflex + combining dot-below, or Devanagari conjuncts): **predicted failure**. `previous_boundary`/`next_boundary` (`crates/akar-components/src/text_edit.rs:117-131`) advance by exactly one `char`. A base character followed by one or more combining marks is multiple `char`s but one grapheme cluster. Backspace at the end of such a cluster will delete only the last combining mark, not the whole visual character — the cursor appears to not move / the character appears unchanged after one Backspace press, requiring N presses to remove what the user perceives as one character.
- Multi-codepoint emoji / ZWJ sequences (e.g. family emoji `👨‍👩‍👧‍👦`, several codepoints joined by U+200D ZERO WIDTH JOINER): **predicted failure**, same mechanism as above but more visually severe — Backspace/Delete will step through and can leave a "broken" emoji rendering (dangling ZWJ or an isolated component emoji) rather than deleting the whole visual glyph in one keystroke. Left/Right arrow navigation will likewise stop the caret mid-sequence.
- Root cause is single and shared across `text_input.rs` and `textarea.rs`: both files import and call the same `previous_boundary`/`next_boundary` from `text_edit.rs` (confirmed via `grep -n previous_boundary\|next_boundary crates/akar-components/src/{text_input,textarea}.rs`), so a fix in the shared engine (epic 018's design intent) fixes both widgets at once.
- No `unicode-segmentation`-equivalent grapheme logic exists anywhere in `akar-components` today (confirmed by grep — the crate is present transitively only via `cosmic-text` in `Cargo.lock`).

### Task 2 — Line-Breaking Verification for CJK Text

**Status:** Done — re-verified 2026-08-12. All three cosmic-text citations resolve exactly (`buffer.rs:262` = `wrap: Wrap::WordOrGlyph` in `Buffer`'s constructor; `layout.rs:111-120` = the `Wrap` enum with the quoted `WordOrGlyph` doc comment; `shape.rs:870` = `for (end_lb, _) in unicode_linebreak::linebreaks(span)`), and `text_pipeline.rs:61-97` still never calls `set_wrap`. The wrapping conclusion is now **verified by execution at the cosmic-text layer** with an out-of-repo probe crate (no akar source touched): a 33-character Chinese string in a 120px-wide `Buffer` with default settings breaks into 5 visual lines of widths `112.00, 112.00, 112.00, 112.00, 80.00` — i.e. 7 ideographs per line, character-level breaking with no overflow, with `buffer.wrap()` reporting `WordOrGlyph`. What remains unverified is only the akar render path around it (Task 8).

**Readiness:** Ready for implementation (research task; no akar-side code change required — see Task 8 for the remaining visual confirmation)

- `crates/akar-core/src/text_pipeline.rs:61-97` (`TextPipeline::set_text`) constructs the `glyphon::Buffer`, calls `buffer.set_metrics`, `buffer.set_size`, and `buffer.set_text(..., glyphon::Shaping::Advanced, None)`, then `buffer.shape_until_scroll`. It never calls `buffer.set_wrap` — wrap mode is left at cosmic-text's own default.
- Read directly from `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/cosmic-text-0.18.2/src/buffer.rs:262`: the default is `Wrap::WordOrGlyph`. Per `cosmic-text-0.18.2/src/layout.rs:111-120`, this wraps at the word level and falls back to glyph-level breaking when a "word" doesn't fit — this is already the more permissive of the two non-`None` options, not the strict `Wrap::Word` that would risk failing to wrap unbroken CJK runs.
- More importantly, cosmic-text's definition of "word" for line-breaking purposes is not whitespace-delimited. `cosmic-text-0.18.2/src/shape.rs:870` calls `unicode_linebreak::linebreaks(span)` — the UAX #14 Unicode Line Breaking Algorithm — to compute break opportunities before grouping into shaping "words." UAX #14 assigns break-opportunity classes to CJK ideographs such that (with limited exceptions for punctuation-pairing rules) a break opportunity exists between nearly every pair of adjacent CJK characters, independent of whitespace.
- Conclusion: CJK paragraph wrapping in `paragraph`/`textarea` already works correctly today with zero configuration from akar's `TextPipeline`, because akar does not override cosmic-text's already-correct default. No akar-side fix is required for Task 2's concern. This should be confirmed visually once a GPU/display environment is available (a CJK screenshot through `--component paragraph` or `--component textarea` is a low-cost follow-up, listed as a Task below, primarily to catch any font-fallback interaction with epic 022 rather than because the wrap logic itself is in doubt).

### Task 3 — Codebase Assumption Audit

**Status:** Done — re-verified 2026-08-12; every `file:line` in the punch list below resolves correctly against current source (`text_edit.rs:117-131`, `text_pipeline.rs:368-379` with `cluster.chars().count()` on line 371, `text_input.rs:189`, `textarea.rs:28-30` and `28-37`). One item is amended: see "Correction to punch-list item 2" in the review section — `glyph_boundary_x` is not merely an approximation that disappears once the cursor is grapheme-correct.

**Readiness:** Ready for implementation (research task; no code owed)

Punch list (see Research above for the full grep methodology and line citations):

1. `crates/akar-components/src/text_edit.rs:117-131` — `previous_boundary`/`next_boundary` operate on codepoints, not grapheme clusters. **Root cause of Task 1's findings.**
2. `crates/akar-core/src/text_pipeline.rs:368-379` (`glyph_boundary_x`) — interpolates caret x-position within a shaped glyph cluster using codepoint count (`cluster.chars().count()`), not grapheme-cluster count. Same root cause as #1, surfaces as a caret-geometry approximation rather than a broken edit. **Amended 2026-08-12:** this is a permanent mis-measurement, not a transient one that disappears once #1 is fixed — see "Correction to punch-list item 2" in the review section and Task 7.
3. `crates/akar-components/src/text_input.rs:189` — password masking via `"*".repeat(value.chars().count())` renders one asterisk per codepoint, not per grapheme cluster; a multi-codepoint grapheme in a password field leaks its codepoint count as extra asterisks. Minor; independent of #1/#2's navigation fix but shares the same "what counts as one visible character" question.
4. `crates/akar-components/src/textarea.rs:28-30` (`character_column`) and `textarea.rs:32-37` (`position_at_character_column`) — Up/Down vertical navigation computes column position by codepoint count. Same root cause as #1; a grapheme-aware column function is needed for Up/Down to land the caret in the visually-equivalent column on multi-codepoint-grapheme lines.
5. `akar-layout` — clean. It owns only taffy tree resolution; all text measurement is delegated to `akar_core::TextPipeline` via the measure callback (`DEVELOP.md`'s "Component lifecycle: construct, compute, paint"), so there is no separate character-counting logic to audit there.
6. All other `akar-components` widgets (`alert.rs`, `avatar.rs`, `badge.rs`, `card.rs`, `data_item.rs`, `data_list.rs`, `label.rs`, `paragraph.rs`, `select.rs`, `tabs.rs`, etc.) — `.len()`/`.chars()` usages found by grep are on `Vec`s (draw-list lengths, option lists, container/card counts), not on user-facing text content. No latent i18n bugs found outside the text-editing engine.
7. `~/Projects/daftprompt/src/ui/` — no TODO/FIXME/unicode/grapheme/CJK/non-ASCII comments found (`grep -rniE` returned no matches). The one akar text widget it uses (`akar_text_input`, aliased in `render.rs:48`, called at `render.rs:1138`) is used as-is with no bespoke width/character-counting workaround, so daftprompt has not (yet) surfaced or worked around any of the issues in #1-#4 above — consistent with them being real but not yet hit by daftprompt's current usage (a single-line search box, unlikely to have exercised combining-mark or ZWJ input).

### Task 4 — Scope Proposal for First Implementation Pass

**Status:** Done — converted into Tasks 5-9 below. Re-verified 2026-08-12 with two corrections applied: the `unicode-segmentation` version pin (see Task 5) and the `glyph_boundary_x` "may reduce to an assertion" hypothesis, which is now disproved (see Task 7).

**Readiness:** Ready for implementation (research task; no code owed)

**Proposed fix, concretely.** Adopt `unicode-segmentation` (already resolved in the workspace's `Cargo.lock` transitively via `cosmic-text`, so no new dependency-tree risk) as a **direct** dependency of `akar-components` (where `text_edit.rs` lives). Replace the codepoint-stepping logic in the shared text-editing engine with grapheme-cluster-stepping logic, in one place, so both `text_input.rs` and `textarea.rs` inherit the fix without per-widget changes:

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

**Readiness:** Ready for implementation

- Add `unicode-segmentation = "1"` to `[workspace.dependencies]` in the root `Cargo.toml`, then depend on it via `unicode-segmentation.workspace = true` in `crates/akar-components/Cargo.toml`. **Correction applied 2026-08-12:** do *not* pin `1.13.3` as the earlier draft said. The root `Cargo.toml:15-23` convention is loose major/minor requirements (`wgpu = "29"`, `glyphon = "0.11"`, `glam = "0.33"`, `taffy = "0.11"`, `winit = "0.30"`, `thiserror = "2"`, `log = "0.4"`, `bytemuck = { version = "1", ... }`) — not exact patch pins. `"1"` still resolves to the 1.13.3 already in `Cargo.lock` (verified: it is the only version of that crate in the lockfile), so there is no dependency bump either way. Using the workspace table matches `crates/akar-components/Cargo.toml`, which already declares `glam.workspace = true` / `glyphon.workspace = true` and no per-crate version pins. Note also that Task 7 **does** require the same crate inside `akar-core` (see Task 7 — this is now settled, not conditional); add `unicode-segmentation.workspace = true` to `crates/akar-core/Cargo.toml` too rather than duplicating a version.
- Reimplement `previous_boundary`/`next_boundary` in `crates/akar-components/src/text_edit.rs:117-131` using `UnicodeSegmentation::grapheme_indices(true)` (extended grapheme clusters) instead of `char_indices()`/`chars()`.
- Update `normalize_position` (`text_edit.rs:37-43`) so externally-supplied `TextEditState` positions are snapped to a grapheme-cluster boundary, not merely a `char` boundary. Keep the existing floor-only (snap-backwards) semantics; `usize::MAX` must still clamp to `value.len()`, which is always a grapheme boundary. Compatibility with epic 018's suite was checked case by case on 2026-08-12 and no existing test changes result — see "Does grapheme snapping in `normalize_position` break the epic-018 tests?" in the review section for the per-test analysis and the one behavioural caveat to keep in mind.
- Add test fixtures the current suite lacks: a ZWJ family emoji (e.g. `"👨‍👩‍👧‍👦"`), a Vietnamese or Devanagari combining-mark string, and mixed ASCII/CJK/combining-mark content, covering Backspace, Delete, Left, Right, and select-all.
- Verify no regression against the existing codepoint-level fixtures (`aé🙂` style) already in `text_edit.rs`'s test module.
- No `text_input.rs`/`textarea.rs` changes should be required for Backspace/Delete/Left/Right beyond this shared-engine change, since both already call through `previous_boundary`/`next_boundary`; confirm this by re-running (not editing) their existing tests.

### Task 6 — Grapheme-Aware Vertical Navigation and Password Masking

**Status:** Not Started

**Readiness:** Ready for implementation (depends on Task 5 only for the shared `unicode-segmentation` dependency declaration)

- Reimplement `character_column` and `position_at_character_column` in `crates/akar-components/src/textarea.rs:28-37` using grapheme-cluster counts (via the same `unicode-segmentation` dependency added in Task 5) so `move_vertical`/Up/Down land in the visually-equivalent column on lines containing multi-codepoint grapheme clusters.
- Change `crates/akar-components/src/text_input.rs:189` from `value.chars().count()` to a grapheme-cluster count for password-mask asterisk rendering.
- Add tests: Up/Down navigation across textarea lines containing combining-mark/emoji sequences of differing codepoint-vs-grapheme length; masked `text_input` asterisk count for a value containing a ZWJ sequence (expect one asterisk per grapheme cluster, not per codepoint).

### Task 7 — Grapheme-Aware Caret Geometry

**Status:** Not Started — but its one open question is now **answered** (2026-08-12), so the task is no longer conditional.

**Readiness:** Ready for implementation

- **Answered: yes, a grapheme-corrected cursor can still land strictly inside a shaped glyph cluster.** Measured on this machine with an out-of-repo `cosmic-text` 0.18.2 probe using the same `Shaping::Advanced` + default `Attrs` path as `text_pipeline.rs:88-93`. The Arabic lam-alef mandatory ligature `"\u{0644}\u{0627}"` is **two** extended grapheme clusters (boundaries at bytes 0 and 2) shaped into **one** `LayoutGlyph` spanning `0..4`. Byte 2 is a legal grapheme-correct cursor position strictly inside that glyph's range, so the interpolation branch in `glyph_boundary_x` stays reachable forever. The "may reduce to an assertion/simplification" hypothesis in Task 4 is therefore **disproved** — do not delete the interpolation.
- **The codepoint-vs-grapheme fraction demonstrably differs.** `"\u{0644}\u{064E}\u{0627}"` (lam + fatha + alef) shapes to a single glyph spanning `0..6` containing 3 codepoints but 2 grapheme clusters, with the grapheme boundary at byte 4. Today's formula gives `2/3 ≈ 0.667` of the glyph advance; a grapheme-based formula gives `1/2 = 0.5`. So this is a real behaviour change, not a no-op refactor. Change the interpolation fraction from codepoint count to grapheme-cluster count, and add `unicode-segmentation.workspace = true` to `crates/akar-core/Cargo.toml` (this is now required, not conditional).
- **The proposed equal-width grapheme subdivision matches cosmic-text 0.18.2's own cursor hit-testing contract.** `Buffer::hit` in the pinned source (`buffer.rs:922-1018`) takes the same `LayoutGlyph` cluster, counts `cluster.grapheme_indices(true)`, divides `glyph.w` evenly by that count, and reverses the selected boundary according to `glyph.level.is_rtl()`. Implement `glyph_boundary_x` as the forward mapping of that rule, and add a round-trip-style test against `Buffer::hit` where practical. This is stronger guidance than inventing a new ligature-caret policy in akar.
- **Two further shaping facts a fix must not trip over**, from the same probe:
  - A glyph cluster can also *subdivide* one grapheme cluster. Thai `"\u{0E01}\u{0E33}"` (ko kai + sara am) is one grapheme (`0..6`) but shapes to glyphs `0..3` and `3..6` — so `index` at a grapheme boundary is not guaranteed to be a glyph-range endpoint, and an "assert index is a glyph boundary" style simplification would be wrong in the other direction too.
  - Several `LayoutGlyph`s can share an identical `start..end`. Devanagari `"\u{0915}\u{094D}\u{0937}\u{093F}"` yields two glyphs both spanning `0..12`; Thai `"\u{0E01}\u{0E33}"` yields two glyphs both spanning `3..6` with widths `0.00` then `9.51`; `"e\u{0301}\u{0301}\u{0301}"` yields three glyphs all spanning `0..7` with widths `8.20, 0.00, 0.00`. `caret_x` (`text_pipeline.rs:366-369`) selects with `.find(|glyph| index >= glyph.start && index <= glyph.end)`, i.e. the **first** match. Cosmic-text's `Buffer::hit` also walks glyphs in order and accepts the first glyph whose x-range contains the point, so matching its grapheme subdivision is the appropriate v1 contract. Add a regression test for a zero-width/shared-range case; a more sophisticated ligature-caret model would require upstream shaping data and is out of scope.
  - Latin "coding ligature" candidates (`"office fluffy"`, `"a != b -> c"`) did **not** ligate under the default system font on this machine — every glyph mapped 1:1 to one byte. Multi-grapheme glyph clusters are therefore a script/font property (Arabic, Indic), not a Latin-font property, and any test fixture must use a script that actually produces them.
- Add/extend `text_pipeline.rs`'s existing `#[cfg(test)] mod tests` (which already builds `glyphon::Buffer`s directly, see `text_pipeline.rs:385-397`) with a combining-mark or CJK+combining case exercising `text_geometry`'s caret output.

### Task 8 — Visual Confirmation of CJK Wrapping and Grapheme Navigation

**Status:** Not Started

**Readiness:** Ready for implementation, with ordering constraint — first capture the broken baseline using the known-good coordinate path below, before Task 5 lands. Adding or repairing a labeled demo fixture and adding CJK sample content are ordinary implementation work within this task, not external blockers. After Tasks 5-7, run the same fixture for the corrected captures. The underlying claims of Tasks 1 and 2 are already verified by execution; this task creates durable repository-toolchain evidence.

- Tasks 1 and 2 have since been verified by execution (see the review section); what this task still owes is captured before/after evidence in the repo's own toolchain. Fixture gaps found on 2026-08-12:
  - `--dump-layout` reports `0 0 0 0` for `form_name`, `form_notes` and every other label except `alert` and `tab_bar`, because the demo opens on the `List` tab and inactive-tab widgets are never laid out. Label-addressed `click @form_name` therefore resolves to a zero rect and clicks at the origin.
  - Consequently the repo's own epic-018 scripts do not currently exercise what they claim: running `examples/demo-rust/scripts/text_edit_backspace.txt` produced four screenshots that are byte-identical to each other *and* to an untouched idle-frame capture (same MD5). This should be fixed or the scripts should be re-pointed, independently of this epic.
  - Working path found: click the `Form` tab by coordinates (`click 679 123` at the demo's default 800x600), then click the Name field by coordinates (`click 400 208`), then use `type "..."`. That sequence produces distinct, correct screenshots. `paste @form_name "..."` after the same coordinate-based focus produced no visible text in this pass and needs investigation before being used as a fixture.
  - The demo contains no CJK text and no CJK-capable fixture, so the implementation must add sample content to `examples/demo-rust` (or an equivalent isolatable fixture) before capturing the CJK screenshot.
- Capture the visual evidence the debug toolchain is built for:
  - A `--component paragraph` or `--component textarea` screenshot of a long CJK (Chinese or Japanese) string at a fixed width, confirming character-level wrapping with no overflow.
  - A `--script` reproduction (per `examples/demo-rust/scripts/text_edit_*.txt` conventions) driving `text_input`/`textarea` through Backspace/Delete/Left/Right over a ZWJ emoji and a combining-diacritic string, both before Task 5 (to document the bug) and after (to prove the fix), saved as before/after screenshots per `AGENTS.md`'s scripted-input conventions.
- Run `akar-diff --compare` between the before/after captures as evidence of the behavior change (not a regression gate, since the change is intentional).

### Task 9 — C ABI, Documentation, and Verification

**Status:** Not Started

**Readiness:** Ready for implementation (the C ABI question below is now settled; the task itself still gates on Tasks 5-7 landing)

- **Confirmed 2026-08-12 by reading `crates/akar-c-api/src/lib.rs`: no C ABI shape change is needed.** `AkarTextEditState` is `{ cursor: u32, anchor: u32 }` (`lib.rs:182-185`). Both text-edit entry points convert it to and from `akar_components::TextEditState` purely by integer cast — `akar_text_input_ex` at `lib.rs:1807-1809` (in) and `lib.rs:1825-1828` (out), `akar_textarea_ex` at `lib.rs:2037-2039` (in) and `lib.rs:2056-2059` (out). The struct is also embedded by value in the two options structs (`lib.rs:1676`, `lib.rs:1846`). Nothing in the FFI layer inspects, steps, or validates offsets, so changing stepping granularity inside `akar-components` is invisible to the ABI and `akar.h` should regenerate identically. Still verify the regeneration produces no diff, and call it out explicitly if it does.
- Add a C integration test under `crates/akar-c-api/tests/` covering Backspace/Delete over a multi-codepoint grapheme cluster passed across the C boundary as UTF-8 bytes, matching the existing C ABI test conventions from epic 018 Task 7. Concretely: extend `crates/akar-c-api/tests/text_edit.c` (142 lines; it already builds raw UTF-8 byte arrays for this purpose, e.g. the `utf8_paste` fixture at `text_edit.c:107-119`), which is compiled as a static library and driven from Rust by the single `#[test]` in `crates/akar-c-api/tests/c_text_edit.rs`. No new test harness is needed.
- Run `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace`.
- Update this epic's `Status:` to `Done` (with commit reference, per this repo's convention) once Tasks 5-9 are complete and verified.

---

## Acceptance Criteria

- [ ] Backspace, Delete, Left, and Right in both `text_input` and `textarea` operate on extended grapheme clusters, not codepoints — a single keystroke deletes/crosses one visually-perceived character for ZWJ emoji sequences and combining-mark scripts (Vietnamese, Devanagari), verified by unit tests in `text_edit.rs`.
- [ ] `normalize_position` snaps externally-supplied `TextEditState` positions to a grapheme-cluster boundary, not merely a `char` boundary.
- [ ] `textarea`'s Up/Down vertical navigation (`character_column`/`position_at_character_column`) lands the caret in the visually-equivalent column on lines containing multi-codepoint grapheme clusters.
- [ ] Password masking (`text_input.rs`'s masked mode) renders one asterisk per grapheme cluster, not per codepoint.
- [ ] Caret geometry (`glyph_boundary_x`) interpolates by grapheme-cluster count rather than codepoint count. The alternative "simplify it away" outcome is ruled out: mid-glyph-cluster grapheme boundaries are reachable (Arabic lam-alef), so the interpolation must be kept and corrected, not removed.
- [ ] Existing codepoint-level Unicode test fixtures (`aé🙂` style) remain green — this is a granularity fix, not a behavior regression for simple multi-byte text. Baseline on 2026-08-12 at commit `7413111`: `cargo test --workspace` passes with 338 tests (akar-c-api 32 + api.rs 11 + c_text_edit.rs 1 + akar-components 204 + akar-core 30 + akar-diff 3 + akar-layout 42 + text_measurement.rs 10 + akar-winit 5), 0 failures.
- [ ] CJK line-breaking in `paragraph`/`textarea` is confirmed visually correct via a captured screenshot (Task 8), matching the conclusion — now measured at the cosmic-text layer — that no akar-side wrap configuration is needed. Note this criterion cannot be met without adding CJK sample content to `examples/demo-rust`; if that is judged out of scope, downgrade the criterion to the already-obtained cosmic-text-layer measurement and say so explicitly rather than leaving it unchecked.
- [ ] No `AkarTextEditState` C ABI shape change; `akar.h` regenerates identically. If this assumption turns out false, it is called out explicitly rather than silently absorbed.
- [ ] A C integration test exercises Backspace/Delete over a multi-codepoint grapheme cluster across the C ABI boundary.
- [ ] `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` all pass.
- [ ] This epic still does not introduce locale formatting, translation, pluralization, IME composition, or any "current locale" concept — scope stays limited to grapheme/codepoint correctness and the CJK line-breaking verification.

---

## Review — 2026-08-12 verification pass

Every "Confirmed finding" and every `file:line` citation above was re-checked against current source at commit `7413111`. Claims are labelled **verified by execution** (a command was run and its output observed) or **verified by source reading**. Environment note: this pass ran on macOS with a real GPU and display, so the original file's repeated "no live GPU/display in this research environment" caveat is false here and has been removed from the affected task statuses.

### Citations checked

- All `crates/` citations resolve: `text_edit.rs:1-5`, `:23`, `:37-43`, `:117-131`; `text_input.rs:110-117` (Backspace), `:118-125` (Delete), `:189` (masking); `textarea.rs:18-37`, `:136-143` (Backspace), `:144-151` (Delete), `:154-157` (Left/Right/Up/Down); `text_pipeline.rs:61-97` (`set_text`), `:242` (`text_geometry`), `:368-379` (`glyph_boundary_x`, with `cluster.chars().count()` on `:371`), `:382-397` (test module and `shaped_buffer`). One trivial drift: the `text_input.rs` Left/Right arms are at `:128-137`, not `:127-137`.
- All vendored `cosmic-text-0.18.2` citations resolve: `buffer.rs:262`, `layout.rs:111-120`, `shape.rs:870`.
- The claim that both widgets route through the same two functions is verified by grep: `previous_boundary`/`next_boundary` are called only at `text_input.rs:114,122,130,135` and `textarea.rs:140,148,155,157`.

### Dependency-resolution correction

`cargo tree -i unicode-segmentation` shows a single resolved version, **1.13.3**, with exactly one reverse dependency: `cosmic-text 0.18.2` → `glyphon 0.11.0`. The epic's claim that `winit` also pulls it in is wrong. Separately, `unicode-width 0.2.2` comes from `codespan-reporting` → `naga 29.0.4` → `wgpu`, again not `winit`. The 1.12.0 copy in the local cargo registry is not part of this workspace's lockfile. The Task 5 instruction to pin `1.13.3` was also inconsistent with the root `Cargo.toml:15-23` convention (loose major/minor requirements) and has been corrected to `"1"`.

### Task 1 reproduced live

Verified by execution through the real `demo-rust` binary, no source changes, script file kept out of the repo:

1. `click 679 123` (Form tab, by coordinate), `click 400 208` (Name field), `type "abc"` — screenshot shows `abc`.
2. `type "e\u{0302}\u{0323}"` (decomposed Vietnamese `ệ`: one grapheme cluster, three codepoints) — screenshot changes.
3. Three consecutive `key Backspace` presses produce three *distinct* screenshots; the second one renders `abce` (base letter surviving with both marks stripped), and the third returns to a frame byte-identical to the `abc` capture.

So deleting one visually-perceived character currently costs three Backspace presses, and intermediate frames display a partially-decomposed character. Task 1's prediction is confirmed as observed behaviour, not inference. The ZWJ-emoji half of Task 1 was not reproduced live because the `paste` script step did not deliver text in this pass (see Task 8 blockers); it remains verified by source reading only, via the same `previous_boundary` code path.

### Correction to punch-list item 2 (`glyph_boundary_x`)

Task 3 item 2 and Task 4 both describe `glyph_boundary_x` as an approximation that "may reduce to an assertion/simplification" once the cursor is grapheme-correct. That is wrong, measured with an out-of-repo `cosmic-text` 0.18.2 probe. See Task 7 for the numbers. Summary: Arabic lam-alef is two graphemes in one glyph, so a grapheme-correct cursor still lands mid-glyph; the codepoint fraction and grapheme fraction genuinely differ (`0.667` vs `0.5` for lam + fatha + alef); Thai sara am shows one grapheme split across two glyphs; and several scripts produce multiple `LayoutGlyph`s sharing an identical byte range, which interacts with `caret_x`'s first-match `find` at `text_pipeline.rs:366-369`.

Consequence for Task 5's Arabic examples: the caret-geometry portion of this epic overlaps [[023]]'s subject matter in the sense that the clearest reproduction cases are RTL scripts. The *code* ownership is still unambiguous — [[023]] owns direction-aware caret *movement* (its Task 10 adds `visual_previous`/`visual_next` wrappers around `previous_boundary`/`next_boundary`), this epic owns the *granularity* inside those same two functions and inside `glyph_boundary_x`. The two changes are additive and can land in either order; whichever lands second should re-run the other's tests.

### Does grapheme snapping in `normalize_position` break the epic-018 tests?

No. Each existing fixture was checked by hand against extended grapheme boundaries:

- `normalizes_invalid_utf8_positions` — `"aé🙂"`, cursor `3` is both a char and a grapheme boundary; anchor `99` clamps to `value.len()`, always a grapheme boundary.
- `replaces_unicode_and_newline_selection` — cursor `7` in `"aé\n🙂z"` is mid-`🙂`, already floored to `4` today; `4` is also a grapheme boundary.
- `invalid_external_selection_is_normalized_before_deletion` — cursor `2` is mid-`é`, floored to `1` today; `1` is also a grapheme boundary.
- `backspace_and_delete_ranges_are_unicode_safe`, `select_all_covers_unicode_value`, `copy_returns_selected_text_only` — every position used is a grapheme boundary in `"aé🙂"` (each of `a`, `é` precomposed, `🙂` is its own cluster).

Two caveats for the implementer, neither of which requires a design change:

- `replace_selection` (`text_edit.rs:45-54`) stores the post-insert cursor as `range.start + replacement.len()` **without** normalizing. That is correct for sequential typing (after inserting a combining mark, the cursor sits at the end of the enlarged cluster, which is a boundary), but the invariant is worth an explicit test so a future floor-only snap cannot silently drag the caret back to the start of a cluster mid-typing.
- `"\r\n"` is a single grapheme cluster. `normalize_paste` (`text_edit.rs:68-75`) rewrites `\r\n` and `\r` to `\n` on every paste, so no buffer should contain a CRLF, but a `TextEditState` handed in by a C caller over a string the caller built itself could. Floor-snapping handles it safely; just do not assume `\n`-adjacent positions are always boundaries.

### Task 8 fixture gaps and working path

Verified by execution. `--dump-layout` at HEAD reports real rects for only two labels, `alert 200 48 600 48` and `tab_bar 200 104 600 40`; all 22 others, including `form_name`, `form_notes` and `navbar_dropdown`, report `0 0 0 0`. Running the repo's own `examples/demo-rust/scripts/text_edit_backspace.txt` produced four captures with identical MD5s, also identical to an idle-frame capture — the script's `click @form_name` lands at the origin because the demo opens on the `List` tab and the Form widgets never lay out. Coordinate-based clicking works and was used for the Task 1 reproduction above. `paste @form_name "..."` after coordinate-based focus produced no visible text and needs investigation. None of this is an external blocker: Task 8 can capture its pre-fix baseline through the coordinate path, then repair or replace the fixture as part of implementation.

### Baseline

`cargo test --workspace` passes at HEAD: 338 tests, 0 failures, exit code 0.

## Notes for Future Work

- Locale-aware number/date/currency formatting is explicitly out of scope for akar itself; if there is ever a companion crate, it would be a separate, optional, application-facing library, not part of `akar-core`/`akar-components`.
- Input Method Editor (IME) composition (needed for Chinese/Japanese/Korean text input) is already flagged as future work in epic 018's Notes and is closely related to this epic's concerns but large enough to warrant its own epic when prioritized.
- Translation/string-resource management is explicitly not akar's concern — it belongs to the application layer, consistent with akar not owning any application state.
