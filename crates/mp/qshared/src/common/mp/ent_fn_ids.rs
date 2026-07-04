//! Entity fn-pointer dispatch fn-ID enums (ruling 2), hoisted to `mp_qshared`
//! so `gentity_t`'s dispatch fields can name them.
//!
//! Ruling 2 (`docs/handoffs/jampgame-fork-discovery.md`): per-field fn-ID enums
//! replace Raven's stored fn pointers; `PartialEq` replaces fn-address compares;
//! stored as `Option<EntXxx>` where Raven stored a nullable fn pointer.
//!
//! `gentity_t` lives in this crate (the abi seam names `*mut gentity_t`), and the
//! fork-2 flip turns its 7 stored dispatch fn-pointer fields (`think`, `reached`,
//! `blocked`, `touch`, `use_`, `pain`, `die`) into `Option<EntXxx>`. That field
//! type must therefore be visible in `mp_qshared`, below `mp_game`. The central
//! match dispatch (`dispatch_think`, …) stays in `mp_game`'s `ent_fn_enums`
//! (it names the handler fns), which re-exports these enums so existing imports
//! keep resolving.
//! Field signatures source: `oracle/oracle/codemp/game/g_local.h:284-290`
#![allow(non_camel_case_types)]

/// Raven `think` fn-pointer targets (84 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntThink {
    /// `oracle/oracle/codemp/game/g_trigger.c:1039`
    AimAtTarget,
    /// `oracle/oracle/codemp/game/g_combat.c:675`
    BodyRid,
    /// `oracle/oracle/codemp/game/g_client.c:973`
    BodySink,
    /// `oracle/oracle/codemp/game/g_items.c:254`
    CreateShield,
    /// `oracle/oracle/codemp/game/g_weapon.c:1310`
    DEMP2_AltDetonate,
    /// `oracle/oracle/codemp/game/g_weapon.c:1164`
    DEMP2_AltRadiusDamage,
    /// `oracle/oracle/codemp/game/w_saber.c:6235`
    DeadSaberThink,
    /// `oracle/oracle/codemp/game/g_weapon.c:2740`
    DetPackBlow,
    /// `oracle/oracle/codemp/game/NPC_behavior.c:185`
    Disappear,
    /// `oracle/oracle/codemp/game/w_saber.c:6342`
    DownedSaberThink,
    /// `oracle/oracle/codemp/game/g_items.c:1776`
    EWebThink,
    /// `oracle/oracle/codemp/game/g_items.c:2779`
    FinishSpawningItem,
    /// `oracle/oracle/codemp/game/g_missile.c:215`
    G_ExplodeMissile,
    /// `oracle/oracle/codemp/game/g_utils.c:932`
    G_FreeEntity,
    /// `oracle/oracle/codemp/game/g_misc.c:638`
    G_PortalifyEntities,
    /// `oracle/oracle/codemp/game/g_object.c:72`
    G_RunObject,
    /// `oracle/oracle/codemp/game/g_vehicles.c:186`
    G_VehicleSpawn,
    /// `oracle/oracle/codemp/game/g_misc.c:907`
    HolocronThink,
    /// `oracle/oracle/codemp/game/g_misc.c:1142`
    InitShooter_Finish,
    /// `oracle/oracle/codemp/game/g_client.c:346`
    JMSaberThink,
    /// `oracle/oracle/codemp/game/g_combat.c:3391`
    LimbThink,
    /// `oracle/oracle/codemp/game/g_ICARUScb.c:1865`
    MoveOwner,
    /// `oracle/oracle/codemp/game/NPC_spawn.c:862`
    NPC_Begin,
    /// `oracle/oracle/codemp/game/NPC.c:115`
    NPC_RemoveBody,
    /// `oracle/oracle/codemp/game/NPC_spawn.c:1814`
    NPC_ShySpawn,
    /// `oracle/oracle/codemp/game/NPC_spawn.c:1765`
    NPC_Spawn_Go,
    /// `oracle/oracle/codemp/game/NPC.c:1826`
    NPC_Think,
    /// `oracle/oracle/codemp/game/g_items.c:2288`
    RespawnItem,
    /// `oracle/oracle/codemp/game/g_mover.c:615`
    ReturnToPos1,
    /// `oracle/oracle/codemp/game/w_saber.c:286`
    SaberUpdateSelf,
    /// `oracle/oracle/codemp/game/g_items.c:171`
    ShieldGoSolid,
    /// `oracle/oracle/codemp/game/g_items.c:123`
    ShieldThink,
    /// `oracle/oracle/codemp/game/g_saga.c:1382`
    SiegeItemThink,
    /// `oracle/oracle/codemp/game/g_ICARUScb.c:4271`
    SolidifyOwner,
    /// `oracle/oracle/codemp/game/g_items.c:1277`
    SpecialItemThink,
    /// `oracle/oracle/codemp/game/g_team.c:714`
    Team_DroppedFlagThink,
    /// `oracle/oracle/codemp/game/g_mover.c:1727`
    Think_BeginMoving,
    /// `oracle/oracle/codemp/game/g_mover.c:1217`
    Think_MatchTeam,
    /// `oracle/oracle/codemp/game/g_mover.c:1802`
    Think_SetupTrainTargets,
    /// `oracle/oracle/codemp/game/g_mover.c:1168`
    Think_SpawnNewDoorTrigger,
    /// `oracle/oracle/codemp/game/g_trigger.c:789`
    Think_Strike,
    /// `oracle/oracle/codemp/game/g_target.c:78`
    Think_Target_Delay,
    /// `oracle/oracle/codemp/game/g_weapon.c:2471`
    TrapThink,
    /// `oracle/oracle/codemp/game/g_mover.c:710`
    Use_BinaryMover_Go,
    /// `oracle/oracle/codemp/game/g_weapon.c:3610`
    WP_VehWeapSetSolidToOwner,
    /// `oracle/oracle/codemp/game/g_weapon.c:1549`
    WP_flechette_alt_blow,
    /// `oracle/oracle/codemp/game/g_ICARUScb.c:569`
    anglerCallback,
    /// `oracle/oracle/codemp/game/g_trigger.c:1927`
    asteroid_field_think,
    /// `oracle/oracle/codemp/game/g_trigger.c:1922`
    asteroid_move_to_start,
    /// `oracle/oracle/codemp/game/g_misc.c:1176`
    check_recharge,
    /// `oracle/oracle/codemp/game/g_weapon.c:4828`
    emplaced_gun_update,
    /// `oracle/oracle/codemp/game/g_misc.c:2758`
    faller_think,
    /// `oracle/oracle/codemp/game/g_mover.c:2398`
    funcBBrushDieGo,
    /// `oracle/oracle/codemp/game/g_trigger.c:1757`
    func_timer_think,
    /// `oracle/oracle/codemp/game/g_mover.c:3027`
    func_usable_think,
    /// `oracle/oracle/codemp/game/g_mover.c:2995`
    func_wait_return_solid,
    /// `oracle/oracle/codemp/game/g_misc.c:2387`
    fx_runner_link,
    /// `oracle/oracle/codemp/game/g_misc.c:2266`
    fx_runner_think,
    /// `oracle/oracle/codemp/game/g_weapon.c:2244`
    laserTrapExplode,
    /// `oracle/oracle/codemp/game/g_weapon.c:2367`
    laserTrapThink,
    /// `oracle/oracle/codemp/game/g_misc.c:305`
    locateCamera,
    /// `oracle/oracle/codemp/game/g_misc.c:2659`
    maglock_link,
    /// `oracle/oracle/codemp/game/g_misc.c:2830`
    misc_faller_think,
    /// `oracle/oracle/codemp/game/g_misc.c:3417`
    misc_weapon_shooter_aim,
    /// `oracle/oracle/codemp/game/g_misc.c:3391`
    misc_weapon_shooter_fire,
    /// `oracle/oracle/codemp/game/g_trigger.c:32`
    multi_trigger_run,
    /// `oracle/oracle/codemp/game/g_items.c:708`
    pas_think,
    /// `oracle/oracle/codemp/game/g_weapon.c:2320`
    proxMineThink,
    /// `oracle/oracle/codemp/game/g_misc.c:3267`
    ref_link,
    /// `oracle/oracle/codemp/game/g_weapon.c:1651`
    rocketThink,
    /// `oracle/oracle/codemp/game/w_saber.c:6917`
    saberBackToOwner,
    /// `oracle/oracle/codemp/game/w_saber.c:7117`
    saberFirstThrown,
    /// `oracle/oracle/codemp/game/g_target.c:754`
    scriptrunner_run,
    /// `oracle/oracle/codemp/game/g_items.c:702`
    sentryExpire,
    /// `oracle/oracle/codemp/game/g_trigger.c:1533`
    shipboundary_think,
    /// `oracle/oracle/codemp/game/g_target.c:401`
    target_laser_start,
    /// `oracle/oracle/codemp/game/g_target.c:349`
    target_laser_think,
    /// `oracle/oracle/codemp/game/g_target.c:554`
    target_location_linkup,
    /// `oracle/oracle/codemp/game/g_weapon.c:1936`
    thermalDetonatorExplode,
    /// `oracle/oracle/codemp/game/g_weapon.c:1972`
    thermalThinkStandard,
    /// `oracle/oracle/codemp/game/g_trigger.c:872`
    trigger_always_think,
    /// `oracle/oracle/codemp/game/g_trigger.c:549`
    trigger_cleared_fire,
    /// `oracle/oracle/codemp/game/g_turret_G2.c:829`
    turretG2_base_think,
    /// `oracle/oracle/codemp/game/g_turret.c:505`
    turret_base_think,
}

