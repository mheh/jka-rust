#![allow(non_camel_case_types, non_snake_case)]

/// Raven `cgameExport_t` — cgame module export function indices.
///
/// Type definition source: `oracle/code/client/vmachine.h:13-39`
#[repr(i32)]
pub enum cgameExport_t {
	CG_INIT,
	CG_SHUTDOWN,
	CG_CONSOLE_COMMAND,
	CG_DRAW_ACTIVE_FRAME,
	CG_CROSSHAIR_PLAYER,
	CG_CAMERA_POS,
	CG_CAMERA_ANG,
	// Ghoul2 Insert Start
	CG_RESIZE_G2_BOLT,
	CG_RESIZE_G2,
	CG_RESIZE_G2_BONE,
	CG_RESIZE_G2_SURFACE,
	CG_RESIZE_G2_TEMPBONE,
	// Ghoul2 Insert End
	CG_DRAW_DATAPAD_HUD,
	CG_DRAW_DATAPAD_OBJECTIVES,
	CG_DRAW_DATAPAD_WEAPONS,
	CG_DRAW_DATAPAD_INVENTORY,
	CG_DRAW_DATAPAD_FORCEPOWERS,
}
