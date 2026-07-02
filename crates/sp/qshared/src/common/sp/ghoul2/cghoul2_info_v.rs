#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `CGhoul2Info_v` — a handle into the global Ghoul2 model instance
/// array (a `vector<CGhoul2Info>` accessed indirectly through `mItem`).
///
/// Raven: the instance vector itself lives in a shared `IGhoul2InfoArray`
/// (`TheGhoul2InfoArray()` / `TheGameGhoul2InfoArray()`); this handle only
/// stores the array slot index and defers all access/lifetime behavior to
/// that array's `Alloc`/`Free`/`Get` methods.
///
/// Type definition source: `oracle/oracle/code/game/ghoul2_shared.h:326-452`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CGhoul2Info_v {
    pub mItem: c_int,
}

const _: () = assert!(core::mem::size_of::<CGhoul2Info_v>() == 4);
const _: () = assert!(core::mem::offset_of!(CGhoul2Info_v, mItem) == 0);
