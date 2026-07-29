# Epic 020: Component-Based Webpage Sample

**Status:** Not Started
**Goal:** Rebuild the webpage sample using akar's component catalog instead of raw draw calls, proving that our building blocks are production-ready for real-world web-style layouts.

**Prerequisite:** Epic 019 is `Status: Done`.

---

## Agenda

Epic 019 successfully proved that a multimodal agent can build a full webpage layout from reference screenshots alone — the agent iterated on the MiMo landing page using screenshot feedback until every section, spacing, font, and color matched. The result is visually correct and was achieved with zero human intervention on the rendering code. That was the hard part, and it worked.

But Epic 019 took a deliberate shortcut: it bypassed the component catalog entirely. Every section — navbar, hero, cards, content blocks, footer — was built with raw `push_quad`/`push_text` calls. This means our components, the building blocks that downstream developers actually use, remain untested against a real web design.

**This epic's agenda is to shift focus from "can it look right?" to "do our components work?".** We rebuild the same page (or a comparable akar marketing page) using only `akar-components` widget functions: `navbar`, `heading`, `paragraph`, `button`, `badge`, `card`, `stat`, `link`, `separator`, `tab_bar`. Where a component can't express the design that Epic 019 achieved with raw calls, we close the gap — adding font-size overrides, color overrides, new text-bearing components, and composed card/link components.

The visual bar is what Epic 019 already demonstrated. We are not inventing new visual design; we are proving that the component API can reach the same quality. If a component can't match what raw calls achieved, that component needs to be fixed — not the page simplified.

### Why this matters

- **For agents:** A component catalog that can't build real pages is useless. This epic validates that agents can compose akar components into production layouts.
- **For the C ABI:** Every styling gap we close here (font-size, color overrides, text styles) directly improves what non-Rust developers can express through `akar.h`.
- **For the library:** Components that only work in a demo sidebar with fixed theme colors aren't ready for real applications. Stress-testing them against a webpage is the fastest way to find what's missing.

---

## Design Decisions

### TextStyle struct for text-bearing components

Rather than adding `font_size: Option<f32>` and `color: Option<u32>` params to every function, introduce a `TextStyle` struct:

```rust
#[derive(Clone, Copy, Debug, Default)]
pub struct TextStyle {
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub color: Option<u32>,
    pub font_weight: Option<glyphon::Weight>,
    pub font_family: Option<glyphon::FamilyOwned>,
}
```

Components that render text (`label`, `heading`, `paragraph`, `button`, `badge`, `stat`) accept `style: Option<&TextStyle>`. When `None`, they fall back to theme defaults. When `Some`, each `Some(field)` overrides the theme default, and each `None(field)` keeps the theme default. This keeps the common case (theme-driven) ergonomic while allowing per-instance overrides.

### Optional style params, not breaking changes

All new styling params are `Option<_>` with `None` preserving existing behavior. Existing call sites compile without changes. The new sample page is the first consumer of the override paths.

### Card as a composed component, not a layout primitive

The card component creates layout nodes (header, content, footer) and renders a `BoxStyle` background. It returns `CardSlots { header, content, footer }` — NodeIds where the caller adds children. This mirrors how `navbar` works with `NavbarSlots`. The caller places `heading`, `label`, `button`, etc. into the slots.

### No new components for hero/footer sections

Hero and footer are layout patterns, not reusable components. The sample page builds them by composing existing components (`heading`, `paragraph`, `button`, `separator`, `label`) with taffy flex layouts. This exercises the component catalog without creating single-use abstractions.

---

## Tasks

### Task 1 — TextStyle struct and theme font tokens

**Status:** Not Started

- Add `TextStyle` struct to `akar-components/src/text_style.rs` with optional fields: `font_size`, `line_height`, `color`, `font_weight`, `font_family`.
- Add `font_size_xl: f32` (20.0) and `font_size_xxl: f32` (24.0) tokens to `AkarTheme`. Update both `AKAR_THEME_DARK` and `AKAR_THEME_LIGHT` presets.
- Add a `resolve` helper that merges `TextStyle` overrides with theme defaults into `glyphon::Metrics` and `glyphon::Attrs`.
- Export `TextStyle` from `lib.rs`.
- Add unit tests: default TextStyle resolves to theme values; partial overrides merge correctly; full overrides take precedence.

### Task 2 — Heading component

**Status:** Not Started

