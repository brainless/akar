# Epic 027: Virtualized Data Grid MVP

**Status:** Done
**Goal:** Add a production-credible MVP data-grid component for applications whose primary UX is tabular data: fixed-height virtualized rows, horizontally virtualized caller-sized columns, a sticky header, two-axis scrolling, selection/navigation responses, and a full-screen example populated with deterministic fake data.

**Prerequisite:** Epic 017 (Reusable Data Items and Lists) and Epic 026 (Component Showcase and Variant Screenshots) are `Status: Done`.

---

## Problem Statement

akar can already render a fixed-height virtualized `data_list`, and `DEVELOP.md` describes how an application can combine `list_clip`, visibility checks, and draw-list culling to keep a large table proportional to its visible cells. It does not, however, provide a data-grid component. An application currently has to design column geometry, horizontal virtualization, header/body clipping, sticky headers, cell hit-testing, keyed text-buffer identity, keyboard navigation, selection feedback, and sorting affordances itself.

That is too much application-owned rendering infrastructure when a grid is the application's core UX. The MVP must be useful for database browsers, log viewers, admin tools, and spreadsheet-like read-heavy interfaces without turning akar into a data-model, query, or spreadsheet engine.

This epic deliberately builds on Epic 017's ownership boundary:

- The application owns records, stable row keys, loading, filtering, sorting, mutation, and persistence.
- akar owns grid geometry, virtualization, clipping, drawing, input hit-testing, focus/navigation state, and typed interaction responses.
- The Rust design must translate to a flat C ABI without closures, generic records, or caller-side heap allocation requirements.

## Current-State Findings

- `crates/akar-components/src/data_list.rs` virtualizes fixed-height rows vertically and owns only `scroll_y`; it has no columns, sticky header, horizontal scroll, or cell semantics.
- `crates/akar-components/src/scroll_area.rs` is also vertical-only, although `InputState::scroll_delta` already carries both `x` and `y`.
- `akar.h` exports `akar_data_list_begin/end` and `akar_data_item`, but no table or data-grid API.
- Taffy's CSS Grid support is a layout facility, not a virtualized data-grid widget. This epic must not equate the two or create one Taffy node per logical cell.
- No fake-data crate is currently declared in the workspace or any example manifest. `Cargo.lock` contains only transitive `fastrand`, which is not a semantic faker API. The full-screen example will add the `fake` crate as an example-only dependency. At planning time the current documented release is `fake` 5.1, which supports deterministic `fake_with_rng` generation and re-exports its compatible `rand`; re-check the resolved version when implementation starts and commit the resulting lockfile change.

---

## Architecture Decisions

### ADR-020: The Grid Is an Immediate-Mode View, Not a Data Store

**Decision:** The data grid never owns application rows or cell values. The caller supplies counts, stable row keys, column descriptors, and text for visible cells each frame.

**Consequences:**

- Sorting and filtering are requested through interaction responses; the application performs them and supplies the resulting row order on the next frame.
- Selection state remains caller-visible and serializable. akar may retain the focused/active cell needed for keyboard navigation, but must not own a hidden selected-record collection.
- The public API does not accept a Rust iterator, callback, closure, trait object, or generic record type.
- A C caller can drive the same begin/header/body/end lifecycle as a Rust caller.

### ADR-021: Virtualize Geometry, Do Not Build a Layout Node per Cell

**Decision:** The grid has one ordinary layout node for its viewport. Header and cell rectangles are derived arithmetically from that viewport, fixed row height, header height, scroll offsets, and caller-provided column widths.

**Consequences:**

- Row visibility is O(1) through the existing `list_clip` behavior, with a small configurable overscan.
- Column visibility is computed from cumulative caller-sized widths. The MVP may scan the visible boundary over tens or hundreds of columns, but must not iterate rows × columns outside the visible ranges. If the implementation stores prefix offsets, their lifetime and invalidation rules must be explicit and C-compatible.
- A grid with 100,000 rows and 100 columns creates no 10-million-node Taffy tree and performs no per-frame work proportional to 10 million cells.
- Zero rows, zero columns, zero-area viewports, columns wider than the viewport, and total content smaller than the viewport are valid inputs and must not panic.

