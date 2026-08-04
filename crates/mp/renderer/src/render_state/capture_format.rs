//! `CaptureFormat` — which file a pending screenshot becomes.

/// The encoding a [`CaptureRequest`](super::capture_request::CaptureRequest) asks the render thread for.
/// Raven has one command per format, and each writes its own extension, so the request carries the choice across.
///
/// - `Tga`: uncompressed 24-bit TGA, the `screenshot_tga` command.
/// - `Jpeg`: baseline JPEG at the given quality, the `screenshot` command.
///
/// Raven passes 95 for the quality on its one call.
///
/// Source: `oracle/codemp/renderer/tr_init.cpp:537-596`
pub enum CaptureFormat {
    Tga,
    Jpeg { quality: u8 },
}
