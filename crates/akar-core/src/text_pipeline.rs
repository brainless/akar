use crate::TextCall;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextGeometry {
    pub caret: Option<[f32; 4]>,
    pub selection: Vec<[f32; 4]>,
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
}

impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let font_system = glyphon::FontSystem::new();
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
        }
    }

    pub fn set_text(
        &mut self,
        buffer_id: Option<u64>,
        text: &str,
        metrics: glyphon::Metrics,
        width: Option<f32>,
        height: Option<f32>,
    ) -> u64 {
        let id = buffer_id.unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        });

        let buffer = self
            .buffers
            .entry(id)
            .or_insert_with(|| glyphon::Buffer::new(&mut self.font_system, metrics));

        buffer.set_metrics(&mut self.font_system, metrics);

        if let Some(w) = width {
            buffer.set_size(&mut self.font_system, Some(w), height);
        }

        buffer.set_text(
            &mut self.font_system,
            text,
            &glyphon::Attrs::new(),
            glyphon::Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        id
    }

    pub fn remove_buffer(&mut self, buffer_id: u64) {
        self.buffers.remove(&buffer_id);
    }

    pub fn measure(&mut self, buffer_id: u64, width: Option<f32>) -> glam::Vec2 {
        let Some(buffer) = self.buffers.get_mut(&buffer_id) else {
            return glam::Vec2::ZERO;
        };

        if let Some(w) = width {
            buffer.set_size(&mut self.font_system, Some(w), None);
            buffer.shape_until_scroll(&mut self.font_system, false);
        }

        let mut max_w: f32 = 0.0;
        let mut last_bottom: f32 = 0.0;
        for run in buffer.layout_runs() {
            max_w = max_w.max(run.line_w);
            last_bottom = run.line_top + run.line_height;
        }

        glam::Vec2::new(max_w, last_bottom)
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
        TextPipeline::new(&device, &queue, format)
    }

    #[test]
    fn set_text_returns_id_and_update_same_id() {
        let mut pipeline = create_pipeline();
        let metrics = glyphon::Metrics::new(16.0, 20.0);

        let id1 = pipeline.set_text(None, "hello", metrics, None, None);
        assert_eq!(id1, 1);

        let id2 = pipeline.set_text(None, "world", metrics, None, None);
        assert_eq!(id2, 2);

        pipeline.set_text(Some(id1), "updated", metrics, Some(200.0), None);
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
}
