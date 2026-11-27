#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod entry;

#[cfg(all(feature = "mimalloc", not(feature = "alloc-stats")))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(all(feature = "alloc-stats", feature = "mimalloc"))]
#[global_allocator]
static ALLOCATOR: test_alloc::TrackingAllocator<mimalloc::MiMalloc> =
    test_alloc::TrackingAllocator(mimalloc::MiMalloc);

#[cfg(all(feature = "alloc-stats", not(feature = "mimalloc")))]
#[global_allocator]
static ALLOCATOR: test_alloc::TrackingAllocator<std::alloc::System> =
    test_alloc::TrackingAllocator(std::alloc::System);

/// One stable Windows identity for the launcher, the game window and the shortcuts, so a pinned
/// taskbar icon keeps working across updates and groups the game under the same button.
#[cfg(windows)]
fn set_app_identity() {
    let id: Vec<u16> = "Ahwfr.Cinnamatica"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    // SAFETY: `id` is a NUL-terminated UTF-16 string that outlives the call.
    unsafe {
        windows_sys::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID(id.as_ptr());
    }
}

/// Runs the client through its shared startup and exit dispatch.
fn main() {
    #[cfg(windows)]
    set_app_identity();
    cinnabar::lifecycle::record_launch();
    entry::dispatch();
}
