#![allow(non_camel_case_types, non_snake_case)]

/// Raven `renderCommand_t` — render command types.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:2048-2059`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum renderCommand_t {
    RC_END_OF_LIST = 0,
    RC_SET_COLOR = 1,
    RC_STRETCH_PIC = 2,
    RC_SCISSOR = 3,
    RC_ROTATE_PIC = 4,
    RC_ROTATE_PIC2 = 5,
    RC_DRAW_SURFS = 6,
    RC_DRAW_BUFFER = 7,
    RC_SWAP_BUFFERS = 8,
    RC_WORLD_EFFECTS = 9,
}
