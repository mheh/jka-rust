#![allow(non_camel_case_types, non_snake_case)]

/// Raven `fuzzyseperator_t` — fuzzy logic weight tree separator node.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_weight.h:19-29`
#[repr(C)]
pub struct fuzzyseperator_t {
    pub index: i32,
    pub value: i32,
    pub r#type: i32,
    pub weight: f32,
    pub minweight: f32,
    pub maxweight: f32,
    pub child: *mut fuzzyseperator_t,
    pub next: *mut fuzzyseperator_t,
}

pub type fuzzyseperator_s = fuzzyseperator_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<fuzzyseperator_t>() == 40);
    assert!(core::mem::offset_of!(fuzzyseperator_t, index) == 0);
    assert!(core::mem::offset_of!(fuzzyseperator_t, value) == 4);
    assert!(core::mem::offset_of!(fuzzyseperator_t, r#type) == 8);
    assert!(core::mem::offset_of!(fuzzyseperator_t, weight) == 12);
    assert!(core::mem::offset_of!(fuzzyseperator_t, minweight) == 16);
    assert!(core::mem::offset_of!(fuzzyseperator_t, maxweight) == 20);
    assert!(core::mem::offset_of!(fuzzyseperator_t, child) == 24);
    assert!(core::mem::offset_of!(fuzzyseperator_t, next) == 32);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<fuzzyseperator_t>() == 32);
    assert!(core::mem::offset_of!(fuzzyseperator_t, index) == 0);
    assert!(core::mem::offset_of!(fuzzyseperator_t, value) == 4);
    assert!(core::mem::offset_of!(fuzzyseperator_t, r#type) == 8);
    assert!(core::mem::offset_of!(fuzzyseperator_t, weight) == 12);
    assert!(core::mem::offset_of!(fuzzyseperator_t, minweight) == 16);
    assert!(core::mem::offset_of!(fuzzyseperator_t, maxweight) == 20);
    assert!(core::mem::offset_of!(fuzzyseperator_t, child) == 24);
    assert!(core::mem::offset_of!(fuzzyseperator_t, next) == 28);
};