/// Raven `reached` fn-pointer targets (4 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntReached {
    /// `oracle/oracle/codemp/game/g_mover.c:634`
    Reached_BinaryMover,
    /// `oracle/oracle/codemp/game/g_mover.c:1739`
    Reached_Train,
    /// `oracle/oracle/codemp/game/g_ICARUScb.c:667`
    moveAndRotateCallback,
    /// `oracle/oracle/codemp/game/g_ICARUScb.c:603`
    moverCallback,
}

/// Raven `blocked` fn-pointer targets (2 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntBlocked {
    /// `oracle/oracle/codemp/game/g_mover.c:1018`
    Blocked_Door,
    /// `oracle/oracle/codemp/game/g_ICARUScb.c:635`
    Blocked_Mover,
}

/// Raven `touch` fn-pointer targets (27 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntTouch {
    /// `oracle/oracle/codemp/game/g_misc.c:786`
    HolocronTouch,
    /// `oracle/oracle/codemp/game/g_client.c:385`
    JMSaberTouch,
    /// `oracle/oracle/codemp/game/g_combat.c:3387`
    LimbTouch,
    /// `oracle/oracle/codemp/game/w_saber.c:6229`
    SaberBounceSound,
    /// `oracle/oracle/codemp/game/w_saber.c:360`
    SaberGotHit,
    /// `oracle/oracle/codemp/game/g_items.c:515`
    SentryTouch,
    /// `oracle/oracle/codemp/game/g_items.c:230`
    ShieldTouch,
    /// `oracle/oracle/codemp/game/g_saga.c:1477`
    SiegeItemTouch,
    /// `oracle/oracle/codemp/game/g_mover.c:1633`
    Touch_Button,
    /// `oracle/oracle/codemp/game/g_mover.c:1078`
    Touch_DoorTrigger,
    /// `oracle/oracle/codemp/game/g_items.c:2362`
    Touch_Item,
    /// `oracle/oracle/codemp/game/g_trigger.c:350`
    Touch_Multi,
    /// `oracle/oracle/codemp/game/g_mover.c:1489`
    Touch_Plat,
    /// `oracle/oracle/codemp/game/g_mover.c:1507`
    Touch_PlatCenterTrigger,
    /// `oracle/oracle/codemp/game/g_weapon.c:3569`
    WP_TouchVehMissile,
    /// `oracle/oracle/codemp/game/g_weapon.c:2645`
    charge_stick,
    /// `oracle/oracle/codemp/game/g_misc.c:2730`
    faller_touch,
    /// `oracle/oracle/codemp/game/g_mover.c:2663`
    funcBBrushTouch,
    /// `oracle/oracle/codemp/game/g_trigger.c:1299`
    hurt_touch,
    /// `oracle/oracle/codemp/game/g_trigger.c:1597`
    hyperspace_touch,
    /// `oracle/oracle/codemp/game/g_trigger.c:1494`
    shipboundary_touch,
    /// `oracle/oracle/codemp/game/g_trigger.c:1442`
    space_touch,
    /// `oracle/oracle/codemp/game/w_saber.c:7080`
    thrownSaberTouch,
    /// `oracle/oracle/codemp/game/g_weapon.c:2296`
    touchLaserTrap,
    /// `oracle/oracle/codemp/game/g_weapon.c:165`
    touch_NULL,
    /// `oracle/oracle/codemp/game/g_trigger.c:901`
    trigger_push_touch,
    /// `oracle/oracle/codemp/game/g_trigger.c:1197`
    trigger_teleporter_touch,
}

