#![allow(non_camel_case_types, non_snake_case)]

// Non-XBOX build; `#define SIEGE_CLASS_DESC_LEN 4096`.
// Source: `oracle/codemp/game/bg_saga.h:45-48`
pub const SIEGE_CLASS_DESC_LEN: usize = 4096;

/// Raven `siegeClassDesc_t` — siege class description text buffer.
///
/// Type definition source: `oracle/codemp/game/bg_saga.h:49-52`
#[repr(C)]
pub struct siegeClassDesc_t {
    pub desc: [core::ffi::c_char; SIEGE_CLASS_DESC_LEN],
}

const _: () = assert!(core::mem::size_of::<siegeClassDesc_t>() == 4096);
const _: () = assert!(core::mem::offset_of!(siegeClassDesc_t, desc) == 0);
