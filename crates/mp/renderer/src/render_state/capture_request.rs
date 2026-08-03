//! `CaptureRequest` — one pending screenshot, carried to the render thread.

/// One `screenshot_tga` command, resolved on the sim side and answered on the
/// render thread.
///
/// The sim side owns the filesystem, so it runs Raven's free-number scan and
/// resolves the OS path before the request travels. The render thread owns the
/// presented texture, so it does the readback, the TGA write, and the
/// `Wrote %s` print that follows the write.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:705-759`
pub struct CaptureRequest {
    /// The resolved OS path `FS_BuildOSPath` produced, not the qpath. The
    /// render thread has no `Common` to resolve one.
    pub os_path: String,
    /// Raven's `screenshot_tga silent` argument. It drops the `Wrote %s`
    /// print, and nothing else.
    pub silent: bool,
}
