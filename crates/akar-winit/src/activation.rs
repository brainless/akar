use std::time::Instant;
use winit::window::Window;

/// The result of a startup activation poll. Terminal results remain unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationProgress {
    /// Poll again at this deadline, merged with the host's own wake deadlines.
    Pending { wake_at: Instant },
    /// The application and window have remained focused long enough to settle.
    Focused,
    /// Activation did not settle within the one-second startup budget.
    TimedOut,
    /// The platform or launch is ineligible, or activation was cancelled.
    Skipped,
}

/// Explicitly opted-in, one-shot startup focus recovery for unbundled macOS apps.
///
/// Create after winit starts and poll on its main thread with the same window.
/// Bundled apps, non-Regular activation policies, and invisible windows are
/// skipped. Other platforms are no-ops. This never changes host control flow.
///
/// Do not opt in for intentionally inactive launches, including
/// `WindowAttributes::with_active(false)` or macOS
/// `with_activate_ignoring_other_apps(false)`: winit does not expose those
/// original preferences through `Window`. Cancel when closing or abandoning
/// startup. A completed helper never reactivates after a later focus loss.
pub struct StartupActivation {
    #[cfg(target_os = "macos")]
    machine: Machine,
}

impl StartupActivation {
    /// Opt in to bounded Dock/application/window activation recovery.
    pub fn for_unbundled_launch() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            machine: Machine::default(),
        }
    }

    /// Advance startup recovery, without sleeping or changing event-loop policy.
    ///
    /// Use a monotonic `now` from `Instant::now()`. Frequent calls do not issue
    /// additional activation requests before the returned wake deadline.
    pub fn poll(&mut self, window: &Window, now: Instant) -> ActivationProgress {
        #[cfg(target_os = "macos")]
        {
            macos::poll(&mut self.machine, window, now)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (window, now);
            ActivationProgress::Skipped
        }
    }

    /// Stop pending recovery without performing any platform actions.
    /// Already terminal results remain unchanged.
    pub fn cancel(&mut self) {
        #[cfg(target_os = "macos")]
        self.machine.cancel();
    }
}

#[cfg(any(target_os = "macos", test))]
use std::time::Duration;

#[cfg(any(target_os = "macos", test))]
const STARTUP_TIMEOUT: Duration = Duration::from_secs(1);
#[cfg(any(target_os = "macos", test))]
const POLL_INTERVAL: Duration = Duration::from_millis(50);
#[cfg(any(target_os = "macos", test))]
const FOCUS_SETTLE_INTERVAL: Duration = Duration::from_millis(100);

#[cfg(any(target_os = "macos", test))]
#[derive(Default)]
enum Phase {
    #[default]
    Initial,
    WaitingForDock,
    WaitingForApp,
    WaitingForWindow,
}

#[cfg(any(target_os = "macos", test))]
#[derive(Debug, PartialEq, Eq)]
enum Action {
    ActivateDock,
    Activate,
    Focus,
}

#[cfg(any(target_os = "macos", test))]
#[derive(Default)]
struct Machine {
    phase: Phase,
    deadline: Option<Instant>,
    focused_since: Option<Instant>,
    wake_at: Option<Instant>,
    terminal: Option<ActivationProgress>,
}

#[cfg(any(target_os = "macos", test))]
impl Machine {
    fn cancel(&mut self) {
        self.terminal.get_or_insert(ActivationProgress::Skipped);
    }

    fn poll(
        &mut self,
        now: Instant,
        eligible: bool,
        app_active: bool,
        window_focused: bool,
    ) -> (ActivationProgress, Option<Action>) {
        if let Some(progress) = self.terminal {
            return (progress, None);
        }
        if !eligible {
            self.cancel();
            return (ActivationProgress::Skipped, None);
        }
        let deadline = *self.deadline.get_or_insert(now + STARTUP_TIMEOUT);
        if now >= deadline {
            self.terminal = Some(ActivationProgress::TimedOut);
            return (ActivationProgress::TimedOut, None);
        }
        if !(app_active && window_focused) {
            self.focused_since = None;
        }
        if let Some(wake_at) = self.wake_at.filter(|wake_at| now < *wake_at) {
            return (ActivationProgress::Pending { wake_at }, None);
        }
        let action = if matches!(self.phase, Phase::WaitingForDock) {
            // Complete the asynchronous Dock handoff even if focus is transiently ours.
            self.phase = Phase::WaitingForApp;
            Some(Action::Activate)
        } else if app_active && window_focused {
            let focused_since = *self.focused_since.get_or_insert(now);
            if now.saturating_duration_since(focused_since) >= FOCUS_SETTLE_INTERVAL {
                self.terminal = Some(ActivationProgress::Focused);
                return (ActivationProgress::Focused, None);
            }
            None
        } else if app_active && !matches!(self.phase, Phase::WaitingForWindow) {
            self.phase = Phase::WaitingForWindow;
            Some(Action::Focus)
        } else if matches!(self.phase, Phase::Initial) {
            self.phase = Phase::WaitingForDock;
            Some(Action::ActivateDock)
        } else {
            None
        };
        let wake_at = (now + POLL_INTERVAL).min(deadline);
        self.wake_at = Some(wake_at);
        (ActivationProgress::Pending { wake_at }, action)
    }
}

