# Epic 002 — Improvements Before Epic 003

These are real issues found during post-epic review. All are small fixes. None require architectural changes. Fix these before starting Epic 003.

---

## 1. Clippy failures break CI (4 errors)

`cargo clippy --workspace -- -D warnings` fails. These must pass before any CI pipeline is added.

### 1a. Missing `Default` impls — three structs

`InputState`, `DrawList`, and `Layout` each have `pub fn new() -> Self` but no `impl Default`. Clippy's `new_without_default` lint fails on all three.

**Fix:** Add `impl Default` for each by delegating to `Self::new()`.

Files:
- `crates/akar-core/src/input.rs` — after `impl InputState { pub fn new()... }`
- `crates/akar-core/src/draw_list.rs` — after `impl DrawList { pub fn new()... }`
- `crates/akar-layout/src/lib.rs` — after `impl Layout { pub fn new()... }`

Pattern (same for all three):
```rust
impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
```

### 1b. Manual slice size calculation — `quad_pipeline.rs:135`

```rust
// current — triggers manual_slice_size_calculation
let required_size = quads.len() * mem::size_of::<QuadCall>();

// fix
let required_size = std::mem::size_of_val(quads);
```

File: `crates/akar-core/src/quad_pipeline.rs:135`

---

## 2. `str::from_utf8_unchecked` in C ABI is unsound

`crates/akar-c-api/src/lib.rs:222`:
```rust
let label_str = unsafe { std::str::from_utf8_unchecked(label_bytes) };
```

If a C caller passes non-UTF-8 bytes, this is undefined behavior. The C ABI is an explicit trust boundary.

**Fix:** Use `std::str::from_utf8` and return a no-op result on invalid input:
```rust
let Ok(label_str) = std::str::from_utf8(label_bytes) else {
    return AkarButtonResult { clicked: false, hovered: false, pressed: false };
};
```

---

## 3. `dim_color` and `lighten_color` are identical functions

`crates/akar-components/src/button.rs:29–43`: Both functions have identical bodies — they multiply each RGB channel by `factor` and clamp. The only difference is the name and the value of `factor` passed by the caller (`0.8` for dim, `1.1` for lighten).

**Fix:** Remove `lighten_color`, keep `dim_color`, rename it to `scale_color`. Update all call sites. This eliminates the dead-code smell and makes the intent explicit at the call site.

```rust
fn scale_color(c: u32, factor: f32) -> u32 {
    let r = (((c >> 24) & 0xFF) as f32 * factor).min(255.0) as u32;
    let g = (((c >> 16) & 0xFF) as f32 * factor).min(255.0) as u32;
    let b = (((c >> 8) & 0xFF) as f32 * factor).min(255.0) as u32;
    let a = c & 0xFF;
    (r << 24) | (g << 16) | (b << 8) | a
}
```

Call sites become self-documenting: `scale_color(theme.primary, 0.8)` (dim on press), `scale_color(theme.primary, 1.1)` (lighten on hover).

---

## Acceptance Criteria

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero errors.
- [ ] Passing invalid UTF-8 bytes into `akar_button` returns `AkarButtonResult { false, false, false }` instead of UB.
- [ ] `button.rs` has one `scale_color` function instead of two identical ones.
- [ ] All existing tests continue to pass (`cargo test --workspace`).
