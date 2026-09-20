use std::sync::Arc;
use std::time::{Duration, Instant};

use akar_components::{
    data_grid_begin, data_grid_body_begin, data_grid_body_end, data_grid_cell, data_grid_end,
    data_grid_handle_keyboard, data_grid_header_begin, data_grid_header_cell, data_grid_header_end,
    DataGridAlign, DataGridColumn, DataGridSortDirection, DataGridState, DataGridStyle,
    AKAR_THEME_DARK,
};
use akar_core::AkarCore;
use akar_layout::{Layout, NodeId, PageConfig, Size, Style};
use akar_winit::process_window_event;
use fake::rand::rngs::StdRng;
use fake::rand::SeedableRng;
use fake::{Fake, Faker};
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, InstanceDescriptor, PresentMode, TextureUsages,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowAttributes},
};

mod script;
use script::{parse_script, ScriptRunner};

const DEFAULT_ROWS: usize = 100_000;
const DEFAULT_SEED: u64 = 42;

struct Record {
    original_index: usize,
    amount: f64,
    cells: [String; 8],
}

fn compare_records(a: &Record, b: &Record, column: usize, ascending: bool) -> std::cmp::Ordering {
    let ordering = match column {
        0 => a.original_index.cmp(&b.original_index),
        6 => a
            .amount
            .partial_cmp(&b.amount)
            .unwrap_or(std::cmp::Ordering::Equal),
        _ => a.cells[column]
            .bytes()
            .map(|byte| byte.to_ascii_lowercase())
            .cmp(
                b.cells[column]
                    .bytes()
                    .map(|byte| byte.to_ascii_lowercase()),
            ),
    };
    if ascending {
        ordering
    } else {
        ordering.reverse()
    }
}

const COLUMN_NAMES: [&str; 8] = [
    "ID", "Name", "Email", "Company", "City", "Status", "Amount", "Created",
];

const COLUMN_WIDTHS: [f32; 8] = [80.0, 140.0, 200.0, 150.0, 120.0, 100.0, 110.0, 120.0];

fn columns() -> Vec<DataGridColumn> {
    COLUMN_WIDTHS
        .iter()
        .enumerate()
        .map(|(i, &w)| DataGridColumn {
            key: (i as u64 + 1) * 100,
            width: w,
            align: if i == 6 {
                DataGridAlign::Right
            } else {
                DataGridAlign::Left
            },
        })
        .collect()
}

fn generate_records(count: usize, seed: u64) -> Vec<Record> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut records = Vec::with_capacity(count);

    for i in 0..count {
        let id = (i as u64 + 1) * 1000;
        let name: String = Faker.fake_with_rng(&mut rng);
        let email: String = Faker.fake_with_rng(&mut rng);
        let company: String = Faker.fake_with_rng(&mut rng);
        let city: String = Faker.fake_with_rng(&mut rng);
        let status_idx = i % 5;
        let status = match status_idx {
            0 => "Active",
            1 => "Pending",
            2 => "Inactive",
            3 => "Suspended",
            _ => "Active",
        };
        let amount = (i as f64 + 1.0) * 137.50 * ((i as f64 * 0.01).sin() + 1.5);
        let month = (i % 12) + 1;
        let day = (i % 28) + 1;
        let created = format!("2025-{month:02}-{day:02}");

        let cells = [
            format!("{id}"),
            name,
            email,
            company,
            city,
            status.to_string(),
            format!("${amount:.2}"),
            created,
        ];

        records.push(Record {
            original_index: i,
            amount,
            cells,
        });
    }

    records
}

struct CliConfig {
    rows: usize,
    seed: u64,
    screenshot: Option<String>,
    delay: f64,
    exit: bool,
    script: Option<String>,
    dump_layout: bool,
    dump_frame: Option<String>,
}

fn parse_cli() -> CliConfig {
    let mut rows = DEFAULT_ROWS;
    let mut seed = DEFAULT_SEED;
    let mut screenshot = None;
    let mut delay = 1.0;
    let mut exit = false;
    let mut script = None;
    let mut dump_layout = false;
    let mut dump_frame = None;

    let mut args = std::env::args().peekable();
    args.next();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rows" => {
                if let Some(v) = args.next() {
                    rows = v.parse().unwrap_or(DEFAULT_ROWS);
                }
            }
            "--seed" => {
                if let Some(v) = args.next() {
                    seed = v.parse().unwrap_or(DEFAULT_SEED);
                }
            }
            "--screenshot" => {
                screenshot = args.next();
            }
            "--delay" => {
                if let Some(v) = args.next() {
                    delay = v.parse().unwrap_or(1.0);
                }
            }
            "--exit" => {
                exit = true;
            }
            "--script" => {
                script = args.next();
            }
            "--dump-layout" => {
                dump_layout = true;
            }
            "--dump-frame" => {
                dump_frame = args.next();
            }
            other => {
                eprintln!("Unknown argument: {other}");
                std::process::exit(1);
            }
        }
    }

    CliConfig {
        rows,
        seed,
        screenshot,
        delay,
        exit,
        script,
        dump_layout,
        dump_frame,
    }
}

