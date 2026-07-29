Files are modified (not staged), which is what the task requires ("I will commit"). Let me summarize.

Summary in upstream format:

Implemented Task 1 of Epic 020. Deliverables met per the contract.

**Files changed:**
- `DEVELOP.md` — added "Component lifecycle: construct, compute, paint" section under Architecture Notes.
- `crates/akar-core/src/lib.rs` — re-exported `TextMeasureInput` and `TextMeasureResult`.
- `crates/akar-core/src/text_pipeline.rs` — added `TextMeasureInput`/`TextMeasureResult` types and `TextPipeline::measure_with_metadata`; `measure(Vec2)` now wraps the new method. Added 4 unit tests.
- `crates/akar-layout/Cargo.toml` — added `akar-core` dep + `glyphon` dev-dep.
- `crates/akar-layout/src/lib.rs` — added `AkarNodeContext::text()`/`empty()` constructors, `Layout::compute_with_text`, and `default_measure_fn` free function.
- `crates/akar-layout/tests/text_measurement.rs` — 10 integration tests.

**Public API surface added (signatures only):**
- `akar_core::TextMeasureInput { known_width, known_height, available_width }`
- `akar_core::TextMeasureResult { width, height }`
- `akar_core::TextPipeline::measure_with_metadata(buffer_id, TextMeasureInput) -> TextMeasureResult`
- `akar_layout::AkarNodeContext::text(u64) -> Self` and `AkarNodeContext::empty() -> Self`
- `akar_layout::Layout::compute_with_text(root, (Option<f32>, Option<f32>), &mut TextPipeline)`
- `akar_layout::default_measure_fn<'a>(&'a mut TextPipeline) -> impl FnMut(...) + 'a`

**Tests added:**
- `crates/akar-core/src/text_pipeline.rs` (in `#[cfg(test)] mod tests`):
  - `measure_with_metadata_known_dimensions_short_circuit`
  - `measure_with_metadata_wrap_increases_height`
  - `measure_with_metadata_explicit_newlines_count`
  - `measure_with_metadata_missing_buffer_returns_zero`
- `crates/akar-layout/tests/text_measurement.rs`:
  - `single_line_intrinsic_size`
  - `wrapped_multi_line_height`
  - `explicit_newlines_add_height`
  - `constrained_width_produces_wrapped_height`
  - `sequential_compute_with_text_stable`
  - `empty_context_returns_zero_size`
  - `empty_context_in_auto_parent_shrinks_to_zero`
  - `default_measure_fn_respects_known_dimensions`
  - `measure_with_metadata_returns_zero_for_missing_buffer`
  - `measure_with_metadata_paints_and_measures_use_same_buffer`

**Validation:** `cargo test --workspace` → 276 tests pass, 0 failures. `cargo fmt --check` passes. `cargo clippy` on my changed files introduces no new warnings; pre-existing clippy lints in `crates/akar-core/src/screenshot.rs` and `crates/akar-layout/src/lib.rs` (unrelated `length(5.0)` style) are present at HEAD and out of scope.

**Open risks:** None significant. The `AkarCore::mock()` path uses a real `wgpu` adapter at test time, which matches the existing test pattern in the crate; CI already supports this. Three pre-existing clippy/warnings are visible alongside the new tests but they exist in untouched code.

**Recommended next step:** Move to Task 2.