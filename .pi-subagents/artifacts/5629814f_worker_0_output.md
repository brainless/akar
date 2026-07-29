## Task 7 Implementation Complete

### Changes Made

#### 1. Color contrast helper (`color.rs`)
- Added `pub fn contrast_color(bg: u32) -> u32` using WCAG relative luminance formula
- Added `contrast_color_dark_bg_returns_white` and `contrast_color_light_bg_returns_black` tests
- Made `color` module public (was `pub(crate)`) and exported `contrast_color` from `lib.rs`

#### 2. Button styled API (`button.rs`)
- Added `ButtonStyle` struct with `fill`, `hover_fill`, `pressed_fill`, `border_color`, `content_color`, `text_style` fields
- Added `button_styled()` function that accepts `ButtonStyle` and applies custom colors while preserving variant behavior as fallback
- Refactored `button()` to delegate to `button_styled()` with `ButtonStyle::empty()`
- Added `styled_uses_custom_fill` and `styled_preserves_zero_area` tests

#### 3. Badge styled API (`badge.rs`)
- Added `BadgeStyle` struct with `fill`, `border_color`, `content_color`, `text_style` fields
- Added `badge_styled()` function with custom color overrides
- Refactored `badge()` to delegate to `badge_styled()` with `BadgeStyle::empty()`
- Added `styled_uses_custom_fill` and `styled_preserves_zero_area` tests

#### 4. Separator styled API (`separator.rs`)
- Added `SeparatorStyle` struct with `color`, `thickness` fields
- Added `separator_styled()` function with custom color/thickness
- When `thickness` is `Some`, centers the line within the layout rect; when `None`, uses full rect (preserving original behavior)
- Refactored `separator()` to delegate to `separator_styled()` with `SeparatorStyle::empty()`
- Added `styled_uses_custom_color` and `styled_preserves_zero_area` tests

#### 5. Stat styled API (`stat.rs`)
- Added `StatStyle` struct with `title_color`, `value_color`, `description_color`, `title_text_style`, `value_text_style`, `description_text_style` fields
- Added `stat_styled()` function with custom text color overrides
- Refactored `stat()` to delegate to `stat_styled()` with `StatStyle::empty()`
- Added `styled_uses_custom_fill` and `styled_preserves_zero_area` tests

#### 6. Tab bar styled API (`tabs.rs`)
- Added `TabBarStyle` struct with `active_color`, `inactive_color`, `indicator_color` fields
- Added `tab_bar_styled()` function with custom color overrides for active/inactive tabs and indicator
- Refactored `tab_bar()` to delegate to `tab_bar_styled()` with `TabBarStyle::empty()`
- Added `styled_uses_custom_fill` and `styled_preserves_zero_area` tests

#### 7. Exports (`lib.rs`)
- Updated all module exports to include new types and `_styled` functions
- Made `color` module public with `contrast_color` re-export

### Files Changed
- `crates/akar-components/src/color.rs` - Added `contrast_color` function and tests
- `crates/akar-components/src/button.rs` - Added `ButtonStyle`, `button_styled`, tests
- `crates/akar-components/src/badge.rs` - Added `BadgeStyle`, `badge_styled`, tests
- `crates/akar-components/src/separator.rs` - Added `SeparatorStyle`, `separator_styled`, tests
- `crates/akar-components/src/stat.rs` - Added `StatStyle`, `stat_styled`, tests
- `crates/akar-components/src/tabs.rs` - Added `TabBarStyle`, `tab_bar_styled`, tests
- `crates/akar-components/src/lib.rs` - Updated exports

### Validation
- `cargo test -p akar-components`: 204 passed, 0 failed
- `cargo fmt -p akar-components`: Clean (no formatting issues)
- All existing tests continue to pass (original APIs unchanged)
- All new styled API tests pass
- No new compiler warnings introduced

### Implementation Notes
- All `_styled` functions follow the same pattern: accept a style struct with `Option` fields, use `unwrap_or`/`unwrap_or_else` to fall back to theme defaults
- Original API functions delegate to styled variants with empty style structs, ensuring backward compatibility
- The `button_styled` test required setting mouse position outside the button rect to avoid the hover state (mock input defaults to (0,0))