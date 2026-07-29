# Task for worker

## Task 5 — Card Lifecycle and Composition

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md (read for architecture, conventions, component contract)

### What to implement

Add a composed `card` component with construct/paint lifecycle to `crates/akar-components/src/card.rs`.

The card is a composed component with body and optional header/footer slots. Construction creates stable layout nodes; paint draws background, border, radius, shadow, and separators without mutating layout.

#### Types

```rust
pub struct CardSlots {
    pub header: Option<NodeId>,
    pub body: NodeId,
    pub footer: Option<NodeId>,
}

pub struct CardLayout {
    pub direction: FlexDirection,  // default Column
    pub gap: f32,                  // default theme.spacing_lg or similar
    pub padding: f32,              // default theme.padding_x
    pub has_header: bool,
    pub has_footer: bool,
}

pub struct CardStyle {
    pub background: u32,
    pub border_color: u32,
    pub border_width: f32,
    pub corner_radii: [f32; 4],
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub shadow_color: u32,
    pub shadow_offset: [f32; 2],
    pub separator_color: u32,
}
```

#### Construction: `card_layout`

```rust
pub fn card_layout(layout: &mut Layout, node_id: NodeId, options: &CardLayout) -> CardSlots
```

- Sets the root node's display to Flex, direction from options, gap, padding.
- Creates body node (always present).
- Creates header node only if `options.has_header`.
- Creates footer node only if `options.has_footer`.
- Sets children on root node in order: [header?, body, footer?].
- Returns `CardSlots` with the node IDs.
- The caller adds content to the returned slot nodes.

#### Paint: `card`

```rust
pub fn card(core: &mut AkarCore, layout: &Layout, node_id: NodeId, slots: &CardSlots, style: &CardStyle)
```

- Gets the resolved rect from layout.
- Zero-area guard.
- Draws background quad with border, corner radii, shadow.
- Draws separators between populated regions (header-body, body-footer) as 1px quads.
- Does NOT mutate layout — read-only on `Layout`.

#### Default style

Add a `CardStyle::default(theme: &AkarTheme) -> CardStyle` that uses:
- `background: theme.base_100`
- `border_color: theme.base_300`
- `border_width: theme.border_width`
- `corner_radii: [theme.radius_box; 4]`
- `shadow_blur: 0.0` (no shadow by default)
- `separator_color: theme.base_300`

Also add `BoxStyle::card()` if it doesn't already exist — check `box_style.rs`.

### Files to read first

1. `crates/akar-components/src/box_style.rs` — BoxStyle, BoxShadow, existing card/panel helpers
2. `crates/akar-components/src/container.rs` — how container draws background
3. `crates/akar-components/src/navbar.rs` — example of construct pattern with slots
4. `crates/akar-components/src/separator.rs` — separator drawing
5. `crates/akar-components/src/lib.rs` — module declarations
6. `crates/akar-components/src/theme.rs` — AkarTheme tokens
7. `crates/akar-core/src/lib.rs` — QuadCall, AkarCore
8. `crates/akar-layout/src/lib.rs` — Layout, NodeId, Style, Display, FlexDirection

### Key constraints from the epic

- Card is a composed component, not a primitive layout container.
- Empty optional slots are not created.
- Separators are painted only between populated regions.
- Paint functions never add children, replace children, or overwrite caller-owned layout properties.
- Stable caller-owned NodeIds survive every frame.

### Exports

In `lib.rs`:
```rust
pub mod card;
pub use card::{card as akar_card, card_layout as akar_card_layout, CardLayout, CardSlots, CardStyle};
```

### Tests

- `card_layout_creates_body_only` — has_header=false, has_footer=false → body only
- `card_layout_creates_header_body_footer` — all three slots present
- `card_layout_returns_distinct_slot_ids` — header, body, footer are different NodeIds
- `zero_area_does_nothing` — zero-size node, no draw calls
- `card_renders_background` — sized node with slots, assert quad present
- `card_separator_between_header_body` — when header+body present, separator drawn

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments, self-documenting code

After implementing, run `cargo test -p akar-components` and `cargo fmt` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/2df8720f/inline
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