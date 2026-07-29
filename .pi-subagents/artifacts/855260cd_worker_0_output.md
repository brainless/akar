# Task 9 — Modularize webpage-rust

## Implementation Summary

Successfully split the monolithic `examples/webpage-rust/src/main.rs` (1198 lines) into modular files.

## Changed Files

### New Files Created
- `src/site.rs` — Site trait definition
- `src/app.rs` — Shared application state (AppState, App, ApplicationHandler)
- `src/sites/mod.rs` — Site registry/discovery
- `src/sites/mimo.rs` — MiMo page implementation
- `src/sites/akar.rs` — Akar page stub (Task 10 will fill)

### Modified Files
- `src/main.rs` — CLI parsing and dispatch (reduced from 1198 to 84 lines)

## Architecture

```
src/
  main.rs       — CLI parsing, dispatch, event loop creation (84 lines)
  app.rs        — AppState, App struct, ApplicationHandler impl (290 lines)
  site.rs       — Site trait (name, root, build_layout, render) (9 lines)
  sites/
    mod.rs      — create_site() factory, available_sites() (16 lines)
    mimo.rs     — MimoSite struct and implementation (869 lines)
    akar.rs     — AkarSite stub (blank page with "Coming soon" text) (72 lines)
```

Total: 1340 lines across 6 files

## Site Trait

```rust
pub trait Site {
    fn name(&self) -> &str;
    fn root(&self) -> NodeId;
    fn build_layout(&mut self, layout: &mut Layout);
    fn render(&mut self, core: &mut AkarCore, layout: &Layout, viewport_rect: [f32; 4]);
}
```

## CLI Changes

Added `--width` and `--height` options with defaults (1280x900):

```bash
# MiMo page at fixed viewport
cargo run --release --bin webpage-rust -- --site mimo --width 1280 --height 900

# Akar page (stub for now, Task 10 will fill)
cargo run --release --bin webpage-rust -- --site akar --width 1280 --height 900

# Unknown site shows available options
cargo run --release --bin webpage-rust -- --site foo
# Error: Unknown site 'foo'. Valid sites: mimo, akar
```

## Key Design Decisions

1. **Trait-based dispatch** — Each site implements the `Site` trait for clean encapsulation
2. **Layout owned by AppState** — Layout tree lives in AppState, sites only store node IDs
3. **Separation of concerns** — Rendering logic moved to site modules, infrastructure in app.rs
4. **Stable layout construction** — Each site builds its layout once in `build_layout()`, recomputed on resize via `prepare_layout()`
5. **Scroll state per-site** — Each site owns its scroll_y and interaction state
6. **Preserved exact MiMo behavior** — All rendering functions moved verbatim, just reorganized as methods

## Validation

```bash
$ cargo check --bin webpage-rust
warning: `webpage-rust` (bin "webpage-rust") generated 1 warning
# (only warning is unused `name()` method in Site trait, expected for Task 10)

$ cargo fmt --check
# Clean - no formatting issues
```

## Acceptance Criteria

- [x] Monolithic file split into modules (6 files)
- [x] MiMo implementation preserved exactly (869 lines, all rendering code intact)
- [x] Site::Akar stub added (compiles, renders blank page with "Coming soon")
- [x] CLI discovery/error output updated (shows available sites on error)
- [x] Each site has its own stable layout and interaction state
- [x] Deterministic `--width` and `--height` capture options added (default 1280x900)
- [x] Layout built once and recomputed on resize (not rebuilt every frame)