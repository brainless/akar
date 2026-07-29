# Task for worker

## Task 6 — Navbar Lifecycle Correction

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md (read for architecture, conventions, component contract)

### Current state

The existing `navbar` in `crates/akar-components/src/navbar.rs` currently:
1. Mutates the root node's style (sets display, flex_direction, align_items, size)
2. Creates child slot nodes (start, center, end) every call
3. Draws a background container via `container()`
4. Returns `NavbarSlots`

This violates the construct/paint lifecycle: construction should be separate from painting, and paint must not mutate layout.

### What to implement

Split navbar into `navbar_layout` (construction) and `navbar` (paint), following the same pattern as `card_layout`/`card`.

#### Types to add

```rust
pub struct NavbarStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
}
```

#### Construction: `navbar_layout`

```rust
pub fn navbar_layout(layout: &mut Layout, node_id: NodeId, theme: &AkarTheme) -> NavbarSlots
```

- Creates start, center, end child nodes with appropriate flex styles.
- Sets children on root node.
- Configures root node as flex row with center alignment.
- Does NOT draw anything.
- Returns `NavbarSlots { start, center, end }`.

#### Paint: `navbar`

```rust
pub fn navbar(core: &mut AkarCore, layout: &Layout, node_id: NodeId, style: &NavbarStyle)
```

- Gets rect from layout.
- Zero-area guard.
- Draws background quad with border, corner radii.
- Does NOT mutate layout, does NOT create child nodes.

### Key constraints from the epic

- Preserve caller-owned root size, padding, gap, min/max constraints, and placement.
- Keep stable start, center, and end slots.
- Page-specific height belongs to the caller's Taffy layout.
- Retain or provide a clear migration for the existing API.

### Migration strategy

Keep the old `navbar` function as a convenience that does both construction and paint in one call (for backward compatibility), but mark it as the "combined" entry point. The new split API (`navbar_layout` + `navbar`) is the preferred path.

Actually, looking at the epic more carefully: "Retain or provide a clear migration for the existing API." The simplest approach is to rename the current function and provide the new split API. But since the existing `navbar` is already used in demo-rust and the webpage sample, we should keep backward compatibility.

Best approach: keep the existing `navbar(core, layout, node_id, theme)` function signature but refactor it to use the new internal structure. Add `navbar_layout` and `navbar` (paint-only) as the new preferred API. The old `navbar` becomes a combined convenience that calls both.

### Files to read first

1. `crates/akar-components/src/navbar.rs` — current implementation
2. `crates/akar-components/src/card.rs` — reference for the construct/paint split pattern
3. `crates/akar-components/src/container.rs` — how container draws
4. `crates/akar-components/src/box_style.rs` — BoxStyle
5. `crates/akar-components/src/lib.rs` — module declarations
6. `crates/akar-components/src/theme.rs` — AkarTheme tokens
7. `crates/akar-core/src/lib.rs` — QuadCall, AkarCore

### Exports

In `lib.rs`, update:
```rust
pub mod navbar;
pub use navbar::{
    navbar as akar_navbar, navbar_layout as akar_navbar_layout,
    NavbarSlots, NavbarStyle,
};
```

### Tests

- `navbar_layout_creates_three_slots` — start, center, end are distinct
- `navbar_layout_does_not_draw` — no draw calls after construction
- `navbar_paint_draws_background` — sized node, assert quad present
- `zero_area_does_nothing` — zero-size node, no draw calls
- `navbar_preserves_caller_layout` — caller sets root style, paint doesn't overwrite it

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments

After implementing, run `cargo test -p akar-components` and `cargo fmt` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/3793e45f/inline
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

Review gate: required by reviewer.

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
    },
    {
      "id": "criterion-2",
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