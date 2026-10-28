/// Makes the next presents of `surface` land in the current Core Animation
/// transaction instead of on a later display refresh.
///
/// During a window resize AppKit commits the new layer bounds in the current
/// transaction, so a frame presented asynchronously reaches the screen after
/// the bounds change and the compositor scales it to fit. Enable this around
/// a frame rendered synchronously from `WindowEvent::Resized`, then disable it
/// again: presenting in a transaction blocks until the GPU schedules the frame.
///
/// No-op on platforms other than macOS and on non-Metal surfaces.
pub fn set_presents_with_transaction(surface: &wgpu::Surface<'_>, enabled: bool) {
    #[cfg(target_os = "macos")]
    // SAFETY: only a CAMetalLayer property is toggled; the HAL surface is not retained.
    if let Some(hal) = unsafe { surface.as_hal::<wgpu::hal::api::Metal>() } {
        hal.render_layer()
            .lock()
            .setPresentsWithTransaction(enabled);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (surface, enabled);
}
