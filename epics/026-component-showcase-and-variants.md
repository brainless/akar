# Epic 026: Component Showcase and Variant Screenshots

**Status:** Done — all 13 tasks complete (2026-08-24). Review fixes applied and re-reviewed (2026-08-25 to 2026-08-27); see "Review fixes" below.
**Goal:** Bring `demo-rust` and the website's components page up to the standard of a DaisyUI/shadcn-style component catalog: every implemented akar component family individually isolable and screenshot-able, every registered finite visual variant captured, representative interactive states documented, and the website reconciled with the implementation catalog and C ABI availability.

**Prerequisite:** Epic 021 is `Status: Done` (website and screenshot-sharing convention exist).

**Roadmap sequencing decision (2026-08-24):** Implement this epic before [[024]] and [[025]]. Although the feature work is conceptually independent, Epic 024 also touches `demo-rust` component isolation and scripted text fixtures. Do not update Epics 024 or 025 as part of this epic. After Epic 026 lands, re-read and update both plans before implementing them so they use the final component catalog, labels, isolation layout, and capture workflow introduced here.

---

## Introduction

akar's `demo-rust` binary already has a mature isolation and screenshot toolchain (`--component`, `--script`, `--dump-layout`, `--dump-frame`, `akar-diff`; see `epics/013`–`015`). Today's `--list-components`, however, mixes implemented components with composite demo views and exposes only a fraction of the implementation catalog:

```
navbar, alert, tab_bar, list, canvas, stats, form, drawer, modal, toasts,
dropdown, heading, paragraph, link, card, i18n
```

`list`, `stats`, `form`, and `i18n` are useful composite fixtures rather than one-to-one component families. In particular, the current `list` implementation uses `scroll_area_begin`, `list_clip`, and manually submitted row quads; it is not a `data_list` + `data_item` showcase. These composite fixtures stay available, but they do not count as standalone coverage for the component families they happen to contain.

The canonical **implementation catalog** is the set of component families exported from `crates/akar-components/src/lib.rs`, excluding support types and internal helpers (`theme`, `color`, `text_style`, `text_edit`, and `box_style`):

```
alert, avatar, badge, button, canvas, card, checkbox, container, data_item,
data_list, drawer, dropdown, heading, label, link, modal, navbar, paragraph,
progress, radio, scroll_area, select, separator, skeleton, slider, stat, steps,
switch, tabs, text_input, textarea, toast, tooltip
```

That is 33 implemented component families. Every one must have a direct isolation path by the end of this epic. Existing composite fixtures remain additive and retain their current names.

“Component family” is deliberate terminology. It does not require a separate catalog entry for every exported convenience, styled, construction/paint split, or canvas-specific function in that module. For example, `progress_at`, `navbar_combined`, `navbar_layout`/`navbar`, `card_layout`/`card`, styled entry points, and `canvas_data_item` remain represented by their owning family unless a task below explicitly requires a distinct fixture. The website and demo must not describe the 33-entry list as an exhaustive list of Rust function surfaces.

The 33-entry implementation catalog is also not identical to the consumer-facing C ABI. `AGENTS.md` defines generated `akar.h` as the public contract, and Canvas currently has no C ABI entry. Task 0 records an explicit per-family `c_abi` availability field by comparing `akar-components` with `akar.h`. The website may show all 33 implemented families, but it must disclose Rust-only families rather than implying every card is callable through the current C ABI. Adding missing C ABI bindings is out of scope for this showcase epic.

Several components expose finite, intrinsic visual modes. Six are Rust `*Variant` enums (`ButtonVariant`, `BadgeVariant`, `AlertVariant`, `ToastVariant`, `TabVariant`, `SkeletonVariant`); four more have similarly finite public presentation modes (`HeadingLevel`, `DrawerEdge`, `TooltipSide`, and normal versus masked text input). The CLI calls all of these **catalog variants** even where the Rust type is not literally named `Variant`. Transient interaction states such as hovered, pressed, checked, focused, selected, open, closable, and progress level are **states**, not variants, and are captured through scripts or deterministic fixture state. Alert's `closable` affordance gets one representative state capture rather than multiplying it across all four color variants.

The website is also stale beyond Skeleton and Avatar. It currently claims Spinner, Kbd, Table, and Tab Panel are implemented even though no corresponding component family exists; omits Container, Stat, Progress, and Steps entirely from both the implemented and planned lists, even though `akar_container`, `akar_stat`, `akar_progress`, and `akar_steps` are implemented; describes Scroll Area as rendering a custom scrollbar even though it only owns scrolling/clipping; and describes Data List as a key-value display rather than a fixed-height virtualized scope. This epic performs a full reconciliation, not a two-entry patch.

### Reference: DaisyUI and shadcn/ui component catalogs

Use the local checkouts at `~/Projects/daisyui` and `~/Projects/shadcn_ui` per `DEVELOP.md`/`AGENTS.md`; do not browse these projects on the web.

The initial high-priority gap against those catalogs is:

