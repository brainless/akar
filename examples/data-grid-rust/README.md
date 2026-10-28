# Data grid example

The example uses the operating system's decorated, resizable window. To run with
optional window diagnostics:

```sh
AKAR_DEBUG_WINDOW=1 cargo run --bin data-grid-rust 2>&1 | tee -a /tmp/akar-native-resize.log
```

Each launch logs `resize-mode=native-only`, its PID, startup activation, focus,
mouse buttons, and resize events. Event lines include elapsed milliseconds and
previous/current physical sizes; pointer movement is recorded for button events
without logging each movement. Logs establish activation and size changes;
visually observe cursor appearance and rendering stability during resize checks.
Native resizing preserves synchronous resize rendering and macOS transaction
presentation.

On macOS, the native-only path was checked on an initial terminal launch and two
quit/relaunch cycles. All three logs showed width-only, height-only, and combined
size changes. The user confirmed normal resize cursors throughout and no manual
window or Dock activation before testing. The initial launch briefly lost focus
and recovered before dragging; these checks do not establish immediately stable
startup focus. Startup activation behavior is unchanged.
