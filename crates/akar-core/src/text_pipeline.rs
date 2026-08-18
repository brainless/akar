use crate::font_source::{
    FontLoadError, FontRequest, FontSelection, FontSource, TextPipelineConfig,
};
use crate::TextCall;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextGeometry {
    pub caret: Option<[f32; 4]>,
    pub selection: Vec<[f32; 4]>,
}

/// Physical caret direction for keyboard navigation. This is intentionally
/// *not* `AkarDirection`: caret motion must follow the shaped paragraph's own
/// bidi direction (an RTL app can contain an LTR value, and vice versa), not
/// app-level layout direction. See `TextPipeline::move_caret`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretMotion {
    Left,
    Right,
}

/// A caret's position in shaped visual order.
///
/// `line` identifies the source-text line, `run` identifies a wrapped visual
/// run within that line, and `x` preserves bidi-aware ordering within the run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShapedCaretPosition {
    pub line: usize,
    pub run: usize,
    pub x: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMeasureInput {
    pub known_width: Option<f32>,
    pub known_height: Option<f32>,
    pub available_width: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMeasureResult {
    pub width: f32,
    pub height: f32,
}

pub struct TextPipeline {
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    #[allow(dead_code)]
    cache: glyphon::Cache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    buffers: HashMap<u64, glyphon::Buffer>,
    next_id: u64,
    font_families: Vec<String>,
    font_sources: Vec<(Arc<Vec<u8>>, u32)>,
}

const LOCALE: &str = "en-US";

impl TextPipeline {
    /// Builds a pipeline with the default configuration (bundled fonts, no
    /// system scan).
    pub fn new_default(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        Self::new(device, queue, surface_format, TextPipelineConfig::default())
    }

    /// # Panics
    ///
    /// Panics if the resulting font database contains no faces. akar requires
    /// at least one font: cosmic-text panics with an opaque "no default font
    /// found" on the first shaping call otherwise. This happens when `akar-core`
    /// is built with `--no-default-features` (no `bundled-font`) and the caller
    /// supplies neither `TextPipelineConfig::fonts` nor
    /// `FontSource::BundledPlusSystemScan`.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        config: TextPipelineConfig,
    ) -> Self {
        let mut db = glyphon::fontdb::Database::new();
        if config.font_source == FontSource::BundledPlusSystemScan {
            db.load_system_fonts();
        }
        for bytes in config.fonts {
            db.load_font_source(glyphon::fontdb::Source::Binary(Arc::new(bytes)));
        }
        assert!(
            !db.is_empty(),
            "akar: no fonts available. akar-core was built without the default \
             `bundled-font` feature, so the caller must supply a font via \
             TextPipelineConfig::fonts (or opt into \
             FontSource::BundledPlusSystemScan) before any text is shaped."
        );

        let font_system = glyphon::FontSystem::new_with_locale_and_db(LOCALE.to_string(), db);
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let viewport = glyphon::Viewport::new(device, &cache);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, surface_format);
        let renderer =
            glyphon::TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            renderer,
            buffers: HashMap::new(),
            next_id: 1,
            font_families: Vec::new(),
            font_sources: Vec::new(),
        }
    }

    /// Loads font bytes (TTF/OTF/TTC/OTC) into the live font database and
    /// returns a handle for the loaded family.
    ///
    /// v1 accepts only sources containing exactly one distinct family: a
    /// collection spanning several families is rejected with
    /// `FontLoadError::MultipleFamilies` and nothing is loaded.
    pub fn load_font_bytes(&mut self, data: Vec<u8>) -> Result<u32, FontLoadError> {
        if data.is_empty() {
            return Err(FontLoadError::InvalidFontData(
                "empty byte slice".to_string(),
            ));
        }

        if let Some((_, handle)) = self
            .font_sources
            .iter()
            .find(|(source, _)| source.as_slice() == data.as_slice())
        {
            return Ok(*handle);
        }

        let source = Arc::new(data);
        let db = self.font_system.db_mut();
        let ids = db.load_font_source(glyphon::fontdb::Source::Binary(source.clone()));
        if ids.is_empty() {
            return Err(FontLoadError::InvalidFontData(
                "no parsable font face in data".to_string(),
            ));
        }

        // Only the primary name counts: fontdb lists every localized alias of
        // one family in `FaceInfo::families` (English first), so counting all
        // of them would reject a single-face font that merely carries a
        // non-Latin alias.
        let mut families: Vec<String> = Vec::new();
        for id in ids.iter() {
            let Some(face) = db.face(*id) else { continue };
            let Some((name, _)) = face.families.first() else {
                continue;
            };
            if !families.iter().any(|existing| existing == name) {
                families.push(name.clone());
            }
        }

        if families.len() == 1 {
            let handle = self.register_family(families.remove(0));
            self.font_sources.push((source, handle));
            return Ok(handle);
        }

        let db = self.font_system.db_mut();
        for id in ids.iter() {
            db.remove_face(*id);
        }
        if families.is_empty() {
            Err(FontLoadError::EmptyFontSource)
        } else {
            Err(FontLoadError::MultipleFamilies(families.len()))
        }
    }

    fn register_family(&mut self, name: String) -> u32 {
        if let Some(index) = self
            .font_families
            .iter()
            .position(|existing| *existing == name)
        {
            return index as u32;
        }
        self.font_families.push(name);
        (self.font_families.len() - 1) as u32
    }

    /// Resolves a handle returned by `load_font_bytes` to its family name.
    pub fn family_name(&self, handle: u32) -> Option<&str> {
        self.font_families.get(handle as usize).map(String::as_str)
    }

    pub fn set_text(
        &mut self,
        buffer_id: Option<u64>,
        text: &str,
        metrics: glyphon::Metrics,
        width: Option<f32>,
        height: Option<f32>,
        attrs: Option<glyphon::Attrs>,
    ) -> u64 {
        set_text_impl(
            &mut self.font_system,
            &mut self.buffers,
            &mut self.next_id,
            buffer_id,
            text,
            metrics,
            width,
            height,
            attrs.as_ref().unwrap_or(&glyphon::Attrs::new()),
            None,
        )
    }

    /// Shapes text from an owned, `Copy` font request. `FontSelection::Named`
    /// handles are resolved against this pipeline's family registry here, so
    /// callers never hold a borrow of a registry name.
    ///
    /// An unknown handle falls back to the generic sans-serif family.
    ///
    /// `align`, when set, is passed straight to `Buffer::set_text` so
    /// cosmic-text aligns the shaped text within the full component-width
    /// buffer itself. Callers must not *also* apply a manual x-offset for
    /// `Start`/`End` alignment on top of this — see
    /// `text_style::resolve_align` and `crates/akar-components/src/paragraph.rs`
    /// for the intended direction-aware mapping. Passing `None` here leaves
    /// cosmic-text's own paragraph-direction-derived default alignment in
    /// effect.
    #[allow(clippy::too_many_arguments)]
    pub fn set_text_styled(
        &mut self,
        buffer_id: Option<u64>,
        text: &str,
        metrics: glyphon::Metrics,
        width: Option<f32>,
        height: Option<f32>,
        font: FontRequest,
        align: Option<glyphon::cosmic_text::Align>,
    ) -> u64 {
        let family = resolve_font_family(&self.font_families, font.family);
        let attrs = glyphon::Attrs::new()
            .family(family)
            .weight(glyphon::Weight(font.weight));

        set_text_impl(
            &mut self.font_system,
            &mut self.buffers,
            &mut self.next_id,
            buffer_id,
            text,
            metrics,
            width,
            height,
            &attrs,
            align,
        )
    }

    pub fn remove_buffer(&mut self, buffer_id: u64) {
        self.buffers.remove(&buffer_id);
    }

    pub fn measure(&mut self, buffer_id: u64, width: Option<f32>) -> glam::Vec2 {
        let result = self.measure_with_metadata(
            buffer_id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: width,
            },
        );
        glam::Vec2::new(result.width, result.height)
    }

    pub fn measure_with_metadata(
        &mut self,
        buffer_id: u64,
        input: TextMeasureInput,
    ) -> TextMeasureResult {
        let Some(buffer) = self.buffers.get_mut(&buffer_id) else {
            return TextMeasureResult::default();
        };

        if let (Some(w), Some(h)) = (input.known_width, input.known_height) {
            return TextMeasureResult {
                width: w,
                height: h,
            };
        }

        let width_constraint = input
            .known_width
            .or(input.available_width)
            .map(|w| w.max(0.0));

        let current = buffer.size();
        if width_constraint != current.0 {
            buffer.set_size(&mut self.font_system, width_constraint, None);
            buffer.shape_until_scroll(&mut self.font_system, false);
        }

        let mut max_w: f32 = 0.0;
        let mut total_height: f32 = 0.0;
        for run in buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
            total_height = run.line_top + run.line_height;
        }

        let width = match input.known_width {
            Some(w) => w,
            None => max_w,
        };
        let height = match input.known_height {
            Some(h) => h,
            None => total_height,
        };

        TextMeasureResult { width, height }
    }

    pub fn geometry(
        &self,
        buffer_id: u64,
        text: &str,
        cursor: usize,
        anchor: usize,
    ) -> TextGeometry {
        self.buffers
            .get(&buffer_id)
            .map_or_else(TextGeometry::default, |buffer| {
                text_geometry(buffer, text, cursor, anchor)
            })
    }

    /// Moves `offset` (akar's global byte offset into the shaped buffer's
    /// text) one physical `Left`/`Right` step, using cosmic-text's own
    /// `Buffer::cursor_motion`. cosmic-text picks `Previous`/`Next` based on
    /// the shaped line's base bidi direction, so this follows the
    /// *paragraph's* direction rather than any app-level layout direction —
    /// correct for an LTR value inside an RTL app and vice versa.
    ///
    /// Returns `offset` unchanged if `buffer_id` is unknown or cosmic-text
    /// can't resolve a motion (e.g. an empty buffer).
    pub fn move_caret(&mut self, buffer_id: u64, offset: usize, motion: CaretMotion) -> usize {
        let Some(buffer) = self.buffers.get_mut(&buffer_id) else {
            return offset;
        };
        let cursor = global_offset_to_cursor(&buffer.lines, offset);
        let cosmic_motion = match motion {
            CaretMotion::Left => glyphon::cosmic_text::Motion::Left,
            CaretMotion::Right => glyphon::cosmic_text::Motion::Right,
        };
        let Some((new_cursor, _)) =
            buffer.cursor_motion(&mut self.font_system, cursor, None, cosmic_motion)
        else {
            return offset;
        };
        cursor_to_global_offset(&buffer.lines, new_cursor)
    }

    /// The caret's shaped source-line, wrapped-run, and x position. Source
    /// line and run order make multiline comparisons stable; x retains the
    /// bidi-aware visual order for endpoints in the same run.
    pub fn caret_position(&self, buffer_id: u64, offset: usize) -> Option<ShapedCaretPosition> {
        let buffer = self.buffers.get(&buffer_id)?;
        let cursor = global_offset_to_cursor(&buffer.lines, offset);
        let mut run_in_line = 0;
        for run in buffer.layout_runs() {
            if run.line_i != cursor.line {
                continue;
            }
            if let Some(x) = caret_x(&run, cursor.index) {
                return Some(ShapedCaretPosition {
                    line: cursor.line,
                    run: run_in_line,
                    x,
                });
            }
            run_in_line += 1;
        }
        None
    }

    /// The shaped-glyph x position of the caret at `offset`.
    pub fn caret_x(&self, buffer_id: u64, offset: usize) -> Option<f32> {
        self.caret_position(buffer_id, offset)
            .map(|position| position.x)
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_width: u32,
        viewport_height: u32,
        scale_factor: f32,
        text_calls: &[TextCall],
    ) -> Result<(), glyphon::PrepareError> {
        self.viewport.update(
            queue,
            glyphon::Resolution {
                width: viewport_width,
                height: viewport_height,
            },
        );

        let text_areas: Vec<glyphon::TextArea<'_>> = text_calls
            .iter()
            .filter_map(|call| {
                let buffer = self.buffers.get(&call.buffer_id)?;
                let color = glyphon::Color::rgba(
                    (call.color[0] * 255.0).round() as u8,
                    (call.color[1] * 255.0).round() as u8,
                    (call.color[2] * 255.0).round() as u8,
                    (call.color[3] * 255.0).round() as u8,
                );
                Some(glyphon::TextArea {
                    buffer,
                    left: call.x * scale_factor,
                    top: call.y * scale_factor,
                    scale: scale_factor,
                    bounds: glyphon::TextBounds {
                        left: call.clip[0] as i32,
                        top: call.clip[1] as i32,
                        right: (call.clip[0] + call.clip[2]) as i32,
                        bottom: (call.clip[1] + call.clip[3]) as i32,
                    },
                    default_color: color,
                    custom_glyphs: &[],
                })
            })
            .collect();

        self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            text_areas,
            &mut self.swash_cache,
        )
    }

    pub fn render<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), glyphon::RenderError> {
        self.renderer.render(&self.atlas, &self.viewport, pass)
    }

    pub fn trim_atlas(&mut self) {
        self.atlas.trim();
    }
}

