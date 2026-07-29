# Task 3 — Heading and Paragraph Components

## Summary

Implemented `heading` and `paragraph` components in `crates/akar-components/src/` following the existing `label` component pattern.

## Changes

### New files
- **`crates/akar-components/src/heading.rs`** — `HeadingLevel` enum (H1, H2, H3, H4) and `heading()` function with per-level font size/weight defaults, `TextStyle` overrides, zero-area guard, text buffer creation via `set_text`, and `TextCall` push.
- **`crates/akar-components/src/paragraph.rs`** — `paragraph()` function with `font_size_base`, `FontWeight::Normal`, `line_height = font_size * 1.5`, wrapping enabled, `TextStyle` overrides, zero-area guard, text buffer creation, and `TextCall` push.

### Modified files
- **`crates/akar-components/src/lib.rs`** — Added `pub mod heading` and `pub mod paragraph` with public re-exports `akar_heading`, `HeadingLevel`, and `akar_paragraph`.

## Design decisions

1. **Follows label pattern exactly** — both components are paint-time only (no layout mutation). The caller is responsible for setting up `AkarNodeContext` on the node before `Layout::compute` if intrinsic measurement is needed.
2. **Uses `resolved_to_attrs` and `resolved_to_metrics`** — unlike `label` which passes `None` for attrs, heading and paragraph pass resolved attrs to `set_text` so font weight and family are respected.
3. **`resolve_text_style` merges defaults with overrides** — three-layer resolution (theme defaults -> component defaults -> per-instance override) as specified in the epic.
4. **`#[repr(C)]` on `HeadingLevel`** — ready for C ABI coverage in Task 8.
5. **Paragraph defaults to `wrap: true`** — headings default to `wrap: false`.

## Tests added

### heading tests
- `zero_area_does_not_push_text` — verifies zero-size node produces no draw calls
- `heading_h1_renders_text` — verifies sized node produces at least 1 text call
- `heading_all_levels_render` — iterates H1-H4, each produces text output
- `heading_with_style_override` — verifies color override is accepted

### paragraph tests
- `zero_area_does_not_push_text` — verifies zero-size node produces no draw calls
- `paragraph_renders_text` — verifies sized node produces text output
- `paragraph_with_long_text` — longer string renders successfully
- `paragraph_with_style_override` — verifies color and font_size overrides are accepted

## Validation

- `cargo test --workspace` — all tests pass (174 in akar-components, 0 failures)
- `cargo clippy -p akar-components` — no new warnings (pre-existing warnings in akar-core/akar-layout only)
- All 8 new heading/paragraph tests pass

## Open risks/questions

None. The implementation is paint-time only; intrinsic text measurement integration will be wired up when these components are used in the webpage sample (Task 10) where callers set up `AkarNodeContext` before compute.

## Recommended next step

Task 4 (Link component) or Task 5 (Card lifecycle) — both depend on this work.