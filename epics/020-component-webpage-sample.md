# Epic 020: Component-Based Webpage Sample

**Status:** In Progress
**Goal:** Rebuild the webpage sample using akar's component catalog instead of raw draw calls, proving that the component, layout, text, and C ABI building blocks are production-ready for real-world web-style layouts.

**Prerequisite:** Epic 019 is `Status: Done`.

**Progress:**
- [x] Task 1 — Component lifecycle and text measurement foundation
- [x] Task 2 — Typography types, theme tokens, and resolution
- [x] Task 3 — Heading and paragraph components
- [x] Task 4 — Link component
- [x] Task 5 — Card lifecycle and composition
- [x] Task 6 — Navbar lifecycle correction
- [x] Task 7 — Styled button, badge, separator, stat, and tab bar APIs
- [x] Task 8 — C ABI coverage
- [x] Task 9 — Modularize webpage-rust

---

## Agenda

Epic 019 proved that a multimodal agent can build a full webpage layout from reference screenshots alone. The MiMo landing page was built with raw `push_quad` and `push_text` calls and refined through screenshot feedback until its layout, spacing, typography, and colors matched the reference.

This epic shifts the question from "can akar draw the result?" to "can downstream developers build it through akar's public component APIs?"

Build a comparable akar marketing page using `akar-components`: navbar, heading, paragraph, button, badge, card, stat, link, separator, and tab bar. Where the component catalog cannot express the required design, improve the component API rather than simplifying the page.

This is also the first integrated test of four systems that must agree:

1. Components create stable, reusable structure without mutating the layout tree during paint.
2. Text participates in Taffy measurement so wrapping determines layout height.
3. Styling is expressive without exposing glyphon types as akar's public contract.
4. New capabilities are available through both Rust and the generated C ABI.

The visual bar remains the quality demonstrated by Epic 019. The new page does not need to copy MiMo's design, but it must look like a deliberate production landing page rather than an expanded component gallery.

### Why this matters

- **For agents:** Agents need composable APIs with stable identities, predictable layout, and screenshot-verifiable behavior.
- **For downstream developers:** The public component catalog must support real application and webpage composition without raw renderer access.
- **For the C ABI:** Styling and typography must be expressible through language-neutral akar-owned types.
- **For the library:** Real composition exposes lifecycle, intrinsic sizing, wrapping, scrolling, and styling gaps that isolated demo panels do not.

---

## Design Decisions

### Component lifecycle: construct, compute, paint

Components that own internal layout structure must not create or replace nodes during paint. The frame lifecycle is:

1. Construct the layout tree and component slots.
2. Compute Taffy layout.
3. Paint components using resolved rectangles.

Slot-bearing components expose separate construction and paint operations:

```rust
pub fn card_layout(layout: &mut Layout, node_id: NodeId, options: &CardLayout) -> CardSlots;
pub fn card(core: &mut AkarCore, layout: &Layout, node_id: NodeId, style: &CardStyle);
```

Navbar adopts the same split. Construction happens once, or only when the application deliberately rebuilds its layout. Paint functions never add children, replace children, or overwrite caller-owned layout properties.

Stable caller-owned `NodeId`s are used every frame so widget IDs and text buffers remain stable.

### Text participates in layout

Text-bearing leaves use `AkarNodeContext` and the Taffy measurement callback to return intrinsic sizes. Measurement uses the available width, resolved typography, and glyphon/cosmic-text shaping.

Paragraph height is derived from wrapped text. Heading, label, link, button, badge, and stat text can provide intrinsic sizes where their parent layout does not provide explicit dimensions.

Text measurement and painting use the same metrics, family, weight, width, wrapping behavior, and DPI assumptions. Text is not shaped outside the active scissor when callers use the existing visibility and virtualization APIs.

### Akar-owned typography types