fn resolve_font_family(font_families: &[String], selection: FontSelection) -> glyphon::Family<'_> {
    match selection {
        FontSelection::SansSerif => glyphon::Family::SansSerif,
        FontSelection::Serif => glyphon::Family::Serif,
        FontSelection::Monospace => glyphon::Family::Monospace,
        FontSelection::Named(handle) => font_families
            .get(handle as usize)
            .map_or(glyphon::Family::SansSerif, |name| {
                glyphon::Family::Name(name)
            }),
    }
}

#[allow(clippy::too_many_arguments)]
fn set_text_impl(
    font_system: &mut glyphon::FontSystem,
    buffers: &mut HashMap<u64, glyphon::Buffer>,
    next_id: &mut u64,
    buffer_id: Option<u64>,
    text: &str,
    metrics: glyphon::Metrics,
    width: Option<f32>,
    height: Option<f32>,
    attrs: &glyphon::Attrs,
    align: Option<glyphon::cosmic_text::Align>,
) -> u64 {
    let id = buffer_id.unwrap_or_else(|| {
        let id = *next_id;
        *next_id += 1;
        id
    });

    let buffer = buffers
        .entry(id)
        .or_insert_with(|| glyphon::Buffer::new(font_system, metrics));

    buffer.set_metrics(font_system, metrics);

    if let Some(w) = width {
        buffer.set_size(font_system, Some(w), height);
    }

    buffer.set_text(font_system, text, attrs, glyphon::Shaping::Advanced, align);
    buffer.shape_until_scroll(font_system, false);

    id
}

