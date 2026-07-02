#![allow(non_camel_case_types, non_snake_case)]

/// Raven `renderCommandList_t` — a fixed-size byte buffer of encoded render
/// commands, plus a cursor tracking how many bytes are used.
///
/// Raven: none.
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:2180-2183`
#[repr(C)]
pub struct renderCommandList_t {
    pub cmds: [u8; 0x40000],
    pub used: i32,
}

const _: () = assert!(core::mem::size_of::<renderCommandList_t>() == 262148);
const _: () = assert!(core::mem::offset_of!(renderCommandList_t, cmds) == 0);
const _: () = assert!(core::mem::offset_of!(renderCommandList_t, used) == 262144);
