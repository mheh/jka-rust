#![allow(non_camel_case_types, non_snake_case)]

/// Raven `hstring` — interned/hashed string handle (stores only an id, not the
/// characters themselves).
///
/// Raven: no header comment.
/// Type definition source: `oracle/oracle/code/qcommon/hstring.h:13-79`
#[repr(C)]
pub struct hstring {
    mId: i32,
}

const _: () = assert!(core::mem::size_of::<hstring>() == 4);
const _: () = assert!(core::mem::offset_of!(hstring, mId) == 0);
