//! Entity fn-pointer dispatch fn-ID enums, hoisted to `mp_qshared`
//! so `gentity_t`'s dispatch fields can name them.
//!
//! Per-field fn-ID enums replace Raven's stored fn pointers; `PartialEq`
//! replaces fn-address compares; stored as `Option<EntXxx>` where Raven stored
//! a nullable fn pointer.
//!
//! `gentity_t` lives in this crate (the abi seam names `*mut gentity_t`), and its
//! 7 stored dispatch fn-pointer fields (`think`, `reached`,
//! `blocked`, `touch`, `use_`, `pain`, `die`) are `Option<EntXxx>`. That field
//! type must therefore be visible in `mp_qshared`, below `mp_game`. The central
//! match dispatch (`dispatch_think`, …) stays in `mp_game`'s `ent_fn_enums`
//! (it names the handler fns), which re-exports these enums so existing imports
//! keep resolving.
//! Field signatures source: `oracle/codemp/game/g_local.h:284-290`
#![allow(non_camel_case_types)]

use core::marker::PhantomData;
use core::num::NonZeroU8;

/// Maps a `#[repr(u8)]` fieldless fn-ID enum to/from a 0-based index (its
/// discriminant). Implemented for every `EntXxx` enum below via
/// [`impl_ent_fn!`]; consumed by [`FnId`] to store the id as a `NonZeroU8`.
pub trait EntFn: Copy {
    /// The enum's discriminant, `0..N`.
    fn to_index(self) -> u8;
    /// Reverse of [`to_index`](EntFn::to_index).
    ///
    /// # Safety
    /// `i` must be a discriminant produced by `to_index` (i.e. `i < N`), so the
    /// `#[repr(u8)]` transmute lands on a real variant.
    unsafe fn from_index(i: u8) -> Self;
}

macro_rules! impl_ent_fn {
    ($ty:ty) => {
        impl EntFn for $ty {
            #[inline]
            fn to_index(self) -> u8 {
                self as u8
            }
            #[inline]
            unsafe fn from_index(i: u8) -> Self {
                // SAFETY: `$ty` is a `#[repr(u8)]` fieldless enum with contiguous
                // discriminants `0..N` (both are 1 byte); the caller guarantees
                // `i` is such a discriminant.
                unsafe { core::mem::transmute::<u8, Self>(i) }
            }
        }
    };
}

/// Niche-guaranteed handle to a `#[repr(u8)]` fn-ID enum `E`, stored where Raven
/// stored a nullable fn pointer (`gentity_t.think`, `.touch`, …).
///
/// The storage is `Option<NonZeroU8>` holding `discriminant + 1`, so `None`
/// (no handler) is the **all-zero** bit pattern — std guarantees this for
/// `Option<NonZero*>` and for `#[repr(transparent)]` structs around it (see
/// `core::option` "Representation"). Byte-wise zeroing a `gentity_t`
/// (`write_bytes`, `zeroed_box`, `memset`) therefore yields `None` handlers by
/// construction, matching Raven's C NULL-fn-pointer semantics with no post-zero
/// fixup. This makes the historic "zeroed `touch` == `Some(HolocronTouch)`"
/// niche hazard structurally impossible.
#[repr(transparent)]
pub struct FnId<E: EntFn> {
    raw: Option<NonZeroU8>,
    _marker: PhantomData<E>,
}

impl<E: EntFn> FnId<E> {
    /// The "no handler" value; its bit pattern is all-zero (see type docs).
    pub const NONE: Self = Self {
        raw: None,
        _marker: PhantomData,
    };

    /// Wraps a handler id.
    #[inline]
    pub fn some(e: E) -> Self {
        // `to_index()` is `0..N`, so `+ 1` is `1..=N` — always nonzero.
        let id = e.to_index() + 1;
        Self {
            raw: Some(unsafe { NonZeroU8::new_unchecked(id) }),
            _marker: PhantomData,
        }
    }