### ADR-022: Fixed Rows, Caller-Sized Columns, Sticky Header

**Decision:** MVP rows have one fixed positive height. Each column has a stable `u64` key and a caller-supplied fixed width. The header is sticky vertically and scrolls horizontally with the body.

Invalid or non-finite dimensions must be normalized to documented safe values rather than propagating NaNs into hit-testing or scissor rectangles. Column resizing, reordering, pinning, and auto-fit are deferred, so a column's width does not change as a side effect of this component in the MVP.

The grid uses distinct header and body scissors. Body cells must never paint over the sticky header, and horizontally scrolled header labels must stay aligned with body cells.

### ADR-023: Stable Cell Identity Is `(Grid, Row Key, Column Key)`

**Decision:** Any stable widget/text-buffer identity for a cell is composed from the grid's namespaced widget ID, the caller-provided row key, and the column key—not the visible row/column slot.

**Consequences:**

- Scrolling or sorting cannot attach a shaped text buffer, focus, or edit state to the new record occupying a screen position.
- The key-composition helper must have deterministic unit tests for namespace separation, row separation, column separation, and frame-to-frame stability.
- The MVP's cells are display text, but identity is designed now so a later editable/custom-cell epic does not require an incompatible rewrite.

### ADR-024: Interaction Reports Intent; Application Policy Stays Outside

**Decision:** The component reports header clicks and active/selected cell changes. It does not sort records or impose single-row versus multi-row application policy.

MVP interaction includes:

- hover and pressed feedback for headers and body cells;
- click to activate a cell and report its stable row/column keys;
- Up/Down/Left/Right, Home/End, PageUp/PageDown, and Tab/Shift+Tab movement while the grid owns focus;
- automatic scroll-to-reveal for the active cell;
- Enter as an activation response for the current cell;
- header-click response plus caller-provided `None | Ascending | Descending` sort indicator state.

The application decides whether a cell click selects a row, selects a cell, opens a record, toggles multi-selection, or does nothing. The full-screen example uses single-row selection and a three-state sortable header cycle to demonstrate a complete policy.

### ADR-025: Text-First Cells Define the MVP Boundary

**Decision:** MVP header and body cells render a single line of clipped display text with left, center, or right alignment. Styling supports header, alternating rows, hover, active cell, selected row, grid lines, padding, and text color through an `AkarDataGridStyle` derived from the active theme.

This makes the component immediately useful and keeps draw-call count predictable. Arbitrary nested components, editable cells, multiline/variable-height rows, images, and per-cell portals are deferred. The API must leave room for future custom cell painters without making them part of MVP acceptance.

### ADR-026: Debuggability Is Part of the Component Contract

**Decision:** The implementation is not complete until an agent can isolate, drive, capture, inspect, and diff the grid without external screen-capture tools.

The grid must integrate with the existing debug loop:

- `demo-rust --component data_grid --screenshot ... --exit` for a tight isolated capture;
- checked-in `--script` fixtures for horizontal/vertical scroll, header sort, cell selection, and keyboard navigation;
- `--dump-layout` for the grid viewport and stable targets;
- `--dump-frame` to inspect header/body scissors, culled draw calls, z-order, and input state;
- `akar-diff --compare` for pixel-exact before/after regression checks.

The full-screen example must also expose the capture options needed for direct integration verification. At minimum: `--screenshot`, `--delay`, `--exit`, `--script`, `--dump-layout`, and `--dump-frame`. Reuse or extract the demo tooling where practical; do not depend on OS-level screenshots.

---

## Public API Direction

Exact names may change during Task 1 review, but the layering and ownership may not. A representative Rust surface is:

