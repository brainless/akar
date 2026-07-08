# Epic 013: Screenshot Utility

**Status:** In Progress
**Goal:** Add a window screenshot capability to akar and expose it through the `demo-rust` binary via a CLI flag. After a configurable delay the demo captures its own window and writes it to a PNG file, enabling visual regression checks and UI glitch analysis without external tools.

**Revision note (2026-07-08):** The original Approach A design added `COPY_SRC` directly to the surface's `TextureUsages`. Research against wgpu internals and egui's production capture code (see "Implementation Plan" below) showed this is unsafe: surface `COPY_SRC` support is a live driver capability (`wgpu-hal/src/vulkan/adapter.rs` queries `VkSurfaceCapabilitiesKHR.supportedUsageFlags`), not a guarantee, and requesting an unsupported usage fails at `surface.configure()` time (`wgpu-core::present::ConfigureSurfaceError::UnsupportedUsage`) — i.e. at startup, unconditionally, even when no screenshot is requested. The plan below replaces the direct-surface-flag approach with egui's pattern: render the captured frame into a plain intermediate `wgpu::Texture` (which has no driver-dependent usage restrictions) and blit it to the surface, gated so it only happens on the frame a screenshot is actually taken.

**Primary Use Case:** Enable coding agent debug workflows. When developers use AI coding agents (Claude, Cursor, etc.) to build UIs with akar, the agents need a way to "see" what the UI looks like. By providing out-of-the-box screenshot support, akar allows coding agents to capture the current UI state, analyze it for bugs or styling issues, and iterate on improvements without requiring developers to manually take and share screenshots.

**Prerequisite:** Epic 012 is `Status: Done` and `cargo clippy --workspace -- -D warnings` passes clean.

---

## Approaches Considered

### Approach A: wgpu Surface Readback (Recommended)

**How it works:** Read pixels directly from the wgpu surface texture after rendering, using `copy_texture_to_buffer` and `map_async`.

**Advantages:**
- **Cross-platform by default** - Same code works on macOS, Windows, Linux. No platform-specific implementations.
- **Zero new dependencies** - wgpu already provides everything needed.
- **No permission issues** - macOS screen recording permission is NOT required (unlike platform capture APIs).
- **Captures exactly what akar rendered** - No OS chrome, no overlapping windows, no taskbar. For agent debugging, you want to see akar's output, not the compositor's output.
- **Deterministic** - Platform capture can be affected by window visibility, overlapping windows, display scaling. wgpu readback is pixel-perfect from the render target.
- **Simpler implementation** - ~50 lines in akar-core vs ~200+ lines of platform-specific code in akar-winit.
- **Aligns with akar's architecture** - Screenshot is a rendering concern, not a windowing concern. `akar-core` owns the wgpu pipelines and render flow.

**Platform support:**
- The revised mechanism (intermediate texture, see Implementation Plan) does not request `COPY_SRC` on the surface at all, so it has no per-driver surface-capability dependency. It works identically on Metal, DX12, and Vulkan.
- **GLES** - out of scope; akar targets desktop backends only.

**Implementation location:** `akar-core` (not `akar-winit`)

**Reference implementation:** egui's `CaptureState` in `egui-wgpu/src/capture.rs` demonstrates this pattern in production. Its key insight (`capture.rs:6-13`): `COPY_SRC` is not an allowed flag for the surface texture on all platforms, so egui never requests it there — it always renders the capturable frame into a dedicated offscreen texture instead, and only pays for that texture + a blit pass on frames where a capture was actually requested (`egui-wgpu/src/winit.rs:636-645`, gated by a `capture` bool).

### Approach B: Platform Window Capture (Original Epic Design)

**How it works:** Use platform-specific APIs to capture the window from the OS compositor:
- macOS: `CGWindowListCreateImage` via core-graphics
- Windows: `BitBlt` or `PrintWindow` via winapi
- Linux: X11/Wayland capture APIs

**Advantages:**
- Captures the entire window including OS decorations (if desired)
- Works even if surface texture doesn't support `COPY_SRC` (e.g., GLES)

**Disadvantages:**
- **Heavy dependencies** - Adds `objc2`, `objc2-app-kit`, `objc2-foundation`, `core-graphics` on macOS; `windows` crate on Windows; `x11rb` or similar on Linux
- **Permission issues** - macOS requires screen recording permission for `CGWindowListCreateImage`
- **Non-deterministic** - Captures whatever the compositor shows, which may include overlapping windows or be affected by display scaling
- **Complex implementation** - Requires per-platform code with `#[cfg(target_os = ...)]`
- **Captures more than needed** - For agent debugging, you want akar's output, not the full window chrome