    /// Recovers the handler enum, or `None` for the no-handler value.
    #[inline]
    pub fn get(self) -> Option<E> {
        // SAFETY: `raw` was built by `some` from a real discriminant, so
        // `nz.get() - 1` is a valid index for `E::from_index`.
        self.raw.map(|nz| unsafe { E::from_index(nz.get() - 1) })
    }

    #[inline]
    pub const fn is_some(self) -> bool {
        self.raw.is_some()
    }

    #[inline]
    pub const fn is_none(self) -> bool {
        self.raw.is_none()
    }

    /// Panics if this is [`NONE`](FnId::NONE) — mirrors `Option::unwrap`.
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> E {
        self.get().unwrap()
    }
}

impl<E: EntFn> Clone for FnId<E> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<E: EntFn> Copy for FnId<E> {}

impl<E: EntFn> Default for FnId<E> {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

impl<E: EntFn> PartialEq for FnId<E> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
impl<E: EntFn> Eq for FnId<E> {}

impl<E: EntFn + core::fmt::Debug> core::fmt::Debug for FnId<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.get().fmt(f)
    }
}

impl<E: EntFn> From<E> for FnId<E> {
    #[inline]
    fn from(e: E) -> Self {
        Self::some(e)
    }
}

impl<E: EntFn> From<Option<E>> for FnId<E> {
    #[inline]
    fn from(o: Option<E>) -> Self {
        match o {
            Some(e) => Self::some(e),
            None => Self::NONE,
        }
    }
}