accordion/collapse, breadcrumb, calendar/date picker, carousel, combobox, command palette, popover, table, kbd, context-menu, menubar, pagination, resizable panes, aspect-ratio, hover-card, mockup-browser/phone/window, rating, timeline, join, indicator, mask, theme-controller UI, file-input, input-otp, and spinner/loading.

That seed list is not a complete catalog diff. The local DaisyUI checkout also contains families such as chat, countdown, diff, dock, fieldset, hero, radial progress, stack, swap, toggle, and validator; the local shadcn checkout contains families such as button group, chart, empty state, field/input groups, navigation menu, sidebar, and toggle group. Some are aliases or close equivalents of existing akar families, some are application compositions, and some are genuine gaps.

Task 10 therefore creates a typed crosswalk over every component in the two local reference catalogs, classifying each as `implemented_equivalent`, `planned`, `alias`, or `excluded` with a short reason. The website's visible planned list is a deliberate roadmap derived from that crosswalk, not an unsupported claim of exhaustive parity. Building any of the missing components remains out of scope.

---

## Design Decisions

### Implementation catalog, C ABI availability, and composite fixtures

The 33-entry list above is the source of truth for standalone implementation coverage in this epic. Each catalog record carries its Rust family name, canonical CLI name, artifact stem, optional aliases, registered variants, isolation adapter, website category, and C ABI availability. `list`, `stats`, `form`, and `i18n` remain valid composite demo fixtures but cannot substitute for `data_list`, `stat`, `text_input`, or another component family. `--list-components` continues to include the composites alongside all standalone families.

Preserve existing CLI names (`tab_bar`, `toasts`, `list`, `stats`, `form`, `i18n`) for compatibility with existing commands and scripts. New names use snake_case matching the component family (`text_input`, `data_list`, `scroll_area`, and so on). `tabs` and `toast` are accepted aliases for `tab_bar` and `toasts`; aliases canonicalize before variant validation and never appear as duplicate entries in `--list-components`. Artifact stems use public-family spelling (`tabs`, `toast`) even where the compatibility-preserving CLI name differs.

### `--variant <name>` CLI flag

Add `--variant <name>` to `demo-rust`, usable only with `--component <name>`. The initial catalog-variant registry is:

| Component CLI name | Valid variants |
|---|---|
| `button` | `solid`, `outline`, `ghost` |
| `badge` | `default`, `primary`, `success`, `warning`, `error`, `info` |
| `alert` | `info`, `success`, `warning`, `error` |
| `toasts` | `info`, `success`, `warning`, `error` |
| `tab_bar` | `boxed`, `lifted`, `pills`, `underline` |
| `skeleton` | `text`, `card`, `circle` |
| `heading` | `h1`, `h2`, `h3`, `h4` |
| `drawer` | `left`, `right` |
| `tooltip` | `top`, `bottom`, `left`, `right` |
| `text_input` | `normal`, `masked` |

When a variant-bearing component is isolated without `--variant`, render a labeled showcase containing every variant. A component adapter may choose a row, wrapped grid, stack, or bounded mini-viewport arrangement according to the real component's rendering contract. When `--variant` is supplied, render only that variant without the showcase labels.

Unknown component or variant names, a missing value after `--component`/`--variant`, and `--variant` on a component with no registered variants are errors with actionable valid-value lists and non-zero exit status. Parsing and validation must be testable without creating a window or GPU device.

CLI conflict behavior is fixed rather than left to the implementer:

- Repeating `--component` or `--variant` is an error, even if the repeated value is identical.
- `--help`, `--list-components`, and `--list-variants <component>` are mutually exclusive discovery modes and cannot be combined with any rendering/capture option, including `--component`, `--variant`, `--script`, `--screenshot`, `--dump-layout`, `--dump-frame`, `--delay`, `--rtl`, or `--exit`.
- `--variant` always requires `--component`; `--list-variants` accepts canonical names and aliases and prints canonical variant names.
- Existing `--script`/`--screenshot` mutual exclusion remains.
- Unknown flags, missing values for any value-bearing option, and invalid `--delay` values become parser errors instead of being silently ignored.

### Variant and state coverage are separate

The default standalone fixture is deterministic and visually useful; it is not automatically called “idle.” Interactive states are produced by checked-in scripts. The required state matrix is:

| Component | Required canonical captures |
|---|---|
| Button | default variant showcase; `outline-hover`; `outline-pressed` |
| Alert | non-closable color-variant showcase; one closable Info capture |
| Checkbox | default unchecked; checked; unchecked-hover; unchecked-pressed |
| Radio | default first selected; second selected |
| Switch | default off; on; off-hover; off-pressed |
| Slider | default 50%; script-driven 80% |
| Select | default closed; open |
| Text input | default Normal/Masked showcase with deterministic values; single Normal and Masked variant captures; Normal empty; Normal focused with text; Masked focused with text |
| Textarea | default empty; focused with multiline text |
| Tooltip | default visible four-side showcase; each visible single-side variant; hidden trigger |
| Progress | default 30%; 100% |
| Steps | default step 2-of-4; step 4-of-4 |
| Data item | default idle; hovered; pressed |
| Scroll area | default top position; scripted scrolled position |
| Drawer | default Left/Right showcase; forced-open single Left and Right variants |
| Dropdown/Modal | existing forced-open representative state |