/// Raven `use` fn-pointer targets (53 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntUse {
    /// `oracle/oracle/codemp/game/g_mover.c:2930`
    GlassUse,
    /// `oracle/oracle/codemp/game/NPC_spawn.c:1839`
    NPC_Spawn,
    /// `oracle/oracle/codemp/game/NPC_reactions.c:1008`
    NPC_Use,
    /// `oracle/oracle/codemp/game/NPC_spawn.c:2175`
    NPC_VehicleSpawnUse,
    /// `oracle/oracle/codemp/game/g_saga.c:1182`
    SiegeIconUse,
    /// `oracle/oracle/codemp/game/g_saga.c:1576`
    SiegeItemUse,
    /// `oracle/oracle/codemp/game/g_client.c:140`
    SiegePointUse,
    /// `oracle/oracle/codemp/game/g_mover.c:863`
    Use_BinaryMover,
    /// `oracle/oracle/codemp/game/g_items.c:2765`
    Use_Item,
    /// `oracle/oracle/codemp/game/g_trigger.c:343`
    Use_Multi,
    /// `oracle/oracle/codemp/game/g_misc.c:1107`
    Use_Shooter,
    /// `oracle/oracle/codemp/game/g_trigger.c:801`
    Use_Strike,
    /// `oracle/oracle/codemp/game/g_target.c:82`
    Use_Target_Delay,
    /// `oracle/oracle/codemp/game/g_misc.c:2569`
    Use_Target_Escapetrig,
    /// `oracle/oracle/codemp/game/g_target.c:10`
    Use_Target_Give,
    /// `oracle/oracle/codemp/game/g_target.c:132`
    Use_Target_Print,
    /// `oracle/oracle/codemp/game/g_target.c:113`
    Use_Target_Score,
    /// `oracle/oracle/codemp/game/g_misc.c:2543`
    Use_Target_Screenshake,
    /// `oracle/oracle/codemp/game/g_target.c:259`
    Use_Target_Speaker,
    /// `oracle/oracle/codemp/game/g_trigger.c:1138`
    Use_target_push,
    /// `oracle/oracle/codemp/game/g_target.c:47`
    Use_target_remove_powerups,
    /// `oracle/oracle/codemp/game/g_misc.c:1331`
    ammo_generic_power_converter_use,
    /// `oracle/oracle/codemp/game/g_misc.c:1753`
    ammo_power_converter_use,
    /// `oracle/oracle/codemp/game/g_saga.c:1236`
    decompTriggerUse,
    /// `oracle/oracle/codemp/game/g_weapon.c:4804`
    emplaced_gun_realuse,
    /// `oracle/oracle/codemp/game/g_mover.c:2516`
    funcBBrushUse,
    /// `oracle/oracle/codemp/game/g_mover.c:2021`
    func_static_use,
    /// `oracle/oracle/codemp/game/g_trigger.c:1763`
    func_timer_use,
    /// `oracle/oracle/codemp/game/g_mover.c:3050`
    func_usable_use,
    /// `oracle/oracle/codemp/game/g_misc.c:2313`
    fx_runner_use,
    /// `oracle/oracle/codemp/game/g_misc.c:1921`
    health_power_converter_use,
    /// `oracle/oracle/codemp/game/g_trigger.c:1280`
    hurt_use,
    /// `oracle/oracle/codemp/game/g_misc.c:134`
    misc_dlight_use,
    /// `oracle/oracle/codemp/game/g_misc.c:2789`
    misc_faller_create,
    /// `oracle/oracle/codemp/game/g_misc.c:3401`
    misc_weapon_shooter_use,
    /// `oracle/oracle/codemp/game/NPC_AI_Sentry.c:64`
    sentry_use,
    /// `oracle/oracle/codemp/game/g_misc.c:1230`
    shield_power_converter_use,
    /// `oracle/oracle/codemp/game/g_saga.c:1317`
    siegeEndUse,
    /// `oracle/oracle/codemp/game/g_saga.c:1060`
    siegeTriggerUse,
    /// `oracle/oracle/codemp/game/g_target.c:912`
    target_activate_use,
    /// `oracle/oracle/codemp/game/g_target.c:611`
    target_counter_use,
    /// `oracle/oracle/codemp/game/g_target.c:919`
    target_deactivate_use,
    /// `oracle/oracle/codemp/game/g_target.c:534`
    target_kill_use,
    /// `oracle/oracle/codemp/game/g_target.c:392`
    target_laser_use,
    /// `oracle/oracle/codemp/game/g_target.c:945`
    target_level_change_use,
    /// `oracle/oracle/codemp/game/g_target.c:972`
    target_play_music_use,
    /// `oracle/oracle/codemp/game/g_target.c:681`
    target_random_use,
    /// `oracle/oracle/codemp/game/g_target.c:479`
    target_relay_use,
    /// `oracle/oracle/codemp/game/g_target.c:839`
    target_scriptrunner_use,
    /// `oracle/oracle/codemp/game/g_target.c:440`
    target_teleporter_use,
    /// `oracle/oracle/codemp/game/g_turret_G2.c:959`
    turretG2_base_use,
    /// `oracle/oracle/codemp/game/g_turret.c:604`
    turret_base_use,
    /// `oracle/oracle/codemp/game/g_mover.c:3215`
    use_wall,
}