/// Raven `think` fn-pointer targets (84 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntThink {
    /// `oracle/codemp/game/g_trigger.c:1039`
    AimAtTarget,
    /// `oracle/codemp/game/g_combat.c:675`
    BodyRid,
    /// `oracle/codemp/game/g_client.c:973`
    BodySink,
    /// `oracle/codemp/game/g_items.c:254`
    CreateShield,
    /// `oracle/codemp/game/g_weapon.c:1310`
    DEMP2_AltDetonate,
    /// `oracle/codemp/game/g_weapon.c:1164`
    DEMP2_AltRadiusDamage,
    /// `oracle/codemp/game/w_saber.c:6235`
    DeadSaberThink,
    /// `oracle/codemp/game/g_weapon.c:2740`
    DetPackBlow,
    /// `oracle/codemp/game/NPC_behavior.c:185`
    Disappear,
    /// `oracle/codemp/game/w_saber.c:6342`
    DownedSaberThink,
    /// `oracle/codemp/game/g_items.c:1776`
    EWebThink,
    /// `oracle/codemp/game/g_items.c:2779`
    FinishSpawningItem,
    /// `oracle/codemp/game/g_missile.c:215`
    G_ExplodeMissile,
    /// `oracle/codemp/game/g_utils.c:932`
    G_FreeEntity,
    /// `oracle/codemp/game/g_misc.c:638`
    G_PortalifyEntities,
    /// `oracle/codemp/game/g_object.c:72`
    G_RunObject,
    /// `oracle/codemp/game/g_vehicles.c:186`
    G_VehicleSpawn,
    /// `oracle/codemp/game/g_misc.c:907`
    HolocronThink,
    /// `oracle/codemp/game/g_misc.c:1142`
    InitShooter_Finish,
    /// `oracle/codemp/game/g_client.c:346`
    JMSaberThink,
    /// `oracle/codemp/game/g_combat.c:3391`
    LimbThink,
    /// `oracle/codemp/game/g_ICARUScb.c:1865`
    MoveOwner,
    /// `oracle/codemp/game/NPC_spawn.c:862`
    NPC_Begin,
    /// `oracle/codemp/game/NPC.c:115`
    NPC_RemoveBody,
    /// `oracle/codemp/game/NPC_spawn.c:1814`
    NPC_ShySpawn,
    /// `oracle/codemp/game/NPC_spawn.c:1765`
    NPC_Spawn_Go,
    /// `oracle/codemp/game/NPC.c:1826`
    NPC_Think,
    /// `oracle/codemp/game/g_items.c:2288`
    RespawnItem,
    /// `oracle/codemp/game/g_mover.c:615`
    ReturnToPos1,
    /// `oracle/codemp/game/w_saber.c:286`
    SaberUpdateSelf,
    /// `oracle/codemp/game/g_items.c:171`
    ShieldGoSolid,
    /// `oracle/codemp/game/g_items.c:123`
    ShieldThink,
    /// `oracle/codemp/game/g_saga.c:1382`
    SiegeItemThink,
    /// `oracle/codemp/game/g_ICARUScb.c:4271`
    SolidifyOwner,
    /// `oracle/codemp/game/g_items.c:1277`
    SpecialItemThink,
    /// `oracle/codemp/game/g_team.c:714`
    Team_DroppedFlagThink,
    /// `oracle/codemp/game/g_mover.c:1727`
    Think_BeginMoving,
    /// `oracle/codemp/game/g_mover.c:1217`
    Think_MatchTeam,
    /// `oracle/codemp/game/g_mover.c:1802`
    Think_SetupTrainTargets,
    /// `oracle/codemp/game/g_mover.c:1168`
    Think_SpawnNewDoorTrigger,
    /// `oracle/codemp/game/g_trigger.c:789`
    Think_Strike,
    /// `oracle/codemp/game/g_target.c:78`
    Think_Target_Delay,
    /// `oracle/codemp/game/g_weapon.c:2471`
    TrapThink,
    /// `oracle/codemp/game/g_mover.c:710`
    Use_BinaryMover_Go,
    /// `oracle/codemp/game/g_weapon.c:3610`
    WP_VehWeapSetSolidToOwner,
    /// `oracle/codemp/game/g_weapon.c:1549`
    WP_flechette_alt_blow,
    /// `oracle/codemp/game/g_ICARUScb.c:569`
    anglerCallback,
    /// `oracle/codemp/game/g_trigger.c:1927`
    asteroid_field_think,
    /// `oracle/codemp/game/g_trigger.c:1922`
    asteroid_move_to_start,
    /// `oracle/codemp/game/g_misc.c:1176`
    check_recharge,
    /// `oracle/codemp/game/g_weapon.c:4828`
    emplaced_gun_update,
    /// `oracle/codemp/game/g_misc.c:2758`
    faller_think,
    /// `oracle/codemp/game/g_mover.c:2398`
    funcBBrushDieGo,
    /// `oracle/codemp/game/g_trigger.c:1757`
    func_timer_think,
    /// `oracle/codemp/game/g_mover.c:3027`
    func_usable_think,
    /// `oracle/codemp/game/g_mover.c:2995`
    func_wait_return_solid,
    /// `oracle/codemp/game/g_misc.c:2387`
    fx_runner_link,
    /// `oracle/codemp/game/g_misc.c:2266`
    fx_runner_think,
    /// `oracle/codemp/game/g_weapon.c:2244`
    laserTrapExplode,
    /// `oracle/codemp/game/g_weapon.c:2367`
    laserTrapThink,
    /// `oracle/codemp/game/g_misc.c:305`
    locateCamera,
    /// `oracle/codemp/game/g_misc.c:2659`
    maglock_link,
    /// `oracle/codemp/game/g_misc.c:2830`
    misc_faller_think,
    /// `oracle/codemp/game/g_misc.c:3417`
    misc_weapon_shooter_aim,
    /// `oracle/codemp/game/g_misc.c:3391`
    misc_weapon_shooter_fire,
    /// `oracle/codemp/game/g_trigger.c:32`
    multi_trigger_run,
    /// `oracle/codemp/game/g_items.c:708`
    pas_think,
    /// `oracle/codemp/game/g_weapon.c:2320`
    proxMineThink,
    /// `oracle/codemp/game/g_misc.c:3267`
    ref_link,
    /// `oracle/codemp/game/g_weapon.c:1651`
    rocketThink,
    /// `oracle/codemp/game/w_saber.c:6917`
    saberBackToOwner,
    /// `oracle/codemp/game/w_saber.c:7117`
    saberFirstThrown,
    /// `oracle/codemp/game/g_target.c:754`
    scriptrunner_run,
    /// `oracle/codemp/game/g_items.c:702`
    sentryExpire,
    /// `oracle/codemp/game/g_trigger.c:1533`
    shipboundary_think,
    /// `oracle/codemp/game/g_target.c:401`
    target_laser_start,
    /// `oracle/codemp/game/g_target.c:349`
    target_laser_think,
    /// `oracle/codemp/game/g_target.c:554`
    target_location_linkup,
    /// `oracle/codemp/game/g_weapon.c:1936`
    thermalDetonatorExplode,
    /// `oracle/codemp/game/g_weapon.c:1972`
    thermalThinkStandard,
    /// `oracle/codemp/game/g_trigger.c:872`
    trigger_always_think,
    /// `oracle/codemp/game/g_trigger.c:549`
    trigger_cleared_fire,
    /// `oracle/codemp/game/g_turret_G2.c:829`
    turretG2_base_think,
    /// `oracle/codemp/game/g_turret.c:505`
    turret_base_think,
}

