# Task for worker

## Task 7 — Styled Button, Badge, Separator, Stat, and Tab Bar APIs

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md

### What to implement

Add styled API variants for button, badge, separator, stat, and tab bar. Also add a shared color-contrast helper.

### 1. Color contrast helper (`color.rs`)

Add a `contrast_color(bg: u32) -> u32` function that returns either black or white text depending on the background luminance. Use the WCAG relative luminance formula:

```rust
pub fn contrast_color(bg: u32) -> u32 {
    let r = ((bg >> 24) & 0xFF) as f32 / 255.0;
    let g = ((bg >> 16) & 0xFF) as f32 / 255.0;
    let b = ((bg >> 8) & 0xFF) as f32 / 255.0;
    let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
    if luminance > 0.5 { 0x000000FF } else { 0xFFFFFFFF }
}
```

Add a test for this.

### 2. Button styled API (`button.rs`)

Add `ButtonStyle`:
```rust
pub struct ButtonStyle {
    pub fill: Option<u32>,
    pub hover_fill: Option<u32>,
    pub pressed_fill: Option<u32>,
    pub border_color: Option<u32>,
    pub content_color: Option<u32>,
    pub text_style: Option<TextStyle>,
}
```

Add `akar_button_styled(core, layout, node_id, label, variant, style: &ButtonStyle, theme) -> ButtonResult`.

The styled variant:
- Uses `style.fill` if set, otherwise falls back to the current variant-based fill logic.
- Uses `style.hover_fill` / `style.pressed_fill` for hover/pressed states when set.
- Uses `style.border_color` if set.
- Uses `style.content_color` if set, otherwise falls back to variant-based text color.
- Applies `style.text_style` to the text if set.

Keep the existing `akar_button` function unchanged.

### 3. Badge styled API (`badge.rs`)

Add `BadgeStyle`:
```rust
pub struct BadgeStyle {
    pub fill: Option<u32>,
    pub border_color: Option<u32>,
    pub content_color: Option<u32>,
    pub text_style: Option<TextStyle>,
}
```

Add `akar_badge_styled(core, layout, node_id, text, variant, style: &BadgeStyle, theme)`.

The styled variant:
- Uses `style.fill` if set, otherwise variant-based fill.
- Uses `style.content_color` if set, otherwise variant-based content color.
- Applies `style.text_style` to the text.

Keep the existing `akar_badge` unchanged.

### 4. Separator style (`separator.rs`)

Read the existing separator first. Add `SeparatorStyle`:
```rust
pub struct SeparatorStyle {
    pub color: Option<u32>,
    pub thickness: Option<f32>,
}
```

Add `akar_separator_styled(core, layout, node_id, style: &SeparatorStyle, theme)`.

The styled variant uses `style.color` if set (otherwise `theme.base_300`), and `style.thickness` if set (otherwise 1.0).

Keep the existing `akar_separator` unchanged.

### 5. Stat text style (`stat.rs`)

Read the existing stat. The stat already accepts theme colors. For the styled API, we need to allow overriding the text colors.

Add `StatStyle`:
```rust
pub struct StatStyle {
    pub title_color: Option<u32>,
    pub value_color: Option<u32>,
    pub description_color: Option<u32>,
    pub title_text_style: Option<TextStyle>,
    pub value_text_style: Option<TextStyle>,
    pub description_text_style: Option<TextStyle>,
}
```

Add `akar_stat_styled(core, layout, node_id, title, value, description, style: &StatStyle, theme)`.

Keep the existing `akar_stat` unchanged.

### 6. Tab bar style (`tabs.rs`)

Read the existing tab_bar. Add minimal style options:
```rust
pub struct TabBarStyle {
    pub active_color: Option<u32>,
    pub inactive_color: Option<u32>,
    pub indicator_color: Option<u32>,
}
```

Add `akar_tab_bar_styled(...)` that accepts this style. Keep the existing `akar_tab_bar` unchanged.

### Files to read first

1. `crates/akar-components/src/button.rs` — current button implementation
2. `crates/akar-components/src/badge.rs` — current badge implementation
3. `crates/akar-components/src/separator.rs` — current separator
4. `crates/akar-components/src/stat.rs` — current stat
5. `crates/akar-components/src/tabs.rs` — current tab_bar
6. `crates/akar-components/src/color.rs` — existing color utilities
7. `crates/akar-components/src/lib.rs` — module declarations
8. `crates/akar-components/src/text_style.rs` — TextStyle

### Exports

Update `lib.rs` to export all new types and functions.

### Tests

For each styled API:
- `styled_uses_custom_fill` — verify custom fill is rendered
- `styled_preserves_zero_area` — zero-area guard works
- Common-case API still works (existing tests cover this)

For contrast_color:
- `contrast_color_dark_bg_returns_white`
- `contrast_color_light_bg_returns_black`

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments

After implementing, run `cargo test -p akar-components` and `cargo fmt` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/5629814f/inline
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