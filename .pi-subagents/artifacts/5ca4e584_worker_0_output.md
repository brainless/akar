All 17 test suites pass with zero failures. Final summary of what was changed:

**Files changed:**
- `crates/akar-components/src/text_style.rs` (new) — typography types, resolver, mapping helpers
- `crates/akar-components/src/theme.rs` — added `muted_content` and 6 heading/font-size tokens
- `crates/akar-components/src/lib.rs` — exports for the new module

**Public API surface added:**
```rust
pub enum FontFamily { SansSerif, Serif, Monospace }
pub enum FontWeight { Normal, Medium, Semibold, Bold }
pub enum TextAlign { Start, Center, End }
pub struct TextStyle { font_size, line_height, color, font_weight, font_family, align, wrap: Option<...> }
impl TextStyle { pub fn empty() -> Self }
pub(crate) struct ResolvedTextStyle { ...concrete fields... }
pub(crate) fn resolve_text_style(theme, defaults, override_style) -> ResolvedTextStyle
pub(crate) fn resolved_to_attrs(rt) -> glyphon::Attrs
pub(crate) fn resolved_to_metrics(rt) -> glyphon::Metrics

// In AkarTheme:
// pub muted_content: u32
// pub font_size_xl, font_size_xxl, font_size_heading_1..4: f32
```

**Tests added (12 in `text_style.rs`):**
- `theme_default_h1_uses_heading_1_size_and_bold`
- `partial_override_only_changes_listed_fields`
- `full_override_replaces_every_field`
- `wrap_true_propagates_from_override`
- `center_alignment_maps_to_text_align_center`
- `resolved_to_attrs_maps_family_and_weight`
- `font_weight_enum_maps_to_glyphon_weights`
- `resolved_to_metrics_uses_font_size_and_line_height`
- `theme_heading_sizes_match_spec`
- `theme_default_when_defaults_match_theme`
- `cascade_order_defaults_then_override`
- `defaults_with_no_override_propagates_defaults`

**Validation:**
- `cargo test --workspace --no-fail-fast`: 17 test suites, all pass (166 component tests including 12 new)
- `cargo fmt --check`: clean
- `cargo clippy --workspace --all-targets`: zero new warnings (the 2 lib warnings in `akar-components` are pre-existing in `modal.rs`)