- Add `heading` function to `akar-components/src/heading.rs`.
- Signature: `pub fn heading(core, layout, node_id, text, level: HeadingLevel, style: Option<&TextStyle>, theme)`
- `HeadingLevel` enum: `H1`, `H2`, `H3`, `H4`. Each maps to a default font size from the theme (H1 = `font_size_xxl * 1.5` ~36px, H2 = `font_size_xxl` ~24px, H3 = `font_size_lg` ~18px, H4 = `font_size_base` ~16px).
- Renders 1 text call. Font weight defaults to `Weight::BOLD` for H1-H3, `Weight::SEMIBOLD` for H4.
- `TextStyle` overrides apply per field; unset fields fall back to level defaults.
- Zero-area guard (existing convention).
- Export from `lib.rs` as `akar_heading`.
- Unit tests: zero area, each level renders correct font size, style overrides work.

### Task 3 — Paragraph component

**Status:** Not Started

- Add `paragraph` function to `akar-components/src/paragraph.rs`.
- Signature: `pub fn paragraph(core, layout, node_id, text, style: Option<&TextStyle>, theme)`
- Renders 1 text call with wrapping (`width` set from layout rect width).
- Default font size: `theme.font_size_base`. Default line height: `font_size * 1.5` (generous for body text readability).
- `TextStyle` overrides apply.
- Zero-area guard.
- Export from `lib.rs` as `akar_paragraph`.
- Unit tests: zero area, text wrapping at width, style overrides.

### Task 4 — Card component

**Status:** Not Started

- Add `card` function to `akar-components/src/card.rs`.
- Signature: `pub fn card(core, layout, node_id, style: &BoxStyle, theme) -> CardSlots`
- `CardSlots { header, body, footer }` — three NodeIds for caller to populate.
- Creates internal layout: header (flex column, padding), body (flex column, flex_grow 1.0, padding), footer (flex row, padding, border-top separator).
- Renders 1 background quad via `container()`.
- If the caller doesn't need a slot, they simply don't add children to it (zero-height, no rendering).
- Export from `lib.rs` as `akar_card`, `CardSlots`.
- Unit tests: zero area, slots created, background rendered, children can be added to slots.

### Task 5 — Link component

**Status:** Not Started

- Add `link` function to `akar-components/src/link.rs`.
- Signature: `pub fn link(core, layout, node_id, text, style: Option<&TextStyle>, theme) -> LinkResult`
- `LinkResult { clicked: bool, hovered: bool }`.
- Renders 1 text call. Default color: `theme.primary`. On hover: underline effect via a thin bottom-border quad.
- `TextStyle` overrides apply.
- Zero-area guard.
- Export from `lib.rs` as `akar_link`, `LinkResult`.
- Unit tests: zero area, hover state, click detection, style overrides.

### Task 6 — Button and Badge color overrides

**Status:** Not Started

- **Button:** Add optional `accent: Option<u32>` param. When `Some`, the button uses this color instead of `theme.primary` for all variants (Solid fill, Outline border, Ghost hover). Text color for Solid auto-contrasts (white for dark, black for light, simple luminance check). Default `None` preserves theme behavior.
- **Badge:** Add optional `accent: Option<u32>` param. When `Some`, badge background uses this color with white/black content auto-contrast. Default `None` preserves variant-driven theme colors.
- Update all existing call sites (demo-rust, webpage-rust) to pass `None` — no behavior change.
- Unit tests: override produces different fill color than theme default; None preserves original behavior.

### Task 7 — Navbar bg and height overrides

**Status:** Not Started

- Add optional `bg: Option<u32>` and `height: Option<f32>` params to `navbar`.
- When `bg` is `Some`, use it instead of `theme.base_200` for the panel background.
- When `height` is `Some`, set the node's height to that value instead of `auto`.
- Update existing call sites to pass `None`.
- Unit tests: custom bg renders different fill; custom height sets layout size.

### Task 8 — Separator color override

**Status:** Not Started

- Add optional `color: Option<u32>` param to `separator`.
- When `Some`, use it instead of `theme.base_300`. When `None`, preserve existing behavior.
- Update existing call sites to pass `None`.
- Unit tests: custom color renders different fill.

### Task 9 — Sample page implementation (`--site akar`)

**Status:** Not Started

- Add `Site::Akar` variant to the `webpage-rust` binary.
- Implement the akar marketing page using components:

**Page structure (top to bottom):**

