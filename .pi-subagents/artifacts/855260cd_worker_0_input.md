# Task for worker

## Task 9 — Modularize webpage-rust

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md

### Current state

`examples/webpage-rust/src/main.rs` is a monolithic 1198-line file containing the MiMo webpage implementation. It needs to be split into modules.

### What to implement

Split the monolithic binary into application/dispatch code and per-site modules:

```
examples/webpage-rust/src/
  main.rs       — CLI parsing, dispatch, rendering loop
  app.rs        — shared application state and rendering
  site.rs       — Site trait or enum
  sites/
    mod.rs      — site registry/discovery
    mimo.rs     — MiMo page implementation
    akar.rs     — Akar page (empty stub for now, Task 10 will fill it)
```

### Requirements from the epic

1. Preserve the MiMo implementation and rendering behavior while moving it.
2. Add `Site::Akar` and update CLI discovery/error output.
3. Give each site its own stable layout and interaction state.
4. Add deterministic `--width` and `--height` capture options, or define a fixed screenshot viewport.
5. Build each site's layout once and recompute it on resize; do not create component nodes every frame.

### Implementation approach

1. **Read `examples/webpage-rust/src/main.rs` fully** to understand the current structure.
2. Identify the MiMo-specific code vs shared code.
3. Create a `Site` trait or enum that both sites implement.
4. Move MiMo code to `sites/mimo.rs`.
5. Create a stub `sites/akar.rs` with placeholder content.
6. Add CLI support for `--site akar` (currently only supports `--site mimo` or defaults to it).
7. Add `--width` and `--height` CLI options for deterministic viewport capture.
8. Extract shared rendering/state code to `app.rs`.

### Site trait pattern

```rust
pub trait Site {
    fn name(&self) -> &str;
    fn build_layout(&mut self, layout: &mut Layout, core: &mut AkarCore, width: f32, height: f32);
    fn paint(&mut self, core: &mut AkarCore, layout: &Layout, theme: &AkarTheme);
}
```

Or use an enum if simpler:
```rust
pub enum SiteKind {
    Mimo(MimoSite),
    Akar(AkarSite),
}
```

### CLI changes

Current CLI likely uses `--site mimo` or defaults. Add:
- `--site akar` support
- `--width <PX>` (default 1280)
- `--height <PX>` (default 900)

### Files to read first

1. `examples/webpage-rust/src/main.rs` — the full monolithic file
2. `examples/webpage-rust/Cargo.toml` — dependencies
3. `examples/demo-rust/src/main.rs` — reference for CLI patterns and rendering loop

### Key constraints

- Preserve MiMo rendering behavior exactly.
- Layout must be built once and recomputed on resize (not rebuilt every frame).
- Each site has its own stable layout and interaction state.
- The `--site akar` stub should compile and render something minimal (even just a blank page or "Coming soon" text).

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments

After implementing, run `cargo check --bin webpage-rust` and `cargo fmt` to verify compilation.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/855260cd/inline
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
`criteriaSatisfied[].status` must be exactly one of: satisfied, not-satisfied, not-applicable.
`commandsRun[].result` must be exactly one of: passed, failed, not-run.
`manualNotes` and `notes` are optional strings; an empty string means no note and does not satisfy `manual-notes` evidence.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```