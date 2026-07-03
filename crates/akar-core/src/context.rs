use crate::draw_list::DrawCall;
use crate::{DrawList, InputState, QuadPipeline, TextPipeline};

pub struct AkarCore {
    pub draw_list: DrawList,
    pub input: InputState,
    pub(crate) quad_pipeline: QuadPipeline,
    pub(crate) text_pipeline: TextPipeline,
    #[allow(dead_code)]
    surface_format: wgpu::TextureFormat,
    viewport_width: u32,
    viewport_height: u32,
    scale_factor: f32,
}

impl AkarCore {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            draw_list: DrawList::new(),
            input: InputState::new(),
            quad_pipeline: QuadPipeline::new(device, surface_format),
            text_pipeline: TextPipeline::new(device, queue, surface_format),
            surface_format,
            viewport_width: 0,
            viewport_height: 0,
            scale_factor: 1.0,
        }
    }

    pub fn begin_frame(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.draw_list.begin_frame(scale_factor);
        self.input.begin_frame();
        self.viewport_width = width;
        self.viewport_height = height;
        self.scale_factor = scale_factor;
    }

    pub fn end_frame<'a>(
        &'a mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'a>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let text_calls: Vec<_> = self
            .draw_list
            .text_calls()
            .iter()
            .filter_map(|c| match c {
                DrawCall::Text(t) => Some(t.clone()),
                _ => None,
            })
            .collect();

        self.text_pipeline.prepare(
            device,
            queue,
            self.viewport_width,
            self.viewport_height,
            self.scale_factor,
            &text_calls,
        )?;

        self.quad_pipeline.flush(
            device,
            queue,
            pass,
            &self.draw_list.sorted_quads(),
            self.viewport_width,
            self.viewport_height,
        );

        self.text_pipeline.trim_atlas();
        self.text_pipeline.render(pass)?;

        Ok(())
    }
}
