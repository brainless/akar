# Epic 005: C ABI MVP

**Status:** Planned
**Goal:** Make `akar-c-api` minimally complete: fill the layout C API gap, generate a valid `akar.h`, and prove the surface with an integration test. No wgpu-native integration, no full C demo, no canvas ABI. Defer those explicitly.

**Prerequisite:** Epic 004 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean before Task 1 begins.

---

## Architecture Decision Records

### ADR-012: Simplified Layout Constructors, Not Raw Style Exposure

**Decision:** The C API exposes simplified node constructors — `akar_new_leaf`, `akar_new_flex_row`, `akar_new_flex_col` — rather than a direct mapping of taffy's `Style` struct.

**Rationale:** `taffy::Style` has ~20 fields covering flex, grid, sizing, padding, margin, and alignment. Exposing it directly over the C ABI would require a large `AkarStyle` struct in `akar.h` and a brittle field-by-field mapping that breaks every time taffy updates. The simplified constructors cover the 95% case (flex-grow leaves, fixed-size leaves, row and column containers) without binding to taffy's internal type system.

**Consequences:**
- C callers can replicate the page layout pattern (header, sidebar, main) using 2–3 constructor types.
- Power users who need full flex control must use the Rust API directly; this is expected and acceptable for v1.
- Adding more constructors (`akar_new_fixed_leaf`, `akar_new_flex_node_with_gap`) is additive and non-breaking.

---

### ADR-013: `akar_ctx_mock` for Testing; No GPU Required in Integration Tests

**Decision:** Expose `akar_ctx_mock() -> *mut AkarCtx` that creates a context using `AkarCore::mock()` (which does request a real headless wgpu adapter). Integration tests call `akar_ctx_mock` instead of `akar_ctx_new`, so they do not need a real window surface or texture format.

**Rationale:** `akar_ctx_new` requires a valid wgpu `Device*`, `Queue*`, and a surface format — none of which are available in a headless test environment without significant setup. `AkarCore::mock()` already exists and handles headless adapter creation. Exposing it as a C function lets integration tests exercise the full lifecycle (create, layout, compute, query, free) without a display.

**Consequences:**
- `akar_ctx_mock` is documented as "for testing only; not intended for production use."
- The function is always compiled in (not `#[cfg(test)]`) because integration tests link against the compiled library.
- A real rendering test (requiring a surface) is deferred.

---

### ADR-014: Deferred Items

**wgpu-native-based C demo (`examples/demo-c/`):** Deferred. A real C program that renders with akar requires the caller to supply wgpu device/queue pointers, which in practice means using wgpu-native (the C bindings to wgpu) for GPU and windowing setup. This is a significant integration task outside the scope of the ABI itself.

**Canvas C ABI (`akar_canvas_begin`, `akar_canvas_end`):** Deferred per ADR-011 from Epic 004. The canvas Rust API is stable, but the C surface for `CanvasPainter` (an owned buffer across the FFI boundary) requires an opaque heap-allocated handle with explicit alloc/free — a meaningful addition.

**Full `akar_page` helper:** Deferred. The page layout pattern (header, sidebar, main body) is common enough to deserve a convenience C function, but it returns multiple node IDs and requires a result struct. Deferred until the basic layout API is validated.

---

## Tasks

### Task 1: Layout C API

**Goal:** Six new `extern "C"` functions and one result struct that let a C caller build a node tree, run layout, and query resolved rects. All functions operate on `ctx.layout`.

**Add to `crates/akar-c-api/src/lib.rs`:**

