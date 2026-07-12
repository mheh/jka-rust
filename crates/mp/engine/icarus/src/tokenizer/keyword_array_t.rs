#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

/// Raven `keywordArray_t` — a keyword string paired with its token value.
///
/// Type definition source: `oracle/codemp/game/../icarus/tokenizer.h:77-81`
#[repr(C)]
pub struct keywordArray_t {
    pub m_keyword: *mut c_char,
    pub m_tokenvalue: i32,
}

const _: () = assert!(core::mem::offset_of!(keywordArray_t, m_keyword) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<keywordArray_t>() == 16);
    assert!(core::mem::offset_of!(keywordArray_t, m_tokenvalue) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<keywordArray_t>() == 8);
    assert!(core::mem::offset_of!(keywordArray_t, m_tokenvalue) == 4);
};