pub fn text_geometry(
    buffer: &glyphon::Buffer,
    text: &str,
    cursor: usize,
    anchor: usize,
) -> TextGeometry {
    let cursor = normalized_position(text, cursor);
    let anchor = normalized_position(text, anchor);
    let selection_start = cursor.min(anchor);
    let selection_end = cursor.max(anchor);
    let line_starts = line_starts(text);
    let (cursor_line, cursor_index) = line_position(text, &line_starts, cursor);
    let mut geometry = TextGeometry::default();

    for run in buffer.layout_runs() {
        if selection_start != selection_end {
            let line_start = line_starts.get(run.line_i).copied().unwrap_or(text.len());
            let local_start = selection_start
                .saturating_sub(line_start)
                .min(run.text.len());
            let local_end = selection_end.saturating_sub(line_start).min(run.text.len());
            if local_start < local_end {
                if let Some((x, width)) = selected_span(&run, local_start, local_end) {
                    geometry
                        .selection
                        .push([x, run.line_top, width, run.line_height]);
                }
            }
        }

        if geometry.caret.is_none() && run.line_i == cursor_line {
            if let Some(x) = caret_x(&run, cursor_index) {
                geometry.caret = Some([x, run.line_top, 2.0, run.line_height]);
            }
        }
    }

    geometry
}