```rust
pub struct DataGridColumn {
    pub key: u64,
    pub width: f32,
    pub align: DataGridAlign,
}

pub struct DataGridState {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub active_row_key: u64,
    pub active_column_key: u64,
    pub has_active_cell: bool,
}

pub struct DataGridResponse {
    pub viewport_rect: [f32; 4],
    pub header_rect: [f32; 4],
    pub body_rect: [f32; 4],
    pub visible_rows: std::ops::Range<usize>,
    pub visible_columns: std::ops::Range<usize>,
    pub activated: Option<DataGridCellRef>,
    pub header_clicked: Option<u64>,
}

pub fn data_grid_begin(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    state: &mut DataGridState,
    row_count: usize,
    row_keys: &[u64],
    row_height: f32,
    header_height: f32,
    columns: &[DataGridColumn],
    style: &DataGridStyle,
) -> DataGridResponse;

pub fn data_grid_header_begin(core: &mut AkarCore, response: &DataGridResponse);
pub fn data_grid_header_cell(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    response: &DataGridResponse,
    column_index: usize,
    label: &str,
    sort: DataGridSortDirection,
) -> DataGridHeaderResponse;
pub fn data_grid_header_end(core: &mut AkarCore);

pub fn data_grid_body_begin(core: &mut AkarCore, response: &DataGridResponse);
pub fn data_grid_cell(
    core: &mut AkarCore,
    layout: &Layout,
    node: NodeId,
    response: &DataGridResponse,
    row_index: usize,
    row_key: u64,
    column_index: usize,
    text: &str,
    selected_row: bool,
) -> DataGridCellResponse;
pub fn data_grid_body_end(core: &mut AkarCore);
pub fn data_grid_end(core: &mut AkarCore);
```

The implementation may consolidate scopes if it proves balanced and C-safe, but it must preserve separate header/body clipping and must not hold a mutable borrow of `AkarCore` across caller rendering calls. C responses use explicit `has_*` flags plus indices/keys rather than Rust `Option` or heap-backed ranges. The generated `akar.h` is the contract and is never edited manually.

---

## Implementation Tasks

### Task 0 — Preflight, Dependency Audit, and Baselines

**Files:** workspace manifests/lockfile, `.gitignore`, local `.artifacts/epic027/` only as needed

1. Re-read Epics 017 and 026 and inspect current `data_list`, input, text pipeline, draw-list scissor, C ABI, demo catalog, script runner, frame dump, and capture-manifest implementations before changing code.
2. Confirm the fake-data audit. Add `fake = "5.1"` only to the new example crate when that crate is created; use its re-exported compatible RNG unless a separate direct dependency is demonstrably necessary. Do not add a faker dependency to library crates.
3. Capture same-machine pre-change baselines for `data_list`, `scroll_area`, and the full demo using the existing screenshot utility. Store them under gitignored `.artifacts/epic027/baselines/` with commands, scale factor, theme, source commit, and checksums.
4. Record baseline results for `cargo fmt --check`, `cargo test --workspace`, `cargo check --workspace`, and `cargo clippy --workspace --all-targets --no-deps -- -D warnings` before feature edits.

**Acceptance criteria:**

- The dependency decision and current baseline are reproducible.
- No library crate acquires random/fake-data dependencies.
- Existing list/scroll visuals can be compared after shared-path changes.

### Task 1 — Seal Geometry, State, and C-Compatible API Contracts

**Files:** `crates/akar-components/src/data_grid.rs` (new), `crates/akar-components/src/lib.rs`

1. Define column, style, state, response, cell reference, alignment, and sort-direction types.
2. Define the begin/header/body/end lifecycle and document legal call ordering.
3. Define dimension normalization, overscan, empty-grid behavior, and scroll clamping.
4. Require `row_keys.len() == row_count` on the Rust path and an equivalent validated pointer/count pair on the C path. Define how active row/column keys are reconciled when rows are sorted, filtered, or removed without adding an O(total rows) scan to every unchanged frame.
5. Define which values are caller-owned and which state akar mutates immediately before drawing.
6. Review the proposed surface against the flat C ABI before implementation proceeds.

**Acceptance criteria:**

- The API requires no closures, generic records, or retained caller strings.
- No API exposes Taffy or glyphon types.
- The empty and zero-area cases are explicit.
- The C mapping is mechanical rather than requiring a later redesign.

### Task 2 — Pure Two-Axis Grid Geometry and Virtualization

**Files:** `crates/akar-components/src/data_grid.rs`