Public component APIs do not expose glyphon types. Introduce akar-owned types that resolve to glyphon internally and map cleanly to C:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontFamily {
    #[default]
    SansSerif,
    Serif,
    Monospace,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontWeight {
    #[default]
    Normal,
    Medium,
    Semibold,
    Bold,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TextStyle {
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub color: Option<u32>,
    pub font_weight: Option<FontWeight>,
    pub font_family: Option<FontFamily>,
    pub align: Option<TextAlign>,
    pub wrap: Option<bool>,
}
```

Named font families are deferred until akar has an explicit font-registration and C string ownership design.

Style resolution has three layers:

```text
theme semantic default -> component or variant default -> per-instance override
```

The resolved internal style contains concrete glyphon metrics and attributes. Rust and C APIs share the same semantics.

### Additive styled APIs

Appending positional `Option` parameters is still a Rust source break and scales poorly. Preserve existing common-case functions and add style/options entry points where practical:

```rust
akar_button(..., variant, theme)
akar_button_styled(..., &ButtonStyle, theme)
```

New components may accept an options struct from their first release. Options structs own future expansion and avoid repeated signature-wide migrations.

If an existing function must change, document it as a deliberate pre-alpha migration rather than describing it as non-breaking.

### Layout ownership

The caller owns size, min/max size, margin, and placement of the component root node. Component layout constructors configure only component-owned internal slots and documented internal behavior.

Navbar and card must not replace the complete root `Style`. Page-specific height belongs to the page's Taffy style or a layout options struct used during construction, not to the paint function.

### Card composition

Card is a composed component, not a primitive layout container. It has body and optional header/footer slots:

```rust
pub struct CardSlots {
    pub header: Option<NodeId>,
    pub body: NodeId,
    pub footer: Option<NodeId>,
}
```

`CardLayout` selects which optional slots exist and owns internal direction, gaps, and padding. Empty optional slots are not created. Separators are painted only between populated regions.

`CardStyle` owns background, border, corner radius, shadow, and separator appearance. It may be constructed from `BoxStyle::card(theme)`.

### Page sections remain composition patterns

Hero, feature grid, content section, showcase, and footer are page layout patterns, not new library components. The sample builds them from components and Taffy flex/grid layout.

### Caller-owned interaction state

Link navigation, active tab, scrolling, and other application state remain caller-owned. Components report interaction immediately and paint the post-interaction state in the same frame where applicable.

---

## Tasks

### Task 1 — Component lifecycle and text measurement foundation

**Status:** Done

- Document the construct/compute/paint protocol in `DEVELOP.md`.
- Extend `AkarNodeContext` or introduce the minimum equivalent context required for text measurement.
- Integrate glyphon/cosmic-text measurement into `Layout::compute` consumers.
- Ensure measurement respects known dimensions, available width, metrics, family, weight, wrapping, and explicit newlines.
- Ensure text measurement and painting share the same resolved style inputs.
- Add tests for single-line intrinsic size, wrapped multi-line height, explicit newlines, constrained width, and stable recomputation after width/style changes.
- Use the local Taffy `examples/cosmic_text` implementation as the primary design reference.

### Task 2 — Typography types, theme tokens, and resolution

**Status:** Done

- Add akar-owned `FontFamily`, `FontWeight`, `TextAlign`, and `TextStyle` types.
- Add semantic heading tokens to `AkarTheme`:
  - `font_size_xl: 20.0`
  - `font_size_xxl: 24.0`
  - `font_size_heading_1`
  - `font_size_heading_2`
  - `font_size_heading_3`
  - `font_size_heading_4`
- Add any required semantic muted-content token rather than hardcoding secondary paragraph colors.
- Update dark and light presets.
- Add an internal resolved-text-style type and three-layer resolver.
- Map akar font types to glyphon only inside the implementation.
- Export the akar-owned public types from `lib.rs`.
- Add tests for theme defaults, component defaults, partial overrides, full overrides, alignment, wrapping, and font mapping.

### Task 3 — Heading and paragraph components

**Status:** Done

- Add `HeadingLevel::{H1, H2, H3, H4}`.
- Add `heading` with level defaults and `TextStyle` overrides.
- Default H1-H3 to bold and H4 to semibold.
- Add `paragraph` with wrapping enabled and a default line height of `font_size * 1.5`.
- Make both components participate in intrinsic text measurement.
- Keep explicit node width/height constraints authoritative.
- Add zero-area guards.
- Export as `akar_heading` and `akar_paragraph`.
- Add tests for every heading level, style resolution, wrapping, measured multi-line height, explicit newlines, alignment, and zero width/height.

### Task 4 — Link component

**Status:** Done

- Add `link` returning `LinkResult { clicked, hovered, pressed }`.
- Default to the theme's primary color with `TextStyle` overrides.
- Size and position the hover underline from measured glyph geometry rather than the full node width.
- Use the foreground quad layer where necessary so the underline is visible with global text rendering.
- Keep URL opening and navigation caller-owned.
- Participate in intrinsic text measurement and default to no wrapping.
- Export as `akar_link`.
- Add tests for zero area, hover, press, click, measured underline width, and style overrides.

### Task 5 — Card lifecycle and composition

**Status:** Done

- Add `CardLayout`, `CardStyle`, and `CardSlots`.
- Add a build-time `card_layout` operation that creates stable body and requested optional slots.
- Add a paint-time `card` operation that does not mutate `Layout`.
- Define padding, gap, separator, and empty-slot behavior explicitly.
- Render background, border, corner radius, and shadow through existing component/core primitives.
- Verify card shadows under a scroll-area scissor.
- Export `akar_card_layout`, `akar_card`, and associated types.
- Add tests for stable slots, optional slot omission, child layout, background, separators, shadow fields, and zero area.

### Task 6 — Navbar lifecycle correction

**Status:** Done

- Split navbar structure construction from painting.
- Preserve caller-owned root size, padding, gap, min/max constraints, and placement.
- Keep stable start, center, and end slots.
- Introduce `NavbarStyle` for background, border, and other paint properties.
- Keep page-specific height in the caller's Taffy layout.
- Retain or provide a clear migration for the existing API.
- Add tests for stable slots, caller-style preservation, custom background, child composition, and zero area.

### Task 7 — Styled button, badge, separator, stat, and tab bar APIs

**Status:** Done

- Add `ButtonStyle` with optional text, fill, hover fill, pressed fill, border, and content-color overrides.
- Preserve the existing button common-case entry point and add `akar_button_styled`.
- Add `BadgeStyle` with text, fill, border, and content-color overrides while preserving semantic variants.
- Preserve the existing badge common-case entry point and add `akar_badge_styled`.
- Add a shared color-contrast helper in `color.rs` for opaque custom fills. Explicit content color always wins.
- Add `SeparatorStyle` with color, thickness, and orientation/inset behavior as needed by the page.
- Apply the same `TextStyle` resolution path to stat text.
- Add the minimum tab-bar style options needed by the showcase and keep active state caller-owned.
- Add tests that common-case APIs preserve current rendering and styled APIs affect every documented state.
- Update demo call sites only where a deliberate migration requires it.

### Task 8 — C ABI coverage

**Status:** Done

- Add C-compatible representations for font family, font weight, text alignment, text style, heading level, and component style/options types.
- Define explicit presence semantics for optional C fields; do not expose Rust `Option` layout directly.
- Add C functions for heading, paragraph, link, card construction/painting, and styled variants of updated components.
- Define null-style behavior as equivalent to theme/component defaults.
- Keep all component logic in `akar-components`; the C crate delegates.
- Add C integration tests for default and overridden typography, link interaction, card slots, and styled component colors.
- Regenerate `akar.h` with cbindgen; never edit it manually.
- Add ABI size/alignment or generated-header compile checks where appropriate.

### Task 9 — Modularize `webpage-rust`

**Status:** Done

- Split the monolithic binary into application/dispatch code and per-site modules, for example:

```text
src/
  main.rs
  app.rs
  site.rs
  sites/
    mod.rs
    mimo.rs
    akar.rs
```

- Preserve the MiMo implementation and rendering behavior while moving it.
- Add `Site::Akar` and update CLI discovery/error output.
- Give each site its own stable layout and interaction state.
- Add deterministic `--width` and `--height` capture options, or define a fixed screenshot viewport.
- Build each site's layout once and recompute it on resize; do not create component nodes every frame.

### Task 10 — Akar marketing page

**Status:** Not Started

- Implement `--site akar` using component functions only. Raw `push_quad` and `push_text` are forbidden in the Akar site module.
- Use a dedicated scrollable content layout or a single scoped scroll transform so components can query `layout.rect(node)` directly. Do not manually rewrite every component rectangle.
- Retain caller-owned `scroll_y` and active-tab state.
- Register stable labels for scripted interactive verification.

Page structure:

1. **Navbar** — logo in the start slot and Features, Components, and GitHub links in the end slot.
2. **Hero** — centered H1, wrapped subtitle, and Solid/Outline calls to action. Use a large serif H1 override suitable for a marketing page.
3. **Stats** — three stat components for component count, C ABI, and immediate mode.
4. **Feature cards** — three composed cards containing headings and paragraphs.
5. **Why akar** — H2, body copy, and numbered H4/paragraph items.
6. **Component showcase** — semantic badge variants without overrides, one custom-styled badge, button variants, and an interactive tab bar.
7. **Footer** — separator, multi-column heading/link composition, and copyright label.

- Use responsive wrapping or column changes so the page remains coherent at a narrower viewport.
- Ensure empty slots, long paragraphs, and footer content contribute correctly to computed content height.

### Task 11 — Demo isolation and verification

**Status:** Not Started

- Register heading, paragraph, link, and card with demo-rust component isolation.
- Capture idle isolation screenshots for all new components.
- Capture scripted link hover and button hover/press states.
- Capture tab state before and after a click.
- Capture the Akar page at a fixed desktop viewport and at least one narrower viewport.
- Script scrolling to the footer and capture the result.
- Use `--dump-frame` to verify card shadows and link underlines are present and not incorrectly scissor-culled.
- Capture a MiMo baseline before implementation and compare the final MiMo screenshot pixel-exactly at the same viewport.
- Run:

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo run --release --bin webpage-rust -- --site akar --width 1280 --height 900 --screenshot /tmp/akar-page.png --exit
cargo run --release --bin webpage-rust -- --site mimo --width 1280 --height 900 --screenshot /tmp/mimo-page.png --exit
```

---

## Cross-Cutting Change Summary

| Area | Currently | After This Epic |
|---|---|---|
| Component lifecycle | Some components create slots and paint in one call | Slot construction precedes Taffy compute; paint is layout-read-only |
| Text layout | Webpage measurement returns `Size::ZERO` | Text provides intrinsic width and wrapped height to Taffy |
| Typography API | Component-local glyphon metrics | Akar-owned semantic types with a shared resolver |
| Styled APIs | Fixed theme values or growing positional signatures | Additive options/style entry points |
| Heading/body text | Manual `set_text` and `push_text` | Measured heading and paragraph components |
| Link text | No link component | Intrinsically sized interactive link with glyph-width underline |
| Card | `BoxStyle` only | Stable composed slots plus separate paint |
| Navbar | Creates slots, overwrites root style, and paints together | Stable construction, preserved caller layout, separate style/paint |
| Color overrides | Theme-derived only in many components | State-aware component styles with explicit content colors |
| C ABI | No coverage for new component styling | C-compatible types, functions, tests, and regenerated header |
| Webpage sample | Monolithic MiMo-only raw rendering | Per-site modules and component-only Akar page |
| Capture | Monitor-dependent | Deterministic viewport plus responsive and interactive captures |

---

## Acceptance Criteria

- [ ] Component lifecycle is documented and slot-bearing components do not mutate layout during paint.
- [ ] Text measurement participates in Taffy layout and wrapped paragraphs determine their computed height.
- [ ] Public component typography APIs use akar-owned types rather than glyphon types.
- [ ] Existing common-case button and badge behavior is preserved through retained or clearly migrated APIs.
- [ ] `--site akar` uses navbar, heading, paragraph, button, badge, card, stat, link, separator, and tab bar components.
- [ ] The Akar site module contains no raw `push_quad` or `push_text` calls.
- [ ] Headings render with the intended semantic sizes and weights; the hero uses a large serif H1.
- [ ] Body text wraps correctly and following content is positioned from its measured height.
- [ ] Links expose hover/press/click state and underline only their measured text width.
- [ ] Buttons and badges support explicit state-aware styling and readable custom content colors.
- [ ] Cards have stable optional slots, border, radius, separator behavior, and visible unclipped shadows.
- [ ] Navbar preserves caller-owned layout properties and exposes stable start/center/end slots.
- [ ] Active tab and scroll position remain caller-owned and update correctly.
- [ ] The page remains coherent at the fixed desktop and narrower verification viewports.
- [ ] Scrolling reaches a correctly laid-out multi-column footer.
- [ ] Equivalent new capabilities are exposed through the generated C ABI and covered by C integration tests.
- [ ] `akar.h` is regenerated with cbindgen and compiles in the C integration suite.
- [ ] The fixed-viewport MiMo screenshot is pixel-identical to its pre-change baseline.
- [ ] New components are available through demo-rust isolation and have idle/interactive screenshots where applicable.
- [ ] Unit tests cover zero width/height, style resolution, interaction, intrinsic measurement, wrapping, stable slots, and common-case compatibility.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo clippy --workspace -- -D warnings` passes.

---

## Explicit Deferrals

- Named/custom font-family registration and C string ownership.
- Accessibility roles and navigation semantics beyond the existing v1 punt.
- Automatic URL opening or host navigation callbacks.
- Responsive breakpoint abstractions in `akar-components`; the sample may use caller-owned viewport logic.
- General retained component trees. Only stable layout construction and immediate-mode painting are required.
