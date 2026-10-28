use std::sync::OnceLock;
use std::time::Instant;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::WindowEvent,
    window::Window,
};

pub(crate) fn tracing_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("AKAR_DEBUG_WINDOW").is_some())
}

pub struct WindowDebug {
    started: Instant,
    last_size: PhysicalSize<u32>,
    position: Option<PhysicalPosition<f64>>,
}

impl WindowDebug {
    pub fn new(window: &Window) -> Self {
        let debug = Self {
            started: Instant::now(),
            last_size: window.inner_size(),
            position: None,
        };
        if tracing_enabled() {
            eprintln!(
                "[window] ready pid={} elapsed-ms=0 exe={:?} resize-mode=native-only size={:?} scale={} focused={} resizable={} maximized={} fullscreen={}",
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
        debug
    }

    pub fn event(&mut self, window: &Window, event: &WindowEvent) {
        if !tracing_enabled() {
            return;
        }
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = Some(*position);
                return;
            }
            WindowEvent::CursorLeft { .. } => self.position = None,
            WindowEvent::Focused(_)
            | WindowEvent::CursorEntered { .. }
            | WindowEvent::Resized(_)
            | WindowEvent::ScaleFactorChanged { .. }
            | WindowEvent::MouseInput { .. }
            | WindowEvent::CloseRequested => {}
            _ => return,
        }
        eprintln!(
            "[window] pid={} elapsed-ms={} event={event:?} previous-size={:?} current-size={:?} pointer={:?} scale={} focused={} resizable={} maximized={} fullscreen={}",
            std::process::id(),
            self.started.elapsed().as_millis(),
            self.last_size,
            window.inner_size(),
            self.position,
            window.scale_factor(),
            window.has_focus(),
            window.is_resizable(),
            window.is_maximized(),
            window.fullscreen().is_some(),
        );
        if let WindowEvent::Resized(size) = event {
            self.last_size = *size;
        }
    }
}