1. Implement pure helpers for normalized column widths, cumulative offsets, total content dimensions, clamped `scroll_x/scroll_y`, visible row range, visible column range, header/body/cell rectangles, and scroll-to-reveal.
2. Reuse `akar_core::list_clip` semantics for rows where appropriate; do not fork subtly different edge behavior without tests.
3. Support a small documented overscan on both axes without returning indices outside valid ranges.
4. Add table-driven tests for empty data, one cell, partial rows/columns, oversized cells, exact boundaries, large scroll values, viewport resize, non-finite dimensions, and 100,000 × 100 logical grids.

**Acceptance criteria:**

- Geometry tests need no GPU or window.
- Per-frame iteration is bounded by column metadata plus visible/overscan cells, never total rows × columns.
- Header and body x coordinates remain identical under horizontal scroll.

### Task 3 — Grid Frame, Styling, and Header/Body Scopes

**Files:** `crates/akar-components/src/data_grid.rs`, `theme.rs`, `lib.rs`

1. Implement themed outer surface, sticky header, alternating-row backgrounds, grid lines, active/selected fills, padding, and text colors.
2. Implement distinct, intersecting header and body scissors with balanced begin/end scopes.
3. Handle two-axis wheel/trackpad deltas while hovered and clamp state in the same frame before drawing.
4. Ensure drawing reflects updated scroll and interaction state immediately, per the component contract.
5. Add `MockDrawList` tests for scope balance, scissor intersection, zero-area behavior, and post-input rendering.

**Acceptance criteria:**

- Body calls cannot render over the sticky header.
- Header and body remain horizontally aligned at every scroll position.
- Existing nested scissor behavior remains correct when a grid is placed in another clipped region.

### Task 4 — Visible Header and Text Cell Rendering

**Files:** `crates/akar-components/src/data_grid.rs`, text/style helpers as required

1. Render only visible/overscan header and body cells.
2. Render a single clipped line of text with left, center, and right alignment and deterministic truncation/overflow behavior.
3. Compose stable text-buffer IDs from grid namespace, row key, and column key. Header identity uses grid namespace plus column key in a domain separate from body cells.
4. Draw caller-provided sort indicators without embedding sorting logic.
5. Add unit tests proving stable IDs across scrolling/reordering and separation across rows, columns, headers, and portal namespaces.

**Acceptance criteria:**

- Scrolling a new record into an old screen slot cannot reuse the previous record's cell identity.
- A large logical grid shapes and submits text only for the visible/overscan cells.
- Long content is clipped to its cell and never bleeds into adjacent columns.

### Task 5 — Pointer Interaction and Application-Owned Selection Intent

**Files:** `crates/akar-components/src/data_grid.rs`

1. Implement header/cell hover, press, and click hit-testing from arithmetic rectangles.
2. Return stable row/column keys and logical indices for activation; do not expose only screen-slot coordinates.
3. Give the grid a stable focus ID when clicked while preserving other component focus behavior.
4. Apply hover, active-cell, and caller-selected-row visual precedence deterministically and test it.
5. Cover partially visible first/last rows and columns so clipped portions are not clickable outside their scissors.

**Acceptance criteria:**

- Sorting/reordering between frames does not redirect an action to a screen slot.
- Pointer input outside the body/header clip cannot activate a hidden cell.
- Selection policy remains application-owned.

### Task 6 — Keyboard Navigation and Scroll-to-Reveal

**Files:** `crates/akar-components/src/data_grid.rs`, input handling only if a missing key is proven necessary

1. Implement the keyboard behavior listed in ADR-024 while the grid owns focus.
2. Clamp navigation over empty data and after row/column count shrinks.
3. Scroll the active cell fully into view without moving a sticky header or overscrolling content.
4. Return Enter activation and focus/active-cell changes as typed responses.
5. Add non-GPU tests for navigation, modifiers, page movement, tab direction, resize, sorting/reordering, and focus loss.

**Acceptance criteria:**

- Every navigation path is deterministic and testable through `InputState`.
- The active cell remains visible after keyboard movement.
- Keyboard input is ignored when another widget owns focus.

### Task 7 — C ABI and Generated Header

