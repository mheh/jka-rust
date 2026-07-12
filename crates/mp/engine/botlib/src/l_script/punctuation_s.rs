#![allow(non_camel_case_types, non_snake_case)]

/// Raven `punctuation_t` — a punctuation character sequence node in a script's
/// punctuation table linked list.
///
/// Raven: punctuation character(s) / punctuation indication / next punctuation.
/// Type definition source: `oracle/codemp/botlib/l_script.h:133-138`
#[repr(C)]
pub struct punctuation_t {
    /// punctuation character(s)
    pub p: *mut core::ffi::c_char,
    /// punctuation indication
    pub n: core::ffi::c_int,
    /// next punctuation
    pub next: *mut punctuation_t,
}

pub type punctuation_s = punctuation_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<punctuation_t>() == 24);
    assert!(core::mem::offset_of!(punctuation_t, p) == 0);
    assert!(core::mem::offset_of!(punctuation_t, n) == 8);
    assert!(core::mem::offset_of!(punctuation_t, next) == 16);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<punctuation_t>() == 12);
    assert!(core::mem::offset_of!(punctuation_t, p) == 0);
    assert!(core::mem::offset_of!(punctuation_t, n) == 4);
    assert!(core::mem::offset_of!(punctuation_t, next) == 8);
};