fn normalized_position(text: &str, position: usize) -> usize {
    let mut position = position.min(text.len());
    while position > 0 && !text.is_char_boundary(position) {
        position -= 1;
    }
    position
}

fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    starts.extend(
        text.match_indices('\n')
            .map(|(index, _)| index + 1)
            .filter(|&index| index <= text.len()),
    );
    starts
}

fn line_position(text: &str, starts: &[usize], position: usize) -> (usize, usize) {
    let line = starts
        .partition_point(|&start| start <= position)
        .saturating_sub(1);
    let start = starts[line];
    let line_end = text[start..]
        .find('\n')
        .map_or(text.len(), |index| start + index);
    (line, position.min(line_end) - start)
}

/// Converts akar's global byte offset (into the concatenation of every
/// `BufferLine`'s text plus its line-ending separator) into a cosmic-text
/// `Cursor`. Does **not** assume `\n`-only separators: it reads each line's
/// own `LineEnding` so `\r\n`/`\r`/`\n\r` documents (were one ever shaped)
/// round-trip too, even though akar's own paste/insert normalization only
/// ever produces `\n`.
fn global_offset_to_cursor(lines: &[glyphon::BufferLine], offset: usize) -> glyphon::Cursor {
    let mut start = 0usize;
    for (index, line) in lines.iter().enumerate() {
        let text_len = line.text().len();
        if offset <= start + text_len || index + 1 == lines.len() {
            return glyphon::Cursor::new(index, offset.saturating_sub(start).min(text_len));
        }
        start += text_len + line.ending().as_str().len();
    }
    glyphon::Cursor::new(0, 0)
}

/// Inverse of `global_offset_to_cursor`.
fn cursor_to_global_offset(lines: &[glyphon::BufferLine], cursor: glyphon::Cursor) -> usize {
    let mut start = 0usize;
    for (index, line) in lines.iter().enumerate() {
        let text_len = line.text().len();
        if index == cursor.line {
            return start + cursor.index.min(text_len);
        }
        start += text_len + line.ending().as_str().len();
    }
    start
}

fn selected_span(
    run: &glyphon::cosmic_text::LayoutRun<'_>,
    start: usize,
    end: usize,
) -> Option<(f32, f32)> {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    for glyph in run.glyphs {
        let overlap_start = start.max(glyph.start);
        let overlap_end = end.min(glyph.end);
        if overlap_start >= overlap_end {
            continue;
        }
        let left = glyph_boundary_x(run.text, glyph, overlap_start);
        let right = glyph_boundary_x(run.text, glyph, overlap_end);
        min_x = min_x.min(left.min(right));
        max_x = max_x.max(left.max(right));
    }
    (min_x.is_finite() && max_x > min_x).then_some((min_x, max_x - min_x))
}

fn caret_x(run: &glyphon::cosmic_text::LayoutRun<'_>, index: usize) -> Option<f32> {
    if run.glyphs.is_empty() {
        return (index == 0).then_some(0.0);
    }
    let first = run.glyphs.first()?;
    let last = run.glyphs.last()?;
    let run_start = run
        .glyphs
        .iter()
        .map(|glyph| glyph.start)
        .min()
        .unwrap_or(0);
    let run_end = run.glyphs.iter().map(|glyph| glyph.end).max().unwrap_or(0);
    if index < run_start || index > run_end {
        return None;
    }
    if index == run_end {
        return Some(if last.level.is_rtl() {
            last.x
        } else {
            last.x + last.w
        });
    }
    if index == run_start {
        return Some(if first.level.is_rtl() {
            first.x + first.w
        } else {
            first.x
        });
    }
    run.glyphs
        .iter()
        .find(|glyph| index >= glyph.start && index <= glyph.end)
        .map(|glyph| glyph_boundary_x(run.text, glyph, index))
}