1. **Navbar** — `akar_navbar` with logo text ("akar") in start slot, nav links ("Features", "Components", "GitHub") in end slot via `akar_link`. Custom bg to match page aesthetic.

2. **Hero section** — Flex column with centered alignment. `akar_heading` H1 ("GPU-accelerated UI for any language"), `akar_paragraph` subtitle, row of two `akar_button` calls (Solid "Get Started" + Outline "View on GitHub"). Custom `TextStyle` for the H1 (large serif font).

3. **Stats row** — Three `akar_stat` calls in a flex row: "30+ Components", "C ABI", "Immediate Mode".

4. **Feature cards** — Three `akar_card` calls in a flex row. Each card body contains an `akar_heading` H3 + `akar_paragraph` description. Cards use `BoxStyle::card(theme)`.

5. **Content section** — "Why akar?" heading via `akar_heading` H2, `akar_paragraph` body text, numbered items rendered with `akar_heading` H4 for item titles + `akar_paragraph` for descriptions.

6. **Component showcase** — `akar_badge` row (Primary, Success, Warning, Info variants with accent overrides), `akar_button` variants row, `akar_tab_bar` for interactive demo.

7. **Footer** — Flex row of 3-4 link columns. Each column: `akar_heading` H4 section title + `akar_label`/`akar_link` items. `akar_separator` above. Copyright `akar_label` at bottom.

- Each section is a taffy flex column/row with proper gaps and padding.
- The page scrolls via `scroll_area_begin`/`scroll_area_end`.
- Register the akar page in the CLI `--site` flag dispatcher.

### Task 10 — Verification and screenshots

**Status:** Not Started

- Run `cargo run --release --bin webpage-rust -- --site akar --screenshot /tmp/akar-page.png --exit` and verify the page renders correctly.
- Run `cargo run --release --bin webpage-rust -- --site mimo --screenshot /tmp/mimo-page.png --exit` to verify the existing MiMo page is unaffected.
- Run `cargo test --workspace` to verify no regressions.
- Run `cargo clippy --workspace -- -D warnings` to verify no new warnings.
- Capture component isolation screenshots for new/updated components via `demo-rust --component`.

---

## Cross-Cutting Styling Summary

| Feature | Currently | After This Epic |
|---|---|---|
| Font size override | Hardcoded per component (`font_size_base`) | `TextStyle.font_size: Option<f32>` on text-bearing components |
| Font weight | Default only (Normal) | `TextStyle.font_weight: Option<Weight>` — bold headings, semibold labels |
| Font family | Default only (SansSerif) | `TextStyle.font_family: Option<FamilyOwned>` — serif headings |
| Line height | Hardcoded `* 1.2` | `TextStyle.line_height: Option<f32>` — tighter headings, looser body |
| Text color | Theme-derived or explicit `color: u32` param | `TextStyle.color: Option<u32>` — unified override path |
| Background color | Theme-derived only (most components) | Per-component `Option<u32>` overrides on button, badge, navbar, separator |
| Card styling | `BoxStyle` exists, no component | New `card` component with header/body/footer slots |
| Heading text | Manual `set_text`+`push_text` | New `heading` component with level-based sizing |
| Body text | Manual `set_text`+`push_text` | New `paragraph` component with wrapping |
| Link text | Not available | New `link` component with hover underline |

---

## Acceptance Criteria

- [ ] `cargo run --release --bin webpage-rust -- --site akar --screenshot /tmp/akar-page.png --exit` produces a clean screenshot of the component-based akar landing page.
- [ ] The page uses `akar_navbar`, `akar_heading`, `akar_paragraph`, `akar_button`, `akar_badge`, `akar_card`, `akar_stat`, `akar_link`, `akar_separator`, and `akar_tab_bar` — no raw `push_quad`/`push_text` outside scroll container internals.
- [ ] Headings render in serif font at correct sizes (H1 ~36px, H2 ~24px, H3 ~18px).
- [ ] Body text wraps correctly at container width.
- [ ] Buttons show primary (Solid) and secondary (Outline) variants with distinct colors.
- [ ] Badges show multiple color variants.
- [ ] Feature cards display with shadow, border, and rounded corners.
- [ ] Footer has multi-column link layout with separator above.
- [ ] Vertical scrolling works with mouse wheel.
- [ ] `--site mimo` page renders identically to before (no regressions).
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.
- [ ] All new components have unit tests (zero-area guard, style overrides, interaction states).
- [ ] All existing component call sites compile with `None` for new optional params.
