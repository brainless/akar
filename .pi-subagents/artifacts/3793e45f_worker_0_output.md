## Task 6 — Navbar Lifecycle Correction: Implementation Complete

### What changed

Split the monolithic `navbar` function into construct/paint lifecycle matching the `card_layout`/`card` pattern:

**`navbar_layout` (construction):**
- Creates start, center, end child slot nodes with flex styles
- Configures root node as flex row with center alignment
- Sets children on root
- Does NOT draw anything
- Returns `NavbarSlots { start, center, end }`

**`navbar` (paint):**
- Takes `&AkarCore`, `&Layout` (immutable), `NodeId`, `&NavbarStyle`
- Reads rect from layout, applies zero-area guard
- Draws background quad with border, corner radii
- Does NOT mutate layout, does NOT create child nodes

**`navbar_combined` (backward compat):**
- Same signature as the old `navbar(core, layout, node_id, theme)`
- Internally calls `navbar_layout` + `navbar` paint
- Exported as `akar_navbar_combined`

**`NavbarStyle` type added:**
- `background: u32`, `border_color: u32`, `border_width: f32`, `corner_radii: [f32; 4]`
- `NavbarStyle::default(theme)` derives values from theme tokens

### Files changed

1. `crates/akar-components/src/navbar.rs` — Full rewrite with split API, `NavbarStyle`, `navbar_combined`, and 7 tests
2. `crates/akar-components/src/lib.rs` — Updated exports to include `akar_navbar_layout`, `akar_navbar_combined`, `NavbarStyle`
3. `examples/demo-rust/src/main.rs` — Updated import and both call sites from `akar_navbar` to `akar_navbar_layout`

### Tests added (7 total, all in `navbar::tests`)

| Test | Verifies |
|---|---|
| `navbar_layout_creates_three_slots` | start, center, end are distinct NodeIds |
| `navbar_layout_does_not_draw` | Construction emits zero draw calls |
| `navbar_paint_draws_background` | Sized node produces exactly one quad |
| `zero_area_does_nothing` | Zero-size node produces no quads |
| `navbar_preserves_caller_layout` | Paint does not modify computed layout rects |
| `children_can_be_added_to_slots` | Slot children lay out in correct left-to-right order |
| `navbar_combined_does_layout_and_paint` | Backward-compat entry produces slots AND background |

### Validation

- `cargo test -p akar-components`: 192 passed, 0 failed
- `cargo fmt --check`: clean
- `cargo check --bin demo-rust`: compiles (warnings only, pre-existing)
- `cargo check --bin webpage-rust`: compiles (warnings only, pre-existing)

### Migration note

Demo-rust already separated construction from paint in practice (called `akar_navbar` once inside `if is_none()`, then painted background via separate `akar_container`). The migration was straightforward: swap `akar_navbar(core, layout, id, theme)` to `akar_navbar_layout(layout, id, theme)` in the two call sites.