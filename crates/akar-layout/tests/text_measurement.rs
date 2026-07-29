//! Integration tests for `Layout::compute_with_text` and `default_measure_fn`.
//!
//! These tests exercise the text-measurement bridge between `akar-core`'s
//! `TextPipeline` and `akar-layout`'s Taffy measure callback. They mirror the
//! `~/Projects/taffy/examples/cosmic_text` pattern: text-bearing leaves carry
//! a `glyphon::Buffer` keyed by `AkarNodeContext::text_buffer_id`, and the
//! measure callback pulls the intrinsic size from that buffer.

use akar_core::{AkarCore, TextMeasureInput};
use akar_layout::{AkarNodeContext, Layout, NodeId, Style};

fn mock_core() -> AkarCore {
    AkarCore::mock()
}

fn text_buffer(core: &mut AkarCore, text: &str) -> u64 {
    core.text_pipeline.set_text(
        None,
        text,
        glyphon::Metrics::new(16.0, 24.0),
        None,
        None,
        None,
    )
}

fn text_leaf(layout: &mut Layout, core: &mut AkarCore, text: &str) -> NodeId {
    let buffer_id = text_buffer(core, text);
    layout.new_leaf_with_context(Style::default(), AkarNodeContext::text(buffer_id))
}

fn measure_with_known(
    core: &mut AkarCore,
    buffer_id: u64,
    known_width: Option<f32>,
    known_height: Option<f32>,
    available_width: Option<f32>,
) -> akar_core::TextMeasureResult {
    core.text_pipeline.measure_with_metadata(
        buffer_id,
        TextMeasureInput {
            known_width,
            known_height,
            available_width,
        },
    )
}

#[test]
fn single_line_intrinsic_size() {
    let mut core = mock_core();
    let buffer_id = text_buffer(&mut core, "Hello world");

    let size = measure_with_known(&mut core, buffer_id, None, None, None);

    assert!(
        size.width > 0.0,
        "intrinsic width should be > 0, got {}",
        size.width
    );
    assert!(
        size.height > 0.0,
        "intrinsic height should be > 0, got {}",
        size.height
    );
    assert!(
        size.height <= 24.0 + 0.5,
        "single-line height should be one line height, got {}",
        size.height
    );
}

#[test]
fn wrapped_multi_line_height() {
    let mut core = mock_core();
    let long_text = "The quick brown fox jumps over the lazy dog and then runs away into the night without looking back";
    let buffer_id = text_buffer(&mut core, long_text);

    let wide = measure_with_known(&mut core, buffer_id, None, None, Some(800.0));
    let narrow = measure_with_known(&mut core, buffer_id, None, None, Some(80.0));

    assert!(
        narrow.height > wide.height,
        "narrow width should produce taller wrapped text (wide H={}, narrow H={})",
        wide.height,
        narrow.height
    );
    assert!(
        narrow.width <= 80.0 + 0.5,
        "narrow intrinsic width should not exceed constraint, got {}",
        narrow.width
    );
}

#[test]
fn explicit_newlines_add_height() {
    let mut core = mock_core();
    let buffer_id = text_buffer(&mut core, "line1\nline2\nline3");

    let size = measure_with_known(&mut core, buffer_id, None, None, None);

    assert!(
        size.height >= 24.0 * 3.0 - 1.0,
        "three-line explicit newlines should yield >= 3x line height, got {}",
        size.height
    );
}

#[test]
fn constrained_width_produces_wrapped_height() {
    let mut core = mock_core();
    let buffer_id = text_buffer(
        &mut core,
        "the quick brown fox jumps over the lazy dog and keeps running forever",
    );

    let wide = measure_with_known(&mut core, buffer_id, None, None, Some(800.0));
    let narrow = measure_with_known(&mut core, buffer_id, None, None, Some(80.0));

    assert!(
        narrow.height > wide.height,
        "narrow width should produce taller wrapped text (wide H={}, narrow H={})",
        wide.height,
        narrow.height
    );
}