/// Raven `pain` fn-pointer targets (14 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntPain {
    /// `oracle/oracle/codemp/game/g_weapon.c:2768`
    DetPackPain,
    /// `oracle/oracle/codemp/game/g_items.c:1484`
    EWebPain,
    /// `oracle/oracle/codemp/game/g_mover.c:2924`
    GlassPain,
    /// `oracle/oracle/codemp/game/NPC_AI_Atst.c:119`
    NPC_ATST_Pain,
    /// `oracle/oracle/codemp/game/NPC_AI_Rancor.c:703`
    NPC_Rancor_Pain,
    /// `oracle/oracle/codemp/game/NPC_AI_Wampa.c:433`
    NPC_Wampa_Pain,
    /// `oracle/oracle/codemp/game/g_items.c:155`
    ShieldPain,
    /// `oracle/oracle/codemp/game/g_saga.c:1547`
    SiegeItemPain,
    /// `oracle/oracle/codemp/game/g_turret.c:35`
    TurretBasePain,
    /// `oracle/oracle/codemp/game/g_turret_G2.c:210`
    TurretG2Pain,
    /// `oracle/oracle/codemp/game/g_turret.c:11`
    TurretPain,
    /// `oracle/oracle/codemp/game/g_weapon.c:4810`
    emplaced_gun_pain,
    /// `oracle/oracle/codemp/game/g_mover.c:2532`
    funcBBrushPain,
    /// `oracle/oracle/codemp/game/g_mover.c:3108`
    func_usable_pain,
}