/// Raven `reached` fn-pointer targets (4 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntReached {
    /// `oracle/codemp/game/g_mover.c:634`
    Reached_BinaryMover,
    /// `oracle/codemp/game/g_mover.c:1739`
    Reached_Train,
    /// `oracle/codemp/game/g_ICARUScb.c:667`
    moveAndRotateCallback,
    /// `oracle/codemp/game/g_ICARUScb.c:603`
    moverCallback,
}

/// Raven `blocked` fn-pointer targets (2 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntBlocked {
    /// `oracle/codemp/game/g_mover.c:1018`
    Blocked_Door,
    /// `oracle/codemp/game/g_ICARUScb.c:635`
    Blocked_Mover,
}

/// Raven `touch` fn-pointer targets (27 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntTouch {
    /// `oracle/codemp/game/g_misc.c:786`
    HolocronTouch,
    /// `oracle/codemp/game/g_client.c:385`
    JMSaberTouch,
    /// `oracle/codemp/game/g_combat.c:3387`
    LimbTouch,
    /// `oracle/codemp/game/w_saber.c:6229`
    SaberBounceSound,
    /// `oracle/codemp/game/w_saber.c:360`
    SaberGotHit,
    /// `oracle/codemp/game/g_items.c:515`
    SentryTouch,
    /// `oracle/codemp/game/g_items.c:230`
    ShieldTouch,
    /// `oracle/codemp/game/g_saga.c:1477`
    SiegeItemTouch,
    /// `oracle/codemp/game/g_mover.c:1633`
    Touch_Button,
    /// `oracle/codemp/game/g_mover.c:1078`
    Touch_DoorTrigger,
    /// `oracle/codemp/game/g_items.c:2362`
    Touch_Item,
    /// `oracle/codemp/game/g_trigger.c:350`
    Touch_Multi,
    /// `oracle/codemp/game/g_mover.c:1489`
    Touch_Plat,
    /// `oracle/codemp/game/g_mover.c:1507`
    Touch_PlatCenterTrigger,
    /// `oracle/codemp/game/g_weapon.c:3569`
    WP_TouchVehMissile,
    /// `oracle/codemp/game/g_weapon.c:2645`
    charge_stick,
    /// `oracle/codemp/game/g_misc.c:2730`
    faller_touch,
    /// `oracle/codemp/game/g_mover.c:2663`
    funcBBrushTouch,
    /// `oracle/codemp/game/g_trigger.c:1299`
    hurt_touch,
    /// `oracle/codemp/game/g_trigger.c:1597`
    hyperspace_touch,
    /// `NPC_TouchFunc`'s universal NPC touch handler assign.
    /// `oracle/codemp/game/NPC_spawn.c:199-206`
    NPC_Touch,
    /// `oracle/codemp/game/g_trigger.c:1494`
    shipboundary_touch,
    /// `oracle/codemp/game/g_trigger.c:1442`
    space_touch,
    /// `oracle/codemp/game/w_saber.c:7080`
    thrownSaberTouch,
    /// `oracle/codemp/game/g_weapon.c:2296`
    touchLaserTrap,
    /// `oracle/codemp/game/g_weapon.c:165`
    touch_NULL,
    /// `oracle/codemp/game/g_trigger.c:901`
    trigger_push_touch,
    /// `oracle/codemp/game/g_trigger.c:1197`
    trigger_teleporter_touch,
}