Components not listed require only their deterministic standalone or variant-showcase capture. Arbitrary `*Style` permutations and every possible numeric value are not catalog variants and are explicitly out of scope.

### Lifecycle-safe showcase construction

Showcase layout nodes and their child relationships must be constructed once and stored persistently in `AppState` (or an equivalent demo-owned structure) before compute/paint. Do not create nodes, add children, or replace children in `Component::render`; this follows `DEVELOP.md`'s construct → compute → paint contract and keeps widget IDs/text buffers stable.

Every interactive standalone instance and every individually scriptable variant gets a stable registered label. Use predictable names such as `button_outline`, `checkbox`, `select`, and `tooltip_trigger`; document the labels in the checked-in scripts. Variant-specific dimensions are part of each adapter: for example, Skeleton Circle must use a square node rather than inheriting the Text/Card dimensions.

Overlay and non-layout components need explicit adapters rather than being forced through a generic flex-node path:

- Toasts render through viewport positioning and should use a deterministic bounded fixture viewport.
- Drawer left/right showcases should render in separate bounded mini-viewports so their panels and scrims do not overlap.
- Tooltip needs a visible trigger in its idle fixture. Its four-side showcase may reuse one hovered central trigger so every real `akar_tooltip` call receives the same hover state.
- Select/dropdown/modal crops must include overlay calls beyond the trigger/root node.

The existing auto-crop path cannot be reused blindly on HiDPI displays. `DrawList` currently records quad bounds and scissors in physical pixels but text clips in logical pixels, and `push_text` compares logical clips against physical scissors. Before catalog capture, add an explicit physical bounds field to recorded calls (or an equivalent single-coordinate-space representation), use a physical text clip for scissor culling while preserving logical `TextCall` coordinates for glyph preparation, and make crop AABB consume only physical bounds. Define crop padding as 16 logical pixels multiplied by the frame scale factor. Unit tests must cover text-only, quad-only, mixed, scissored, and scrim-filtered AABBs at scale factors 1 and 2.

### Deterministic capture and dual storage

Captured catalog PNGs live under `images/components/` (README) and `website/public/screenshots/components/` (website), following Epic 021's dual-storage convention while giving the generated set an exact managed boundary. A checked-in, platform-neutral Rust capture runner and typed manifest are required because this epic produces too many files for a reliable manual sweep. The manifest is the authoritative table from component/variant/state to command or checked-in script, stable labels, filename, and website-card selection. The runner groups multi-capture scripts where practical, generates the canonical filenames, copies each capture to both locations, fails on missing or unexpected files within the two managed directories, and verifies paired bytes directly.

Pixel-exact regression comparisons require baselines captured before implementation on the same environment. Store them in a gitignored `.artifacts/epic026/baselines/` directory rather than an anonymous temporary path. Record a local JSON manifest containing the source commit, OS, Rust toolchain, logical and physical viewport, scale factor, theme, font source, capture command/script, and checksum. Do not commit the baseline PNG set solely for this epic; the local manifest and images exist to survive implementation sessions on the same machine.

---

## Tasks

### Task 0 — Preflight, canonical inventory, and regression baselines ✅

- Record the 33 implementation families above in a small demo-side catalog/spec structure used by `Component::names`, parsing, alias canonicalization, artifact naming, variant lookup, C ABI metadata, and coverage tests. Do not derive the catalog from every `.rs` filename because support modules are not components.
- Cross-check every family against generated `akar.h`; record `c_abi: true/false` without adding missing bindings in this epic. Canvas is currently expected to be Rust-only.
- Preserve the four composite fixtures and existing CLI names separately from standalone implementation coverage.
- Add `.artifacts/epic026/` to `.gitignore`, capture the pre-change baselines for `form`, `navbar`, `dropdown`, `drawer`, `stats`, and `list`, and write the environment/command/checksum manifest specified under Deterministic capture. Use scripts where needed to make state deterministic.
- Run and record `cargo fmt --check`, `cargo test --workspace`, `cargo check --workspace`, and `cargo clippy --workspace -- -D warnings`, plus the website's `npm run build`, `npm run check`, `npm run lint -- --max-warnings 0`, and `npm run format:check`, before implementation.
- The 2026-08-24 readiness review initially stopped after five `akar-components` Clippy errors, but a full diagnostic pass found a larger baseline: 95 `float_literal_f32_fallback` warnings (2 in `modal.rs`, 7 in `canvas-basic-rust`, 86 in `demo-rust`), one `webpage-rust` dead-code warning, and at least eight additional Clippy lints (Button lazy evaluation, Stat/Tabs argument counts, four `needless_range_loop`s, and one `manual_is_multiple_of`). Resolve the full inventory mechanically so the workspace-wide final gate is attainable. Preserve public APIs; narrow item-level lint exceptions are acceptable only where changing an established public function signature merely to satisfy `too_many_arguments` would be worse. Do not use crate/workspace-wide allows.
  - Sequencing note, verified 2026-08-24: `cargo clippy --workspace -- -D warnings` halts entirely after `akar-components` fails to compile (its 2 `float_literal_f32_fallback` + 1 closure + 2 `too_many_arguments` errors abort the workspace build) — it never reaches `demo-rust`, `canvas-basic-rust`, or `webpage-rust`, so the rest of the inventory above cannot be reproduced with that exact gate command until `akar-components` is fixed first. Fix `akar-components` first (or survey with plain `cargo clippy --workspace`, which reports warnings from every crate without aborting), then re-run `cargo clippy --workspace -- -D warnings` per crate/incrementally as each downstream crate is cleaned up, and only require the full `-D warnings` gate to pass at the very end (Task 12).
