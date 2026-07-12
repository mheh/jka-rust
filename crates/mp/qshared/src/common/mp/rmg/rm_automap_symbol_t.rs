//! `rmAutomapSymbol_t` — an automap symbol marker (relocated to `mp_qshared`).
//!
//! Per RMG-D4d / RMG-D2(b) (ruling 21) the rosetta's original port in
//! `mp_engine_client` relocates here into a new `rmg/` folder mirroring
//! `oracle/codemp/RMG/RM_Manager.h` ownership, so `mp_engine_rmg` (which already
//! depends on `mp_qshared`, not on `mp_engine_client`) names it directly — no
//! `rmg → mp_engine_client` edge. `CRMManager::GetAutomapSymbol` returns
//! `Option<&RmAutomapSymbol>`.

use crate::shared::vec3_t;

/// Raven `rmAutomapSymbol_t` — an automap symbol marker.
///
/// Type definition source: `oracle/codemp/client/client.h:143-149`
#[repr(C)]
pub struct RmAutomapSymbol {
    pub mType: i32,
    pub mSide: i32,
    pub mOrigin: vec3_t,
}

const _: () = assert!(core::mem::size_of::<RmAutomapSymbol>() == 20);
const _: () = assert!(core::mem::offset_of!(RmAutomapSymbol, mType) == 0);
const _: () = assert!(core::mem::offset_of!(RmAutomapSymbol, mSide) == 4);
const _: () = assert!(core::mem::offset_of!(RmAutomapSymbol, mOrigin) == 8);