fn glyph_boundary_x(text: &str, glyph: &glyphon::cosmic_text::LayoutGlyph, index: usize) -> f32 {
    let cluster = &text[glyph.start..glyph.end];
    let boundary = index.clamp(glyph.start, glyph.end) - glyph.start;
    let total = cluster.chars().count().max(1);
    let before = cluster[..boundary].chars().count();
    let fraction = before as f32 / total as f32;
    if glyph.level.is_rtl() {
        glyph.x + glyph.w * (1.0 - fraction)
    } else {
        glyph.x + glyph.w * fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shaped_buffer(text: &str, width: f32) -> glyphon::Buffer {
        let mut font_system = glyphon::FontSystem::new();
        let mut buffer = glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(16.0, 20.0));
        buffer.set_size(&mut font_system, Some(width), None);
        buffer.set_text(
            &mut font_system,
            text,
            &glyphon::Attrs::new(),
            glyphon::Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut font_system, false);
        buffer
    }

    fn create_pipeline() -> TextPipeline {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            ..Default::default()
        }))
        .expect("no suitable adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to create device");
        let format = wgpu::TextureFormat::Bgra8UnormSrgb;
        TextPipeline::new(&device, &queue, format, TextPipelineConfig::default())
    }

    #[test]
    fn bundled_config_measures_latin_text() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(None, "Hello world", metrics, None, None, None);

        let result = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: None,
            },
        );

        assert!(result.width > 0.0, "latin text must measure non-zero width");
        assert!(result.height > 0.0);
    }

    #[test]
    fn uncovered_script_renders_tofu_without_panicking() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(None, "你好世界", metrics, None, None, None);

        let result = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: None,
            },
        );

        assert!(result.width > 0.0);
        assert!(result.height > 0.0);
    }

    /// Byte-exact fingerprint of everything the renderer derives from shaping:
    /// which face each glyph came from, which glyph it is, where it sits, and
    /// its bidi level. Floats are hashed by their raw bits, so this is an
    /// equality check on bytes, not on approximate geometry.
    fn glyph_signature(pipeline: &TextPipeline, buffer_id: u64) -> Vec<u8> {
        let buffer = pipeline.buffers.get(&buffer_id).expect("buffer exists");
        let mut bytes = Vec::new();
        for run in buffer.layout_runs() {
            bytes.extend_from_slice(&run.line_i.to_le_bytes());
            bytes.extend_from_slice(&run.line_w.to_bits().to_le_bytes());
            bytes.extend_from_slice(&run.line_top.to_bits().to_le_bytes());
            for glyph in run.glyphs {
                bytes.extend_from_slice(format!("{:?}", glyph.font_id).as_bytes());
                bytes.extend_from_slice(&glyph.glyph_id.to_le_bytes());
                bytes.extend_from_slice(&glyph.start.to_le_bytes());
                bytes.extend_from_slice(&glyph.end.to_le_bytes());
                bytes.extend_from_slice(&glyph.x.to_bits().to_le_bytes());
                bytes.extend_from_slice(&glyph.y.to_bits().to_le_bytes());
                bytes.extend_from_slice(&glyph.w.to_bits().to_le_bytes());
                bytes.extend_from_slice(&glyph.x_offset.to_bits().to_le_bytes());
                bytes.extend_from_slice(&glyph.y_offset.to_bits().to_le_bytes());
                bytes.push(glyph.level.number());
            }
        }
        bytes
    }

    /// Task 8 regression check: the default `FontSource::Bundled` path is the
    /// reproducibility guarantee akar's screenshot diffing depends on. Two
    /// independently constructed pipelines (i.e. two contexts on the default
    /// `AKAR_FONT_SOURCE_BUNDLED` path) must shape the same input into
    /// byte-identical glyph output on the same machine.
    #[test]
    fn bundled_source_shapes_identically_across_contexts() {
        const SAMPLE: &str = "Hello world — akar 022 fonts";
        let metrics = glyphon::Metrics::new(16.0, 20.0);

        let signature = |request: FontRequest| {
            assert_eq!(
                TextPipelineConfig::default().font_source,
                FontSource::Bundled
            );
            let mut pipeline = create_pipeline();
            let id =
                pipeline.set_text_styled(None, SAMPLE, metrics, Some(400.0), None, request, None);
            glyph_signature(&pipeline, id)
        };

        for request in [
            FontRequest::default(),
            FontRequest {
                family: FontSelection::SansSerif,
                weight: 700,
            },
            FontRequest {
                family: FontSelection::Monospace,
                weight: 400,
            },
        ] {
            let first = signature(request);
            let second = signature(request);
            assert!(!first.is_empty(), "sample text must produce glyphs");
            assert_eq!(
                first, second,
                "bundled-source shaping must be byte-identical across contexts for {request:?}"
            );
        }
    }

    #[cfg(feature = "bundled-font")]
    #[test]
    fn load_font_bytes_registers_single_family() {
        let mut pipeline = create_pipeline();
        let handle = pipeline
            .load_font_bytes(crate::font_source::IBM_PLEX_SANS_REGULAR.to_vec())
            .expect("single-family font loads");
        assert_eq!(pipeline.family_name(handle), Some("IBM Plex Sans"));

        let again = pipeline
            .load_font_bytes(crate::font_source::IBM_PLEX_SANS_SEMIBOLD.to_vec())
            .expect("same family reloads");
        assert_eq!(handle, again, "same family reuses its handle");
    }

    #[cfg(feature = "bundled-font")]
    #[test]
    fn load_font_bytes_is_idempotent_for_identical_source() {
        let mut pipeline = create_pipeline();
        let bytes = crate::font_source::IBM_PLEX_SANS_REGULAR.to_vec();
        let faces_before = pipeline.font_system.db().faces().count();

        let handle = pipeline
            .load_font_bytes(bytes.clone())
            .expect("font source loads");
        let faces_after_first_load = pipeline.font_system.db().faces().count();
        let again = pipeline
            .load_font_bytes(bytes)
            .expect("identical font source reload succeeds");

        assert_eq!(handle, again, "identical source reuses its handle");
        assert!(faces_after_first_load > faces_before);
        assert_eq!(
            pipeline.font_system.db().faces().count(),
            faces_after_first_load,
            "identical source must not add duplicate faces"
        );
    }

    /// A single-face font whose family carries a localized alias must load:
    /// fontdb reports the aliases as extra `families` entries, which an
    /// alias-counting implementation mistakes for a multi-family collection.
    /// Skipped where the sample font is not installed.
    #[test]
    fn load_font_bytes_accepts_localized_family_alias() {
        const ALIASED_FONT: &str = "/System/Library/Fonts/Supplemental/Mishafi.ttf";
        let Ok(bytes) = std::fs::read(ALIASED_FONT) else {
            return;
        };

        let mut pipeline = create_pipeline();
        let handle = pipeline
            .load_font_bytes(bytes)
            .expect("a localized alias is not a second family");
        assert_eq!(pipeline.family_name(handle), Some("Mishafi"));
    }

    #[test]
    fn load_font_bytes_rejects_garbage() {
        let mut pipeline = create_pipeline();
        assert!(matches!(
            pipeline.load_font_bytes(vec![0u8; 64]),
            Err(FontLoadError::InvalidFontData(_))
        ));
        assert!(matches!(
            pipeline.load_font_bytes(Vec::new()),
            Err(FontLoadError::InvalidFontData(_))
        ));
        assert_eq!(pipeline.family_name(0), None);
    }

    #[cfg(feature = "bundled-font")]
    #[test]
    fn set_text_styled_named_handle_shapes_text() {
        let mut pipeline = create_pipeline();
        let handle = pipeline
            .load_font_bytes(crate::font_source::IBM_PLEX_SANS_REGULAR.to_vec())
            .expect("single-family font loads");

        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text_styled(
            None,
            "Hello world",
            metrics,
            None,
            None,
            FontRequest {
                family: FontSelection::Named(handle),
                weight: 400,
            },
            None,
        );

        let result = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: None,
            },
        );
        assert!(result.width > 0.0);
        assert!(result.height > 0.0);
    }

    #[test]
    fn named_handle_resolves_to_registered_family_name() {
        let families = vec!["Distinct Test Family".to_string()];

        assert!(matches!(
            resolve_font_family(&families, FontSelection::Named(0)),
            glyphon::Family::Name("Distinct Test Family")
        ));
        assert!(matches!(
            resolve_font_family(&families, FontSelection::Named(1)),
            glyphon::Family::SansSerif
        ));
    }

    #[test]
    fn set_text_styled_unknown_handle_falls_back_to_sans_serif() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);

        let named = pipeline.set_text_styled(
            None,
            "Hello world",
            metrics,
            None,
            None,
            FontRequest {
                family: FontSelection::Named(9999),
                weight: 400,
            },
            None,
        );
        let generic = pipeline.set_text_styled(
            None,
            "Hello world",
            metrics,
            None,
            None,
            FontRequest::default(),
            None,
        );

        let input = TextMeasureInput {
            known_width: None,
            known_height: None,
            available_width: None,
        };
        let named = pipeline.measure_with_metadata(named, input);
        let generic = pipeline.measure_with_metadata(generic, input);
        assert!(named.width > 0.0);
        assert_eq!(named.width, generic.width);
    }

    #[test]
    fn set_text_returns_id_and_update_same_id() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);

        let id1 = pipeline.set_text(None, "hello", metrics, None, None, None);
        assert_eq!(id1, 1);

        let id2 = pipeline.set_text(None, "world", metrics, None, None, None);
        assert_eq!(id2, 2);

        pipeline.set_text(Some(id1), "updated", metrics, Some(200.0), None, None);
        assert_eq!(pipeline.buffers.len(), 2);
    }

    #[test]
    fn geometry_uses_shaped_unicode_widths() {
        let text = "Wié🙂";
        let buffer = shaped_buffer(text, 500.0);
        let geometry = text_geometry(&buffer, text, text.len(), 0);

        assert_eq!(geometry.selection.len(), 1);
        assert!(geometry.selection[0][2] > 0.0);
        assert_eq!(
            geometry.caret.expect("caret")[0],
            geometry.selection[0][0] + geometry.selection[0][2]
        );
    }

    #[test]
    fn geometry_splits_wrapped_and_multiline_selection_into_runs() {
        let text = "one two three four\né🙂 next";
        let buffer = shaped_buffer(text, 55.0);
        let geometry = text_geometry(&buffer, text, text.len(), 0);

        assert!(geometry.selection.len() >= 3);
        assert!(geometry
            .selection
            .windows(2)
            .any(|pair| pair[0][1] != pair[1][1]));
        assert!(geometry.caret.is_some());
    }

    #[test]
    fn geometry_handles_empty_and_invalid_positions() {
        let buffer = shaped_buffer("", 0.0);
        let geometry = text_geometry(&buffer, "", usize::MAX, usize::MAX);

        assert!(geometry.selection.is_empty());
        assert_eq!(geometry.caret, Some([0.0, 0.0, 2.0, 20.0]));
    }

    #[test]
    fn measure_with_metadata_known_dimensions_short_circuit() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(None, "abcdef", metrics, None, None, None);

        let result = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: Some(120.0),
                known_height: Some(40.0),
                available_width: Some(100.0),
            },
        );

        assert_eq!(result.width, 120.0);
        assert_eq!(result.height, 40.0);
    }

    #[test]
    fn measure_with_metadata_wrap_increases_height() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let text = "the rain in spain stays mainly in the plain";
        let id = pipeline.set_text(None, text, metrics, None, None, None);

        let wide = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: Some(800.0),
            },
        );
        let narrow = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: Some(40.0),
            },
        );

        assert!(narrow.height > wide.height, "wrapping yields taller text");
        // The longest unbreakable word can exceed the wrap width, so the
        // measured width is bounded by that word, not by the constraint.
        assert!(narrow.width < wide.width, "wrapping narrows measured width");
    }

    #[test]
    fn measure_with_metadata_explicit_newlines_count() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(None, "a\nb\nc\nd", metrics, None, None, None);

        let result = pipeline.measure_with_metadata(
            id,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: None,
            },
        );

        assert!(
            result.height >= 20.0 * 4.0 - 1.0,
            "four lines should yield >= 4x line height, got {}",
            result.height
        );
    }

    #[test]
    fn measure_with_metadata_missing_buffer_returns_zero() {
        let mut pipeline = create_pipeline();
        let result = pipeline.measure_with_metadata(
            4242,
            TextMeasureInput {
                known_width: None,
                known_height: None,
                available_width: Some(100.0),
            },
        );
        assert_eq!(result.width, 0.0);
        assert_eq!(result.height, 0.0);
    }

    #[test]
    fn global_offset_cursor_round_trips_across_multiline_text() {
        let text = "one\ntwo\nthree";
        let buffer = shaped_buffer(text, 500.0);

        for offset in [0, 1, 3, 4, 7, 8, 12, text.len()] {
            let cursor = global_offset_to_cursor(&buffer.lines, offset);
            assert_eq!(
                cursor_to_global_offset(&buffer.lines, cursor),
                offset,
                "offset {offset} must round-trip through Cursor {{ line: {}, index: {} }}",
                cursor.line,
                cursor.index
            );
        }

        // Cursor.index is line-local, not the global offset: line 2 ("three")
        // starts at global offset 8, so global offset 12 is index 4 on line 2,
        // not global offset 12 reused as an index.
        assert_eq!(
            global_offset_to_cursor(&buffer.lines, 0),
            glyphon::Cursor::new(0, 0)
        );
        assert_eq!(
            global_offset_to_cursor(&buffer.lines, 3),
            glyphon::Cursor::new(0, 3),
            "end of line 0, before its separator"
        );
        assert_eq!(
            global_offset_to_cursor(&buffer.lines, 4),
            glyphon::Cursor::new(1, 0),
            "start of line 1, just past the separator"
        );
        assert_eq!(
            global_offset_to_cursor(&buffer.lines, 8),
            glyphon::Cursor::new(2, 0),
            "start of line 2"
        );
        assert_eq!(
            global_offset_to_cursor(&buffer.lines, text.len()),
            glyphon::Cursor::new(2, 5),
            "end of the last line"
        );
    }

    #[test]
    fn move_caret_right_crosses_line_separator_to_global_offset() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let text = "one\ntwo";
        let id = pipeline.set_text(None, text, metrics, Some(500.0), None, None);

        // End of "one" (offset 3, right before the '\n') moving right must
        // land at the start of "two" (global offset 4), not at line-local
        // index 4 misread as a global offset.
        let moved = pipeline.move_caret(id, 3, CaretMotion::Right);
        assert_eq!(moved, 4);

        let back = pipeline.move_caret(id, moved, CaretMotion::Left);
        assert_eq!(back, 3);
    }

    /// ASCII-only tests cannot distinguish "moved by app layout direction"
    /// from "moved by the shaped paragraph's own direction" because both
    /// would agree on plain LTR text. This asserts the paragraph-direction
    /// discovery itself: at the very start of a buffer, physical `Right`
    /// advances through an LTR paragraph but is a no-op on an RTL one
    /// (visually already at the paragraph's leading edge), and `Left` does
    /// the reverse — using real Arabic and Hebrew text, not transliteration.
    #[test]
    fn move_caret_follows_the_shaped_paragraphs_own_direction() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);

        let ltr_id = pipeline.set_text(None, "hello", metrics, Some(500.0), None, None);
        assert_eq!(pipeline.move_caret(ltr_id, 0, CaretMotion::Right), 1);
        assert_eq!(pipeline.move_caret(ltr_id, 0, CaretMotion::Left), 0);

        // Arabic: "مرحبا" (hello), a pure-RTL paragraph.
        let arabic_id = pipeline.set_text(None, "مرحبا", metrics, Some(500.0), None, None);
        assert_eq!(
            pipeline.move_caret(arabic_id, 0, CaretMotion::Right),
            0,
            "Right at the start of an RTL paragraph must not move logically \
             backward past its leading (visually rightmost) edge"
        );
        let after_left = pipeline.move_caret(arabic_id, 0, CaretMotion::Left);
        assert_ne!(
            after_left, 0,
            "Left at the start of an RTL paragraph must advance logically, \
             since visual-left is toward the paragraph's end"
        );

        // Hebrew: "שלום" (hello/peace), a pure-RTL paragraph.
        let hebrew_id = pipeline.set_text(None, "שלום", metrics, Some(500.0), None, None);
        assert_eq!(pipeline.move_caret(hebrew_id, 0, CaretMotion::Right), 0);
        assert_ne!(pipeline.move_caret(hebrew_id, 0, CaretMotion::Left), 0);
    }

    /// An RTL app can still contain an LTR value: this adapter takes no
    /// `AkarDirection` at all, so an LTR value shapes and moves the same way
    /// regardless of what an enclosing layout direction would have been.
    #[test]
    fn move_caret_on_ltr_value_is_direction_parameter_free() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(None, "value", metrics, Some(500.0), None, None);

        assert_eq!(pipeline.move_caret(id, 0, CaretMotion::Right), 1);
        assert_eq!(pipeline.move_caret(id, 5, CaretMotion::Left), 4);
    }

    #[test]
    fn move_caret_missing_buffer_is_a_no_op() {
        let mut pipeline = create_pipeline();
        assert_eq!(pipeline.move_caret(9999, 3, CaretMotion::Left), 3);
        assert_eq!(pipeline.move_caret(9999, 3, CaretMotion::Right), 3);
    }

    #[test]
    fn caret_x_missing_buffer_returns_none() {
        let pipeline = create_pipeline();
        assert_eq!(pipeline.caret_x(9999, 0), None);
    }

    #[test]
    fn caret_x_increases_left_to_right_for_ltr_text() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(None, "value", metrics, Some(500.0), None, None);

        let start_x = pipeline.caret_x(id, 0).expect("caret x at start");
        let end_x = pipeline.caret_x(id, 5).expect("caret x at end");
        assert!(end_x > start_x, "LTR caret x must increase toward the end");
    }

    #[test]
    fn caret_position_exposes_source_line_order() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let text = "long first line\nx";
        let id = pipeline.set_text(None, text, metrics, Some(500.0), None, None);

        let first = pipeline.caret_position(id, 10).expect("first-line caret");
        let second = pipeline
            .caret_position(id, "long first line\n".len())
            .expect("second-line caret");
        assert_eq!((first.line, first.run), (0, 0));
        assert_eq!((second.line, second.run), (1, 0));
        assert!(first.x > second.x, "test must exercise misleading x order");
    }

    #[test]
    fn caret_position_exposes_wrapped_run_order() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);
        let id = pipeline.set_text(
            None,
            "one two three four five",
            metrics,
            Some(55.0),
            None,
            None,
        );

        let first = pipeline.caret_position(id, 0).expect("first visual run");
        let last = pipeline
            .caret_position(id, "one two three four five".len())
            .expect("last visual run");
        assert_eq!(first.line, 0);
        assert_eq!(last.line, 0);
        assert!(last.run > first.run, "narrow width must wrap the text");
    }

    /// `set_text_styled`'s `align` parameter must reach cosmic-text's own
    /// `Buffer::set_text` so the buffer aligns itself within the full
    /// component-width buffer, rather than akar computing a manual x-offset
    /// on top of it (Epic 023 Task 9's resolved design). Short text in a wide
    /// buffer is the simplest way to observe this: `Align::Right` must push
    /// the run's glyphs toward the right edge of the buffer, further right
    /// than the buffer's own default (left) alignment would.
    #[test]
    fn set_text_styled_align_right_shifts_glyphs_right_of_default() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);

        let default_id = pipeline.set_text_styled(
            None,
            "hi",
            metrics,
            Some(400.0),
            None,
            FontRequest::default(),
            None,
        );
        let right_id = pipeline.set_text_styled(
            None,
            "hi",
            metrics,
            Some(400.0),
            None,
            FontRequest::default(),
            Some(glyphon::cosmic_text::Align::Right),
        );

        let default_buffer = pipeline.buffers.get(&default_id).expect("default buffer");
        let default_run = default_buffer
            .layout_runs()
            .next()
            .expect("default run shaped");
        let default_x = default_run
            .glyphs
            .first()
            .map(|glyph| glyph.x)
            .unwrap_or(0.0);

        let right_buffer = pipeline
            .buffers
            .get(&right_id)
            .expect("right-aligned buffer");
        let right_run = right_buffer
            .layout_runs()
            .next()
            .expect("right-aligned run shaped");
        let right_x = right_run.glyphs.first().map(|glyph| glyph.x).unwrap_or(0.0);

        assert!(
            right_x > default_x,
            "Align::Right must shift the shaped run right of the default alignment \
             (default {default_x}, right-aligned {right_x})"
        );
    }
}
