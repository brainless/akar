# Epic 009: Static Display and Navigation Components

**Status:** Planned
**Goal:** Complete the Tier 1 component catalog — components that display data without requiring complex interaction or overlay infrastructure. Add the foundational navigation bar that anchors page-level layout.

**Prerequisite:** Epic 008 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean.

---

## Scope

### Components

#### Alert
An informational banner with a semantic variant (Info, Success, Warning, Error) and an optional dismissible close button. Renders as a full-width box with a colored left accent strip, an icon area (text-based icon or colored quad placeholder), and a message label.

Variants map to the same semantic colors as `BadgeVariant` (from Epic 008): `theme.info` / `theme.success` / `theme.warning` / `theme.error`.

#### Stat
A KPI display card: a title string, a large value string, and an optional description string below the value. No interaction. Typically placed inside a row of two to four stat containers.

#### Skeleton
A loading-state placeholder that renders a rounded rect in a muted color (`theme.base_300`) in the shape of the content it will replace (a line of text, a card, an avatar circle). No animation in v1 — a static placeholder. Animation (shimmer) is deferred.

#### Navbar
A horizontal bar — typically the top strip of a page layout — containing left-aligned, center-aligned, and right-aligned slots. Internally a flex row. The `Navbar` component renders the bar background (`BoxStyle::panel`) and resolves three child node regions: `start`, `center`, `end`. The caller populates each region with buttons, labels, or badges.

This is the first structural component that composes sub-regions. Its taffy tree is self-contained within the navbar node.

#### Steps
A horizontal progress indicator showing a sequence of named steps with a current index. Steps before the current index are rendered as complete (primary fill). The current step is highlighted. Steps after are rendered in muted color. No interaction — the active step is passed by the caller.

#### Avatar
A circular or rounded-square frame displaying initials text (when no image is available). In v1, image support is deferred — Avatar renders a colored circle with one or two initials centered inside. Color is derived deterministically from the initials string (hash → theme color slot).

---

## Key Design Decisions

**No new draw list infrastructure required.** All components in this epic are combinations of quads and text. Scissor, z-ordering, and overlay are not needed.

**Navbar slot model.** The `Navbar` takes a single `node_id` (the bar's taffy node) and internally creates three child nodes (`start`, `center`, `end`) as a flex row. It returns a `NavbarSlots { start: NodeId, center: NodeId, end: NodeId }` struct so the caller can populate each slot. This is the first example of a component that returns node IDs for the caller to use as layout parents — a pattern that Tabs (Epic 010) will reuse.

**Stat is a leaf component; no sub-layout.** The title, value, and description are rendered at fixed vertical offsets within the stat rect, not via a sub-taffy tree. If the caller needs flex alignment of multiple stats, they arrange the stat nodes themselves with `Layout::new_with_children`.

**Skeleton mimics shape, not content.** The caller creates a skeleton node in the layout tree with the same style as the real content node it replaces. Swapping `skeleton(core, layout, node)` for `label(core, layout, node, ...)` requires no layout change.

---

## C ABI

All components get `extern "C"` wrappers in `akar-c-api`. `NavbarSlots` is exposed as a repr(C) struct of three `u64` node IDs. `Steps` takes a `const char**` array for step labels.

---

## Demo

The demo gains a top navbar (with a title label, a badge count, and a button), a row of three stat cards below it, and a skeleton placeholder that toggles to real content when a button is clicked. The existing scrollable list from Epic 008 moves below these sections.

---

## Acceptance Criteria

- [ ] `Alert` renders with correct variant color; close button is optional and functional.
- [ ] `Stat` renders title, value, description at correct vertical positions within its rect.
- [ ] `Skeleton` renders a muted rounded rect; no animation.
- [ ] `Navbar` returns three slot NodeIds; caller can add children to each slot.
- [ ] `Steps` renders the correct number of steps with correct fill states.
- [ ] `Avatar` renders a circle with two initials; color is deterministic from the string.
- [ ] All components exposed in `akar.h`.
- [ ] `cargo clippy --workspace -- -D warnings` and `cargo test --workspace` pass clean.
