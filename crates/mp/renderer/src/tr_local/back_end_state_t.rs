#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

use super::back_end_counters_t::backEndCounters_t;
use super::orientationr_t::orientationr_t;
use super::tr_ref_entity_t::trRefEntity_t;
use super::tr_refdef_t::trRefdef_t;
use super::view_parms_t::viewParms_t;

/// Raven `backEndState_t` — persistent state carried between backend renders.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:1279-1292`
#[repr(C)]
pub struct backEndState_t {
    pub refdef: trRefdef_t,
    pub viewParms: viewParms_t,
    // Can't use or as it is a c++ reserved word DREWS 2/2/2002
    pub ori: orientationr_t,
    pub pc: backEndCounters_t,
    pub isHyperspace: qboolean,
    pub currentEntity: *mut trRefEntity_t,
    /// flag for drawing sun
    pub skyRenderedThisView: qboolean,

    /// if qtrue, drawstretchpic doesn't need to change modes
    pub projection2D: qboolean,
    pub color2D: [u8; 4],
    /// shader needs to be finished
    pub vertexes2D: qboolean,
    /// currentEntity will point at this when doing 2D rendering
    pub entity2D: trRefEntity_t,
}

const _: () = assert!(core::mem::offset_of!(backEndState_t, refdef) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<backEndState_t>() == 1416);
    assert!(core::mem::offset_of!(backEndState_t, viewParms) == 448);
    assert!(core::mem::offset_of!(backEndState_t, ori) == 940);
    assert!(core::mem::offset_of!(backEndState_t, pc) == 1064);
    assert!(core::mem::offset_of!(backEndState_t, isHyperspace) == 1112);
    assert!(core::mem::offset_of!(backEndState_t, currentEntity) == 1120);
    assert!(core::mem::offset_of!(backEndState_t, skyRenderedThisView) == 1128);
    assert!(core::mem::offset_of!(backEndState_t, projection2D) == 1132);
    assert!(core::mem::offset_of!(backEndState_t, color2D) == 1136);
    assert!(core::mem::offset_of!(backEndState_t, vertexes2D) == 1140);
    assert!(core::mem::offset_of!(backEndState_t, entity2D) == 1144);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<backEndState_t>() == 1372);
    assert!(core::mem::offset_of!(backEndState_t, viewParms) == 416);
    assert!(core::mem::offset_of!(backEndState_t, ori) == 908);
    assert!(core::mem::offset_of!(backEndState_t, pc) == 1032);
    assert!(core::mem::offset_of!(backEndState_t, isHyperspace) == 1080);
    assert!(core::mem::offset_of!(backEndState_t, currentEntity) == 1084);
    assert!(core::mem::offset_of!(backEndState_t, skyRenderedThisView) == 1088);
    assert!(core::mem::offset_of!(backEndState_t, projection2D) == 1092);
    assert!(core::mem::offset_of!(backEndState_t, color2D) == 1096);
    assert!(core::mem::offset_of!(backEndState_t, vertexes2D) == 1100);
    assert!(core::mem::offset_of!(backEndState_t, entity2D) == 1104);
};
