# Task for worker

You are a delegated subagent running from a fork of the parent session. Treat the inherited conversation as reference-only context, not a live thread to continue. Do not continue or answer prior messages as if they are waiting for a reply. Your sole job is to execute the task below and return a focused result for that task using your tools.

Task:
Implement Task 3 of Epic 020 in /Users/brainless/Projects/akar/.

**Read first** (do not skip):
- /Users/brainless/Projects/akar/epics/020-component-webpage-sample.md (full file, focus on "Task 3 — Heading and paragraph components")
- /Users/brainless/Projects/akar/DEVELOP.md
- /Users/brainless/Projects/akar/AGENTS.md
- /Users/brainless/Projects/akar/crates/akar-components/src/lib.rs (existing exports pattern)
- /Users/brainless/Projects/akar/crates/akar-components/src/text_style.rs (FontFamily/FontWeight/TextStyle/ResolvedTextStyle/resolve_text_style from Task 2 — read this fully; you must use its resolver)
- /Users/brainless/Projects/akar/crates/akar-components/src/label.rs (simple existing component — model after it)
- /Users/brainless/Projects/akar/crates/akar-core/src/text_pipeline.rs (TextPipeline::set_text signature; TextMeasureInput/Result; measure_with_metadata)
- /Users/brainless/Projects/akar/crates/akar-layout/src/lib.rs (Layout::compute_with_text signature and AkarNodeContext from Task 1)

**Task 3 deliverables (from the Epic, exactly):**

1. Add `HeadingLevel::{H1, H2, H3, H4}`.
   - File: new file `crates/akar-components/src/heading.rs`. Define `#[repr(C)] #[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum HeadingLevel { H1, H2, H3, H4 }`.