/// Raven `die` fn-pointer targets (17 distinct assigns
/// in game/*.c bodies).
///
/// Source: `oracle/oracle/codemp/game/g_local.h:285-291`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntDie {
    /// `oracle/oracle/codemp/game/g_weapon.c:2775`
    DetPackDie,
    /// `oracle/oracle/codemp/game/g_items.c:1449`
    EWebDie,
    /// `oracle/oracle/codemp/game/g_mover.c:2875`
    GlassDie,
    /// `oracle/oracle/codemp/game/g_weapon.c:1814`
    RocketDie,
    /// `oracle/oracle/codemp/game/g_items.c:145`
    ShieldDie,
    /// `oracle/oracle/codemp/game/g_saga.c:1553`
    SiegeItemDie,
    /// `oracle/oracle/codemp/game/g_turret.c:51`
    auto_turret_die,
    /// `oracle/oracle/codemp/game/g_combat.c:686`
    body_die,
    /// `oracle/oracle/codemp/game/g_turret.c:112`
    bottom_die,
    /// `oracle/oracle/codemp/game/g_weapon.c:4930`
    emplaced_gun_die,
    /// `oracle/oracle/codemp/game/g_mover.c:2500`
    funcBBrushDie,
    /// `oracle/oracle/codemp/game/g_mover.c:3113`
    func_usable_die,
    /// `oracle/oracle/codemp/game/g_weapon.c:2282`
    laserTrapDelayedExplode,
    /// `oracle/oracle/codemp/game/g_misc.c:2623`
    maglock_die,
    /// `oracle/oracle/codemp/game/g_combat.c:2123`
    player_die,
    /// `oracle/oracle/codemp/game/g_turret_G2.c:236`
    turretG2_die,
    /// `oracle/oracle/codemp/game/g_items.c:940`
    turret_die,
}
