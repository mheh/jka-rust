#![allow(non_camel_case_types, non_snake_case)]

use core::num::NonZeroU32;

/// Raven `Vehicle_t *m_pVehicle` on `centity_t`, as the DEC-46.2 owned-id
/// replacement: the entity number of the vehicle cent the `Vehicle_t` belongs
/// to. `NonZeroU32` because vehicles are NPC ents (entity number >=
/// `MAX_CLIENTS`, never 0), which gives `Option<VehicleId>` the null niche —
/// zero-filled spawn state reads as `None`, Raven's null pointer.
///
/// The referent pool (cgame's owned `Vehicle_t` instances, created by the
/// `G_Create*NPC` family at `oracle/codemp/cgame/cg_players.c:7014-7042`) lands
/// with the wave that transcribes that block; until then ported code only tests
/// presence.
///
/// Source: `oracle/codemp/cgame/cg_local.h:338`, `docs/decisions.md` DEC-46
/// (ruling 2)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct VehicleId(NonZeroU32);

impl VehicleId {
    /// `None` when `ent_num` is 0 — which no vehicle cent ever is.
    pub fn new(ent_num: u32) -> Option<Self> {
        NonZeroU32::new(ent_num).map(VehicleId)
    }

    pub fn ent_num(self) -> u32 {
        self.0.get()
    }
}