/// Raven `use` fn-pointer targets (53 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntUse {
    /// `oracle/codemp/game/g_mover.c:2930`
    GlassUse,
    /// `oracle/codemp/game/NPC_spawn.c:1839`
    NPC_Spawn,
    /// `oracle/codemp/game/NPC_reactions.c:1008`
    NPC_Use,
    /// `oracle/codemp/game/NPC_spawn.c:2175`
    NPC_VehicleSpawnUse,
    /// `oracle/codemp/game/g_saga.c:1182`
    SiegeIconUse,
    /// `oracle/codemp/game/g_saga.c:1576`
    SiegeItemUse,
    /// `oracle/codemp/game/g_client.c:140`
    SiegePointUse,
    /// `oracle/codemp/game/g_mover.c:863`
    Use_BinaryMover,
    /// `oracle/codemp/game/g_items.c:2765`
    Use_Item,
    /// `oracle/codemp/game/g_trigger.c:343`
    Use_Multi,
    /// `oracle/codemp/game/g_misc.c:1107`
    Use_Shooter,
    /// `oracle/codemp/game/g_trigger.c:801`
    Use_Strike,
    /// `oracle/codemp/game/g_target.c:82`
    Use_Target_Delay,
    /// `oracle/codemp/game/g_misc.c:2569`
    Use_Target_Escapetrig,
    /// `oracle/codemp/game/g_target.c:10`
    Use_Target_Give,
    /// `oracle/codemp/game/g_target.c:132`
    Use_Target_Print,
    /// `oracle/codemp/game/g_target.c:113`
    Use_Target_Score,
    /// `oracle/codemp/game/g_misc.c:2543`
    Use_Target_Screenshake,
    /// `oracle/codemp/game/g_target.c:259`
    Use_Target_Speaker,
    /// `oracle/codemp/game/g_trigger.c:1138`
    Use_target_push,
    /// `oracle/codemp/game/g_target.c:47`
    Use_target_remove_powerups,
    /// `oracle/codemp/game/g_misc.c:1331`
    ammo_generic_power_converter_use,
    /// `oracle/codemp/game/g_misc.c:1753`
    ammo_power_converter_use,
    /// `oracle/codemp/game/g_saga.c:1236`
    decompTriggerUse,
    /// `oracle/codemp/game/g_weapon.c:4804`
    emplaced_gun_realuse,
    /// `oracle/codemp/game/g_mover.c:2516`
    funcBBrushUse,
    /// `oracle/codemp/game/g_mover.c:2021`
    func_static_use,
    /// `oracle/codemp/game/g_trigger.c:1763`
    func_timer_use,
    /// `oracle/codemp/game/g_mover.c:3050`
    func_usable_use,
    /// `oracle/codemp/game/g_misc.c:2313`
    fx_runner_use,
    /// `oracle/codemp/game/g_misc.c:1921`
    health_power_converter_use,
    /// `oracle/codemp/game/g_trigger.c:1280`
    hurt_use,
    /// `oracle/codemp/game/g_misc.c:134`
    misc_dlight_use,
    /// `oracle/codemp/game/g_misc.c:2789`
    misc_faller_create,
    /// `oracle/codemp/game/g_misc.c:3401`
    misc_weapon_shooter_use,
    /// `oracle/codemp/game/NPC_AI_Sentry.c:64`
    sentry_use,
    /// `oracle/codemp/game/g_misc.c:1230`
    shield_power_converter_use,
    /// `oracle/codemp/game/g_saga.c:1317`
    siegeEndUse,
    /// `oracle/codemp/game/g_saga.c:1060`
    siegeTriggerUse,
    /// `oracle/codemp/game/g_target.c:912`
    target_activate_use,
    /// `oracle/codemp/game/g_target.c:611`
    target_counter_use,
    /// `oracle/codemp/game/g_target.c:919`
    target_deactivate_use,
    /// `oracle/codemp/game/g_target.c:534`
    target_kill_use,
    /// `oracle/codemp/game/g_target.c:392`
    target_laser_use,
    /// `oracle/codemp/game/g_target.c:945`
    target_level_change_use,
    /// `oracle/codemp/game/g_target.c:972`
    target_play_music_use,
    /// `oracle/codemp/game/g_target.c:681`
    target_random_use,
    /// `oracle/codemp/game/g_target.c:479`
    target_relay_use,
    /// `oracle/codemp/game/g_target.c:839`
    target_scriptrunner_use,
    /// `oracle/codemp/game/g_target.c:440`
    target_teleporter_use,
    /// `oracle/codemp/game/g_turret_G2.c:959`
    turretG2_base_use,
    /// `oracle/codemp/game/g_turret.c:604`
    turret_base_use,
    /// `oracle/codemp/game/g_mover.c:3215`
    use_wall,
}

