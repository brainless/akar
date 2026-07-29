# Task for worker

You are a delegated subagent running from a fork of the parent session. Treat the inherited conversation as reference-only context, not a live thread to continue. Do not continue or answer prior messages as if they are waiting for a reply. Your sole job is to execute the task below and return a focused result for that task using your tools.

Task:
Implement Task 1 of Epic 020 in /Users/brainless/Projects/akar/.

**Read first** (do not skip):
- /Users/brainless/Projects/akar/epics/020-component-webpage-sample.md (full file, focus on "Task 1 — Component lifecycle and text measurement foundation")
- /Users/brainless/Projects/akar/DEVELOP.md
- /Users/brainless/Projects/akar/AGENTS.md
- /Users/brainless/Projects/akar/crates/akar-core/src/text_pipeline.rs (existing text shaping/measurement API)
- /Users/brainless/Projects/akar/crates/akar-layout/src/lib.rs (Layout::compute signature, AkarNodeContext, tree.compute_layout_with_measure signature)
- /Users/brainless/Projects/taffy/examples/cosmic_text/src/main.rs (canonical cosmic-text + Taffy measure_fn reference)
- /Users/brainless/Projects/glyphon/src/ (text shaping API: Buffer::set_size, set_metrics, layout_runs, metrics)

**Task 1 deliverables (from the Epic):**

1. Document the construct/compute/paint protocol in DEVELOP.md (add a new subsection under Architecture Notes titled "Component lifecycle: construct, compute, paint" — explain the three-phase lifecycle and that paint must never mutate the layout tree or replace children).
2. Extend AkarNodeContext or introduce the minimum equivalent context required for text measurement. The existing AkarNodeContext { text_buffer_id: u64 } already exists in akar-layout; it is currently unused. Decide whether to extend it (add `text: Option<String>`, `attrs: Option<glyphon::Attrs>`, `metrics: Option<glyphon::Metrics>`) or add a parallel text-only context. Whichever path keeps the API clean and matches the cosmic_text example is correct. Glyphon types must stay inside akar-core; do not re-export them from akar-layout. Prefer passing the text inputs (text, metrics, attrs, family/weight/width metadata) through the context if you go that route; you may also keep them in a TextMeasurementContext created by akar-core and referenced by buffer id from the layout node.
3. Integrate glyphon/cosmic-text measurement into Layout::compute consumers. The current Layout::compute signature is `compute<F>(root, available, measure_fn)` where F takes `(Size<Option<f32>>, Size<AvailableSpace>, NodeId, Option<&mut AkarNodeContext>, &Style) -> Size<f32>`. Provide a helper (e.g. `AkarCore::default_measure_fn()` or a method on AkarCore that returns the closure) that uses `TextPipeline::measure_with_metadata` to compute intrinsic sizes from text-bearing leaf nodes. text_pipeline currently exposes `measure(buffer_id, Option<f32>) -> Vec2`; extend or wrap it so measurement can also respect known dimensions, available width, family/weight, wrapping, and explicit newlines. Keep glyphon types inside akar-core.
4. Ensure measurement respects known dimensions, available width, metrics, family, weight, wrapping, and explicit newlines. Wrap: when width is constrained and text overflows, height must reflect multiple lines. Newlines: text containing `\n` must produce extra line height per newline. The buffer should be created with the same `Metrics`, family, and attrs as the paint path will use, so intrinsic width == paint-time width.
5. Ensure text measurement and painting share the same resolved style inputs. The same TextPipeline::set_text call must drive both; do not duplicate shaping. Reuse the same buffer_id both for measure and for paint. Provide a helper API on AkarCore (e.g. `prepare_text_node(buffer_id, text, metrics, max_width, attrs) -> u64` returning the buffer id, used by both measure_fn and the eventual paint code) so there is one place where text is shaped.
6. Add tests for:
   - single-line intrinsic size
   - wrapped multi-line height (a long line constrained to a narrow width should produce a height greater than one line)
   - explicit newlines (\n must add to height)
   - constrained width (intrinsic width never exceeds the constraint)
   - stable recomputation after width/style changes (run compute_layout twice with different sizes and assert correct dimensions each time)
   Place tests in crates/akar-layout/tests/text_measurement.rs or alongside the helper in akar-core, whichever fits the crate boundary. Do not introduce a GPU requirement; tests must run with `cargo test --workspace`.
7. Use ~/Projects/taffy/examples/cosmic_text as the primary design reference — keep the measure_fn shape aligned with cosmic_text's pattern (text buffer carried in node context, measure walks layout_runs to compute width and line-height total).

**Constraints (from AGENTS.md):**
- No live GPU in tests; use AkarCore::mock if a core helper is involved.
- No new unsafe code outside akar-c-api.
- No new dependencies without justification.
- Do not touch akar.h; it is cbindgen-generated.
- Do not change unrelated components in this task.

**Conventions (from DEVELOP.md):**
- No emojis in source or docs.
- No comments unless WHY is non-obvious.
- Follow existing module structure; prefer extending existing files (text_pipeline.rs, layout/src/lib.rs) before adding new ones.

**Deliverables:**
- Updated DEVELOP.md with the new "Component lifecycle: construct, compute, paint" section.
- Working text measurement integration in akar-core + akar-layout.
- All tests green. Run `cargo test --workspace --no-fail-fast` and confirm zero failures.
- Run `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` and confirm clean.
- A short summary back to me listing: files changed, public API surface added (signatures only, no full code), and the test names you added.
- Do NOT modify the Epic file's status — I will do that after review.
- Do NOT commit your changes — I will commit.

Stay tightly scoped to Task 1; do not start Task 2 or later.

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