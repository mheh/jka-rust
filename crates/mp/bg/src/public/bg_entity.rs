//! MP `bgEntity_t` copied from Raven `codemp/game/bg_public.h`.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:423-433`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_qshared::common::mp::qcommon::{entityState_t, playerState_t};
use mp_qshared::shared::{vec3_t, entityShared_t};

use crate::vehicles::vehicle_s::Vehicle_t;

/// Raven `bgEntity_t` — the shared bg-side view of an entity (matches the head
/// of `gentity_t`/`centity_t`).
///
/// Raven: Data type(s) must directly correspond to the head of the gentity and
/// centity structures.
/// Type definition source: `oracle/codemp/game/bg_public.h:423-433`
#[repr(C)]
#[derive(Debug)]
pub struct bgEntity_t {
    /// Raven field source: `oracle/codemp/game/bg_public.h:425`
    pub s: entityState_t,
    /// Raven field source: `oracle/codemp/game/bg_public.h:426`
    pub playerState: *mut playerState_t,
    /// Raven `Vehicle_t *m_pVehicle`.
    ///
    /// Raven: vehicle data.
    /// Raven field source: `oracle/codemp/game/bg_public.h:427`
    pub m_pVehicle: *mut Vehicle_t,
    /// G2 instance.
    /// Raven field source: `oracle/codemp/game/bg_public.h:428`
    pub ghoul2: *mut c_void,
    /// Index locally (game/cgame) to anim data for this skel.
    /// Raven field source: `oracle/codemp/game/bg_public.h:429`
    pub localAnimIndex: c_int,
    /// Needed for g2 collision.
    /// Raven field source: `oracle/codemp/game/bg_public.h:430`
    pub modelScale: vec3_t,
    /// Shared entity state: linked, bmodel, mins/maxs, absmin/absmax, ownerNum, etc.
    ///
    /// Raven field source: `oracle/codemp/game/g_local.h:144`
    pub r: entityShared_t,
    /// Set in QuakeEd.
    ///
    /// Raven field source: `oracle/codemp/game/g_local.h:156`
    pub classname: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<bgEntity_t>() == 696);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, s) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, playerState) == 536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, m_pVehicle) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, ghoul2) == 552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, localAnimIndex) == 560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, modelScale) == 564);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, r) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(bgEntity_t, classname) == 688);
