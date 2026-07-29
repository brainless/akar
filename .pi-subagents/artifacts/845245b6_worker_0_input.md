# Task for worker

## Task 11 — Demo Isolation and Verification

**Epic:** epics/020-component-webpage-sample.md (read it fully for context)
**Key references:** DEVELOP.md, AGENTS.md, README.md

### What to implement

This is the verification task. Register new components with demo-rust component isolation, run tests, and capture screenshots.

### Sub-tasks

#### 1. Register new components with demo-rust isolation

Read `examples/demo-rust/src/main.rs` to understand how components are registered with `--list-components` and `--component`. Add registration for:
- `heading` (force H1 level)
- `paragraph` (force with some text)
- `link` (force hover state via script)
- `card` (force with header/footer)

Check how existing components like `button`, `badge`, `navbar` are registered and follow the same pattern.

#### 2. Add labels for scripted interactive verification

Register stable labels for interactive elements in the Akar page that can be targeted by `--script`. Check how mimo.rs registers labels and follow the pattern.

#### 3. Run tests and verify compilation

```bash
cargo fmt --check
cargo test --workspace
cargo clippy --workspace -- -D warnings  # note: pre-existing warnings in akar-core/akar-diff
cargo check --bin webpage-rust
cargo check --bin demo-rust
```

#### 4. Document screenshot commands

Add comments or documentation showing how to capture the Akar page at fixed viewports:

```bash
# Desktop viewport
cargo run --release --bin webpage-rust -- --site akar --width 1280 --height 900 --screenshot /tmp/akar-page.png --exit

# Narrower viewport
cargo run --release --bin webpage-rust -- --site akar --width 768 --height 900 --screenshot /tmp/akar-narrow.png --exit

# MiMo baseline (for comparison)
cargo run --release --bin webpage-rust -- --site mimo --width 1280 --height 900 --screenshot /tmp/mimo-page.png --exit
```

Note: Actual screenshot capture requires a GPU/display environment. In this task, we verify that the code compiles and tests pass. The visual verification will happen when the code is run in an environment with GPU access.

### Files to read first

1. `examples/demo-rust/src/main.rs` — component isolation registration
2. `examples/webpage-rust/src/sites/akar.rs` — the Akar page implementation
3. `examples/webpage-rust/src/sites/mimo.rs` — reference for label registration

### What to verify

- `cargo test --workspace` passes
- `cargo fmt --check` passes  
- `cargo check --bin webpage-rust` passes
- `cargo check --bin demo-rust` passes
- New components are registered with demo-rust isolation
- Stable labels exist for interactive elements

### Coding conventions
- Edition 2021, no emojis, no unnecessary comments

After implementing, run all the verification commands listed above.

---
**Output:**
Write your findings to exactly this path: /Users/brainless/Projects/akar/.pi-subagents/artifacts/outputs/845245b6/inline
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: checked
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, residual-risks, no-staged-files

Review gate: required by reviewer.

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
    },
    {
      "id": "criterion-2",
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