/// Raven `pain` fn-pointer targets (14 distinct assigns in game/*.c bodies,
/// plus the 16 `NPC_PainFunc` dispatch targets assigned by NPC class/weapon
/// switch — `oracle/codemp/game/NPC_spawn.c:103-189`).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntPain {
    /// `oracle/codemp/game/g_weapon.c:2768`
    DetPackPain,
    /// `oracle/codemp/game/g_items.c:1484`
    EWebPain,
    /// `oracle/codemp/game/g_mover.c:2924`
    GlassPain,
    /// `oracle/codemp/game/NPC_AI_Atst.c:119`
    NPC_ATST_Pain,
    /// CLASS_GONK/R2D2/R5D2/MOUSE/PROTOCOL/INTERROGATOR.
    /// `oracle/codemp/game/NPC_AI_Droid.c` (assigned NPC_spawn.c:145-151)
    NPC_Droid_Pain,
    /// CLASS_GALAKMECH. `oracle/codemp/game/NPC_AI_GalakMech.c`
    /// (assigned NPC_spawn.c:167)
    NPC_GM_Pain,
    /// CLASS_HOWLER. `oracle/codemp/game/NPC_AI_Howler.c`
    /// (assigned NPC_spawn.c:142)
    NPC_Howler_Pain,
    /// `ps.weapon == WP_SABER` universal jedi pain. `oracle/codemp/game/NPC_AI_Jedi.c`
    /// (assigned NPC_spawn.c:108)
    NPC_Jedi_Pain,
    /// CLASS_MARK1. `oracle/codemp/game/NPC_AI_Mark1.c`
    /// (assigned NPC_spawn.c:161)
    NPC_Mark1_Pain,
    /// CLASS_MARK2. `oracle/codemp/game/NPC_AI_Mark2.c`
    /// (assigned NPC_spawn.c:164)
    NPC_Mark2_Pain,
    /// CLASS_MINEMONSTER. `oracle/codemp/game/NPC_AI_MineMonster.c`
    /// (assigned NPC_spawn.c:136)
    NPC_MineMonster_Pain,
    /// Default-case generic NPC pain. `oracle/codemp/game/NPC_AI_Default.c`
    /// (assigned NPC_spawn.c:177)
    NPC_Pain,
    /// CLASS_PROBE. `oracle/codemp/game/NPC_AI_ImperialProbe.c`
    /// (assigned NPC_spawn.c:154)
    NPC_Probe_Pain,
    /// `oracle/codemp/game/NPC_AI_Rancor.c:703`
    NPC_Rancor_Pain,
    /// CLASS_REMOTE. `oracle/codemp/game/NPC_AI_Remote.c`
    /// (assigned NPC_spawn.c:133)
    NPC_Remote_Pain,
    /// CLASS_SEEKER. `oracle/codemp/game/NPC_AI_Seeker.c`
    /// (assigned NPC_spawn.c:130)
    NPC_Seeker_Pain,
    /// CLASS_SENTRY. `oracle/codemp/game/NPC_AI_Sentry.c`
    /// (assigned NPC_spawn.c:158)
    NPC_Sentry_Pain,
    /// CLASS_STORMTROOPER/SWAMPTROOPER. `oracle/codemp/game/NPC_AI_Stormtrooper.c`
    /// (assigned NPC_spawn.c:126)
    NPC_ST_Pain,
    /// `oracle/codemp/game/NPC_AI_Wampa.c:433`
    NPC_Wampa_Pain,
    /// `oracle/codemp/game/g_items.c:155`
    ShieldPain,
    /// `oracle/codemp/game/g_saga.c:1547`
    SiegeItemPain,
    /// `oracle/codemp/game/g_turret.c:35`
    TurretBasePain,
    /// `oracle/codemp/game/g_turret_G2.c:210`
    TurretG2Pain,
    /// `oracle/codemp/game/g_turret.c:11`
    TurretPain,
    /// `oracle/codemp/game/g_weapon.c:4810`
    emplaced_gun_pain,
    /// `oracle/codemp/game/g_mover.c:2532`
    funcBBrushPain,
    /// `oracle/codemp/game/g_mover.c:3108`
    func_usable_pain,
}

