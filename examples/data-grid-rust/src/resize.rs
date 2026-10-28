use std::sync::OnceLock;

use winit::{
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, MouseButton, WindowEvent},
    window::{CursorIcon, Window},
};

pub const MIN_WINDOW_SIZE: LogicalSize<f64> = LogicalSize::new(320.0, 240.0);
const EDGE_WIDTH: f64 = 6.0;

pub(crate) fn tracing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("AKAR_DEBUG_WINDOW").is_some())
}

pub fn trace_window_ready(window: &Window) {
    if tracing_enabled() {
        eprintln!(
            "[window] ready pid={} exe={:?} size={:?} scale={} focused={} resizable={} maximized={} fullscreen={}",
            std::process::id(),
            std::env::current_exe(),
            window.inner_size(),
            window.scale_factor(),
            window.has_focus(),
            window.is_resizable(),
            window.is_maximized(),
            window.fullscreen().is_some(),
        );
    }
}

fn resize_allowed(resizable: bool, fullscreen: bool, maximized: bool) -> bool {
    // AppKit's zoomed windows remain resizable; fullscreen windows do not.
    resizable && !fullscreen && (cfg!(target_os = "macos") || !maximized)
}

fn can_resize(window: &Window) -> bool {
    resize_allowed(
        window.is_resizable(),
        window.fullscreen().is_some(),
        window.is_maximized(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edge {
    Right,
    Bottom,
    Corner,
}

fn edge_at(position: PhysicalPosition<f64>, size: PhysicalSize<u32>, scale: f64) -> Option<Edge> {
    let x = position.x;
    let y = position.y;
    if x < 0.0 || y < 0.0 || x >= size.width as f64 || y >= size.height as f64 {
        return None;
    }
    let right = x >= size.width as f64 - EDGE_WIDTH * scale;
    let bottom = y >= size.height as f64 - EDGE_WIDTH * scale;
    match (right, bottom) {
        (true, true) => Some(Edge::Corner),
        (true, false) => Some(Edge::Right),
        (false, true) => Some(Edge::Bottom),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct Drag {
    edge: Edge,
    origin: PhysicalPosition<f64>,
    size: PhysicalSize<u32>,
}

impl Drag {
    fn size_at(self, position: PhysicalPosition<f64>, scale: f64) -> PhysicalSize<u32> {
        let minimum = MIN_WINDOW_SIZE.to_physical::<u32>(scale);
        let width = if self.edge != Edge::Bottom {
            (self.size.width as f64 + position.x - self.origin.x)
                .round()
                .clamp(minimum.width as f64, u32::MAX as f64) as u32
        } else {
            self.size.width
        };
        let height = if self.edge != Edge::Right {
            (self.size.height as f64 + position.y - self.origin.y)
                .round()
                .clamp(minimum.height as f64, u32::MAX as f64) as u32
        } else {
            self.size.height
        };
        PhysicalSize::new(width, height)
    }
}

#[derive(Default)]
pub struct ResizeEdges {
    position: Option<PhysicalPosition<f64>>,
    hovered: Option<Edge>,
    drag: Option<Drag>,
}

impl ResizeEdges {
    fn set_hovered(&mut self, window: &Window, edge: Option<Edge>) {
        if self.hovered != edge {
            self.hovered = edge;
            self.refresh_cursor(window);
        }
    }

    fn refresh_cursor(&self, window: &Window) {
        let cursor = match self.hovered {
            Some(Edge::Right) => CursorIcon::EwResize,
            Some(Edge::Bottom) => CursorIcon::NsResize,
            Some(Edge::Corner) => CursorIcon::NwseResize,
            None => CursorIcon::Default,
        };
        if tracing_enabled() {
            eprintln!("[window] cursor-request={cursor:?} edge={:?}", self.hovered);
        }
        window.set_cursor(cursor);
    }

    pub fn process(&mut self, window: &Window, event: &WindowEvent) -> bool {
        if tracing_enabled() {
            let should_trace = match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.position.is_none()
                        || edge_at(*position, window.inner_size(), window.scale_factor())
                            != self.hovered
                }
                WindowEvent::Focused(_)
                | WindowEvent::CursorEntered { .. }
                | WindowEvent::CursorLeft { .. }
                | WindowEvent::Resized(_)
                | WindowEvent::ScaleFactorChanged { .. }
                | WindowEvent::MouseInput { .. } => true,
                _ => false,
            };
            if should_trace {
                eprintln!(
                    "[window] event={event:?} size={:?} scale={} focused={} resizable={} maximized={} fullscreen={} resize-allowed={} edge={:?}",
                    window.inner_size(),
                    window.scale_factor(),
                    window.has_focus(),
                    window.is_resizable(),
                    window.is_maximized(),
                    window.fullscreen().is_some(),
                    can_resize(window),
                    self.hovered,
                );
            }
        }
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = Some(*position);
                if let Some(drag) = self.drag {
                    let _ =
                        window.request_inner_size(drag.size_at(*position, window.scale_factor()));
                    return true;
                }
                let edge = if can_resize(window) {
                    edge_at(*position, window.inner_size(), window.scale_factor())
                } else {
                    None
                };
                self.set_hovered(window, edge);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    if can_resize(window) {
                        if let Some(origin) = self.position {
                            let size = window.inner_size();
                            if let Some(edge) = edge_at(origin, size, window.scale_factor()) {
                                // macOS does not support winit's drag_resize_window API.
                                self.drag = Some(Drag { edge, origin, size });
                                return true;
                            }
                        }
                    }
                }
                ElementState::Released => {
                    if self.drag.take().is_some() {
                        self.set_hovered(window, None);
                        return true;
                    }
                }
            },
            WindowEvent::Focused(false) | WindowEvent::ScaleFactorChanged { .. } => {
                self.drag = None;
                self.position = None;
                self.set_hovered(window, None);
            }
            WindowEvent::CursorLeft { .. } if self.drag.is_none() => {
                self.position = None;
                self.set_hovered(window, None);
            }
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resizable_zoomed_macos_windows_keep_resize_targets() {
        assert!(resize_allowed(true, false, false));
        assert_eq!(resize_allowed(true, false, true), cfg!(target_os = "macos"));
        assert!(!resize_allowed(true, true, false));
        assert!(!resize_allowed(false, false, false));
    }

    #[test]
    fn resize_targets_use_logical_width_and_exclude_outside_positions() {
        let size = PhysicalSize::new(1600, 1200);
        for (position, expected) in [
            ((1589.0, 400.0), Some(Edge::Right)),
            ((400.0, 1189.0), Some(Edge::Bottom)),
            ((1599.0, 1199.0), Some(Edge::Corner)),
            ((1587.0, 1187.0), None),
            ((1600.0, 400.0), None),
            ((400.0, 1200.0), None),
            ((-1.0, 1199.0), None),
        ] {
            assert_eq!(
                edge_at(PhysicalPosition::from(position), size, 2.0),
                expected
            );
        }
    }

    #[test]
    fn dragging_preserves_other_axis_and_clamps_to_scaled_minimum() {
        let drag = Drag {
            edge: Edge::Corner,
            origin: PhysicalPosition::new(1599.0, 1199.0),
            size: PhysicalSize::new(1600, 1200),
        };
        let position = PhysicalPosition::new(1699.0, 1249.0);
        assert_eq!(drag.size_at(position, 2.0), PhysicalSize::new(1700, 1250));
        assert_eq!(
            Drag {
                edge: Edge::Right,
                ..drag
            }
            .size_at(position, 2.0),
            PhysicalSize::new(1700, 1200)
        );
        assert_eq!(
            Drag {
                edge: Edge::Bottom,
                ..drag
            }
            .size_at(position, 2.0),
            PhysicalSize::new(1600, 1250)
        );
        assert_eq!(
            drag.size_at(PhysicalPosition::new(-1000.0, -1000.0), 2.0),
            PhysicalSize::new(640, 480)
        );
    }
}