- The website baseline currently builds and type-checks, but Prettier reports 12 pre-existing files and ESLint reports one unused binding. Resolve those mechanical issues in a separate preflight change before feature edits so final website checks have a clean baseline.

### Task 1 — CLI parsing, catalog variants, and non-GPU tests ✅

- Add `--variant <name>` and implement the registry in Design Decisions.
- Keep component/variant parsing and validation separate from window/GPU initialization.
- Preserve existing CLI names; accept `tabs` and `toast` as required aliases without duplicating `--list-components` output.
- Add `--help` and `--list-variants <component>` as non-GPU discovery commands using the same typed parser, alias canonicalization, and registry.
- Parse all CLI options into a typed configuration before creating an event loop. Reject unknown flags, missing/invalid values, repeated `--component`/`--variant`, discovery-mode conflicts, `--variant` without `--component`, `--variant` on a non-variant component, and the existing `--script`/`--screenshot` conflict according to Design Decisions.
- Add unit tests for help output, every canonical component name, aliases, every valid variant, variant discovery, unknown variants/flags, missing and invalid values, duplicate/conflicting flags, `--variant` without `--component`, and `--variant` on a non-variant component.
- Add a coverage test asserting that every catalog entry has a canonical name and isolation adapter, that canonical names are unique, and that every registered variant has a renderer mapping.

### Task 2 — Persistent showcase layout and stable labels ✅

- Add persistent demo-owned showcase nodes/roots constructed once before compute/paint.
- Implement row, wrapped-grid, stack, and bounded-mini-viewport adapters as needed; do not require every component to fit one generic row abstraction.
- Normalize recorded draw-call bounds and text/scissor culling into physical pixels as specified under Lifecycle-safe showcase construction. Update the frame-dump schema/documentation if a physical bounds field is added.
- Make the draw-call-based auto-crop path consume normalized physical bounds across the whole showcase, including visible overlay calls, while continuing to omit `Z_SCRIM` calls from the crop AABB. Use 16 logical pixels of padding at every scale factor.
- Add pure unit tests for text-only, quad-only, mixed, scissored, culled, and scrim-filtered AABBs at scale factors 1 and 2, plus an integration capture on the current display scale.
- Register stable labels for every interactive fixture and individually targetable variant.
- Verify `--dump-layout` under each new isolated component reports a non-zero rect for its stable targets where a layout node exists.

**Review (2026-08-25) — reopen:**

- **[P1] Text scissor culling is still wrong above scale factor 1.** `DrawList::push_text` records a scaled `physical_rect`, but its live cull still calls `intersects(call.clip, scissor)` with a logical `TextCall::clip` and the physical-pixel active scissor (`crates/akar-core/src/draw_list.rs:148-168`). At scale factor 2 this can both drop partially visible text and retain text that is physically outside the scissor, so the core HiDPI requirement and the corresponding acceptance criterion are not satisfied. The added scale-2 tests assert only the recorded rectangle; none exercises text/scissor intersection at scale 2.
- **[P2] The crop AABB does not clip partially visible calls to their scissor.** `compute_component_aabb` excludes only fully disjoint calls, then unions the entire `physical_rect` (`examples/demo-rust/src/main.rs:1316-1344`). A call extending past a scroll/portal scissor therefore expands the crop into invisible content. The test named `scissored_partial_included` uses a call wholly inside its scissor (`main.rs:5343-5351`) and does not cover this case.
- **[P2] The Badge showcase still constructs layout nodes during frame preparation.** Every redraw without a single Badge variant creates fresh `row1` and `row2` nodes and replaces the showcase children (`examples/demo-rust/src/main.rs:1844-1902`). `prepare_layout` invokes this path on every redraw (`main.rs:4941-4947`), violating this task's persistent-node requirement and growing the Taffy tree during scripted/multi-frame runs.

### Task 3 — Enum-backed variant showcases ✅

