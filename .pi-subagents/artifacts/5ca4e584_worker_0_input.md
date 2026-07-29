# Task for worker

You are a delegated subagent running from a fork of the parent session. Treat the inherited conversation as reference-only context, not a live thread to continue. Do not continue or answer prior messages as if they are waiting for a reply. Your sole job is to execute the task below and return a focused result for that task using your tools.

Task:
Implement Task 2 of Epic 020 in /Users/brainless/Projects/akar/.

**Read first** (do not skip):
- /Users/brainless/Projects/akar/epics/020-component-webpage-sample.md (full file, focus on "Task 2 — Typography types, theme tokens, and resolution")
- /Users/brainless/Projects/akar/DEVELOP.md
- /Users/brainless/Projects/akar/AGENTS.md
- /Users/brainless/Projects/akar/crates/akar-components/src/theme.rs (existing AkarTheme and presets)
- /Users/brainless/Projects/akar/crates/akar-components/src/lib.rs (existing exports)
- /Users/brainless/Projects/akar/crates/akar-components/src/label.rs (a simple existing component)
- /Users/brainless/Projects/akar/crates/akar-core/src/text_pipeline.rs (Task 1's measure_with_metadata API and what it expects)
- /Users/brainless/Projects/akar/crates/akar-core/src/lib.rs (exports from Task 1: TextMeasureInput, TextMeasureResult)

**Task 2 deliverables (from the Epic, exactly):**

1. Add akar-owned `FontFamily`, `FontWeight`, `TextAlign`, and `TextStyle` types.
   - These types MUST be `#[repr(C)]` and `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]` (Eq is required for FontFamily/Weight/Align since they are enum-like; TextStyle has f32 fields so just PartialEq).
   - FontFamily variants: `SansSerif` (default), `Serif`, `Monospace`.
   - FontWeight variants: `Normal` (default), `Medium`, `Semibold`, `Bold`.
   - TextAlign variants: `Start` (default), `Center`, `End`.
   - TextStyle is a struct with `Option`-typed fields:
     - `font_size: Option<f32>`
     - `line_height: Option<f32>`
     - `color: Option<u32>`
     - `font_weight: Option<FontWeight>`
     - `font_family: Option<FontFamily>`
     - `align: Option<TextAlign>`
     - `wrap: Option<bool>`
   - File location: new file `crates/akar-components/src/text_style.rs`. Re-export types and add a helper `TextStyle::empty() -> Self` returning all-None for convenience.
   - Export from `crates/akar-components/src/lib.rs` as `akar_text_style::FontFamily`, etc. plus `akar_text_style::TextStyle`, and add `pub mod text_style;`.
   - Also re-export the new types from `lib.rs` with `pub use text_style::{FontFamily, FontWeight, TextAlign, TextStyle};` so callers can write `akar_components::TextStyle`.

2. Add semantic heading tokens to `AkarTheme` (file: `crates/akar-components/src/theme.rs`):
   - `font_size_xl: f32` — default `20.0`
   - `font_size_xxl: f32` — default `24.0`
   - `font_size_heading_1: f32` — default `36.0`
   - `font_size_heading_2: f32` — default `30.0`
   - `font_size_heading_3: f32` — default `24.0`
   - `font_size_heading_4: f32` — default `20.0`
   - Update both `AKAR_THEME_DARK` and `AKAR_THEME_LIGHT` to include these fields with the defaults above.
   - Also add a semantic muted-content token `muted_content: u32` rather than hardcoding secondary paragraph colors.
     - `AKAR_THEME_DARK.muted_content = 0xa1a1aaff` (zinc-400-ish)
     - `AKAR_THEME_LIGHT.muted_content = 0x71717aff` (zinc-500-ish)
   - Add it to the struct, both presets, and any test that constructs the full theme.

3. Add an internal resolved-text-style type and three-layer resolver.
   - In `crates/akar-components/src/text_style.rs` (same file), add a `pub(crate) struct ResolvedTextStyle` with concrete (non-Option) fields: `font_size: f32`, `line_height: f32`, `color: u32`, `font_weight: FontWeight`, `font_family: FontFamily`, `align: TextAlign`, `wrap: bool`.
   - Add a public-ish (pub(crate)) resolver function `resolve_text_style(theme: &AkarTheme, defaults: &ResolvedTextStyle, override_style: Option<&TextStyle>) -> ResolvedTextStyle`.
   - Resolution order: theme semantic default -> component or variant `defaults` -> per-instance `override_style` (Option layer). `defaults` is the middle layer that maps a component's chosen variants to a concrete baseline (e.g. HeadingLevel::H1 -> font_size_heading_1 + font_weight=Bold). Test the cascade order.
   - Also expose a small helper `resolved_to_attrs(rt: &ResolvedTextStyle) -> glyphon::Attrs` and `resolved_to_metrics(rt: &ResolvedTextStyle) -> glyphon::Metrics` so callers (Task 3) can shape text from a ResolvedTextStyle without re-implementing the mapping. Place these in `text_style.rs`; they are the only place glyphon types appear and they are `pub(crate)`.

4. Map akar font types to glyphon only inside the implementation.
   - The mapping helpers above (`resolved_to_attrs`, `resolved_to_metrics`) are the boundary. Outside `text_style.rs`, no `glyphon::Family` or `glyphon::Attrs` references should appear in the new code.
   - Acceptable mapping:
     - SansSerif -> `glyphon::Family::SansSerif`
     - Serif -> `glyphon::Family::Serif`
     - Monospace -> `glyphon::Family::Monospace`
     - Normal -> weight 400 (glyphon::Attrs::new().weight(400))
     - Medium -> weight 500
     - Semibold -> weight 600
     - Bold -> weight 700

5. Export the akar-owned public types from `lib.rs` (covered by step 1). They are already exported via `pub use text_style::{...}` and `pub mod text_style;`.

6. Add tests for theme defaults, component defaults, partial overrides, full overrides, alignment, wrapping, and font mapping.
   - File: `crates/akar-components/src/text_style.rs` `#[cfg(test)] mod tests`. Cover at minimum:
     - Default theme resolves HeadingLevel::H1 default (font_size_heading_1, Bold, Start).
     - Partial TextStyle override only changes the listed fields; others remain at default.
     - Full TextStyle override replaces every field.
     - Wrap=true propagates to ResolvedTextStyle.wrap.
     - Center alignment maps to TextAlign::Center.
     - `resolved_to_attrs` maps FontFamily and FontWeight to expected glyphon values.
     - Theme accessor returns the correct heading sizes (just sanity).

**Constraints (from AGENTS.md / DEVELOP.md):**
- No emojis in source or docs.
- No comments unless WHY is non-obvious.
- No new unsafe code; no new dependencies.
- Do not edit `akar.h` (cbindgen-generated); Task 8 will handle C ABI.
- Glyphon types must not leak past `pub(crate)` in text_style.rs.
- Do not change unrelated components in this task. Touch only `theme.rs`, `lib.rs`, the new `text_style.rs`, and `Cargo.toml` if needed (you should not need to touch Cargo.toml).
- Do NOT modify the Epic file's status — I will do that after review.
- Do NOT commit your changes — I will commit.

**Validation:**
- Run `cargo test --workspace --no-fail-fast` and confirm zero failures.
- Run `cargo fmt --check` and confirm clean.
- Run `cargo clippy --workspace --all-targets` (note: WITHOUT -D warnings; the repo has pre-existing clippy noise unrelated to this task) and confirm you introduce NO new warnings.

**Deliverables back to me:**
- Files changed.
- Public API surface added (signatures only).
- Test names added.
- Confirmation that the four builds above are clean.

Stay tightly scoped to Task 2; do not start Task 3 or later.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
`criteriaSatisfied[].status` must be exactly one of: satisfied, not-satisfied, not-applicable.
`commandsRun[].result` must be exactly one of: passed, failed, not-run.
`manualNotes` and `notes` are optional strings; an empty string means no note and does not satisfy `manual-notes` evidence.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```