```rust
#[repr(C)]
pub struct AkarRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Creates a flex-grow leaf node. Use for elements that should expand to fill
/// available space. `flex_grow = 0.0` for a fixed-size leaf (pair with
/// `akar_new_fixed_leaf` when you need explicit sizing).
#[no_mangle]
pub unsafe extern "C" fn akar_new_leaf(ctx: *mut AkarCtx, flex_grow: f32) -> u64 {
    use akar_layout::{Style, Size, Dimension};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        flex_grow,
        flex_shrink: 1.0,
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

/// Creates a fixed-size leaf node. `w` and `h` are in logical pixels.
/// Pass 0.0 for either dimension to use `auto` on that axis.
#[no_mangle]
pub unsafe extern "C" fn akar_new_fixed_leaf(ctx: *mut AkarCtx, w: f32, h: f32) -> u64 {
    use akar_layout::{Style, Size, Dimension, length};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        size: Size {
            width:  if w > 0.0 { length(w) } else { Dimension::auto() },
            height: if h > 0.0 { length(h) } else { Dimension::auto() },
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

/// Creates a flex-row container node (children laid out left-to-right).
#[no_mangle]
pub unsafe extern "C" fn akar_new_flex_row(ctx: *mut AkarCtx) -> u64 {
    use akar_layout::{Style, Display, FlexDirection, Size, Dimension};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        },
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

/// Creates a flex-column container node (children laid out top-to-bottom).
#[no_mangle]
pub unsafe extern "C" fn akar_new_flex_col(ctx: *mut AkarCtx) -> u64 {
    use akar_layout::{Style, Display, FlexDirection, Size, Dimension};
    let ctx = unsafe { &mut *ctx };
    let style = Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        },
        ..Default::default()
    };
    ctx.layout.new_leaf(style).into()
}

/// Adds `child` as the last child of `parent`.
#[no_mangle]
pub unsafe extern "C" fn akar_add_child(ctx: *mut AkarCtx, parent: u64, child: u64) {
    let ctx = unsafe { &mut *ctx };
    let parent_node: akar_layout::NodeId = parent.into();
    let child_node:  akar_layout::NodeId = child.into();
    ctx.layout.add_child(parent_node, child_node);
}

/// Runs layout computation for the tree rooted at `root`.
/// Call once per frame after setting up the tree, before calling component functions.
#[no_mangle]
pub unsafe extern "C" fn akar_layout_compute(
    ctx: *mut AkarCtx,
    root: u64,
    width: f32,
    height: f32,
) {
    use akar_layout::Size;
    let ctx = unsafe { &mut *ctx };
    let root_node: akar_layout::NodeId = root.into();
    ctx.layout.compute(
        root_node,
        (Some(width), Some(height)),
        |_, _, _, _, _| Size::ZERO,
    );
}

/// Returns the resolved screen-space rect for `node` after `akar_layout_compute`.
/// Returns a zero rect if called before compute or for an invalid node.
#[no_mangle]
pub unsafe extern "C" fn akar_layout_rect(ctx: *mut AkarCtx, node: u64) -> AkarRect {
    let ctx = unsafe { &mut *ctx };
    let node_id: akar_layout::NodeId = node.into();
    let [x, y, w, h] = ctx.layout.rect(node_id);
    AkarRect { x, y, w, h }
}
```

**Update `cbindgen.toml`** — add `AkarRect` to the export list:

```toml
[export]
include = ["AkarCtx", "AkarButtonResult", "AkarRect"]
```

**Acceptance criteria:** `cargo check -p akar-c-api` and `cargo clippy -p akar-c-api -- -D warnings` both pass clean.

---

### Task 2: `akar_ctx_mock`

**Goal:** A test-only context constructor that skips the wgpu surface setup so integration tests can create a valid `AkarCtx*` without a real display.

**Add to `crates/akar-c-api/src/lib.rs`:**

```rust
/// Creates a headless context suitable for testing layout and input logic.
/// The GPU pipeline is initialized against a headless wgpu adapter; no surface
/// or real window is required. Do not call `akar_end_frame` on a mock context.
#[no_mangle]
pub unsafe extern "C" fn akar_ctx_mock() -> *mut AkarCtx {
    use akar_components::AKAR_THEME_DARK;
    let core = AkarCore::mock();
    let layout = Layout::new();
    let theme = AKAR_THEME_DARK;
    Box::into_raw(Box::new(AkarCtx {
        core,
        layout,
        theme,
        device: std::ptr::null(),
        queue: std::ptr::null(),
    }))
}
```

**Acceptance criteria:** `cargo check -p akar-c-api` passes. The function is present in the compiled library and callable from integration tests.

---

### Task 3: Generate and Commit `akar.h`

**Goal:** Run `cargo build -p akar-c-api` to invoke `build.rs` and produce `akar.h` at the workspace root. Verify the header contains all exported types and functions, then commit it.

**Verification checklist for `akar.h`:**
- `AkarCtx` opaque struct present.
- `AkarButtonResult` struct with `clicked`, `hovered`, `pressed` bool fields.
- `AkarRect` struct with `x`, `y`, `w`, `h` float fields.
- All `akar_ctx_*`, `akar_begin_frame`, `akar_end_frame` declarations.
- All `akar_input_*`, `akar_set_mouse_pos`, `akar_push_*` declarations.
- All `akar_new_leaf`, `akar_new_fixed_leaf`, `akar_new_flex_row`, `akar_new_flex_col`, `akar_add_child`, `akar_layout_compute`, `akar_layout_rect` declarations.
- `akar_button` declaration.
- `akar_ctx_mock` declaration.
- Include guard `#ifndef AKAR_H` / `#define AKAR_H` / `#endif`.

**If cbindgen misses any function:** The cause is always one of: missing `#[no_mangle]` + `pub`, or the type is not in `[export] include`. Fix in source or `cbindgen.toml`, not in the header directly.

**Acceptance criteria:** `cargo build -p akar-c-api` succeeds. `akar.h` exists at workspace root and passes the checklist above. Header is committed.

---

### Task 4: Rust Integration Tests

**Goal:** A Rust integration test file in `crates/akar-c-api/tests/api.rs` that calls the C-exported functions through their Rust signatures, proving the ABI surface is correct and the layout pipeline works end-to-end.

