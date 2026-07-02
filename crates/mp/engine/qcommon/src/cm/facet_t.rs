#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qboolean;

/// Raven `facet_t` — a single collision facet of a patch collision surface.
///
/// Raven: `numBorders` comment: 3 or four + 6 axial bevels + 4 or 3 * 4 edge bevels.
/// Type definition source: `oracle/oracle/codemp/qcommon/cm_patch.h:83-89`
#[repr(C)]
pub struct facet_t {
    pub surfacePlane: i32,
    pub numBorders: i32,
    pub borderPlanes: [i32; 4 + 6 + 16],
    pub borderInward: [i32; 4 + 6 + 16],
    pub borderNoAdjust: [qboolean; 4 + 6 + 16],
}

const _: () = assert!(core::mem::size_of::<facet_t>() == 320);
const _: () = assert!(core::mem::offset_of!(facet_t, surfacePlane) == 0);
const _: () = assert!(core::mem::offset_of!(facet_t, numBorders) == 4);
const _: () = assert!(core::mem::offset_of!(facet_t, borderPlanes) == 8);
const _: () = assert!(core::mem::offset_of!(facet_t, borderInward) == 112);
const _: () = assert!(core::mem::offset_of!(facet_t, borderNoAdjust) == 216);
