# Task for worker

## Task 3 — Heading and Paragraph Components

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md (read them for architecture, conventions, and component contract)

### What to implement

Add `heading` and `paragraph` components to `crates/akar-components/src/`.

#### Heading

- Add `HeadingLevel` enum with variants `H1`, `H2`, `H3`, `H4`.
- Add `heading(core, layout, node_id, text, level, overrides: Option<TextStyle>, theme)` function.
- Defaults per level:
  - H1: `font_size_heading_1`, `FontWeight::Bold`
  - H2: `font_size_heading_2`, `FontWeight::Bold`
  - H3: `font_size_heading_3`, `FontWeight::Bold`
  - H4: `font_size_heading_4`, `FontWeight::Semibold`
- Font family defaults to `SansSerif`. Color defaults to `theme.base_content`.
- `overrides: Option<TextStyle>` allows partial style overrides (use the existing `resolve_text_style` from `text_style.rs`).
- Must participate in intrinsic text measurement: create a text buffer via `core.text_pipeline.set_text(...)` with the resolved metrics and available width from `rect[2]`, then push a `TextCall`.
- Zero-area guard: return early if `rect[2] == 0.0 || rect[3] == 0.0`.
- Use `layout.widget_id(node_id)` as the buffer key.

#### Paragraph

- Add `paragraph(core, layout, node_id, text, overrides: Option<TextStyle>, theme)` function.
- Default style: `font_size_base`, `FontWeight::Normal`, line height = `font_size * 1.5`, wrapping enabled.
- Color defaults to `theme.base_content`. Use `theme.muted_content` if an `overrides` color is not set. Actually, looking at the epic, paragraphs should use `base_content` by default — but the epic also says "Add any required semantic muted-content token" which already exists in theme as `muted_content`. For the paragraph component, default to `theme.base_content` for now — the caller can override.
- Wrapping: pass `Some(rect[2])` as max width to `set_text`, and ensure the text pipeline wraps.
- Must participate in intrinsic text measurement (same pattern as heading).
- Zero-area guard.
- Use `layout.widget_id(node_id)` as the buffer key.

### Files to read first

1. `crates/akar-components/src/text_style.rs` — TextStyle, FontFamily, FontWeight, TextAlign, resolve_text_style, resolved_to_attrs, resolved_to_metrics, ResolvedTextStyle
2. `crates/akar-components/src/label.rs` — simplest existing text component pattern
3. `crates/akar-components/src/lib.rs` — module declarations and public exports
4. `crates/akar-components/src/theme.rs` — AkarTheme with font_size_heading_1..4, font_size_base, base_content, muted_content
5. `crates/akar-components/src/stat.rs` — example of multi-text rendering with different sizes
6. `crates/akar-core/src/lib.rs` — TextCall, QuadCall, AkarCore, TextPipeline::set_text signature
7. `crates/akar-layout/src/lib.rs` — Layout, NodeId, AkarNodeContext, default_measure_fn, compute_with_text

### Implementation pattern

Follow the `label` component pattern: take `&mut AkarCore`, `&Layout`, `NodeId`, text, style params, `&AkarTheme`. The function is purely paint-time (no layout mutation). Use the existing `resolve_text_style` to merge component defaults with overrides. Use `resolved_to_attrs` and `resolved_to_metrics` to create the glyphon buffer. Use `resolved_to_attrs` for the `Attrs` passed to `set_text`.

For text measurement to work with Taffy, the text buffer must be created BEFORE `Layout::compute` is called, and the node must have an `AkarNodeContext` with the correct `text_buffer_id`. The heading/paragraph components are paint-time only, so the buffer creation happens during paint. The caller is responsible for setting up `AkarNodeContext` on the node if they want intrinsic measurement. The component just needs to use the same buffer ID.

Actually, looking more carefully at the measurement system: `set_text` returns a `buffer_id`, and that buffer_id must be set on the node's `AkarNodeContext` BEFORE `compute_with_text` runs. Since heading/paragraph are paint-time only, they should NOT set the node context — that's the caller's job. But they DO create the buffer during paint.

Wait — looking at the label component, it just calls `set_text` during paint and pushes a `TextCall`. There's no explicit measurement integration in the paint function. The measurement integration would happen at the call site (the application sets up the node context with the buffer ID before compute).

For now, follow the label pattern exactly: create the text buffer during paint, push the text call. The intrinsic measurement integration will be wired up when these components are used in the webpage sample (Task 10) where the caller sets up `AkarNodeContext` before compute.

### Exports

In `lib.rs`, add:
```rust
pub mod heading;
pub use heading::{heading as akar_heading, HeadingLevel};

pub mod paragraph;
pub use paragraph::{paragraph as akar_paragraph};
```

### Tests

Add tests for both components:

**heading tests:**
- `zero_area_does_not_push_text` — node with zero size, assert `draw_list.len() == 0`
- `heading_h1_renders_text` — sized node, assert draw list has at least 1 text call
- `heading_all_levels_render` — verify each level produces output

**paragraph tests:**
- `zero_area_does_not_push_text`
- `paragraph_renders_text` — sized node, assert text call present
- `paragraph_with_long_text` — longer string, assert it renders

Follow the existing test patterns (use `AkarCore::mock()`, `Layout::new()`, `akar_layout::Style` with explicit size).

### Coding conventions (from DEVELOP.md)
- Edition 2021
- No emojis
- No comments unless WHY is non-obvious
- Self-documenting code
- Use `thiserror` for library crates
- No unsafe outside akar-c-api

After implementing, run `cargo test --workspace` and `cargo clippy --workspace -- -D warnings` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/0a30ea67/inline
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

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