#[cfg(any(target_os = "macos", test))]
fn eligible(unbundled: bool, regular: bool, visible: bool) -> bool {
    unbundled && regular && visible
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use objc2::{available, MainThreadMarker};
    use objc2_app_kit::{
        NSApplication, NSApplicationActivationOptions, NSApplicationActivationPolicy,
        NSRunningApplication,
    };
    use objc2_foundation::ns_string;

    pub(super) fn poll(machine: &mut Machine, window: &Window, now: Instant) -> ActivationProgress {
        if let Some(progress) = machine.terminal {
            return progress;
        }
        let Some(mtm) = MainThreadMarker::new() else {
            machine.cancel();
            return ActivationProgress::Skipped;
        };
        let app = NSApplication::sharedApplication(mtm);
        let eligible = eligible(
            NSRunningApplication::currentApplication()
                .bundleIdentifier()
                .is_none(),
            app.activationPolicy() == NSApplicationActivationPolicy::Regular,
            window.is_visible() == Some(true),
        );
        let (progress, action) = machine.poll(now, eligible, app.isActive(), window.has_focus());
        if let Some(action) = action {
            log::debug!("startup activation: {action:?}");
            match action {
                // Terminal-launched processes can be frontmost but inactive in AppKit.
                // Handing activation to the Dock first lets self-activation succeed.
                Action::ActivateDock => {
                    if let Some(dock) =
                        NSRunningApplication::runningApplicationsWithBundleIdentifier(ns_string!(
                            "com.apple.dock"
                        ))
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
                            .activateWithOptions(
                                NSApplicationActivationOptions::ActivateAllWindows,
                            );
                        log::debug!("startup activation accepted={accepted}");
                    }
                }
                Action::Focus => window.focus_window(),
            }
        }
        progress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excludes_bundled_nonregular_and_hidden_launches() {
        for unbundled in [false, true] {
            for regular in [false, true] {
                for visible in [false, true] {
                    assert_eq!(
                        eligible(unbundled, regular, visible),
                        unbundled && regular && visible
                    );
                }
            }
        }
        let now = Instant::now();
        let mut machine = Machine::default();
        assert_eq!(
            machine.poll(now, false, false, false),
            (ActivationProgress::Skipped, None)
        );
        assert_eq!(
            machine.poll(now, true, true, true),
            (ActivationProgress::Skipped, None)
        );
    }

    #[test]
    fn handoff_is_throttled_and_requires_stable_focus() {
        let now = Instant::now();
        let mut machine = Machine::default();
        assert_eq!(
            machine.poll(now, true, false, true).1,
            Some(Action::ActivateDock)
        );
        assert_eq!(
            machine.poll(now + POLL_INTERVAL / 2, true, true, true),
            (
                ActivationProgress::Pending {
                    wake_at: now + POLL_INTERVAL
                },
                None
            )
        );
        assert_eq!(
            machine.poll(now + POLL_INTERVAL, true, true, true).1,
            Some(Action::Activate)
        );
        assert_eq!(
            machine.poll(now + POLL_INTERVAL * 2, true, true, false).1,
            Some(Action::Focus)
        );
        assert_eq!(
            machine.poll(now + POLL_INTERVAL * 3, true, true, false).1,
            None
        );
        machine.poll(now + POLL_INTERVAL * 4, true, true, true);
        machine.poll(
            now + POLL_INTERVAL * 4 + Duration::from_millis(1),
            true,
            false,
            true,
        );
        machine.poll(now + POLL_INTERVAL * 5, true, true, true);
        assert!(matches!(
            machine.poll(now + POLL_INTERVAL * 6, true, true, true).0,
            ActivationProgress::Pending { .. }
        ));
        assert_eq!(
            machine.poll(now + POLL_INTERVAL * 7, true, true, true),
            (ActivationProgress::Focused, None)
        );
        machine.cancel();
        assert_eq!(
            machine.poll(now + STARTUP_TIMEOUT * 2, false, false, false),
            (ActivationProgress::Focused, None)
        );
    }

    #[test]
    fn timeout_is_bounded_and_sticky_even_after_cancellation() {
        let now = Instant::now();
        let mut machine = Machine::default();
        machine.poll(now, true, false, false);
        assert_eq!(
            machine
                .poll(
                    now + STARTUP_TIMEOUT - Duration::from_millis(1),
                    true,
                    false,
                    false
                )
                .0,
            ActivationProgress::Pending {
                wake_at: now + STARTUP_TIMEOUT
            }
        );
        assert_eq!(
            machine.poll(now + STARTUP_TIMEOUT, true, false, false),
            (ActivationProgress::TimedOut, None)
        );
        machine.cancel();
        assert_eq!(
            machine.poll(now + STARTUP_TIMEOUT * 2, true, true, true),
            (ActivationProgress::TimedOut, None)
        );
    }

    #[test]
    fn cancellation_before_or_during_startup_performs_no_later_actions() {
        let now = Instant::now();
        for started in [false, true] {
            let mut machine = Machine::default();
            if started {
                machine.poll(now, true, false, false);
            }
            machine.cancel();
            assert_eq!(
                machine.poll(now + POLL_INTERVAL, true, false, false),
                (ActivationProgress::Skipped, None)
            );
        }
    }

    #[test]
    fn already_active_app_never_hands_activation_to_dock() {
        let now = Instant::now();
        let mut machine = Machine::default();
        assert_eq!(machine.poll(now, true, true, false).1, Some(Action::Focus));
        machine.poll(now + POLL_INTERVAL, true, true, true);
        assert_eq!(
            machine.poll(
                now + POLL_INTERVAL + FOCUS_SETTLE_INTERVAL,
                true,
                true,
                true
            ),
            (ActivationProgress::Focused, None)
        );
    }
}