#[test]
fn sequential_compute_with_text_stable() {
    let mut core = mock_core();
    let mut layout = Layout::new();

    let leaf = text_leaf(&mut layout, &mut core, "navigation menu");
    let root = layout.new_with_children(
        Style {
            display: akar_layout::Display::Flex,
            size: akar_layout::Size {
                width: akar_layout::Dimension::length(200.0),
                height: akar_layout::Dimension::auto(),
            },
            ..Default::default()
        },
        &[leaf],
    );

    layout.compute_with_text(root, (Some(200.0), Some(400.0)), &mut core.text_pipeline);
    let r1 = layout.rect(leaf);

    layout.compute_with_text(root, (Some(400.0), Some(400.0)), &mut core.text_pipeline);
    let r2 = layout.rect(leaf);

    assert!(r1[2] > 0.0);
    assert!(r2[2] > 0.0);
    assert!(r1[3] > 0.0);
    assert!(r2[3] > 0.0);

    layout.compute_with_text(root, (Some(200.0), Some(400.0)), &mut core.text_pipeline);
    let r3 = layout.rect(leaf);
    assert_eq!(r1[2], r3[2], "width must be stable across recompute");
    assert_eq!(r1[3], r3[3], "height must be stable across recompute");
}

#[test]
fn empty_context_returns_zero_size() {
    let mut core = mock_core();
    let mut layout = Layout::new();

    let leaf = layout.new_leaf_with_context(Style::default(), AkarNodeContext::empty());
    let root = layout.new_with_children(
        Style {
            display: akar_layout::Display::Flex,
            flex_direction: akar_layout::FlexDirection::Row,
            size: akar_layout::Size {
                width: akar_layout::Dimension::length(200.0),
                height: akar_layout::Dimension::length(50.0),
            },
            ..Default::default()
        },
        &[leaf],
    );

    layout.compute_with_text(root, (Some(200.0), Some(50.0)), &mut core.text_pipeline);

    let r = layout.rect(leaf);
    assert!(r[2] >= 0.0, "leaf width must be non-negative, got {}", r[2]);
    assert!(
        r[3] >= 0.0,
        "leaf height must be non-negative, got {}",
        r[3]
    );
}

#[test]
fn empty_context_in_auto_parent_shrinks_to_zero() {
    let mut core = mock_core();
    let mut layout = Layout::new();

    let leaf = layout.new_leaf_with_context(Style::default(), AkarNodeContext::empty());
    let root = layout.new_with_children(
        Style {
            display: akar_layout::Display::Flex,
            size: akar_layout::Size {
                width: akar_layout::Dimension::length(200.0),
                height: akar_layout::Dimension::auto(),
            },
            ..Default::default()
        },
        &[leaf],
    );

    layout.compute_with_text(root, (Some(200.0), Some(400.0)), &mut core.text_pipeline);

    let r = layout.rect(leaf);
    assert_eq!(r[2], 0.0, "no text + auto width -> leaf shrinks to 0");
    assert_eq!(r[3], 0.0, "no text + auto height -> leaf shrinks to 0");
}

#[test]
fn default_measure_fn_respects_known_dimensions() {
    let mut core = mock_core();
    let buffer_id = text_buffer(&mut core, "fixed");

    let mut ctx = AkarNodeContext::text(buffer_id);
    let mut measure = akar_layout::default_measure_fn(&mut core.text_pipeline);

    let dummy_node: NodeId = layout_dummy_node();
    let size = measure(
        akar_layout::Size {
            width: Some(100.0),
            height: Some(50.0),
        },
        akar_layout::Size {
            width: akar_layout::AvailableSpace::MaxContent,
            height: akar_layout::AvailableSpace::MaxContent,
        },
        dummy_node,
        Some(&mut ctx),
        &Style::default(),
    );

    assert_eq!(size.width, 100.0, "known width must be authoritative");
    assert_eq!(size.height, 50.0, "known height must be authoritative");
}

fn layout_dummy_node() -> NodeId {
    let mut layout = Layout::new();
    layout.new_leaf(Style::default())
}

#[test]
fn measure_with_metadata_returns_zero_for_missing_buffer() {
    let mut core = mock_core();
    let result = measure_with_known(&mut core, 0xdead, None, None, Some(100.0));
    assert_eq!(result.width, 0.0);
    assert_eq!(result.height, 0.0);
}

#[test]
fn measure_with_metadata_paints_and_measures_use_same_buffer() {
    let mut core = mock_core();
    let buffer_id = text_buffer(&mut core, "stable buffer identity");

    let paint_call_id = core.text_pipeline.set_text(
        Some(buffer_id),
        "stable buffer identity",
        glyphon::Metrics::new(16.0, 24.0),
        Some(200.0),
        None,
        None,
    );
    assert_eq!(
        paint_call_id, buffer_id,
        "re-set with same id must reuse buffer"
    );

    let measure_size = measure_with_known(&mut core, buffer_id, None, None, Some(200.0));
    let paint_size = measure_with_known(&mut core, buffer_id, None, None, Some(200.0));

    assert_eq!(measure_size.width, paint_size.width);
    assert_eq!(measure_size.height, paint_size.height);
}
