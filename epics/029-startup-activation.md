# Epic 029: Opt-in Startup Activation

**Status:** Done (2026-10-28)
**Goal:** Move the data-grid example's bounded macOS unbundled-launch activation workaround into optional `akar-winit` integration without taking ownership of host scheduling.

## Scope

- Export `StartupActivation::for_unbundled_launch()`, `poll(&Window, Instant)`, and `cancel()`.
- Report `Pending { wake_at }`, `Focused`, `TimedOut`, and `Skipped`; terminal results remain stable.
- Bound and throttle the Dock/application/window handoff. Skip bundled, accessory, hidden, and non-macOS applications/windows.
- Keep Objective-C dependencies macOS-only and use the library's `log` facade.
- Migrate data-grid's existing explicit opt-in; keep all host deadlines under host control.
- Document that intentional inactivity must be handled by the caller because winit does not expose its original startup preferences on `Window`.

## Verification

- [x] Deterministic state-machine and scheduling tests: `cargo test -p akar-winit -p data-grid-rust` passes (12 library tests and 3 example tests).
- [x] `cargo fmt --check`, `git diff --check`, and `cargo check -p akar-winit -p data-grid-rust` pass.
- [x] Final workspace regression checks: `cargo fmt --check`, `git diff --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` pass. The workspace test suite was rerun outside the sandbox for Metal adapter access.
- [x] Record source findings and live-verification limitations.

## Findings and limitations

The old example's `drive()` unconditionally replaced control flow with `Wait`
or `WaitUntil`. That would conflict with hosts such as demo-rust, whose
`about_to_wait` owns screenshot deadlines. The shared helper therefore accepts
no event-loop handle. Data-grid recomputes its host schedule before merging
pending activation, so terminal progress cannot leave an expired wake deadline.

Both the local winit checkout and the pinned winit 0.30.13 implementation preserve
bundled applications' manifest-controlled activation policy. The helper checks
bundle identity, Regular activation policy, and window visibility at each poll.
Intentional inactivity remains caller-controlled because winit exposes no getter
for the original `with_active(false)` or activation opt-out setting.

The other four cited applications use ordinary winit window creation. That is
source-level exposure, not reproduced startup failure in each application. Only
data-grid's existing opt-in is migrated. Native macOS code compiles and the pure
state machine covers throttling, transient focus, timeout, eligibility, and
cancellation. No live Dock handoff or Finder/LaunchServices behavior was verified
as part of this change; non-macOS behavior is implemented as a no-op but was not
cross-compiled on this macOS host.

Epics 024 and 025 remain explicitly deferred. This epic does not enable the workaround in the other applications merely because their source uses ordinary winit startup.