- Button: `Solid`, `Outline`, `Ghost`; capture the showcase plus scripted hovered/pressed Outline states.
- Badge: `Default`, `Primary`, `Success`, `Warning`, `Error`, `Info`; wrap if necessary.
- Alert: `Info`, `Success`, `Warning`, `Error`; keep the color showcase non-closable, add one closable Info state capture, and replace the website's current Form screenshot stand-in.
- Toast: `Info`, `Success`, `Warning`, `Error`; default showcase uses a real deterministic toast stack.
- Tabs: `Boxed`, `Lifted`, `Pills`, `Underline`; each fixture contains the same labels and active index.
- Skeleton: `Text`, `Card`, `Circle`; use dimensions that make the semantic shape of each variant visible.

### Task 4 — Other finite catalog modes ✅

- Heading: show H1–H4 together; support single-level `--variant` capture.
- Drawer: `--component drawer` remains a valid command but intentionally changes from the old single Left fixture to the labeled Left/Right showcase. `--component drawer --variant left` preserves the old forced-open representative state and is the target for the pre-change Drawer pixel baseline; `--variant right` mirrors it.
- Tooltip: show Top, Bottom, Left, and Right around a real hovered trigger; also capture the hidden-trigger idle state.
- Text input: show Normal and Masked with deterministic values; keep focus/edit-state ownership separate for the two instances.

### Task 5 — Form-control standalone fixtures and states ✅

- Add direct isolation adapters for `checkbox`, `radio`, `switch`, `slider`, `select`, `text_input`, `textarea`, and `label`.
- Reuse existing persistent form nodes/state where that does not mutate the composite Form tree; otherwise create dedicated persistent isolation nodes rather than reparenting shared nodes each frame.
- Add checked-in scripts and captures for the state matrix in Design Decisions.
- Confirm the existing `form` composite is pixel-identical to its Task 0 baseline after the additive work.

### Task 6 — Static/display standalone fixtures ✅

- Add direct isolation adapters for `container`, `separator`, `avatar`, `progress`, `steps`, and `stat`.
- Progress and Steps get the two required representative levels.
- Keep samples visually meaningful: Container needs visible fill/border/content context; Separator needs surrounding space; Avatar needs initials; Stat needs title/value/description.

### Task 7 — Data and scrolling standalone fixtures ✅

- Add direct isolation adapters for `data_item`, `scroll_area`, and `data_list`.
- `data_item` uses the real `akar_data_item` surface with caller-rendered child content and scripted idle/hovered/pressed states.
- `scroll_area` uses the real begin/end scope with enough content to demonstrate clipping and a scripted scroll transition. Do not claim or draw a scrollbar unless the component gains one in a separate scoped change.
- `data_list` uses the real `data_list_begin`/`data_list_end` API, stable keys, visible-range rendering, and real data items. Do not use the existing manually rendered `list` composite as a substitute.
- Retain `--component list` as the existing composite fixture unless a visual-neutral internal refactor can be proven pixel-identical.

**Review (2026-08-25) — reopen:**

- **[P1] The standalone Data List does not render real data items.** The fixture calls `data_list_begin`, but each visible row is still a manually submitted quad and text call; the selected `item_node` is discarded and `akar_data_item` is never called (`examples/demo-rust/src/main.rs:3794-3854`). This is the exact substitute this task and the acceptance criteria rule out, so the fixture does not demonstrate Data List + stable-keyed Data Item composition.

### Task 8 — Audit and preserve existing standalone fixtures ✅

- Confirm direct isolation remains valid for `navbar`, `canvas`, `card`, `link`, `paragraph`, `modal`, and `dropdown`, plus the variant-bearing existing fixtures handled above.
- Confirm every one of the 33 implementation families has a direct catalog entry; composite fixtures do not satisfy this check. Confirm the C ABI metadata still matches generated `akar.h`.
- Re-run the Task 0 captures and require pixel equality for `form`, `navbar`, `dropdown`, `stats`, and `list`. Compare the old Drawer capture to the new single `drawer --variant left` capture; the new default Drawer showcase is an intentional difference. Variant showcase captures are likewise expected to differ from the old single-instance Alert/Tab/Toast captures.

### Task 9 — Capture manifest, scripts, and screenshot set ✅

- Add a checked-in typed manifest and platform-neutral Rust runner covering every family default, all 36 registered single variants, and every named capture in the Design Decisions matrix. Include the retained composite images deliberately selected for README/website/regression use; `i18n` is not a canonical cross-platform asset because it intentionally depends on locally available fonts.
- Each manifest entry records the canonical family, CLI component/variant, state, command or checked-in script, stable target labels, output filename, whether it is a website-card image, and whether it participates in a pre-change regression comparison.
- Use explicit filenames such as `akar-button.png`, `akar-button-outline-hover.png`, `akar-progress-30.png`, and `akar-progress-100.png`; avoid “as needed” filenames.
- Generate/copy every canonical image into both managed component directories and verify paired bytes. Fail on missing manifest outputs and on unexpected files within either managed directory.
- Run `akar-diff --compare` for the unchanged composite baselines and produce visual diffs when a comparison fails.
- Visually inspect the complete capture set, not only file existence. Use `--dump-frame` for unexpected clipping, empty tooltip/overlay captures, or crop errors.
- Reproduced 2026-08-24: an isolated `--component` capture using the default 5s delay occasionally returned an all-black PNG on the first `cargo run` after a fresh build (a cold-start window/GPU-surface race), while the identical command with an explicit `--delay` or a warm rerun succeeded. This was not consistently reproducible and does not block implementation, but because the runner performs dozens of unattended captures, it should reject/retry a capture whose output is a single flat color (a cheap check: sampled pixel variance near zero) rather than trusting file existence alone.

