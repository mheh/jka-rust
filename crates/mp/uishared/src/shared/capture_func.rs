//! `CaptureFunc` — the enum that replaces Raven's `captureFunc` pointer.

/// Which scroll handler currently owns the mouse.
///
/// Raven kept `static void (*captureFunc)(void *p)` plus `static void
/// *captureData`; every one of the five assignment sites paired a
/// `Scroll_*_Func` with `captureData = &scrollInfo`, and the single call site
/// (`Display_HandleKey`'s idle path) invoked it with that same pointer. The
/// closed function-pointer set becomes an enum dispatched by `match`
/// (translation dictionary), and `captureData` drops out — it was always
/// `&scrollInfo`, which the framework already owns.
///
/// Source: `oracle/codemp/ui/ui_shared.c:99-101,4039-4090,9835-9836`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaptureFunc {
    /// `captureFunc == 0` — nothing has the mouse captured.
    #[default]
    None,
    /// `Scroll_ListBox_AutoFunc`.
    ScrollListBoxAuto,
    /// `Scroll_ListBox_ThumbFunc`.
    ScrollListBoxThumb,
    /// `Scroll_TextScroll_AutoFunc`.
    ScrollTextScrollAuto,
    /// `Scroll_TextScroll_ThumbFunc`.
    ScrollTextScrollThumb,
    /// `Scroll_Slider_ThumbFunc`.
    ScrollSliderThumb,
}