**Implementation location:** `akar-winit` (windowing layer)

### Recommendation

**Use Approach A (wgpu surface readback)** for the following reasons:

1. **Coding agent debug workflows** need to see akar's rendered output, not the OS window chrome. Platform capture adds noise (title bars, borders, overlapping windows) that doesn't help agents debug UI layout or styling issues.

2. **Cross-platform consistency** is critical. akar aims to work on macOS, Windows, and Linux. Approach A provides identical behavior on all platforms with zero platform-specific code.

3. **Zero-dependency addition** keeps akar's dependency tree lean. Approach B adds ~5 platform-specific crates that only provide screenshot functionality.

4. **Simpler maintenance** - One code path to maintain vs three (macOS, Windows, Linux).

5. **Aligns with akar's architecture** - The library doesn't own the swap chain (per AGENTS.md), but it does own the render pipelines. Screenshot capture is a natural extension of the render pipeline, not the windowing layer.

**Decision: the updated Approach A (intermediate-texture readback, described in the Implementation Plan below) is the approach to implement.** Do not implement the original direct-surface-`COPY_SRC` variant described in earlier revisions of this epic — it was superseded by the revision above once the surface-capability risk was found. Any implementation work on this epic should start from the "Implementation Plan" section as written now.

**When to use Approach B instead:**
- If akar needs to support GLES or WebGPU backends, where `copy_texture_to_buffer` from any GPU texture is unreliable or unavailable
- If users need to capture the full OS window including decorations
- If akar's rendering is embedded in a larger application and you need to capture the entire window, not just akar's content

---

## Implementation Plan (Approach A: wgpu Intermediate-Texture Readback)

### Part A: Screenshot capture in `akar-core`

`crates/akar-core/src/screenshot.rs`:

- `CapturedFrame { width: u32, height: u32, rgba: Vec<u8> }` - raw RGBA pixels from the captured frame
- `ScreenshotError` (via `thiserror`):
  - `BufferMapFailed(String)` - GPU buffer mapping failed
  - `EncodingError(String)` - PNG encoding failed
  - (no `SurfaceCopyNotSupported` variant - see below; the intermediate-texture mechanism has no surface-capability dependency to fail on)