**Files:** `crates/akar-c-api/src/lib.rs`, `cbindgen.toml`, `tests/`, generated `akar.h`

1. Add `repr(C)` equivalents for grid columns, state, style, responses, alignment, and sort direction.
2. Expose style defaults and the flat begin/header/body/end functions with pointer/count pairs for columns and UTF-8 pointer/length handling consistent with existing APIs.
3. Validate null pointers, count mismatches, out-of-range indices, and zero-length inputs without undefined behavior.
4. Add C integration tests that compile and exercise an empty grid, a visible grid, scrolling, activation, and balanced scopes.
5. Regenerate `akar.h` through cbindgen; never edit it directly.

**Acceptance criteria:**

- A C caller can render and interact with the same MVP grid without callbacks or caller-side heap allocations beyond its own records/state.
- Rust and C results agree for visible ranges and actions.

### Task 8 — Demo Isolation and Agent Debug Fixtures

**Files:** `examples/demo-rust/` catalog, layout, capture manifest, and scripts

1. Register a standalone `data_grid` component in the typed demo catalog and `--list-components` output.
2. Build its layout nodes once, outside paint, and register a stable non-zero `@data_grid` root label.
3. Add deterministic demo data and dynamic debug targets for representative headers/cells. Prefer semantic targets such as `@data_grid_header_name` and `@data_grid_cell_1003_status`; if the existing label registry cannot represent arithmetic cell rects, add a demo/debug-only rect-target registry rather than creating a Taffy node per cell.
4. Check in scripts and capture-manifest entries for idle, vertical scroll, horizontal scroll, sorted header, selected cell, and keyboard-moved active cell states.
5. Ensure `--dump-frame` records enough information to diagnose header/body scissor and culling failures. Extend the dump schema with grid debug metadata only if ordinary recorded calls/scissors are insufficient.

**Acceptance criteria:**

- `cargo run --bin demo-rust -- --component data_grid --screenshot /tmp/data-grid.png --exit` produces a tight, useful capture.
- `--dump-layout`, semantic scripts, and `--dump-frame` work without coordinate guessing for the canonical interactions.
- Every scripted screenshot is visually inspected, not merely generated.

### Task 9 — Full-Screen Fake-Data Example

**Files:** `examples/data-grid-rust/Cargo.toml`, `examples/data-grid-rust/src/main.rs`, scripts, root workspace manifest

1. Add a dedicated `data-grid-rust` workspace example whose content area is the data grid—no surrounding component showcase or demo navigation.
2. Generate at least 100,000 deterministic records at startup with `fake` 5.1 and a fixed seeded RNG. Use realistic columns such as ID, name, email, company, city, status, amount, and created date. Stable row keys must not depend on the current sorted position.
3. Demonstrate caller-owned sorting by clicking headers, single-row selection, cell activation, keyboard navigation, horizontal/vertical scrolling, and responsive resize.
4. Keep fake-data generation and sort/filter work outside the render loop. Each frame formats/renders only visible cells; cache any expensive derived display strings in the example's record model.
5. Support `--screenshot`, `--delay`, `--exit`, `--script`, `--dump-layout`, and `--dump-frame`. Provide `--rows <N>` and `--seed <N>` while keeping the canonical capture seed fixed in checked-in scripts.
6. Add checked-in scripts that capture the initial grid, far vertical scroll, horizontal scroll, sorted data, selected row, and keyboard navigation.

**Acceptance criteria:**

- The default window is a convincing full-screen tabular application, not a toy three-row fixture.
- Captures are deterministic for a fixed seed, viewport, theme, font source, and scale factor.
- Changing sort order preserves selection/focus by stable row key rather than screen position.
- Frame work and draw calls remain proportional to visible cells when `--rows` increases from 1,000 to 100,000.

### Task 10 — Regression, Performance, and Documentation Gate

**Files:** `DEVELOP.md`, `AGENTS.md`, README/website component catalog where appropriate, epic completion notes

