## Task 11 — Demo Isolation and Verification: Implementation Complete

### Changes Made

#### 1. Registered new components with demo-rust isolation (`examples/demo-rust/src/main.rs`)

Added four new component variants to the `Component` enum and fully implemented their isolation support:

- **`heading`** — Renders an H1 heading with serif font override. Forced initial state: none needed.
- **`paragraph`** — Renders a paragraph with wrapping text. Forced initial state: none needed.
- **`link`** — Renders an interactive link. Stores `LinkResult` in AppState. Forced initial state: none needed.
- **`card`** — Renders a composed card with header/body/footer slots using `akar_card_layout` and `akar_card`. Uses `CardLayout::with_header_footer`. Forced initial state: none needed.

For each component:
- Added new `NodeId` fields to `AppState` (heading_node, paragraph_node, link_node, card_root, card_header, card_body, card_footer)
- Added `card_slots: CardSlots` and `link_result: LinkResult` to AppState
- Implemented `from_name()` parsing for all four names
- Added names to `names()` list (now 15 components total)
- Implemented `render()` for each — calling the appropriate `akar_heading`, `akar_paragraph`, `akar_link`, `akar_card` functions
- Implemented `prepare_isolated_layout()` with fixed viewport sizes
- Implemented `force_state_initial()` as no-op for all four (they have no interactive state to force)
- Created layout nodes during `resumed()` initialization
- Used `akar_card_layout` to create stable card slots with header and footer

Added imports for: `akar_card`, `akar_card_layout`, `akar_heading`, `akar_link`, `akar_paragraph`, `CardLayout`, `CardSlots`, `CardStyle`, `FontFamily`, `HeadingLevel`, `LinkResult`, `TextStyle`

#### 2. Added labels for scripted interactive verification

**demo-rust labels** (for `--script` targeting):
- `heading` → heading_node
- `paragraph` → paragraph_node
- `link` → link_node
- `card` → card_root

**webpage-rust akar site labels** (`examples/webpage-rust/src/sites/akar.rs`):
- Navigation: `navbar_logo`, `nav_features`, `nav_components`, `nav_github`
- Hero: `hero_h1`, `hero_subtitle`, `cta_solid`, `cta_outline`
- Stats: `stat_0`, `stat_1`, `stat_2`
- Cards: `card_0`, `card_1`, `card_2`
- Why section: `why_h2`, `why_h4_0` through `why_h4_3`
- Showcase: `badge_0` through `badge_6`, `btn_solid`, `btn_outline`, `btn_ghost`, `tab_bar`
- Footer: `footer_separator`, `copyright`

### Verification Commands Run

| Command | Result |
|---|---|
| `cargo fmt --check` | Passed |
| `cargo test --workspace` | Passed (342 tests, 0 failures) |
| `cargo check --bin demo-rust` | Passed (warnings are pre-existing float literal fallback) |
| `cargo check --bin webpage-rust` | Passed (pre-existing dead_code warning on `Site::name`) |

Note: `cargo clippy --workspace -- -D warnings` was not run separately as the task notes pre-existing warnings in akar-core/akar-diff. Both binaries compile cleanly with only pre-existing warnings.

### Screenshot Commands (for GPU environment)

```bash
# Desktop viewport
cargo run --release --bin webpage-rust -- --site akar --width 1280 --height 900 --screenshot /tmp/akar-page.png --exit

# Narrower viewport
cargo run --release --bin webpage-rust -- --site akar --width 768 --height 900 --screenshot /tmp/akar-narrow.png --exit

# MiMo baseline
cargo run --release --bin webpage-rust -- --site mimo --width 1280 --height 900 --screenshot /tmp/mimo-page.png --exit

# Isolated heading
cargo run --release --bin demo-rust -- --component heading --screenshot /tmp/heading.png --exit

# Isolated paragraph
cargo run --release --bin demo-rust -- --component paragraph --screenshot /tmp/paragraph.png --exit

# Isolated link (interactive — use script for hover)
cargo run --release --bin demo-rust -- --component link --script /tmp/hover_link.txt --screenshot /tmp/link-hover.png

# Isolated card
cargo run --release --bin demo-rust -- --component card --screenshot /tmp/card.png --exit
```

### Residual Risks
- Actual screenshot capture requires GPU/display environment. Compilation-only verification was done.
- Pre-existing float literal fallback warnings in demo-rust (121 warnings). These are not from our changes — the existing code already uses bare float literals extensively.

### Files Changed
- `examples/demo-rust/src/main.rs` — +290/-10 lines
- `examples/webpage-rust/src/sites/akar.rs` — +26 lines