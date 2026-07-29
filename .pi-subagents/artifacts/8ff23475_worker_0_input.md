# Task for worker

## Task 4 — Link Component

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md (read for architecture, conventions, component contract)

### What to implement

Add a `link` component to `crates/akar-components/src/link.rs`.

#### Link

- Add `LinkResult { clicked: bool, hovered: bool, pressed: bool }`.
- Add `link(core, layout, node_id, text, overrides: Option<TextStyle>, theme) -> LinkResult`.
- Default style: primary color (`theme.primary`) for text, `FontWeight::Normal`, `font_size_base`.
- Wrapping disabled by default (no wrap).
- Zero-area guard: return `LinkResult { false, false, false }` if `rect[2] == 0.0 || rect[3] == 0.0`.
- Hit testing: check `core.input.is_hovering(rect)`, `core.input.is_pressed(rect)`, `core.input.is_clicked(rect)`.
- Draw a hover underline when hovered: draw a thin quad (1-2px height) under the text. The underline width should be based on the measured text width — use the text buffer's measured width. However, since we don't have direct access to the measured width from `set_text`, use a reasonable approximation: measure the text by creating the buffer, then draw the underline quad across the full available width (`rect[2]`) as a reasonable default, OR simply draw the underline to match the text content width. Looking at the epic: "Size and position the hover underline from measured glyph geometry rather than the full node width." The text pipeline's `set_text` returns a buffer_id, but we need the measured size. Check if `TextPipeline` has a method to get the measured size of a buffer after shaping. If not available, use the full rect width as the underline width for now.
- Use the foreground quad layer for the underline so it's visible with global text rendering.
- Text color: `theme.primary` by default, overridable via `TextStyle.color`.
- Font: resolved via `resolve_text_style` with link defaults.
- Draw text via `TextCall` (same pattern as heading/paragraph/label).

### Files to read first

1. `crates/akar-components/src/heading.rs` — recently added, good reference for the pattern
2. `crates/akar-components/src/paragraph.rs` — another reference
3. `crates/akar-components/src/label.rs` — simplest text component
4. `crates/akar-components/src/button.rs` — example of hit testing and quad drawing
5. `crates/akar-components/src/text_style.rs` — TextStyle, resolve_text_style, resolved_to_attrs, resolved_to_metrics
6. `crates/akar-components/src/lib.rs` — module declarations
7. `crates/akar-core/src/lib.rs` — AkarCore, TextCall, QuadCall, TextPipeline methods (check for any `measure` or `get_size` method on TextPipeline)
8. `crates/akar-components/src/theme.rs` — AkarTheme tokens

### Implementation details

The link should:
1. Resolve text style (defaults: primary color, normal weight, base size, no wrap)
2. Create text buffer via `set_text` with resolved metrics
3. Check hover/press/click state
4. If hovered, draw an underline quad beneath the text. The underline quad should be:
   - `rect[0]` x position (same as text)
   - `rect[1] + text_height - underline_offset` y position (below the text baseline area)
   - Use `rect[2]` as max width (full available width)
   - Height: 1-2px
   - Color: `theme.primary`
   - No border, no shadow
5. Push the text call
6. Return `LinkResult`

For the underline, a simple approach: draw a 2px-high quad at `y = rect[1] + metrics.font_size * 1.2 - 2.0` (approximately below the text baseline). Width can be the full `rect[2]` for simplicity — the text will only occupy part of it, but this avoids needing a separate measurement step.

Actually, looking more carefully at the epic requirement: "Size and position the hover underline from measured glyph geometry rather than the full node width." This means we should try to get the actual text width. Check if `TextPipeline` has a way to get the measured width after `set_text`. If `TextPipeline::measure` or similar exists, use it. Otherwise, use `rect[2]` as a practical fallback and note the limitation.

### Exports

In `lib.rs`, add:
```rust
pub mod link;
pub use link::{link as akar_link, LinkResult};
```

### Tests

- `zero_area_returns_all_false` — zero-size node
- `link_renders_text` — sized node, assert draw list has text
- `link_returns_hit_state` — sized node, verify LinkResult fields are accessible

Follow existing test patterns (`AkarCore::mock()`, `Layout::new()`, `akar_layout::Style` with explicit size).

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments, self-documenting code

After implementing, run `cargo test -p akar-components` and `cargo fmt` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/8ff23475/inline
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