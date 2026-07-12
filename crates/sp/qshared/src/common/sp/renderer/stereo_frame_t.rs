//! SP `stereoFrame_t` from `code/renderer/tr_types.h`.

#![allow(non_camel_case_types)]

/// Raven `stereoFrame_t` — which eye a frame is rendered for.
///
/// Type definition source: `oracle/code/renderer/tr_types.h:183-187`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum stereoFrame_t {
    STEREO_CENTER = 0,
    STEREO_LEFT,
    STEREO_RIGHT,
}

const _: () = assert!(core::mem::size_of::<stereoFrame_t>() == 4);
