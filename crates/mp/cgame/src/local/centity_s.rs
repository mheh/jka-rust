#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::shared::{qboolean, vec3_t};

use super::cg_loop_sound_s::cgLoopSound_t;
use super::client_info_t::clientInfo_t;
use super::player_entity_t::playerEntity_t;
use super::player_state_ref::PlayerStateRef;
use super::vehicle_id::VehicleId;

/// Raven `MAX_CG_LOOPSOUNDS`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:321`
pub const MAX_CG_LOOPSOUNDS: usize = 8;

impl centity_t {
    /// All-zero value for take/put-back swaps at borrow-conflicted call sites
    /// (the `refEntity_t::zeroed` pattern). Every POD field is zero-valid, and
    /// the three resolution fields are too: `PlayerStateRef` (0 = None),
    /// `Option<VehicleId>` and `npcClient` (null-niche `Option`s).
    #[must_use]
    pub fn zeroed() -> Self {
        // SAFETY: all-zero is a valid bit pattern for every field, per above.
        unsafe { core::mem::zeroed() }
    }
}

/// Raven `centity_t` — client-side representation of an entity, tracking
/// interpolation, animation, and effects state between snapshots.
///
/// Raven: This comment below is correct, but now m_pVehicle is the first
/// thing in bg shared entity, so it goes first. - AReis
/// rww - entstate must be first, to correspond with the bg shared entity
/// structure.
/// Type definition source: `oracle/codemp/cgame/cg_local.h:333-462`
#[repr(C)]
pub struct centity_t {
    /// Raven: from cg.frame
    pub currentState: entityState_t,
    /// Raven: ptr to playerstate if applicable (for bg ents).
    /// DEC-46.2: resolution enum, resolved via `CgWorld` at use sites.
    pub playerState: PlayerStateRef,
    /// Raven: vehicle data.
    /// DEC-46.2: the vehicle cent's entity number, not a pointer.
    pub m_pVehicle: Option<VehicleId>,
    /// Raven: g2 instance
    pub ghoul2: *mut c_void,
    /// Raven: index locally (game/cgame) to anim data for this skel
    pub localAnimIndex: i32,
    /// Raven: needed for g2 collision
    pub modelScale: vec3_t,

    // Raven: from here up must be unified with bgEntity_t -rww
    /// Raven: from cg.nextFrame, if available
    pub nextState: entityState_t,
    /// Raven: true if next is valid to interpolate to
    pub interpolate: qboolean,
    /// Raven: true if cg.frame holds this entity
    pub currentValid: qboolean,

    /// Raven: move to playerEntity?
    pub muzzleFlashTime: i32,
    pub previousEvent: i32,

    /// Raven: so missile trails can handle dropped initial packets
    pub trailTime: i32,
    pub dustTrailTime: i32,
    pub miscTime: i32,

    pub damageAngles: vec3_t,
    pub damageTime: i32,

    /// Raven: last time this entity was found in a snapshot
    pub snapShotTime: i32,

    pub pe: playerEntity_t,

    pub rawAngles: vec3_t,

    pub beamEnd: vec3_t,

    // Raven: exact interpolated position of entity on this frame
    pub lerpOrigin: vec3_t,
    pub lerpAngles: vec3_t,

    pub ragLastOrigin: vec3_t,
    pub ragLastOriginTime: i32,

    /// Raven: if true only do anims and things on model_root instead of
    /// lower_lumbar, this will be the case for some NPCs.
    pub noLumbar: qboolean,
    pub noFace: qboolean,

    // Raven: For keeping track of the current surface status in relation to
    // the entitystate surface fields.
    pub npcLocalSurfOn: i32,
    pub npcLocalSurfOff: i32,

    pub eventAnimIndex: i32,

    /// Raven: dynamically allocated - always free it, and never stomp over it.
    /// DEC-46.2: owned, so the always-free/never-stomp discipline is the
    /// borrow checker's problem now.
    pub npcClient: Option<Box<clientInfo_t>>,

    pub weapon: i32,

    /// Raven: rww - pointer to ghoul2 instance of the current 3rd person weapon
    pub ghoul2weapon: *mut c_void,

    pub radius: f32,
    pub boltInfo: i32,

    // Raven: sometimes used as a bolt index, but these values are also used
    // as generic values for clientside entities at times
    pub bolt1: i32,
    pub bolt2: i32,
    pub bolt3: i32,
    pub bolt4: i32,

    pub bodyHeight: f32,

    pub torsoBolt: i32,

    pub turAngles: vec3_t,

    pub frame_minus1: vec3_t,
    pub frame_minus2: vec3_t,

    pub frame_minus1_refreshed: i32,
    pub frame_minus2_refreshed: i32,

    /// Raven: pointer to a ghoul2 instance
    pub frame_hold: *mut c_void,

    pub frame_hold_time: i32,
    pub frame_hold_refreshed: i32,

    /// Raven: pointer to a ghoul2 instance
    pub grip_arm: *mut c_void,

    pub trickAlpha: i32,
    pub trickAlphaTime: i32,

    pub teamPowerEffectTime: i32,
    /// Raven: 0 regen, 1 heal, 2 drain, 3 absorb
    pub teamPowerType: qboolean,

    pub isRagging: qboolean,
    pub ownerRagging: qboolean,
    pub overridingBones: i32,

    pub bodyFadeTime: i32,
    pub pushEffectOrigin: vec3_t,

    pub loopingSound: [cgLoopSound_t; MAX_CG_LOOPSOUNDS],
    pub numLoopingSounds: i32,

    pub serverSaberHitIndex: i32,
    pub serverSaberHitTime: i32,
    /// Raven: true if flesh, false if anything else.
    pub serverSaberFleshImpact: qboolean,

    pub ikStatus: qboolean,

    pub saberWasInFlight: qboolean,

    pub smoothYaw: f32,

    pub uncloaking: i32,
    pub cloaked: qboolean,

    pub vChatTime: i32,
}

// Layout asserts retired with the DEC-46.2 reshape (playerState/m_pVehicle/
// npcClient are owned/resolution types now, so C layout parity is gone by
// design). `centity_t` is `cg_local.h` module-private and never crosses the
// seam — bg reaches entity data through the accessor seam, never the
// `bgEntity_t` pointer pun. Same DEC-31 treatment `weaponInfo_t`,
// `localEntity_t` and `markPoly_t` carry.
