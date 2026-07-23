use crate::draw_list::DrawCall;
use crate::screenshot::ScreenshotCapture;
use crate::{
    CapturedFrame, DrawList, InputState, QuadPipeline, ScreenshotError, TextEditKeybindings,
    TextPipeline,
};

pub struct AkarCore {
    pub draw_list: DrawList,
    pub input: InputState,
    pub text_edit_keybindings: TextEditKeybindings,
    pub(crate) quad_pipeline: QuadPipeline,
    foreground_quad_pipeline: QuadPipeline,
    pub text_pipeline: TextPipeline,
    screenshot_capture: ScreenshotCapture,
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
            text_edit_keybindings: TextEditKeybindings::default(),
            quad_pipeline: QuadPipeline::new(device, surface_format),
            foreground_quad_pipeline: QuadPipeline::new(device, surface_format),
            text_pipeline: TextPipeline::new(device, queue, surface_format),
            screenshot_capture: ScreenshotCapture::new(device, surface_format),
            surface_format,
            viewport_width: 0,
            viewport_height: 0,
            scale_factor: 1.0,
        }
    }

    pub fn set_text_edit_keybindings(&mut self, bindings: TextEditKeybindings) {
        self.text_edit_keybindings = bindings;
    }

    pub fn mock() -> Self {
        // Minimal mock for unit testing component logic without a GPU.
        // quad_pipeline and text_pipeline are zero-initialized placeholders
        // and must not be used for actual rendering.
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("no suitable adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to create device");
        Self::new(&device, &queue, wgpu::TextureFormat::Bgra8UnormSrgb)
    }

    pub fn begin_frame(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.draw_list.begin_frame(scale_factor);
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

        let quads = self.draw_list.sorted_quads();
        let foreground_start = quads.partition_point(|quad| quad.z < crate::Z_TEXT_FOREGROUND);
        self.quad_pipeline.flush(
            device,
            queue,
            pass,
            &quads[..foreground_start],
            self.viewport_width,
            self.viewport_height,
        );

        self.text_pipeline.trim_atlas();
        self.text_pipeline.render(pass)?;
        self.foreground_quad_pipeline.flush(
            device,
            queue,
            pass,
            &quads[foreground_start..],
            self.viewport_width,
            self.viewport_height,
        );

        // Clear single-frame input events after all components have read them.
        self.input.begin_frame();

        Ok(())
    }

    pub fn request_screenshot(&mut self) {
        self.screenshot_capture.requested = true;
    }

    pub fn capture_target_view(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Option<wgpu::TextureView> {
        self.screenshot_capture
            .capture_view(device, width, height, self.surface_format)
    }

    pub fn take_screenshot(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: wgpu::CommandEncoder,
        surface_texture: &wgpu::SurfaceTexture,
    ) -> Result<CapturedFrame, ScreenshotError> {
        self.screenshot_capture.take_screenshot(
            device,
            queue,
            encoder,
            surface_texture,
            self.surface_format,
        )
    }
}