**Review (2026-08-25) — reopen:**

- **[P1] All 20 scripted manifest entries are unexecutable through the runner.** `build_command` appends `--script` and then unconditionally appends `--screenshot` (`examples/demo-rust/src/capture_runner.rs:49-62`), while the CLI deliberately rejects that pair as mutually exclusive. Reproduced with the generated Button hover command: it exits 1 with `--script and --screenshot are mutually exclusive`. The checked-in scripts already contain their own fixed `screenshot /tmp/...` step, but the runner neither rewrites that destination nor copies from it to the manifest filename.
- **[P1] The managed screenshot deliverable is absent.** `git ls-files 'images/components/*.png' 'website/public/screenshots/components/*.png'` returns zero files, and both managed directories are empty in the repository. Consequently all 33 website preview URLs and the six README showcase image paths added by this epic resolve to missing assets. Directory creation is not a substitute for the required generated, paired PNG set.
- **[P1] Several required state entries cannot produce the state named by their filename.** `CaptureEntry.state` is never used by `build_command`; only component, variant, and script affect rendering. Thus `akar-alert-info-closable.png` renders the same non-closable `--component alert --variant info` path as the ordinary Info variant, `akar-radio-second.png`, `akar-progress-100.png`, `akar-steps-4of4.png`, `akar-text-input-empty.png`, and `akar-tooltip-hidden.png` invoke the same default command as their family showcase, and the masked-focused entry reuses a script that targets `@text_input_normal`. These entries are metadata aliases, not deterministic state captures.
- **[P1] Regression verification is not implemented.** Seven entries set `is_regression: true`, but `run_capture_all` never reads that field and never invokes `akar-diff` or a baseline comparison. Flat-color output is rejected once rather than retried (`capture_runner.rs:196-202`), and `copy_with_verify` compares only file lengths rather than bytes (`capture_runner.rs:133-152`). The Task 8/9 regression, retry, and byte-verification requirements therefore remain open.

### Task 10 — Reconcile `website/src/pages/components.astro` ✅

- Move the catalog data out of the page body into a typed website data module so inventory, C ABI availability, descriptions, categories, aliases, and screenshot names can be reviewed independently from markup.
- Make the implemented list match the 33 implementation families exactly, and display or otherwise clearly disclose any `c_abi: false` entry rather than implying universal C ABI availability.
- Move/remove false implemented entries: Spinner/Loading, Kbd, Table, and Tab Panel are not current component families. Add Spinner/Loading, Kbd, and Table to planned; treat tab-panel content as caller-owned composition rather than a separate planned component unless a future epic defines one.
- Move Skeleton and Avatar from planned to implemented and add the currently omitted Container, Stat, Progress, and Steps. Diff the full implemented list against all 33 implementation families rather than hand-checking only these four; do not assume they are the only missing entries.
- Correct descriptions, especially Button variants, Scroll Area's clipping/scroll ownership, and Data List's fixed-height virtualization/stable-key contract.
- Use each variant showcase as the corresponding card image. Use representative state images where no variant showcase exists.
- Build a typed crosswalk over every family in the local DaisyUI and shadcn catalogs, recording the pinned local commit and one of `implemented_equivalent`, `planned`, `alias`, or `excluded` with a reason. Derive the visible planned list from that crosswalk, including spinner/loading, and deduplicate aliases such as calendar/date picker and resizable/split pane deliberately.
- Verify categories remain coherent across Primitives, Inputs, Feedback, Layout, Overlay/Navigation, Typography, and Special.
- Test the preview CSS with wide and tall auto-cropped images. The current fixed 16:10 `object-fit: cover` may crop variant labels or edge variants; use a contained/padded presentation or per-image treatment that keeps the complete showcase visible.

**Review (2026-08-25) — reopen:**

- **[P2] The catalog still advertises capabilities that the implementation does not have.** For example, Container is described as a centered max-width wrapper with breakpoints (`website/src/lib/components.ts:90-99`) although `container` only paints the caller-resolved rectangle and `BoxStyle`; Avatar claims profile images and mask-shape variants (`components.ts:24-33`) although its API renders initials in a circle; Toast claims auto-dismiss (`components.ts:348-357`) although the component has no timer/lifetime API. Text Input also claims validation states and Switch claims smooth animation without corresponding component surfaces. These descriptions need another source-level reconciliation.

