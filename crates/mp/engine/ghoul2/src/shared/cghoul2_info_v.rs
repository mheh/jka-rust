#![allow(non_camel_case_types, non_snake_case)]

/// Raven `CGhoul2Info_v` — handle to a vector of `CGhoul2Info` model instances,
/// indexed into the global `IGhoul2InfoArray`.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/codemp/ghoul2/ghoul2_shared.h:328-457`
#[repr(C)]
pub struct CGhoul2Info_v {
    /// don't be bad and muck with this
    pub mItem: i32,
}

const _: () = assert!(core::mem::size_of::<CGhoul2Info_v>() == 4);
const _: () = assert!(core::mem::offset_of!(CGhoul2Info_v, mItem) == 0);
