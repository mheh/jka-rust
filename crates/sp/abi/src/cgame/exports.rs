//! SP cgame exports enum vocabulary.
//!
//! Transcribed from Raven `oracle/code/client/vmachine.h`.
//! These discriminants are ABI wire values; do not renumber them.

#![allow(non_camel_case_types)]

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpCgameExport {
    /// Source: `oracle/code/client/vmachine.h:14`
    CG_INIT,

    /// Source: `oracle/code/client/vmachine.h:15`
    CG_SHUTDOWN,

    /// Source: `oracle/code/client/vmachine.h:16`
    CG_CONSOLE_COMMAND,

    /// Source: `oracle/code/client/vmachine.h:17`
    CG_DRAW_ACTIVE_FRAME,

    /// Source: `oracle/code/client/vmachine.h:18`
    CG_CROSSHAIR_PLAYER,

    /// Source: `oracle/code/client/vmachine.h:19`
    CG_CAMERA_POS,

    /// Source: `oracle/code/client/vmachine.h:20`
    CG_CAMERA_ANG,

    /// Ghoul2 Insert Start
    /// Source: `oracle/code/client/vmachine.h:25`
    CG_RESIZE_G2_BOLT,

    /// Source: `oracle/code/client/vmachine.h:26`
    CG_RESIZE_G2,

    /// Source: `oracle/code/client/vmachine.h:27`
    CG_RESIZE_G2_BONE,

    /// Source: `oracle/code/client/vmachine.h:28`
    CG_RESIZE_G2_SURFACE,

    /// Source: `oracle/code/client/vmachine.h:29`
    CG_RESIZE_G2_TEMPBONE,

    /// Ghoul2 Insert End
    /// Source: `oracle/code/client/vmachine.h:33`
    CG_DRAW_DATAPAD_HUD,

    /// Source: `oracle/code/client/vmachine.h:34`
    CG_DRAW_DATAPAD_OBJECTIVES,

    /// Source: `oracle/code/client/vmachine.h:35`
    CG_DRAW_DATAPAD_WEAPONS,

    /// Source: `oracle/code/client/vmachine.h:36`
    CG_DRAW_DATAPAD_INVENTORY,

    /// Source: `oracle/code/client/vmachine.h:37`
    CG_DRAW_DATAPAD_FORCEPOWERS,
}