1. Document grid ownership, lifecycle, virtualization, stable identity, and deferred features in `DEVELOP.md` and the C-facing API docs.
2. Add the grid to the component catalog/website without reviving the old false claim that a generic Table already existed.
3. Compare the Task 0 `data_list`, `scroll_area`, and demo baselines with `akar-diff`; explain every intentional difference.
4. Capture the isolated grid and all full-screen example states. Inspect images directly and use `--dump-frame` for any clipping/z-order anomaly.
5. Add a counted performance regression test proving that logical row count does not affect visible-cell submission count for a fixed viewport. Add a benchmark only if it is stable and useful; do not make wall-clock timing a flaky CI gate.
6. Run `cargo fmt --check`, `cargo test --workspace`, `cargo check --workspace`, and `cargo clippy --workspace --all-targets --no-deps -- -D warnings`.

**Acceptance criteria:**

- All non-GPU tests and workspace quality gates pass.
- Representative screenshots pass visual review and pixel comparisons.
- Documentation clearly separates the data grid from Taffy CSS Grid and from the lower-level `data_list`.
- The epic is marked Done only after Rust, C ABI, isolated-demo, and full-screen-example paths are all verified.

---

## MVP Scope

### Included

- Fixed-height virtualized rows.
- Caller-sized, horizontally virtualized columns.
- Sticky header with caller-owned sort state and header-click responses.
- Two-axis scrolling and scroll-to-active-cell.
- Single-line clipped text cells with alignment.
- Hover, press, active cell, caller-selected row, alternating row, border, and grid-line styling.
- Pointer activation and keyboard navigation.
- Stable row/column identity across scroll and sort.
- Flat C ABI.
- Isolated demo fixture, scripted captures, frame/layout dumps, and pixel-diff verification.
- Full-screen Rust example with at least 100,000 deterministic fake records.

### Deferred

- Inline cell editing and validation.
- Arbitrary component/custom-renderer cells.
- Variable-height or multiline rows.
- Column resize, reorder, auto-fit, pinning/frozen columns, and grouped headers.
- Multi-cell range selection, clipboard copy/paste, fill handles, and spreadsheet formulas.
- Multi-row selection policy, grouping/tree rows, aggregation, pivoting, and pagination.
- Built-in filtering/search UI or data-source/query APIs.
- Column persistence and application preference storage.
- Accessibility semantics, pending the deferred accessibility epic; keyboard behavior in this MVP must not be presented as accessibility completion.

---

## Epic Acceptance Criteria

- [x] An application can render 100,000 caller-owned records with work proportional to the visible rows and columns.
- [x] Header and body remain aligned and correctly clipped during two-axis scrolling.
- [x] Stable cell identity survives scrolling, sorting, and viewport resize.
- [x] Pointer and keyboard interactions return typed intent without akar owning records or sorting policy.
- [x] Rust and generated C APIs expose equivalent MVP behavior.
- [x] `demo-rust` can isolate, script, screenshot, dump, and pixel-diff the component.
- [x] `data-grid-rust` demonstrates a full-screen, deterministic fake-data grid and exposes the required debug flags.
- [x] Existing `data_list` and `scroll_area` behavior does not regress.
- [x] Workspace tests, formatting, checking, and clippy pass.

---

## Post-completion memory follow-up — 2026-09-20

**Status:** Done.

Same-machine `/usr/bin/time -l` measurements found two avoidable sources of
memory use in the full-screen example. Each fake record retained eight display
strings plus six duplicate string sort keys, and grid body cells left every
stable keyed `glyphon::Buffer` in `TextPipeline` after the row left the
viewport.

The example now stores each display string once and compares the stored fields
without allocating during sorting; numeric ID and amount ordering remain
typed. Grid body text uses an opt-in transient-buffer path. Transient buffers
retain a one-unused-frame grace period so a stationary viewport reuses its
allocations, then are pruned by `AkarCore::begin_frame`. Persistent layout and
text-editing buffers keep their previous lifetime.

At 100,000 rows, maximum RSS fell from 153.5 MiB to 125.4 MiB in debug and from
141.7 MiB to 114.0 MiB in release. After ten large jumps through disjoint row
ranges, RSS remained bounded at 129.1 MiB debug and 120.3 MiB release. The
initial full-grid screenshot remained pixel-identical (0 of 1,024,000 pixels
changed).