/// Raven `die` fn-pointer targets (17 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntDie {
    /// `oracle/codemp/game/g_weapon.c:2775`
    DetPackDie,
    /// `oracle/codemp/game/g_items.c:1449`
    EWebDie,
    /// `oracle/codemp/game/g_mover.c:2875`
    GlassDie,
    /// `oracle/codemp/game/g_weapon.c:1814`
    RocketDie,
    /// `oracle/codemp/game/g_items.c:145`
    ShieldDie,
    /// `oracle/codemp/game/g_saga.c:1553`
    SiegeItemDie,
    /// `oracle/codemp/game/g_turret.c:51`
    auto_turret_die,
    /// `oracle/codemp/game/g_combat.c:686`
    body_die,
    /// `oracle/codemp/game/g_turret.c:112`
    bottom_die,
    /// `oracle/codemp/game/g_weapon.c:4930`
    emplaced_gun_die,
    /// `oracle/codemp/game/g_mover.c:2500`
    funcBBrushDie,
    /// `oracle/codemp/game/g_mover.c:3113`
    func_usable_die,
    /// `oracle/codemp/game/g_weapon.c:2282`
    laserTrapDelayedExplode,
    /// `oracle/codemp/game/g_misc.c:2623`
    maglock_die,
    /// `oracle/codemp/game/g_combat.c:2123`
    player_die,
    /// `oracle/codemp/game/g_turret_G2.c:236`
    turretG2_die,
    /// `oracle/codemp/game/g_items.c:940`
    turret_die,
}

impl_ent_fn!(EntThink);
impl_ent_fn!(EntReached);
impl_ent_fn!(EntBlocked);
impl_ent_fn!(EntTouch);
impl_ent_fn!(EntUse);
impl_ent_fn!(EntPain);
impl_ent_fn!(EntDie);