**File:** `crates/akar-c-api/tests/api.rs`

```rust
use akar_c_api::{
    akar_ctx_mock, akar_ctx_free,
    akar_new_leaf, akar_new_fixed_leaf, akar_new_flex_row, akar_new_flex_col,
    akar_add_child, akar_layout_compute, akar_layout_rect,
    akar_input_begin, akar_set_mouse_pos, akar_push_mouse_button,
    akar_push_scroll, akar_input_end,
    akar_begin_frame,
};

#[test]
fn lifecycle_mock_create_free() {
    let ctx = unsafe { akar_ctx_mock() };
    assert!(!ctx.is_null());
    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn layout_flex_grow_leaf_fills_parent() {
    let ctx = unsafe { akar_ctx_mock() };

    let root  = unsafe { akar_new_flex_col(ctx) };
    let child = unsafe { akar_new_leaf(ctx, 1.0) };
    unsafe { akar_add_child(ctx, root, child) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let rect = unsafe { akar_layout_rect(ctx, child) };
    assert!((rect.w - 800.0).abs() < 1.0, "child.w = {}", rect.w);
    assert!((rect.h - 600.0).abs() < 1.0, "child.h = {}", rect.h);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn layout_fixed_leaf_respected() {
    let ctx = unsafe { akar_ctx_mock() };

    let root  = unsafe { akar_new_flex_row(ctx) };
    let child = unsafe { akar_new_fixed_leaf(ctx, 120.0, 40.0) };
    unsafe { akar_add_child(ctx, root, child) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let rect = unsafe { akar_layout_rect(ctx, child) };
    assert!((rect.w - 120.0).abs() < 1.0, "child.w = {}", rect.w);
    assert!((rect.h - 40.0).abs() < 1.0, "child.h = {}", rect.h);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn layout_two_flex_siblings_fill_row() {
    let ctx = unsafe { akar_ctx_mock() };

    let root = unsafe { akar_new_flex_row(ctx) };
    let left  = unsafe { akar_new_leaf(ctx, 1.0) };
    let right = unsafe { akar_new_leaf(ctx, 1.0) };
    unsafe { akar_add_child(ctx, root, left) };
    unsafe { akar_add_child(ctx, root, right) };
    unsafe { akar_layout_compute(ctx, root, 800.0, 600.0) };

    let lr = unsafe { akar_layout_rect(ctx, left) };
    let rr = unsafe { akar_layout_rect(ctx, right) };
    assert!((lr.w - 400.0).abs() < 1.0, "left.w = {}", lr.w);
    assert!((rr.w - 400.0).abs() < 1.0, "right.w = {}", rr.w);
    assert!((rr.x - 400.0).abs() < 1.0, "right.x = {}", rr.x);

    unsafe { akar_ctx_free(ctx) };
}

#[test]
fn input_feed_does_not_panic() {
    let ctx = unsafe { akar_ctx_mock() };
    unsafe {
        akar_begin_frame(ctx, 800, 600, 1.0);
        akar_input_begin(ctx);
        akar_set_mouse_pos(ctx, 100.0, 200.0);
        akar_push_mouse_button(ctx, 0, true);
        akar_push_scroll(ctx, 0.0, -3.0);
        akar_input_end(ctx);
    }
    unsafe { akar_ctx_free(ctx) };
}
```

**Note on imports:** These tests call `akar_c_api::` functions directly because integration tests in a `cdylib` crate can import from the crate's public namespace. The functions are `pub unsafe extern "C"` so they are accessible.

**Acceptance criteria:** `cargo test -p akar-c-api` passes all 5 tests. `cargo clippy -p akar-c-api -- -D warnings` passes clean.

---

## Acceptance Criteria for Epic 005

- [ ] `cargo clippy --workspace -- -D warnings` passes with zero errors.
- [ ] `cargo test -p akar-c-api` passes all integration tests (no GPU rendering required).
- [ ] `akar.h` is present at the workspace root and contains all exported types and function declarations.
- [ ] Layout C API: `akar_new_leaf`, `akar_new_fixed_leaf`, `akar_new_flex_row`, `akar_new_flex_col`, `akar_add_child`, `akar_layout_compute`, `akar_layout_rect` are all present and correct in `akar.h`.
- [ ] `akar_ctx_mock` is present in `akar.h` and returns a non-null pointer in tests.
- [ ] `AkarRect` struct is in `akar.h` with fields `x`, `y`, `w`, `h`.
- [ ] No changes to `akar-core`, `akar-layout`, `akar-components`, or `akar-winit`.
- [ ] No canvas ABI (`akar_canvas_begin`, `akar_canvas_end`) — deferred per ADR-014.
- [ ] No wgpu-native dependency added.
- [ ] Full `examples/demo-c/` with real rendering is not required — deferred per ADR-014.
