# Task for worker

## Task 10 — Akar Marketing Page

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md

### What to implement

Replace the stub in `examples/webpage-rust/src/sites/akar.rs` with a full Akar marketing page using only akar component APIs. Raw `push_quad` and `push_text` are forbidden in the Akar site module.

### Page structure (from the epic)

1. **Navbar** — logo in start slot, Features/Components/GitHub links in end slot
2. **Hero** — centered H1, wrapped subtitle, Solid/Outline CTAs. Large serif H1 override.
3. **Stats** — three stat components (component count, C ABI, immediate mode)
4. **Feature cards** — three composed cards with headings and paragraphs
5. **Why akar** — H2, body copy, numbered H4/paragraph items
6. **Component showcase** — badge variants, custom badge, button variants, interactive tab bar
7. **Footer** — separator, multi-column heading/link composition, copyright label

### Files to read first

1. `examples/webpage-rust/src/sites/akar.rs` — current stub
2. `examples/webpage-rust/src/sites/mimo.rs` — reference for how a full site is implemented (scrolling, layout, rendering)
3. `examples/webpage-rust/src/app.rs` — AppState, rendering loop, how scroll is handled
4. `crates/akar-components/src/lib.rs` — all available component APIs
5. `crates/akar-components/src/heading.rs` — HeadingLevel, akar_heading
6. `crates/akar-components/src/paragraph.rs` — akar_paragraph
7. `crates/akar-components/src/link.rs` — akar_link
8. `crates/akar-components/src/card.rs` — card_layout, akar_card, CardSlots, CardLayout, CardStyle
9. `crates/akar-components/src/navbar.rs` — navbar_layout, akar_navbar, NavbarSlots, NavbarStyle
10. `crates/akar-components/src/button.rs` — akar_button, akar_button_styled, ButtonVariant
11. `crates/akar-components/src/badge.rs` — akar_badge, BadgeVariant
12. `crates/akar-components/src/separator.rs` — akar_separator
13. `crates/akar-components/src/stat.rs` — akar_stat
14. `crates/akar-components/src/tabs.rs` — akar_tab_bar
15. `crates/akar-components/src/theme.rs` — AKAR_THEME_DARK, AkarTheme tokens

### Implementation approach

The AkarSite struct needs to hold all stable node IDs and interaction state:

```rust
pub struct AkarSite {
    root: NodeId,
    scroll_y: f32,
    // Navbar slots
    navbar_root: NodeId,
    navbar_start: NodeId,
    navbar_end: NodeId,
    // Hero
    hero_root: NodeId,
    h1_node: NodeId,
    subtitle_node: NodeId,
    cta_solid: NodeId,
    cta_outline: NodeId,
    // Stats
    stats_root: NodeId,
    stat1: NodeId,
    stat2: NodeId,
    stat3: NodeId,
    // Feature cards
    cards_root: NodeId,
    card1: NodeId,
    card2: NodeId,
    card3: NodeId,
    // Why akar
    why_root: NodeId,
    // Showcase
    showcase_root: NodeId,
    active_tab: usize,
    // Footer
    footer_root: NodeId,
}
```

### Layout approach

Use a single scrollable content layout. The root node is a flex column that contains all sections. Use Taffy flex layout for vertical stacking.

For scrolling: wrap everything in a scroll area. The `render` method should translate all child rects by `-scroll_y`. Use `akar_components::scroll_area_begin`/`scroll_area_end` if appropriate, or implement manual scroll transform.

Actually, looking at how mimo.rs handles scrolling — it uses a manual scroll transform in the render method. The layout is built at full height, and rendering translates Y coordinates by `-scroll_y`. Follow the same pattern.

### Key constraints from the epic

- **No raw `push_quad` or `push_text`** — only component functions.
- Use `akar_heading`, `akar_paragraph`, `akar_link`, `akar_card`, `akar_navbar`, `akar_button`, `akar_badge`, `akar_separator`, `akar_stat`, `akar_tab_bar`.
- Hero H1 should use a large serif override: `TextStyle { font_size: Some(48.0), font_family: Some(FontFamily::Serif), .. }`.
- Stats: "30+ components", "C ABI", "Immediate mode".
- Feature cards: 3 cards with headings and paragraphs about key features.
- Component showcase: demonstrate badge variants, button variants, and an interactive tab bar.
- Footer: separator, multi-column layout with links, copyright.

### Text content suggestions

**Hero:**
- H1: "akar" (with serif override)
- Subtitle: "A GPU-accelerated, language-neutral UI component library built on wgpu and glyphon."

**Stats:**
- "30+" / "Components"
- "C ABI" / "Language Neutral"
- "Immediate Mode" / "No Framework Opinions"

**Feature cards:**
- "Cross-Platform GPU Rendering" — built on wgpu for native performance
- "Language Neutral C ABI" — use from any language that calls C
- "Composable Components" — buttons, cards, inputs, tables styled out of the box

**Why akar:**
- "Built by agents, debuggable by agents"
- "Batteries-included component catalog"
- "Virtualization first"
- "Canvas LOD with component portals"

**Showcase:**
- Badge variants row
- One custom-styled badge
- Button variant row (Solid, Outline, Ghost)
- Tab bar with 3 tabs

**Footer:**
- Separator
- Multi-column: "Product" (Features, Components, Demo), "Resources" (GitHub, Documentation), "Community" (MIT License)
- Copyright: "akar - MIT License"

### Scroll handling

Follow the mimo.rs pattern:
1. `build_layout` creates all nodes at full content height.
2. `render` applies `-scroll_y` transform to all rendered positions.
3. Input handling in `app.rs` already handles scroll events and updates `scroll_y`.

### Interaction state

- `active_tab: usize` — which tab is selected in the showcase
- Tab bar interaction: `akar_tab_bar` returns `TabBarResponse` with the clicked tab index

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments

After implementing, run `cargo check --bin webpage-rust` and `cargo fmt` to verify.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/4661cc57/inline
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