2. Add `heading` with level defaults and `TextStyle` overrides.
   - Signature (Rust, pub): `pub fn heading(core: &mut AkarCore, layout: &Layout, node_id: NodeId, level: HeadingLevel, text: &str, style: Option<&TextStyle>, theme: &AkarTheme)`.
   - Defaults per level (use Task 2's resolver):
     - H1 -> `font_size_heading_1`, `FontWeight::Bold`, color = theme.base_content
     - H2 -> `font_size_heading_2`, `FontWeight::Bold`, color = theme.base_content
     - H3 -> `font_size_heading_3`, `FontWeight::Bold`, color = theme.base_content
     - H4 -> `font_size_heading_4`, `FontWeight::Semibold`, color = theme.base_content
   - Build the per-component default ResolvedTextStyle using theme values, then call `resolve_text_style(theme, &defaults, style)` to layer overrides.
   - Use `core.text_pipeline.set_text(Some(layout.widget_id(node_id)), text, resolved_to_metrics(&rt), wrap_width, None, Some(resolved_to_attrs(&rt)))` to shape. `wrap_width` should be `Some(rect[2])` when the layout gave a non-zero width; otherwise None (intrinsic).
   - Push `TextCall { buffer_id, x: rect[0], y: rect[1], clip: rect, color: color_to_f32(rt.color), z: 0.0 }`.
   - Do NOT push any quad; heading is text-only.
   - Zero-area guard: if `rect[2] == 0.0 || rect[3] == 0.0`, return immediately (no draw call, no text shaping — same pattern as label.rs).
   - Implementation must participate in intrinsic text measurement: do not push the TextCall if the layout gave zero area. The actual Taffy measurement for heading is set up by callers via Task 1's `AkarNodeContext::text(buffer_id)` + `Layout::compute_with_text`. Heading itself does not configure measurement — it is purely a paint function. Add a doc comment (`///`) on `heading` explaining that the caller must use `Layout::compute_with_text` for wrapped headings to size correctly.

3. Default H1-H3 to bold and H4 to semibold.
   - Already covered in step 2 defaults.

4. Add `paragraph` with wrapping enabled and a default line height of `font_size * 1.5`.
   - File: same `crates/akar-components/src/heading.rs` or new `crates/akar-components/src/paragraph.rs` — keep heading and paragraph in separate files (`heading.rs`, `paragraph.rs`) for clean module boundaries; export from `lib.rs`.
   - Signature: `pub fn paragraph(core: &mut AkarCore, layout: &Layout, node_id: NodeId, text: &str, style: Option<&TextStyle>, theme: &AkarTheme)`.
   - Defaults: `font_size = theme.font_size_base`, `line_height = theme.font_size_base * 1.5`, `wrap = true`, color = `theme.muted_content` (use the new semantic token from Task 2), `font_weight = FontWeight::Normal`, `font_family = FontFamily::SansSerif`, `align = TextAlign::Start`.
   - Apply overrides via `resolve_text_style` and shape with `wrap_width = Some(rect[2])` for wrap, falling back to None on zero width.
   - Zero-area guard identical to heading.

5. Make both components participate in intrinsic text measurement.
   - Document this in doc comments (///) on both functions. The component paint functions themselves do not configure measurement — they read the resolved rect and shape the buffer. The CALLER (e.g. the Akar page in Task 10) is responsible for creating the layout node with `Layout::new_leaf_with_context(Style::default(), AkarNodeContext::text(buffer_id))` and using `Layout::compute_with_text(root, (w, h), &mut core.text_pipeline)` so intrinsic text size flows into Taffy.
   - This split (paint-only) is what Task 1 enabled.

6. Keep explicit node width/height constraints authoritative.
   - When `rect[2]` is finite and > 0, `wrap_width = Some(rect[2])`; otherwise None. Do not fight the layout.

7. Add zero-area guards.
   - Done in steps 2 and 4.

8. Export as `akar_heading` and `akar_paragraph`.
   - In `crates/akar-components/src/lib.rs`, add `pub mod heading; pub mod paragraph;` and `pub use heading::{heading as akar_heading, HeadingLevel};` and `pub use paragraph::paragraph as akar_paragraph;`.

9. Add tests for every heading level, style resolution, wrapping, measured multi-line height, explicit newlines, alignment, and zero width/height.
   - File `crates/akar-components/src/heading.rs` and `crates/akar-components/src/paragraph.rs` `#[cfg(test)] mod tests`.
   - Use `AkarCore::mock()` to get a core without a GPU adapter (it already exists and works in label.rs tests; if it requires a real adapter, fall back to constructing a `TextPipeline` directly with a mock approach — look at `crates/akar-components/src/label.rs` tests as the pattern).
   - Tests at minimum:
     - `heading_h1_uses_heading_1_size_and_bold` — verify the buffer is shaped with the right metrics (introspect `core.text_pipeline` if a getter exists, or assert that the resolved style yields font_size_heading_1 and Bold via `resolve_text_style` directly).
     - `heading_h4_uses_semibold` — assert H4 default weight is Semibold.
     - `heading_zero_area_does_not_push_text` — zero-area node results in no draw_list entries.
     - `heading_override_color_and_size` — TextStyle { color: Some(0xff0000ff), font_size: Some(64.0) } produces a resolved style with those values.
     - `paragraph_default_wraps` — ResolvedTextStyle.wrap is true by default.
     - `paragraph_zero_area_does_not_push_text` — zero-area node results in no draw_list entries.
     - `paragraph_override_align_center` — TextStyle { align: Some(TextAlign::Center) } yields ResolvedTextStyle.align = TextAlign::Center.
     - `paragraph_line_height_is_one_point_five_times_font_size` — defaults yield line_height = 1.5 * font_size.

**Constraints (from AGENTS.md / DEVELOP.md):**
- No emojis.
- No comments unless WHY is non-obvious (doc comments `///` for public APIs are fine and encouraged).
- No new unsafe code; no new dependencies.
- Do not edit `akar.h`.
- Use Task 2's resolver, not a custom one.
- Do NOT modify the Epic file's status — I will do that after review.
- Do NOT commit your changes — I will commit.
- Touch only `crates/akar-components/src/heading.rs` (new), `crates/akar-components/src/paragraph.rs` (new), `crates/akar-components/src/lib.rs`, and `crates/akar-components/src/text_style.rs` if a small helper is genuinely missing. Do not touch unrelated components or crate Cargo.toml.

**Validation:**
- `cargo test --workspace --no-fail-fast` — zero failures.
- `cargo fmt --check` — clean.
- `cargo clippy --workspace --all-targets` — no NEW warnings from your changes (pre-existing ones are fine).

**Deliverables back to me:**
- Files changed.
- Public API surface added (signatures only).
- Test names added.
- Confirmation of clean test/fmt/clippy runs.

Stay tightly scoped to Task 3; do not start Task 4 or later.

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