#![allow(non_camel_case_types, non_snake_case)]

/// Raven `renderCommand_t` — rendering command type.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:2239-2250`
#[repr(i32)]
pub enum renderCommand_t {
	RC_END_OF_LIST = 0,
	RC_SET_COLOR = 1,
	RC_STRETCH_PIC = 2,
	RC_ROTATE_PIC = 3,
	RC_ROTATE_PIC2 = 4,
	RC_DRAW_SURFS = 5,
	RC_DRAW_BUFFER = 6,
	RC_SWAP_BUFFERS = 7,
	RC_WORLD_EFFECTS = 8,
	RC_AUTO_MAP = 9,
}
