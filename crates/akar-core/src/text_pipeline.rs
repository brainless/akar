use crate::TextCall;
use std::collections::HashMap;

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
        let renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

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

        buffer.set_text(&mut self.font_system, text, &glyphon::Attrs::new(), glyphon::Shaping::Advanced, None);
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
