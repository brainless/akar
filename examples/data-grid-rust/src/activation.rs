use std::time::{Duration, Instant};

use objc2::{available, MainThreadMarker};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationOptions, NSRunningApplication, NSWorkspace,
};
use objc2_foundation::ns_string;
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow},
    window::Window,
};

const STARTUP_TIMEOUT: Duration = Duration::from_secs(1);
const POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Default, Debug, PartialEq, Eq)]
enum Phase {
    #[default]
    Initial,
    WaitingForDock,
    WaitingForApp,
    WaitingForWindow,
    Finished,
}

#[derive(Debug, PartialEq, Eq)]
enum Action {
    ActivateDock,
    Activate,
    Focus,
    Wait,
    Finish,
    Timeout,
}

#[derive(Default)]
pub struct StartupActivation {
    phase: Phase,
    deadline: Option<Instant>,
}

impl StartupActivation {
    fn poll(&mut self, now: Instant, app_active: bool, window_focused: bool) -> Action {
        if self.phase == Phase::Finished {
            return Action::Finish;
        }
        if window_focused {
            self.phase = Phase::Finished;
            return Action::Finish;
        }
        let deadline = *self.deadline.get_or_insert(now + STARTUP_TIMEOUT);
        if now >= deadline {
            self.phase = Phase::Finished;
            return Action::Timeout;
        }
        if app_active && self.phase != Phase::WaitingForWindow {
            self.phase = Phase::WaitingForWindow;
            return Action::Focus;
        }
        match self.phase {
            Phase::Initial => {
                self.phase = Phase::WaitingForDock;
                return Action::ActivateDock;
            }
            Phase::WaitingForDock => {
                self.phase = Phase::WaitingForApp;
                return Action::Activate;
            }
            _ => {}
        }
        Action::Wait
    }

    pub fn drive(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        if self.phase == Phase::Finished || event_loop.exiting() {
            return;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            self.phase = Phase::Finished;
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        let now = Instant::now();
        let action = self.poll(now, app.isActive(), window.has_focus());
        if action != Action::Wait {
            trace(&app, window, &action);
        }
        match action {
            // A terminal-launched unbundled process can be reported frontmost while AppKit
            // keeps it inactive, which leaves it without a key window and ignores cursor
            // updates. Handing activation to the Dock first lets the self-activation succeed.
            Action::ActivateDock => {
                if let Some(dock) = NSRunningApplication::runningApplicationsWithBundleIdentifier(
                    ns_string!("com.apple.dock"),
                )
                .firstObject()
                {
                    #[allow(deprecated)]
                    dock.activateWithOptions(
                        NSApplicationActivationOptions::ActivateIgnoringOtherApps,
                    );
                }
            }
            Action::Activate => {
                if available!(macos = 14.0) {
                    app.activate();
                } else {
                    let accepted = NSRunningApplication::currentApplication()
                        .activateWithOptions(NSApplicationActivationOptions::ActivateAllWindows);
                    if crate::resize::tracing_enabled() {
                        eprintln!("[window] activation-request accepted={accepted}");
                    }
                }
            }
            Action::Focus => window.focus_window(),
            Action::Finish | Action::Timeout => {
                event_loop.set_control_flow(ControlFlow::Wait);
                return;
            }
            Action::Wait => {}
        }
        let next_poll = (now + POLL_INTERVAL).min(self.deadline.unwrap());
        event_loop.set_control_flow(ControlFlow::WaitUntil(next_poll));
    }

    pub fn focused(&mut self, event_loop: &ActiveEventLoop) {
        if self.phase != Phase::Finished {
            self.phase = Phase::Finished;
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}

fn trace(app: &NSApplication, window: &Window, action: &Action) {
    if crate::resize::tracing_enabled() {
        let frontmost_pid = NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .map(|application| application.processIdentifier());
        eprintln!(
            "[window] startup-activation action={action:?} app-active={} focused={} frontmost-pid={frontmost_pid:?} pid={}",
            app.isActive(),
            window.has_focus(),
            std::process::id(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waits_for_app_activation_before_requesting_window_focus_once() {
        let now = Instant::now();
        let mut activation = StartupActivation::default();
        assert_eq!(activation.poll(now, false, false), Action::ActivateDock);
        assert_eq!(
            activation.poll(now + POLL_INTERVAL / 2, false, false),
            Action::Activate
        );
        assert_eq!(
            activation.poll(now + POLL_INTERVAL, false, false),
            Action::Wait
        );
        assert_eq!(
            activation.poll(now + POLL_INTERVAL * 2, true, false),
            Action::Focus
        );
        assert_eq!(
            activation.poll(now + POLL_INTERVAL * 3, true, false),
            Action::Wait
        );
        assert_eq!(
            activation.poll(now + POLL_INTERVAL * 4, true, true),
            Action::Finish
        );
        assert_eq!(
            activation.poll(now + POLL_INTERVAL * 5, false, false),
            Action::Finish
        );
    }

    #[test]
    fn denied_activation_stops_at_deadline_without_further_requests() {
        let now = Instant::now();
        let mut activation = StartupActivation::default();
        assert_eq!(activation.poll(now, false, false), Action::ActivateDock);
        assert_eq!(
            activation.poll(now + POLL_INTERVAL, false, false),
            Action::Activate
        );
        assert_eq!(
            activation.poll(now + STARTUP_TIMEOUT, false, false),
            Action::Timeout
        );
        assert_eq!(
            activation.poll(now + STARTUP_TIMEOUT * 2, false, false),
            Action::Finish
        );
    }

    #[test]
    fn already_active_app_skips_activation_request() {
        let mut activation = StartupActivation::default();
        assert_eq!(activation.poll(Instant::now(), true, false), Action::Focus);
    }
}