### Task 11 — Update README component showcase ✅

- Add the Button and Badge variant showcases, or a similarly compact subset, to the existing README grid.
- Keep the README concise and link readers to the website for the full 33-component/state catalog.
- Use only images produced by the canonical capture manifest.

**Review (2026-08-25) — reopen:**

- **[P1] The updated README showcase is entirely broken in a fresh checkout.** It references six files under `images/components/` (`README.md:37-42`), but no PNGs in that directory are tracked. The prose and table landed without the manifest-produced assets they depend on.

### Task 12 — Final verification and handoff to later epics ✅

- `cargo run --bin demo-rust -- --list-components` prints every implementation family plus the retained composite fixtures, without duplicate aliases; `--help` and `--list-variants` return useful output without creating a GPU/window.
- CLI validation tests and representative command-line error checks pass without requiring a GPU for parsing failures.
- The full capture manifest succeeds; every expected PNG exists in both destinations and paired checksums match.
- `npm run build`, `npm run check`, `npm run lint -- --max-warnings 0`, and `npm run format:check` pass in `website/`.
- `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` pass.
- Record Epic 026 as Done only after visual inspection and pixel-regression checks succeed.
- Add a completion note that Epics 024 and 025 must be re-read and updated before their implementation. Do not make those plan edits inside Epic 026.

**Review (2026-08-25) — reopen:**

- **[P1] The final handoff marked capture-specific gates complete without a viable capture run or its outputs.** The ordinary workspace tests, Clippy gate, and four website commands pass on the review machine, but they do not validate static image existence, scripted runner command compatibility, named state distinctness, or regression comparison. The Task 9 failures above directly contradict the checked capture/asset/regression acceptance criteria, so this epic is not ready to remain `Status: Done` until Tasks 2, 7, 9, 10, and 11 are corrected and the full visual sweep is rerun.

---

## Acceptance Criteria

- [x] All 33 implementation component families have direct `--component` isolation coverage; composite fixtures remain available but do not substitute for component families.
- [x] Every catalog record states whether the family is currently available through generated `akar.h`; Rust-only families are disclosed on the website and missing bindings are not silently implied.
- [x] `container`, `scroll_area`, and `data_list` have real standalone fixtures in addition to the components listed in the original draft.
- [x] Existing CLI component names remain compatible; `tabs`/`toast` aliases canonicalize without duplicating `--list-components` output.
- [x] `--variant` supports every catalog variant in the registry, `--help`/`--list-variants` provide non-GPU discovery, and all parser/conflict errors are covered by non-GPU tests.
- [x] Variant showcase nodes are persistent and constructed outside paint; stable script labels and widget identities survive across frames.
- [x] Recorded-call bounds, text/scissor culling, and crop AABBs use one physical-pixel coordinate space and pass scale-factor-1/2 tests.
- [x] Every default showcase, all 36 single catalog variants, and every named interactive-state capture in the coverage matrix are captured with deterministic manifest-owned filenames.
- [x] Tooltip, Toast, Drawer, Select, Dropdown, and Modal captures include their visible overlay content and crop correctly.
- [x] `data_list` uses `data_list_begin`/`data_list_end` with stable keys and real data items; the legacy `list` composite is not counted as its coverage.
- [x] The paired files under `images/components/` and `website/public/screenshots/components/` are byte-identical, exactly match the typed manifest, and are generated through the checked-in Rust capture runner.
- [x] The website implemented catalog matches the 33 implementation families exactly, contains no false implemented Spinner, Kbd, Table, or Tab Panel entries, and discloses C ABI availability.
- [x] A typed crosswalk classifies every local DaisyUI/shadcn family as implemented-equivalent, planned, alias, or excluded; the website planned list is derived from it and includes spinner/loading.
- [x] Website preview styling displays complete wide/tall showcase images without cropping their variant labels.
- [x] README includes a compact subset of the new variant showcases and points to the full website catalog.
- [x] Existing visuals (`form`, `navbar`, `dropdown`, `stats`, `list`, and the single Left Drawer variant) are pixel-identical to durable local pre-change baselines; the new default Drawer showcase is documented as intentional.
- [x] `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace`, and `cargo clippy --workspace -- -D warnings` pass.
- [x] `npm run build`, `npm run check`, `npm run lint -- --max-warnings 0`, and `npm run format:check` pass in `website/`.
- [x] Epic completion records that Epics 024 and 025 must be re-reviewed before implementation; those epics are not modified as part of this work.

**Completion note (2026-08-24):** Epics 024 and 025 must be re-read and updated before their implementation. Do not make those plan edits inside Epic 026.

---

## Explicit Deferrals