- `ScreenshotCapture` struct owning:
  - `texture: Option<wgpu::Texture>` - the offscreen capture target, lazily (re)created at the surface's current size with `RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_SRC` (all three are always legal on a plain `wgpu::Texture`, unlike a surface texture)
  - `blit_pipeline: wgpu::RenderPipeline` + `blit_bind_group_layout` + `sampler` - a single fullscreen-triangle pipeline that samples the capture texture and writes it to the surface view (see egui's `egui-wgpu/src/texture_copy.wgsl` for a ready-made shader to adapt)
  - `requested: bool` - set by `request_screenshot()`, cleared once a capture completes
- `AkarCore::request_screenshot()` - sets `requested = true`; does not affect surface configuration
- `AkarCore::capture_target_view(&mut self, device: &Device, width: u32, height: u32, format: TextureFormat) -> Option<&wgpu::TextureView>` - returns `Some(view into the offscreen texture)` when a screenshot is pending for this frame, `None` otherwise. The caller (the demo's render loop) uses this in place of the surface view as the render pass's color attachment **only on capture frames**; on every other frame rendering targets the surface directly as it does today, so there is no per-frame overhead from this feature when it isn't in use.
- `AkarCore::take_screenshot(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, surface_texture: &wgpu::SurfaceTexture) -> Result<CapturedFrame, ScreenshotError>` - called after the main render pass, while still recording into the same encoder:
  1. Runs the blit pass: capture texture -> `surface_texture` view, so the on-screen window still shows the frame that was "secretly" rendered offscreen.
  2. Records `copy_texture_to_buffer` from the capture texture into a staging buffer.
  3. After `queue.submit(...)`, maps the staging buffer and reads pixels back into a `CapturedFrame`.

**This mirrors egui's `CaptureState` (`egui-wgpu/src/capture.rs`, wired up in `egui-wgpu/src/winit.rs:632-718`):** egui redirects its main render pass to an offscreen texture only on the frame a screenshot is requested (`let target_texture = if capture { &capture_state.texture } else { &output_frame.texture }`), then blits that texture onto the real surface texture and copies it to a buffer in the same encoder. akar's `capture_target_view` / `take_screenshot` split reproduces that gating.

**Implementation details:**

1. **No surface configuration change needed.** `surface_config.usage` stays `RENDER_ATTACHMENT` exactly as today; the capture texture is a separate resource the demo creates through `AkarCore`, not a surface usage flag. This is the change from the original Approach A write-up (see "Revision note" above) — that version required `surface_config.usage = TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC`, which is not safe because surface `COPY_SRC` support is a per-driver capability, not guaranteed by wgpu (see `wgpu-hal/src/vulkan/adapter.rs:3177` `surface_capabilities()`, and `wgpu-core::present::ConfigureSurfaceError::UnsupportedUsage`).

2. **On a capture frame, copy the offscreen capture texture to a staging buffer** (identical padding math to the original plan, just reading from the capture texture instead of the surface texture):
   ```rust
   let unpadded_bytes_per_row = width * 4; // RGBA
   let padded_bytes_per_row = align_to(unpadded_bytes_per_row, 256);
   let buffer_size = padded_bytes_per_row * height;
   
   let staging_buffer = device.create_buffer(&BufferDescriptor {
       size: buffer_size,
       usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
       ..
   });
   
   encoder.copy_texture_to_buffer(
       TexelCopyTextureInfo {
           texture: &capture_texture, // the offscreen texture, not the surface texture
           ..
       },
       TexelCopyBufferInfo {
           buffer: &staging_buffer,
           layout: TexelCopyBufferLayout {
               bytes_per_row: Some(padded_bytes_per_row),
               rows_per_image: Some(height),
               ..
           },
       },
       Extent3d { width, height, .. },
   );
   ```

3. **Map buffer and read pixels:**
   ```rust
   queue.submit(Some(encoder.finish()));
   
   let buffer_slice = staging_buffer.slice(..);
   let (sender, receiver) = std::sync::mpsc::channel();
   buffer_slice.map_async(MapMode::Read, move |result| {
       sender.send(result).unwrap();
   });
   device.poll(PollType::wait_indefinitely()).unwrap();
   receiver.recv().unwrap().unwrap();
   
   let data = buffer_slice.get_mapped_range();
   // Strip padding from each row and handle BGRA→RGBA if needed
   let rgba: Vec<u8> = data.chunks(padded_bytes_per_row as usize)
       .flat_map(|row| row[..unpadded_bytes_per_row as usize].to_vec())
       .collect();
   ```

4. **Handle capture texture format:** Check if the format is `Bgra8Unorm` **or** `Bgra8UnormSrgb` and swap R/B channels if needed. `surface.get_default_config()` commonly returns the sRGB variant on macOS/Metal, and the capture texture is created with the same format as the surface, so the sRGB variant must be checked too, not just the plain one.

### Part B: Dependency updates

`crates/akar-core/Cargo.toml`:
- `thiserror` is already in workspace dependencies (no change needed)
- No new crate dependency for the blit pass: it's a hand-written WGSL shader (`crates/akar-core/src/shaders/blit.wgsl`, adapted from egui's `egui-wgpu/src/texture_copy.wgsl`) plus a `wgpu::RenderPipeline`, same pattern already used by `QuadPipeline`/`TextPipeline`.

`examples/demo-rust/Cargo.toml`:
- Add `png = "0.17"` for PNG encoding
- Add `anyhow = "1"` (matches DEVELOP.md binary error-handling convention)

### Part C: CLI and timer in `demo-rust`

`examples/demo-rust/src/main.rs`:

1. Parse CLI args in `main()`:
   - Recognize `--screenshot <PATH>`
   - No default path; the path is required when the flag is used
2. Extend `App` with:
   - `screenshot_path: Option<String>`
   - `screenshot_taken: bool`
   - `start_time: Option<Instant>`
3. `resumed()` is unchanged from today - `surface_config.usage` stays `TextureUsages::RENDER_ATTACHMENT`. No surface reconfiguration is needed for screenshots.
4. Set `start_time` in `resumed()` after the window and wgpu surface are ready
5. In the render loop, decide per-frame whether this is a capture frame (`screenshot_path.is_some() && !screenshot_taken && delay elapsed`):
   - If it is a capture frame, call `state.core.capture_target_view(&state.device, size.width, size.height, surface_format)` and use the returned view as the render pass's color attachment instead of the surface view. Otherwise render to the surface view exactly as today.
   - After the main render pass ends but before `encoder.finish()`, if this is a capture frame, call `state.core.take_screenshot(&state.device, &state.queue, &mut encoder, &output)`. This records the blit-to-surface and copy-to-buffer, so the surface still gets the frame written to it before `output.present()`.
   - After `queue.submit(...)` and `output.present()`, use the `CapturedFrame` returned by `take_screenshot` to encode a PNG with the `png` crate and write it to `screenshot_path`. Print success or error to stderr, then set `screenshot_taken = true`.
6. The app continues running after the screenshot so the window remains inspectable

---

## Acceptance Criteria

- `cargo check --workspace` passes
- `cargo clippy --workspace -- -D warnings` passes
- `cargo run --bin demo-rust -- --screenshot /tmp/akar_demo.png` renders the window for ~5 seconds, then writes a valid PNG of akar's rendered content to `/tmp/akar_demo.png`
- Running without `--screenshot` behaves exactly as before
- The screenshot contains only akar's rendered UI (no OS window chrome)
- Works identically on macOS, Windows, and Linux - the intermediate-texture mechanism has no dependency on the surface exposing `COPY_SRC`, so there is no driver-dependent failure mode to test around

---

## Open Questions

1. **After the screenshot is saved, should the demo exit automatically or keep running?**  
   Default proposal: keep running so the live window remains inspectable.

2. **Should the `--screenshot` flag accept a default output path, or require an explicit path?**  
   Default proposal: require an explicit path (`--screenshot <PATH>`).

3. **Should we implement a fallback to Approach B (platform capture) for GLES/WebGPU backends?**  
   Default proposal: No. akar targets desktop wgpu (Metal/DX12/Vulkan), where the intermediate-texture mechanism has no known failure mode. GLES and WebGPU support can be added later if needed; revisit whether a new `ScreenshotError` variant is warranted if/when that work starts.

4. **Should the screenshot capture be blocking or non-blocking?**  
   - **Blocking:** Simpler to implement. The frame is captured synchronously after rendering. The app pauses briefly (~10-50ms) while the GPU buffer is mapped.
   - **Non-blocking:** More complex. Use `map_async` with a callback and process the result on the next frame. Avoids any frame stutter.
   - Default proposal: **Blocking** for simplicity. The brief pause is acceptable for a debug tool. If users report stutter, we can optimize to non-blocking later.

5. **Should we support capturing at a specific resolution, or always capture at the current window size?**  
   Default proposal: Always capture at the current window size. Resolution-independent capture adds complexity (requires rendering to an offscreen texture at a different size) and is not needed for agent debugging.

6. **Should the screenshot include only akar's UI, or the entire surface (including the clear color)?**  
   Default proposal: The entire surface. This is what the user sees, and it's simpler to implement. akar's UI is rendered on top of the clear color anyway.

7. **Should we add a keyboard shortcut (e.g., F12) to trigger a screenshot at runtime, in addition to the CLI flag?**  
   Default proposal: No. The CLI flag is sufficient for agent workflows. Adding runtime shortcuts adds complexity and requires input handling logic. Can be added later if users request it.

8. **Should the screenshot API be exposed via the C ABI (`akar-c-api`)?**  
   Default proposal: No, not in this epic. The initial implementation is Rust-only. C ABI exposure can be added in a follow-up epic if non-Rust users need it.

---

## Notes

- The 256-byte alignment requirement for `bytes_per_row` in `copy_texture_to_buffer` is enforced by wgpu. See `wgpu-types/src/lib.rs:116` for `COPY_BYTES_PER_ROW_ALIGNMENT = 256`.
- If the capture texture format is `Bgra8Unorm` or `Bgra8UnormSrgb` (the latter is what `surface.get_default_config()` commonly returns on macOS/Metal), the captured pixels will be in BGRA order. Swap R and B channels to convert to RGBA before encoding, checking both format variants.
- The staging buffer should be reused across frames if screenshots are requested frequently, but for a debug tool, creating a new buffer per screenshot is acceptable.
- macOS may still request screen recording permission if the user runs the app in certain sandboxed environments, but this is rare and not caused by `CGWindowListCreateImage` (which we're not using).
- This epic does not add C ABI bindings; screenshot exposure is Rust-only for now.
- **Why the intermediate-texture approach, not direct surface `COPY_SRC` (superseded design):** requesting `COPY_SRC` on the surface's `TextureUsages` is not portable. `wgpu-hal/src/vulkan/adapter.rs:3177`'s `surface_capabilities()` queries the live Vulkan driver's `VkSurfaceCapabilitiesKHR.supportedUsageFlags`; if a driver doesn't advertise transfer-src support there, `surface.configure()` fails with `wgpu-core::present::ConfigureSurfaceError::UnsupportedUsage` (`wgpu-core/src/present.rs:117-118`) — at startup, every run, regardless of whether `--screenshot` was passed. A plain `wgpu::Texture` has no such restriction, which is why egui's `CaptureState` (`egui-wgpu/src/capture.rs`) and this epic's revised plan render into one instead of flagging the surface directly.