struct AppState {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    core: AkarCore,
    layout: Layout,
    page: akar_layout::PageLayout,
    grid_node: NodeId,
    grid_state: DataGridState,
    records: Vec<Record>,
    row_keys: Vec<u64>,
    columns: Vec<DataGridColumn>,
    sort_col: Option<usize>,
    sort_dir: DataGridSortDirection,
    selected_key: Option<u64>,
}

fn rebuild_row_keys(records: &[Record]) -> Vec<u64> {
    records.iter().map(|r| r.original_index as u64).collect()
}

fn main() {
    let config = parse_cli();

    let script_runner = match config.script {
        Some(ref path) => match std::fs::read_to_string(path) {
            Ok(contents) => match parse_script(&contents) {
                Ok(steps) => Some(ScriptRunner::new(steps)),
                Err(e) => {
                    eprintln!("Failed to parse script '{path}': {e}");
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Failed to read script '{path}': {e}");
                std::process::exit(1);
            }
        },
        None => None,
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop
        .run_app(&mut App {
            state: None,
            config,
            script_runner,
            start_time: None,
            screenshot_taken: false,
            dump_layout_written: false,
            dump_frame_written: false,
        })
        .unwrap();
}

struct App {
    state: Option<AppState>,
    config: CliConfig,
    script_runner: Option<ScriptRunner>,
    start_time: Option<Instant>,
    screenshot_taken: bool,
    dump_layout_written: bool,
    dump_frame_written: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = WindowAttributes::default()
            .with_title("akar data grid")
            .with_inner_size(LogicalSize::new(1280.0, 800.0));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let instance = wgpu::Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        )));
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

        let size = window.inner_size();
        let mut surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface_config.usage = TextureUsages::RENDER_ATTACHMENT;
        surface_config.present_mode = PresentMode::Fifo;
        surface_config.alpha_mode = CompositeAlphaMode::Opaque;
        let surface_format = surface_config.format;
        surface.configure(&device, &surface_config);

        let core = AkarCore::new(
            &device,
            &queue,
            surface_format,
            akar_core::TextPipelineConfig::default(),
        );
        let mut layout = Layout::new();

        let page = layout.page(PageConfig {
            header_height: None,
            footer_height: None,
            sidebar_left_width: None,
            sidebar_right_width: None,
        });

        let grid_node = layout.new_leaf(Style {
            size: Size {
                width: akar_layout::Dimension::percent(1.0),
                height: akar_layout::Dimension::percent(1.0),
            },
            ..Default::default()
        });
        layout.set_children(page.main, &[grid_node]);

        let records = generate_records(self.config.rows, self.config.seed);
        let row_keys = rebuild_row_keys(&records);
        let columns = columns();

        if self.config.screenshot.is_some() {
            self.start_time = Some(Instant::now());
        }

        self.state = Some(AppState {
            window,
            device,
            queue,
            surface,
            surface_config,
            core,
            layout,
            page,
            grid_node,
            grid_state: DataGridState::new(),
            records,
            row_keys,
            columns,
            sort_col: None,
            sort_dir: DataGridSortDirection::None,
            selected_key: None,
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        let has_screenshot = self.config.screenshot.is_some();
        let screenshot_path = self.config.screenshot.clone();
        let delay = self.config.delay;
        let dump_frame = self.config.dump_frame.clone();
        let do_exit = self.config.exit;
        let do_dump_layout = self.config.dump_layout;

        match event {
            WindowEvent::Resized(new_size) => {
                state.surface_config.width = new_size.width;
                state.surface_config.height = new_size.height;
                state
                    .surface
                    .configure(&state.device, &state.surface_config);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let size = state.window.inner_size();
                let scale = state.window.scale_factor() as f32;

                let output = match state.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(t) | CurrentSurfaceTexture::Suboptimal(t) => t,
                    _ => return,
                };

                state.core.begin_frame(size.width, size.height, scale);

                if dump_frame.is_some() && !self.dump_frame_written {
                    state.core.draw_list.start_recording();
                }

                let viewport_rect = [
                    0.0,
                    0.0,
                    size.width as f32 / scale,
                    size.height as f32 / scale,
                ];

                state.layout.compute(
                    state.page.root,
                    (Some(viewport_rect[2]), Some(viewport_rect[3])),
                    |_, _, _, _, _| Size::ZERO,
                );

                if do_dump_layout && !self.dump_layout_written {
                    self.dump_layout_written = true;
                    for (name, rect) in state.layout.labeled_rects() {
                        println!("{} {} {} {} {}", name, rect[0], rect[1], rect[2], rect[3]);
                    }
                    event_loop.exit();
                    return;
                }

                let script_capture_path = if let Some(runner) = self.script_runner.as_mut() {
                    runner.advance(
                        &mut state.core.input,
                        &mut state.core.text_edit_keybindings,
                        &state.layout,
                        Instant::now(),
                    )
                } else {
                    None
                };

                let row_keys = &state.row_keys;
                let selected_slice = if let Some(key) = state.selected_key {
                    vec![key]
                } else {
                    vec![]
                };

                let grid_resp = data_grid_begin(
                    &mut state.core,
                    &state.layout,
                    state.grid_node,
                    &mut state.grid_state,
                    state.records.len(),
                    row_keys,
                    32.0,
                    36.0,
                    &state.columns,
                    &DataGridStyle::from_theme(&AKAR_THEME_DARK),
                );

                data_grid_header_begin(
                    &mut state.core,
                    &grid_resp,
                    &DataGridStyle::from_theme(&AKAR_THEME_DARK),
                );
                for (col_i, col_name) in COLUMN_NAMES.iter().enumerate() {
                    let sort = if state.sort_col == Some(col_i) {
                        state.sort_dir
                    } else {
                        DataGridSortDirection::None
                    };
                    let hdr_resp = data_grid_header_cell(
                        &mut state.core,
                        &state.layout,
                        &grid_resp,
                        state.grid_node,
                        col_i,
                        &state.columns,
                        &DataGridStyle::from_theme(&AKAR_THEME_DARK),
                        col_name,
                        sort,
                    );
                    if hdr_resp.clicked {
                        if state.sort_col == Some(col_i) {
                            state.sort_dir = match state.sort_dir {
                                DataGridSortDirection::Ascending => {
                                    DataGridSortDirection::Descending
                                }
                                DataGridSortDirection::Descending => DataGridSortDirection::None,
                                DataGridSortDirection::None => DataGridSortDirection::Ascending,
                            };
                        } else {
                            state.sort_col = Some(col_i);
                            state.sort_dir = DataGridSortDirection::Ascending;
                        }
                        if state.sort_dir != DataGridSortDirection::None {
                            let col = state.sort_col.unwrap();
                            state.records.sort_by(|a, b| {
                                compare_records(
                                    a,
                                    b,
                                    col,
                                    state.sort_dir == DataGridSortDirection::Ascending,
                                )
                            });
                        } else {
                            state.records.sort_by_key(|r| r.original_index);
                        }
                        state.row_keys = rebuild_row_keys(&state.records);
                    }
                }
                data_grid_header_end(&mut state.core);

                data_grid_body_begin(
                    &mut state.core,
                    &grid_resp,
                    &state.row_keys,
                    &DataGridStyle::from_theme(&AKAR_THEME_DARK),
                    &selected_slice,
                );
                for row_i in grid_resp.visible_rows.clone() {
                    if row_i >= state.records.len() {
                        break;
                    }
                    let record = &state.records[row_i];
                    let row_key = record.original_index as u64;
                    let is_selected = state.selected_key == Some(row_key);
                    for col_i in grid_resp.visible_columns.clone() {
                        if col_i >= state.columns.len() {
                            break;
                        }
                        let cell_resp = data_grid_cell(
                            &mut state.core,
                            &state.layout,
                            &grid_resp,
                            state.grid_node,
                            row_i,
                            row_key,
                            col_i,
                            &state.columns,
                            &DataGridStyle::from_theme(&AKAR_THEME_DARK),
                            &record.cells[col_i],
                            is_selected,
                        );
                        if cell_resp.clicked {
                            state.selected_key = Some(row_key);
                            state.grid_state.active_row_key = row_key;
                            state.grid_state.active_column_key = state.columns[col_i].key;
                            state.grid_state.has_active_cell = true;
                        }
                    }
                }
                data_grid_body_end(&mut state.core);

                data_grid_end(&mut state.core);

                let kb_resp = data_grid_handle_keyboard(
                    &mut state.core,
                    &state.layout,
                    state.grid_node,
                    &mut state.grid_state,
                    state.records.len(),
                    &state.row_keys,
                    &state.columns,
                    &DataGridStyle::from_theme(&AKAR_THEME_DARK),
                );
                if kb_resp.cell_changed || kb_resp.activated {
                    state.selected_key = Some(state.grid_state.active_row_key);
                }

                let normal_capture = !self.screenshot_taken
                    && has_screenshot
                    && self
                        .start_time
                        .is_some_and(|t| t.elapsed() >= Duration::from_secs_f64(delay));
                let is_capture_frame = normal_capture || script_capture_path.is_some();

                if is_capture_frame {
                    state.core.request_screenshot();
                }

                let mut encoder = state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

                {
                    let surface_view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let render_view = if is_capture_frame {
                        state
                            .core
                            .capture_target_view(&state.device, size.width, size.height)
                            .unwrap()
                    } else {
                        surface_view
                    };

                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &render_view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });
                    let _ = state.core.end_frame(&state.device, &state.queue, &mut pass);
                }

                let is_standalone = !has_screenshot && self.script_runner.is_none();
                let is_dump_frame = dump_frame.is_some()
                    && !self.dump_frame_written
                    && (is_capture_frame || is_standalone);
                if is_dump_frame {
                    let path = dump_frame.clone().unwrap();
                    let dump = serde_json::json!({
                        "recorded_calls": state.core.draw_list.recorded_calls(),
                        "labeled_rects": state.layout.labeled_rects(),
                    });
                    match std::fs::File::create(&path) {
                        Ok(file) => {
                            if let Err(e) = serde_json::to_writer_pretty(file, &dump) {
                                eprintln!("Failed to write frame dump '{path}': {e}");
                                std::process::exit(1);
                            }
                            eprintln!("Frame dump written to {path}");
                        }
                        Err(e) => {
                            eprintln!("Failed to create file '{path}': {e}");
                            std::process::exit(1);
                        }
                    }
                    self.dump_frame_written = true;
                    state.core.draw_list.stop_recording();
                    if do_exit && is_standalone {
                        event_loop.exit();
                    }
                }

                if is_capture_frame {
                    let capture_path = if let Some(p) = &script_capture_path {
                        p.clone()
                    } else {
                        screenshot_path.clone().unwrap()
                    };
                    match state
                        .core
                        .take_screenshot(&state.device, &state.queue, encoder, &output)
                    {
                        Ok(frame) => match std::fs::File::create(&capture_path) {
                            Ok(file) => {
                                let mut png_encoder =
                                    png::Encoder::new(file, frame.width, frame.height);
                                png_encoder.set_color(png::ColorType::Rgba);
                                png_encoder.set_depth(png::BitDepth::Eight);
                                match png_encoder.write_header() {
                                    Ok(mut writer) => {
                                        if let Err(e) = writer.write_image_data(&frame.rgba) {
                                            eprintln!("Failed to write PNG data: {e}");
                                            std::process::exit(1);
                                        }
                                        eprintln!("Screenshot saved to {capture_path}");
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to write PNG header: {e}");
                                        std::process::exit(1);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to create file '{capture_path}': {e}");
                                std::process::exit(1);
                            }
                        },
                        Err(e) => {
                            eprintln!("Screenshot failed: {e}");
                            std::process::exit(1);
                        }
                    }
                    self.screenshot_taken = true;
                    if do_exit {
                        event_loop.exit();
                    }
                } else {
                    state.queue.submit(std::iter::once(encoder.finish()));
                }
                output.present();
            }
            _ => {}
        }

        process_window_event(&mut state.core.input, &event);

        if let WindowEvent::RedrawRequested = event {
        } else {
            state.window.request_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(original_index: usize, amount: f64, name: &str) -> Record {
        Record {
            original_index,
            amount,
            cells: std::array::from_fn(|column| {
                if column == 1 {
                    name.to_string()
                } else {
                    String::new()
                }
            }),
        }
    }

    #[test]
    fn record_sorting_preserves_numeric_and_case_insensitive_order() {
        let a = record(1, 20.0, "alice");
        let b = record(2, 10.0, "Bob");

        assert_eq!(compare_records(&a, &b, 0, true), std::cmp::Ordering::Less);
        assert_eq!(
            compare_records(&a, &b, 6, true),
            std::cmp::Ordering::Greater
        );
        assert_eq!(compare_records(&a, &b, 1, true), std::cmp::Ordering::Less);
        assert_eq!(
            compare_records(&a, &b, 1, false),
            std::cmp::Ordering::Greater
        );
    }
}