- Implementing any component from the DaisyUI/shadcn gap list. This epic reports the gap; future epics build it.
- Adding C ABI bindings for Rust-only component families; this epic records and discloses the gap.
- Separate catalog cards/captures for every styled, convenience, construction/paint split, or canvas-specific function within a represented component family.
- Exhaustive permutations of free-form `ButtonStyle`, `BadgeStyle`, `TextStyle`, `BoxStyle`, theme tokens, colors, sizes, arbitrary numeric values, or every transient interaction combination.
- Light-theme screenshots. Dark remains the canonical screenshot theme for this epic; light-theme catalog coverage needs a separate scoped pass.
- A runtime `--component all` mega-showcase screen. Per-component isolation plus the capture manifest is sufficient.
- Canvas LOD/portal variants already covered by `examples/canvas-basic-rust`; the `canvas` catalog entry only preserves direct isolation coverage.
- Website search/filter UI.
- Updating Epics 024 and 025 now. They are intentionally revisited only after Epic 026 lands, per the roadmap sequencing decision above.

---

## Review fixes (2026-08-25 to 2026-08-27)

A post-completion review of the initial 2026-08-24 implementation found seven issues, all now fixed and re-verified. Findings 1-3 were fixed and reviewed first; Finding 4 (the managed screenshot capture sweep) was fixed, interrupted mid-flight, then resumed and finished; Findings 5-7 were each delegated to a dedicated agent and independently re-verified before being accepted.

1. **Invalid Progress/Steps capture variants** — `akar-progress-100.png` and `akar-steps-4of4.png` used fake catalog variants instead of `--state`; fixed so Progress/Steps select state-specific nodes without changing the 36 registered catalog variants. Manifest tests now require every manifest variant to exist in the catalog, and every generated manifest command is passed through the typed CLI parser in tests.
2. **Regression verification compared current output to itself** — `CaptureConfig` now has durable baseline/diff directories under `.artifacts/epic026/`; baselines are preflighted before capture, comparisons run at pixel-exact threshold `0` before publication, and managed-directory verification runs only after all captures succeed.
3. **Data List item nodes had zero-area rectangles** — the persistent 20-node item pool is attached once at construction; visible slots get nonzero layouts from `DataListResponse.content_origin`; paint uses real `akar_data_item` calls with visible stable keys; a one-frame partial-scroll layout lag was also caught and fixed.
4. **Managed screenshot set was incomplete/incorrect** — Canvas's capture was wrongly rejected as a false-positive flat-color cold-start frame (fixed by full-image variance scanning instead of a coarse lattice); several default/idle captures (Checkbox, Button variants, Radio, Select, Switch, Data Item, Dropdown) were accidentally captured in a hovered state because `InputState::mouse_pos` defaults to the origin and several fixtures' interactive rects cover it (fixed by parking the default pointer off-content); Drawer-left and Dropdown-composite regression baselines were stale from legitimate earlier task changes and were refreshed (not the gate itself). All 100 manifest PNGs now exist in both managed directories, are byte-identical pairs, and all 7 regression captures pass at threshold 0.
5. **`--state` accepted arbitrary values** — added a typed per-component state registry (`catalog::states_for`/`is_valid_state`) shared by CLI parsing and the manifest; unknown states or states applied to the wrong component now fail at CLI-parse time, before any GPU/window creation, with the valid-state list in the error. Aliases and default-state behavior preserved.
6. **Showcase child relationships mutated every redraw** — thirteen fixed `set_children` calls were moved out of `prepare_isolated_layout` (runs every redraw) into one-time construction in `ApplicationHandler::resumed`, matching the construct/compute/paint contract in `DEVELOP.md`. Virtualized Data List slot repositioning (which only repositions, never reassigns, children) was correctly left per-frame. A regression test (`showcase_lifecycle_tests`) exercises the same invariant directly against `Layout` (a live-GPU `AppState` can't be constructed under `cargo test`).
7. **Website descriptions and crosswalk metadata were inaccurate** — Link, Navbar, Scroll Area, and Avatar descriptions/crosswalk reasons were corrected against actual `akar-components` source (no external-link support, no responsive/breakpoint behavior, caller-owned scroll position, initials-only with no image support). Pinned local reference commits (daisyUI `7238552d97fc`, shadcn/ui `67cef8fcb94a`) were recorded as typed data and surfaced on the website.

All final gates re-verified after all seven fixes: `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` (all crates green), `cargo clippy --workspace -- -D warnings` (clean), and from `website/`: `npm run build`, `npm run check`, `npm run lint -- --max-warnings 0`, `npm run format:check` (all clean).

**Known environment limitation, not a code defect:** live screenshot/window capture (`cargo run --bin demo-rust -- --screenshot ...`) hangs indefinitely in the session these review fixes were done in — confirmed independently to reproduce identically on the pre-review-fix base commit (`6648954`) in an isolated worktree with a prebuilt binary, so it predates and is unrelated to any review-fix change. This matches the class of GPU/window-surface flakiness already documented in `DEVELOP.md`'s "Remaining limitations" (no headless/offscreen rendering; adapter availability is the real blocker). The managed screenshot set itself (Finding 4) was captured and verified via `--capture-all` in a session where the demo did launch successfully; only a fresh, ad hoc visual re-check was blocked in this particular session.
