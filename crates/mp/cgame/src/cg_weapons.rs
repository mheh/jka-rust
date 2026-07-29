//! Port of `oracle/codemp/cgame/cg_weapons.c` — weapon selection, the viewmodel, and weapon fire effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::bg_saberLoad::BG_SI_SetDesiredLength;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::bg_itemlist::{bg_itemlist, bg_numItems};
use mp_bg::public::entity_flags::{EF_ALT_FIRING, EF_DEAD, EF_FIRING, EF_JETPACK_ACTIVE};
use mp_bg::public::entity_type::entityType_t::{ET_NPC, ET_PLAYER};
use mp_bg::public::force_hand_anims::forceHandAnims_t::HANDEXTEND_NONE;
use mp_bg::public::gametype::GT_CTY;
use mp_bg::public::item_kind::ItemKind;
use mp_bg::public::pers_enum::persEnum_t::PERS_TEAM;
use mp_bg::public::pmtype::pmtype_t;
use mp_bg::public::powerup::{PW_BLUEFLAG, PW_REDFLAG};
use mp_bg::public::stat_index::statIndex_t::{STAT_HEALTH, STAT_WEAPONS};
use mp_bg::public::team::TEAM_SPECTATOR;
use mp_bg::public::viewheight::{CROUCH_VIEWHEIGHT, DEFAULT_VIEWHEIGHT};
use mp_bg::public::weaponstate::weaponstate_t::{WEAPON_CHARGING, WEAPON_CHARGING_ALT};
use mp_bg::weapons::weapon_data::weaponData;
use mp_bg::weapons::weapon_t::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_DET_PACK,
    WP_DISRUPTOR, WP_EMPLACED_GUN, WP_FLECHETTE, WP_MELEE, WP_NONE, WP_NUM_WEAPONS, WP_REPEATER,
    WP_ROCKET_LAUNCHER, WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use mp_bg::weapons::wp_muzzle_point::WP_MuzzlePoint;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::tr_types::{RF_DEPTHHACK, RF_FIRST_PERSON};
use mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS;
use mp_qshared::common::mp::qcommon::saber::saber_info::saberInfo_t;
use mp_qshared::common::mp::qcommon::{playerState_t, MAX_SABERS, PMF_FOLLOW};
use mp_qshared::shared::q_math::{
    _VectorCopy, _VectorMA, vec3_origin, AngleVectors, AnglesToAxis, ByteToDir, VectorClear, PITCH,
    ROLL, YAW,
};
use mp_qshared::shared::{
    addspriteArgStruct_t, ct_table_t, mdxaBone_t, qfalse, qtrue, vec3_t, vec4_t, Eorientations,
    CHAN_AUTO, CHAN_WEAPON, MAX_CLIENTS_I32,
};
use mp_uishared::shared::display_state::DisplayState;
use native_string::{atoi, Q_stricmp};

use crate::cg_draw::colorTable;
use crate::cg_drawtools::{CG_DrawPic, UI_DrawProportionalString};
use crate::cg_ents::{CG_PositionEntityOnTag, CG_PositionRotatedEntityOnTag};
use crate::cg_main::{CG_Argv, CG_Error};
use crate::cg_players::CG_IsMindTricked;
use crate::cg_view::CGCam_Shake;
use crate::cg_weaponinit::CG_RegisterWeapon;
use crate::fx_blaster::{FX_BlasterWeaponHitPlayer, FX_BlasterWeaponHitWall};
use crate::fx_bowcaster::{FX_BowcasterHitPlayer, FX_BowcasterHitWall};
use crate::fx_bryarpistol::{
    FX_BryarAltHitPlayer, FX_BryarAltHitWall, FX_BryarHitPlayer, FX_BryarHitWall,
    FX_ConcussionHitPlayer, FX_ConcussionHitWall, FX_TurretHitPlayer, FX_TurretHitWall,
};
use crate::fx_demp2::{FX_DEMP2_HitPlayer, FX_DEMP2_HitWall};
use crate::fx_disruptor::{FX_DisruptorAltHit, FX_DisruptorAltMiss};
use crate::fx_flechette::{FX_FlechetteWeaponHitPlayer, FX_FlechetteWeaponHitWall};
use crate::fx_heavyrepeater::{
    FX_RepeaterAltHitPlayer, FX_RepeaterAltHitWall, FX_RepeaterHitPlayer, FX_RepeaterHitWall,
};
use crate::fx_rocketlauncher::{FX_RocketHitPlayer, FX_RocketHitWall};
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::clientInfo_t;
use crate::local::impact_sound_t::impactSound_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// FILE-SCOPE CONSTANTS
// Source: `oracle/codemp/cgame/cg_weapons.c:126,890-892`
// ---------------------------------------------------------------------------

/// Raven `ICON_WEAPONS` — the weapon-prong selector for the icon HUD's
/// `cgs.media.currentBackground`.
/// Source: `oracle/codemp/cgame/cg_weapons.c:890`
pub const ICON_WEAPONS: c_int = 0;

/// Raven `ICON_FORCE`.
/// Source: `oracle/codemp/cgame/cg_weapons.c:891`
pub const ICON_FORCE: c_int = 1;

/// Raven `ICON_INVENTORY`.
/// Source: `oracle/codemp/cgame/cg_weapons.c:892`
pub const ICON_INVENTORY: c_int = 2;

// `cg_local.h` timing `#define`s this file reads. They have no ported
// cross-crate home yet, so they land beside their readers — the treatment
// `cg_players.rs` gave `RF_THIRD_PERSON`.

/// Raven `LAND_DEFLECT_TIME` — how long the landing dip drops the viewmodel.
/// Source: `oracle/codemp/cgame/cg_local.h:31`
pub const LAND_DEFLECT_TIME: c_int = 150;

/// Raven `LAND_RETURN_TIME` — the recovery tail after the dip.
/// Source: `oracle/codemp/cgame/cg_local.h:32`
pub const LAND_RETURN_TIME: c_int = 300;

/// Raven `MUZZLE_FLASH_TIME` — how long after a shot the muzzle flash is still
/// drawn, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:47`
pub const MUZZLE_FLASH_TIME: c_int = 20;

/// Raven `WEAPON_SELECT_TIME` — how long a weapon/force/inventory pick keeps
/// the icon HUD up, in msec.
/// Source: `oracle/codemp/cgame/cg_local.h:36`
pub const WEAPON_SELECT_TIME: c_int = 1400;

/// Raven `LAST_USEABLE_WEAPON` — `WP_BRYAR_OLD`, the top of the range a player
/// may cycle to. Same beside-the-reader treatment `g_cmds.rs` gave it.
/// Source: `oracle/codemp/game/bg_weapons.h:43`
pub const LAST_USEABLE_WEAPON: c_int = WP_BRYAR_OLD;

// `tr_types.h`'s renderfx bits and `q_shared.h`'s `UI_*` text flags have no
// ported cross-crate home, so the two this file reads land beside their reader —
// the same file-local-copy story `cg_players.rs`/`cg_drawtools.rs` carry.

/// Raven `UI_CENTER`.
/// Source: `oracle/codemp/game/q_shared.h:488`
const UI_CENTER: c_int = 0x00000001;

/// Raven `UI_SMALLFONT`.
/// Source: `oracle/codemp/game/q_shared.h:491`
const UI_SMALLFONT: c_int = 0x00000010;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Raven `CG_SetGhoul2InfoRef` — copies the ghoul2 half of one `refEntity_t`
/// onto another.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:15-21`
pub fn CG_SetGhoul2InfoRef(ent: &mut refEntity_t, s1: &refEntity_t) {
    ent.ghoul2 = s1.ghoul2;
    _VectorCopy(s1.modelScale, &mut ent.modelScale);
    ent.radius = s1.radius;
    _VectorCopy(s1.angles, &mut ent.angles);
}

/// Raven `CG_RegisterItemVisuals` — precaches item `itemNum`'s world models,
/// its ghoul2 instance and its HUD icon. Registration is once-only: the
/// `registered` latch short-circuits every later call.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:34-115`
pub fn CG_RegisterItemVisuals(ctx: &mut CgContext, itemNum: c_int) {
    let engine = ctx.engine;

    if itemNum < 0 || itemNum >= bg_numItems {
        let msg = format!(
            "CG_RegisterItemVisuals: itemNum {} out of range [0-{}]",
            itemNum,
            bg_numItems - 1
        );
        CG_Error(ctx, &msg);
        return;
    }

    let slot = itemNum as usize;
    if ctx.world.cg_items[slot].registered != qfalse {
        return;
    }

    let item = &bg_itemlist[slot];

    // PORT-NOTE: Raven's `memset(itemInfo, 0, sizeof(&itemInfo))` measures a
    // POINTER, not the struct - 4 bytes on retail, which is exactly the
    // `registered` field the very next line overwrites. Dead, so only the write
    // survives.
    ctx.world.cg_items[slot].registered = qtrue;

    // Raven's `world_model[MAX_ITEM_MODELS]` kept its NULL padding; `GItem`
    // dropped it, so a slot Raven read as NULL reads as the empty name and the
    // engine registers nothing (§F19).
    let worldModel = |n: usize| -> &'static str { item.world_model.get(n).copied().unwrap_or("") };

    let modelName = match item.kind {
        //in CTY the flag model is different
        ItemKind::Team(giTag)
            if (giTag == PW_REDFLAG || giTag == PW_BLUEFLAG)
                && ctx.world.cgs.gametype == GT_CTY =>
        {
            worldModel(1)
        }
        ItemKind::Weapon(giTag)
            if giTag == WP_THERMAL || giTag == WP_TRIP_MINE || giTag == WP_DET_PACK =>
        {
            worldModel(1)
        }
        _ => worldModel(0),
    };
    ctx.world.cg_items[slot].models[0] = trap::R_RegisterModel(engine, modelName);

    /*
    Ghoul2 Insert Start
    */
    // Raven indexes `world_model[0] + strlen - 4` unconditionally, so a name
    // shorter than 4 chars reads before the buffer; here that is simply "not a
    // .glm" (§F19).
    let wm0 = worldModel(0);
    if wm0.len() >= 4 && Q_stricmp(&wm0[wm0.len() - 4..], ".glm") == 0 {
        let handle = trap::G2API_InitGhoul2Model(
            engine,
            &mut ctx.world.cg_items[slot].g2Models[0] as *mut *mut c_void,
            wm0,
            0,
            0,
            0,
            0,
            0,
        );
        if handle < 0 {
            ctx.world.cg_items[slot].g2Models[0] = null_mut();
        } else {
            ctx.world.cg_items[slot].radius[0] = 60.0;
        }
    }
    /*
    Ghoul2 Insert End
    */

    if let Some(icon) = item.icon {
        if matches!(item.kind, ItemKind::Health) {
            //medpack gets nomip'd by the ui or something I guess.
            ctx.world.cg_items[slot].icon = trap::R_RegisterShaderNoMip(engine, icon);
        } else {
            ctx.world.cg_items[slot].icon = trap::R_RegisterShader(engine, icon);
        }
    } else {
        ctx.world.cg_items[slot].icon = 0;
    }

    if let ItemKind::Weapon(giTag) = item.kind {
        CG_RegisterWeapon(ctx, giTag);
    }

    //
    // powerups have an accompanying ring or sphere
    //
    if matches!(
        item.kind,
        ItemKind::Powerup(_) | ItemKind::Health | ItemKind::Armor { .. } | ItemKind::Holdable(_)
    ) {
        // Raven's `if (item->world_model[1])` NULL test is the missing-entry test
        if let Some(&wm1) = item.world_model.get(1) {
            ctx.world.cg_items[slot].models[1] = trap::R_RegisterModel(engine, wm1);
        }
    }
}

/// Raven `CG_MapTorsoToWeaponFrame` — which viewmodel frame goes with the
/// torso animation we're on. `-1` means "no matching frame".
///
/// The busy-holster block is gated on `#define WEAPON_FORCE_BUSY_HOLSTER`
/// (`cg_weapons.c:126`), which this file always defines, so it is unconditional
/// here.
///
/// PORT-NOTE: Raven's `clientInfo_t *ci` parameter is never read in the body,
/// and its only caller ([`CG_AddViewWeapon`], the sole one - Raven's fn is
/// `static`) sources it from the same `CgWorld` this fn takes by `&mut`. The
/// dead parameter is dropped rather than fought for; the caller keeps Raven's
/// "no npcClient means bail" early return.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:140-202`
fn CG_MapTorsoToWeaponFrame(world: &mut CgWorld, frame: c_int, animNum: c_int) -> c_int {
    // no snapshot means no hand extension, which is the reset arm below
    let forceHandExtend = world
        .cg
        .snap_ref()
        .map_or(HANDEXTEND_NONE as c_int, |snap| snap.ps.forceHandExtend);

    if forceHandExtend != HANDEXTEND_NONE as c_int || world.weapons.cgWeapFrameTime > world.cg.time
    {
        // the reason for the after delay is so that it doesn't snap the weapon frame to the "idle" (0) frame
        // for a very quick moment
        if world.weapons.cgWeapFrame < 6 {
            world.weapons.cgWeapFrame = 6;
            world.weapons.cgWeapFrameTime = world.cg.time + 10;
        }

        if world.weapons.cgWeapFrameTime < world.cg.time && world.weapons.cgWeapFrame < 10 {
            world.weapons.cgWeapFrame += 1;
            world.weapons.cgWeapFrameTime = world.cg.time + 10;
        }

        if forceHandExtend != HANDEXTEND_NONE as c_int && world.weapons.cgWeapFrame == 10 {
            world.weapons.cgWeapFrameTime = world.cg.time + 100;
        }

        return world.weapons.cgWeapFrame;
    } else {
        world.weapons.cgWeapFrame = 0;
        world.weapons.cgWeapFrameTime = 0;
    }

    let animations = &world.bg_state.bgHumanoidAnimations;

    if animNum == animNumber_t::TORSO_DROPWEAP1 as c_int {
        let firstFrame = animations[animNum as usize].firstFrame as c_int;
        if frame >= firstFrame && frame < firstFrame + 5 {
            return frame - firstFrame + 6;
        }
    } else if animNum == animNumber_t::TORSO_RAISEWEAP1 as c_int {
        let firstFrame = animations[animNum as usize].firstFrame as c_int;
        if frame >= firstFrame && frame < firstFrame + 4 {
            return frame - firstFrame + 6 + 4;
        }
    } else if animNum == animNumber_t::BOTH_ATTACK1 as c_int
        || animNum == animNumber_t::BOTH_ATTACK2 as c_int
        || animNum == animNumber_t::BOTH_ATTACK3 as c_int
        || animNum == animNumber_t::BOTH_ATTACK4 as c_int
        || animNum == animNumber_t::BOTH_ATTACK10 as c_int
        || animNum == animNumber_t::BOTH_THERMAL_THROW as c_int
    {
        let firstFrame = animations[animNum as usize].firstFrame as c_int;
        if frame >= firstFrame && frame < firstFrame + 6 {
            return 1 + (frame - firstFrame);
        }
    }

    -1
}

/// Raven `CG_CalculateWeaponPosition` — where the viewmodel sits this frame:
/// the view origin/angles plus bob, the landing dip and the idle drift.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:210-255`
pub fn CG_CalculateWeaponPosition(world: &CgWorld, origin: &mut vec3_t, angles: &mut vec3_t) {
    let mut scale: f32;

    _VectorCopy(world.cg.refdef.vieworg, origin);
    _VectorCopy(world.cg.refdef.viewangles, angles);

    // on odd legs, invert some angles
    if (world.cg.bobcycle & 1) != 0 {
        scale = -world.cg.xyspeed;
    } else {
        scale = world.cg.xyspeed;
    }

    // gun angles from bobbing
    angles[ROLL] = (angles[ROLL] as f64 + (scale * world.cg.bobfracsin) as f64 * 0.005) as f32;
    angles[YAW] = (angles[YAW] as f64 + (scale * world.cg.bobfracsin) as f64 * 0.01) as f32;
    angles[PITCH] =
        (angles[PITCH] as f64 + (world.cg.xyspeed * world.cg.bobfracsin) as f64 * 0.005) as f32;

    // drop the weapon when landing
    let delta = world.cg.time - world.cg.landTime;
    if delta < LAND_DEFLECT_TIME {
        origin[2] = (origin[2] as f64
            + world.cg.landChange as f64 * 0.25 * delta as f64 / LAND_DEFLECT_TIME as f64)
            as f32;
    } else if delta < LAND_DEFLECT_TIME + LAND_RETURN_TIME {
        origin[2] = (origin[2] as f64
            + world.cg.landChange as f64
                * 0.25
                * (LAND_DEFLECT_TIME + LAND_RETURN_TIME - delta) as f64
                / LAND_RETURN_TIME as f64) as f32;
    }

    // Raven's stair-climb drop sits under `#if 0` — dead source, not ported.

    // idle drift
    scale = world.cg.xyspeed + 40.0;
    let fracsin = (world.cg.time as f64 * 0.001).sin() as f32;
    angles[ROLL] = (angles[ROLL] as f64 + (scale * fracsin) as f64 * 0.01) as f32;
    angles[YAW] = (angles[YAW] as f64 + (scale * fracsin) as f64 * 0.01) as f32;
    angles[PITCH] = (angles[PITCH] as f64 + (scale * fracsin) as f64 * 0.01) as f32;
}

/// Raven `CG_LightningBolt` — a no-op gate.
///
/// Raven: "NOTENOTE No lightning gun-ish stuff yet." Every use of the local
/// beam (the CPMA "true lightning" trace and the impact flare) is commented
/// out, so all that survives is the durational-weapon test and its early
/// return. Kept because it is Raven's shape and the later waves' callers still
/// make the call.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:269-366`
pub fn CG_LightningBolt(cent: &centity_t, _origin: &vec3_t) {
    // Must be a durational weapon that continuously generates an effect.
    let durational =
        cent.currentState.weapon == WP_DEMP2 && (cent.currentState.eFlags & EF_ALT_FIRING) != 0;
    if !durational {
        return;
    }

    // Raven zeroes a local `refEntity_t beam` here and then never uses it.
}

/// Raven `CG_AddWeaponWithPowerups` — hands the gun to the renderer, then a
/// second time in an electrocution shell while the player is being shocked.
///
/// `powerups` is Raven's unused parameter.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:374-392`
pub fn CG_AddWeaponWithPowerups(ctx: &mut CgContext, gun: &mut refEntity_t, _powerups: c_int) {
    let engine = ctx.engine;

    // add powerup effects
    trap::R_AddRefEntityToScene(engine, gun);

    if ctx.world.cg.predictedPlayerState.electrifyTime > ctx.world.cg.time {
        // add electrocution shell
        let preShader = gun.customShader;
        if (ctx.world.bg_state.rng.rand() & 1) != 0 {
            gun.customShader = ctx.world.cgs.media.electricBodyShader;
        } else {
            gun.customShader = ctx.world.cgs.media.electricBody2Shader;
        }
        trap::R_AddRefEntityToScene(engine, gun);
        gun.customShader = preShader; // set back just to be safe
    }
}

/// Raven `CG_AddPlayerWeapon` — hangs the gun, its barrels, the charge sprite
/// and the muzzle flash off a player refEntity.
///
/// Raven: "Used for both the view weapon (ps is valid) and the world modelother
/// character models (ps is NULL). The main player will have this called for
/// BOTH cases, so effects like light and sound should only be done on the world
/// model case."
///
/// `ps` carries only its presence and `clientNum`; it never aliases `ctx`, so a
/// caller handing us `cg.predictedPlayerState` copies it out first. `team` is
/// Raven's unused parameter.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:404-757`
pub fn CG_AddPlayerWeapon(
    ctx: &mut CgContext,
    parent: &refEntity_t,
    ps: Option<&playerState_t>,
    centNum: usize,
    _team: c_int,
    newAngles: &vec3_t,
    thirdPerson: bool,
) {
    let engine = ctx.engine;

    let weaponNum = ctx.world.entity(centNum).currentState.weapon;

    if ctx.world.entity(centNum).currentState.weapon == WP_EMPLACED_GUN {
        return;
    }

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int
        && ctx.world.entity(centNum).currentState.number
            == ctx.world.cg.predictedPlayerState.clientNum
    {
        //spectator mode, don't draw it...
        return;
    }

    CG_RegisterWeapon(ctx, weaponNum);
    let weapSlot = weaponNum as usize;
    /*
    Ghoul2 Insert Start
    */

    let mut gun = refEntity_t::zeroed();

    // only do this if we are in first person, since world weapons are now handled on the server by Ghoul2
    if !thirdPerson {
        let mut angles: vec3_t = [0.0; 3];

        // add the weapon
        _VectorCopy(parent.lightingOrigin, &mut gun.lightingOrigin);
        gun.shadowPlane = parent.shadowPlane;
        gun.renderfx = parent.renderfx;

        if ps.is_some() {
            // this player, in first person view
            gun.hModel = ctx.world.cg_weapons[weapSlot].viewModel;
        } else {
            gun.hModel = ctx.world.cg_weapons[weapSlot].weaponModel;
        }
        if gun.hModel == 0 {
            return;
        }

        if ps.is_none() {
            // add weapon ready sound
            ctx.world.entity_mut(centNum).pe.lightningFiring = qfalse;

            let eFlags = ctx.world.entity(centNum).currentState.eFlags;
            let number = ctx.world.entity(centNum).currentState.number;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let firingSound = ctx.world.cg_weapons[weapSlot].firingSound;
            let readySound = ctx.world.cg_weapons[weapSlot].readySound;

            if (eFlags & EF_FIRING) != 0 && firingSound != 0 {
                // lightning gun and guantlet make a different sound when fire is held down
                trap::S_AddLoopingSound(engine, number, &lerpOrigin, &vec3_origin, firingSound);
                ctx.world.entity_mut(centNum).pe.lightningFiring = qtrue;
            } else if readySound != 0 {
                trap::S_AddLoopingSound(engine, number, &lerpOrigin, &vec3_origin, readySound);
            }
        }

        CG_PositionEntityOnTag(ctx, &mut gun, parent, parent.hModel, "tag_weapon");

        let (trick1, trick2, trick3, trick4) = {
            let cs = &ctx.world.entity(centNum).currentState;
            (
                cs.trickedentindex,
                cs.trickedentindex2,
                cs.trickedentindex3,
                cs.trickedentindex4,
            )
        };
        // Raven reads `cg.snap->ps.clientNum` unchecked; with no snapshot there is
        // no viewer for the trick to hide from, so we take the not-tricked arm
        // and draw (§F19).
        let mindTricked = match ctx.world.cg.snap_ref().map(|snap| snap.ps.clientNum) {
            Some(clientNum) => {
                CG_IsMindTricked(ctx.world, trick1, trick2, trick3, trick4, clientNum)
            }
            None => false,
        };

        if !mindTricked {
            let powerups = ctx.world.entity(centNum).currentState.powerups;
            //don't draw the weapon if the player is invisible
            CG_AddWeaponWithPowerups(ctx, &mut gun, powerups);
            // Raven's stun-baton tint pass sits commented out right here:
            // `gun.shaderRGBA[0..2] = 25`, `gfx/effects/stunPass`,
            // `RF_RGB_TINT | RF_FIRST_PERSON | RF_DEPTHHACK`, then a second
            // `trap_R_AddRefEntityToScene`.
        }

        if weaponNum == WP_STUN_BATON {
            let mut i = 0;

            while i < 3 {
                let mut barrel = refEntity_t::zeroed();
                _VectorCopy(parent.lightingOrigin, &mut barrel.lightingOrigin);
                barrel.shadowPlane = parent.shadowPlane;
                barrel.renderfx = parent.renderfx;

                if i == 0 {
                    barrel.hModel = trap::R_RegisterModel(
                        engine,
                        "models/weapons2/stun_baton/baton_barrel.md3",
                    );
                } else if i == 1 {
                    barrel.hModel = trap::R_RegisterModel(
                        engine,
                        "models/weapons2/stun_baton/baton_barrel2.md3",
                    );
                } else {
                    barrel.hModel = trap::R_RegisterModel(
                        engine,
                        "models/weapons2/stun_baton/baton_barrel3.md3",
                    );
                }
                angles[YAW] = 0.0;
                angles[PITCH] = 0.0;
                angles[ROLL] = 0.0;

                AnglesToAxis(angles, barrel.axis.as_mut_ptr());

                let handsModel = ctx.world.cg_weapons[weapSlot].handsModel;
                if i == 0 {
                    CG_PositionRotatedEntityOnTag(
                        ctx,
                        &mut barrel,
                        parent, /*&gun*/
                        handsModel,
                        "tag_barrel",
                    );
                } else if i == 1 {
                    CG_PositionRotatedEntityOnTag(
                        ctx,
                        &mut barrel,
                        parent, /*&gun*/
                        handsModel,
                        "tag_barrel2",
                    );
                } else {
                    CG_PositionRotatedEntityOnTag(
                        ctx,
                        &mut barrel,
                        parent, /*&gun*/
                        handsModel,
                        "tag_barrel3",
                    );
                }
                let powerups = ctx.world.entity(centNum).currentState.powerups;
                CG_AddWeaponWithPowerups(ctx, &mut barrel, powerups);

                i += 1;
            }
        } else {
            // add the spinning barrel
            let barrelModel = ctx.world.cg_weapons[weapSlot].barrelModel;
            if barrelModel != 0 {
                let mut barrel = refEntity_t::zeroed();
                _VectorCopy(parent.lightingOrigin, &mut barrel.lightingOrigin);
                barrel.shadowPlane = parent.shadowPlane;
                barrel.renderfx = parent.renderfx;

                barrel.hModel = barrelModel;
                angles[YAW] = 0.0;
                angles[PITCH] = 0.0;
                angles[ROLL] = 0.0;

                AnglesToAxis(angles, barrel.axis.as_mut_ptr());

                let handsModel = ctx.world.cg_weapons[weapSlot].handsModel;
                CG_PositionRotatedEntityOnTag(
                    ctx,
                    &mut barrel,
                    parent, /*&gun*/
                    handsModel,
                    "tag_barrel",
                );

                let powerups = ctx.world.entity(centNum).currentState.powerups;
                CG_AddWeaponWithPowerups(ctx, &mut barrel, powerups);
            }
        }
    }
    /*
    Ghoul2 Insert End
    */

    let mut flash = refEntity_t::zeroed();
    CG_PositionEntityOnTag(ctx, &mut flash, &gun, gun.hModel, "tag_flash");

    _VectorCopy(flash.origin, &mut ctx.world.cg.lastFPFlashPoint);

    // Do special charge bits
    //-----------------------
    let csWeapon = ctx.world.entity(centNum).currentState.weapon;
    let modelindex2 = ctx.world.entity(centNum).currentState.modelindex2;
    let number = ctx.world.entity(centNum).currentState.number;

    if (ps.is_some()
        || ctx.world.cg.renderingThirdPerson != qfalse
        || ctx.world.cg.predictedPlayerState.clientNum != number)
        && ((modelindex2 == WEAPON_CHARGING_ALT as c_int && csWeapon == WP_BRYAR_PISTOL)
            || (modelindex2 == WEAPON_CHARGING_ALT as c_int && csWeapon == WP_BRYAR_OLD)
            || (csWeapon == WP_BOWCASTER && modelindex2 == WEAPON_CHARGING as c_int)
            || (csWeapon == WP_DEMP2 && modelindex2 == WEAPON_CHARGING_ALT as c_int))
    {
        let mut shader: c_int = 0;
        let mut val: f32 = 0.0;
        let mut scale: f32 = 1.0;
        let mut fxSArgs = addspriteArgStruct_t {
            origin: [0.0; 3],
            vel: [0.0; 3],
            accel: [0.0; 3],
            scale: 0.0,
            dscale: 0.0,
            sAlpha: 0.0,
            eAlpha: 0.0,
            rotation: 0.0,
            bounce: 0.0,
            life: 0,
            shader: 0,
            flags: 0,
        };
        let mut flashorigin: vec3_t = [0.0; 3];
        let mut flashdir: vec3_t = [0.0; 3];

        if !thirdPerson {
            _VectorCopy(flash.origin, &mut flashorigin);
            _VectorCopy(flash.axis[0], &mut flashdir);
        } else {
            let mut boltMatrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };

            // `HasGhoul2ModelOnIndex` wants the ADDRESS of the instance slot, not
            // the token — Raven passes `&(cent->ghoul2)`.
            if !trap::G2API_HasGhoul2ModelOnIndex(
                engine,
                &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
                1,
            ) {
                //it's quite possible that we may have have no weapon model and be in a valid state, so return here if this is the case
                return;
            }

            let ghoul2 = ctx.world.entity(centNum).ghoul2;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let modelScale = ctx.world.entity(centNum).modelScale;
            let time = ctx.world.cg.time;

            // go away and get me the bolt position for this frame please
            if !trap::G2API_GetBoltMatrix(
                engine,
                ghoul2,
                1,
                0,
                &mut boltMatrix,
                newAngles,
                &lerpOrigin,
                time,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            ) {
                // Couldn't find bolt point.
                return;
            }

            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::ORIGIN as c_int,
                &mut flashorigin,
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::POSITIVE_X as c_int,
                &mut flashdir,
            );
        }

        let time = ctx.world.cg.time;
        let constantLight = ctx.world.entity(centNum).currentState.constantLight;

        if csWeapon == WP_BRYAR_PISTOL || csWeapon == WP_BRYAR_OLD {
            // Hardcoded max charge time of 1 second
            val = (time - constantLight) as f32 * 0.001;
            shader = ctx.world.cgs.media.bryarFrontFlash;
        } else if csWeapon == WP_BOWCASTER {
            // Hardcoded max charge time of 1 second
            val = (time - constantLight) as f32 * 0.001;
            shader = ctx.world.cgs.media.greenFrontFlash;
        } else if csWeapon == WP_DEMP2 {
            val = (time - constantLight) as f32 * 0.001;
            shader = ctx.world.cgs.media.lightningFlash;
            scale = 1.75;
        }

        if val < 0.0 {
            val = 0.0;
        } else if val > 1.0 {
            val = 1.0;
            if ps.is_some_and(|ps| number == ps.clientNum) {
                CGCam_Shake(ctx.world, /*0.1f*/ 0.2, 100);
            }
        } else if ps.is_some_and(|ps| number == ps.clientNum) {
            CGCam_Shake(ctx.world, val * val * /*0.3f*/ 0.6, 100);
        }

        val += ctx.world.bg_state.rng.random() * 0.5;

        _VectorCopy(flashorigin, &mut fxSArgs.origin);
        VectorClear(&mut fxSArgs.vel);
        VectorClear(&mut fxSArgs.accel);
        fxSArgs.scale = 3.0 * val * scale;
        fxSArgs.dscale = 0.0;
        fxSArgs.sAlpha = 0.7;
        fxSArgs.eAlpha = 0.7;
        fxSArgs.rotation = ctx.world.bg_state.rng.random() * 360.0;
        fxSArgs.bounce = 0.0;
        // Raven writes the float `1.0f` into an `int` field, so it truncates to 1
        fxSArgs.life = 1;
        fxSArgs.shader = shader;
        fxSArgs.flags = 0x08000000;

        //FX_AddSprite( flash.origin, NULL, NULL, 3.0f * val, 0.0f, 0.7f, 0.7f, WHITE, WHITE, random() * 360, 0.0f, 1.0f, shader, FX_USE_ALPHA );
        trap::FX_AddSprite(engine, &mut fxSArgs);
    }

    // make sure we aren't looking at cg.predictedPlayerEntity for LG
    // PORT-NOTE: Raven then tests `(nonPredictedCent - cg_entities) != clientNum`,
    // which is that same index compared with itself - always false, so the
    // fall-back to `cent` is dead and the flash always reads the clientNum slot.
    let nonPredictedNum = ctx.world.entity(centNum).currentState.clientNum as usize;

    // add the flash
    let nonPredEFlags = ctx.world.entity(nonPredictedNum).currentState.eFlags;
    if weaponNum == WP_DEMP2 && (nonPredEFlags & EF_FIRING) != 0 {
        // continuous flash
    } else {
        // impulse flash
        if ctx.world.cg.time - ctx.world.entity(centNum).muzzleFlashTime > MUZZLE_FLASH_TIME {
            return;
        }
    }

    if ps.is_some()
        || ctx.world.cg.renderingThirdPerson != qfalse
        || number != ctx.world.cg.predictedPlayerState.clientNum
    {
        // Make sure we don't do the thirdperson model effects for the local player if we're in first person
        let mut flashorigin: vec3_t = [0.0; 3];
        let mut flashdir: vec3_t = [0.0; 3];
        let mut flash = refEntity_t::zeroed();

        if !thirdPerson {
            CG_PositionEntityOnTag(ctx, &mut flash, &gun, gun.hModel, "tag_flash");
            _VectorCopy(flash.origin, &mut flashorigin);
            _VectorCopy(flash.axis[0], &mut flashdir);
        } else {
            let mut boltMatrix = mdxaBone_t {
                matrix: [[0.0; 4]; 3],
            };

            if !trap::G2API_HasGhoul2ModelOnIndex(
                engine,
                &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void,
                1,
            ) {
                //it's quite possible that we may have have no weapon model and be in a valid state, so return here if this is the case
                return;
            }

            let ghoul2 = ctx.world.entity(centNum).ghoul2;
            let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;
            let modelScale = ctx.world.entity(centNum).modelScale;
            let time = ctx.world.cg.time;

            // go away and get me the bolt position for this frame please
            if !trap::G2API_GetBoltMatrix(
                engine,
                ghoul2,
                1,
                0,
                &mut boltMatrix,
                newAngles,
                &lerpOrigin,
                time,
                Some(&mut ctx.world.cgs.gameModels[0]),
                &modelScale,
            ) {
                // Couldn't find bolt point.
                return;
            }

            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::ORIGIN as c_int,
                &mut flashorigin,
            );
            BG_GiveMeVectorFromMatrix(
                &boltMatrix,
                Eorientations::POSITIVE_X as c_int,
                &mut flashdir,
            );
        }

        if ctx.world.cg.time - ctx.world.entity(centNum).muzzleFlashTime <= MUZZLE_FLASH_TIME + 10 {
            // Handle muzzle flashes
            let eFlags = ctx.world.entity(centNum).currentState.eFlags;
            if (eFlags & EF_ALT_FIRING) != 0 {
                // Check the alt firing first.
                let altMuzzleEffect = ctx.world.cg_weapons[weapSlot].altMuzzleEffect;
                if altMuzzleEffect != 0 {
                    if !thirdPerson {
                        trap::FX_PlayEntityEffectID(
                            engine,
                            altMuzzleEffect,
                            &flashorigin,
                            &flash.axis,
                            -1,
                            -1,
                            -1,
                            -1,
                        );
                    } else {
                        trap::FX_PlayEffectID(
                            engine,
                            altMuzzleEffect,
                            &flashorigin,
                            &flashdir,
                            -1,
                            -1,
                        );
                    }
                }
            } else {
                // Regular firing
                let muzzleEffect = ctx.world.cg_weapons[weapSlot].muzzleEffect;
                if muzzleEffect != 0 {
                    if !thirdPerson {
                        trap::FX_PlayEntityEffectID(
                            engine,
                            muzzleEffect,
                            &flashorigin,
                            &flash.axis,
                            -1,
                            -1,
                            -1,
                            -1,
                        );
                    } else {
                        trap::FX_PlayEffectID(
                            engine,
                            muzzleEffect,
                            &flashorigin,
                            &flashdir,
                            -1,
                            -1,
                        );
                    }
                }
            }
        }

        // add lightning bolt
        CG_LightningBolt(ctx.world.entity(nonPredictedNum), &flashorigin);

        let flashDlightColor = ctx.world.cg_weapons[weapSlot].flashDlightColor;
        if flashDlightColor[0] != 0.0 || flashDlightColor[1] != 0.0 || flashDlightColor[2] != 0.0 {
            // the radius is an int sum in C, then widened for the call
            let intensity = (300 + (ctx.world.bg_state.rng.rand() & 31)) as f32;
            trap::R_AddLightToScene(
                engine,
                &flashorigin,
                intensity,
                flashDlightColor[0],
                flashDlightColor[1],
                flashDlightColor[2],
            );
        }
    }
}

/// Raven `CG_AddViewWeapon` — the first-person gun: fov-clamped drop, the bob
/// and landing dip out of [`CG_CalculateWeaponPosition`], the torso-to-weapon
/// frame map, then everything else hung off it by [`CG_AddPlayerWeapon`].
///
/// `ps` never aliases `ctx`, so a caller handing us `cg.predictedPlayerState`
/// copies it out first.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:766-881`
pub fn CG_AddViewWeapon(ctx: &mut CgContext, ps: &playerState_t) {
    let mut angles: vec3_t = [0.0; 3];
    let mut cgFov = ctx.world.cvars.cg_fov.value;

    if cgFov < 1.0 {
        cgFov = 1.0;
    }
    if cgFov > 97.0 {
        cgFov = 97.0;
    }

    if ps.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR {
        return;
    }

    if ps.pm_type == pmtype_t::PM_INTERMISSION as c_int {
        return;
    }

    // no gun if in third person view or a camera is active
    //if ( cg.renderingThirdPerson || cg.cameraMode) {
    if ctx.world.cg.renderingThirdPerson != qfalse {
        return;
    }

    // allow the gun to be completely removed
    if ctx.world.cvars.cg_drawGun.integer == 0 || ctx.world.cg.predictedPlayerState.zoomMode != 0 {
        if (ctx.world.cg.predictedPlayerState.eFlags & EF_FIRING) != 0 {
            // special hack for lightning gun...
            let mut origin: vec3_t = [0.0; 3];
            _VectorCopy(ctx.world.cg.refdef.vieworg, &mut origin);
            let viewaxis2 = ctx.world.cg.refdef.viewaxis[2];
            _VectorMA(origin, -8.0, viewaxis2, &mut origin);
            CG_LightningBolt(ctx.world.entity(ps.clientNum as usize), &origin);
        }
        return;
    }

    // don't draw if testing a gun model
    if ctx.world.cg.testGun != qfalse {
        return;
    }

    // drop gun lower at higher fov
    // Raven's `-0.2` is a double, so the whole product widens before it lands
    // back in the float
    let fovOffset: f32 = if cgFov > 90.0 {
        (-0.2 * (cgFov - 90.0) as f64) as f32
    } else {
        0.0
    };

    let centNum = ctx.world.cg.predictedPlayerState.clientNum as usize;
    CG_RegisterWeapon(ctx, ps.weapon);
    let weapSlot = ps.weapon as usize;

    let mut hand = refEntity_t::zeroed();

    // set up gun position
    CG_CalculateWeaponPosition(ctx.world, &mut hand.origin, &mut angles);

    let viewaxis = ctx.world.cg.refdef.viewaxis;
    _VectorMA(
        hand.origin,
        ctx.world.cvars.cg_gun_x.value,
        viewaxis[0],
        &mut hand.origin,
    );
    _VectorMA(
        hand.origin,
        ctx.world.cvars.cg_gun_y.value,
        viewaxis[1],
        &mut hand.origin,
    );
    _VectorMA(
        hand.origin,
        ctx.world.cvars.cg_gun_z.value + fovOffset,
        viewaxis[2],
        &mut hand.origin,
    );

    AnglesToAxis(angles, hand.axis.as_mut_ptr());

    // map torso animations to weapon animations
    if ctx.world.cvars.cg_gun_frame.integer != 0 {
        // development tool
        hand.frame = ctx.world.cvars.cg_gun_frame.integer;
        hand.oldframe = hand.frame;
        hand.backlerp = 0.0;
    } else {
        // get clientinfo for animation map
        // Raven picks `ci` here and hands it to `CG_MapTorsoToWeaponFrame`, which
        // never reads it; all that survives is the NPC-without-clientinfo bail.
        if ctx.world.entity(centNum).currentState.eType == ET_NPC as c_int
            && ctx.world.entity(centNum).npcClient.is_none()
        {
            return;
        }

        let (torsoFrame, torsoOldFrame, torsoBacklerp, torsoAnim) = {
            let cent = ctx.world.entity(centNum);
            (
                cent.pe.torso.frame,
                cent.pe.torso.oldFrame,
                cent.pe.torso.backlerp,
                cent.currentState.torsoAnim,
            )
        };

        hand.frame = CG_MapTorsoToWeaponFrame(ctx.world, torsoFrame, torsoAnim);
        hand.oldframe = CG_MapTorsoToWeaponFrame(ctx.world, torsoOldFrame, torsoAnim);
        hand.backlerp = torsoBacklerp;

        // Handle the fringe situation where oldframe is invalid
        if hand.frame == -1 {
            hand.frame = 0;
            hand.oldframe = 0;
            hand.backlerp = 0.0;
        } else if hand.oldframe == -1 {
            hand.oldframe = hand.frame;
            hand.backlerp = 0.0;
        }
    }

    hand.hModel = ctx.world.cg_weapons[weapSlot].handsModel;
    hand.renderfx = RF_DEPTHHACK | RF_FIRST_PERSON; // | RF_MINLIGHT;

    // add everything onto the hand
    let team = ps.persistant[PERS_TEAM as usize];
    CG_AddPlayerWeapon(ctx, &hand, Some(ps), centNum, team, &angles, false);
}

/// Raven `CG_DrawIconBackground` — drives the icon HUD's open/close animation.
///
/// Every `CG_DrawPic` in Raven's body is commented out, so what is left is the
/// `cg.iconSelectTime`/`iconHUDActive`/`iconHUDPercent` bookkeeping the rest of
/// the HUD reads. Raven's now-dead `x2`/`y2`/`xAdd`/`height`/`drawType` locals
/// go with the draws.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:895-1022`
pub fn CG_DrawIconBackground(world: &mut CgWorld) {
    // invenSelectTime/forceSelectTime are floats in cg_t but weaponSelectTime
    // is an int - Raven sums in each field's own width, then floats
    let inTime = world.cg.invenSelectTime + WEAPON_SELECT_TIME as f32;
    let wpTime = (world.cg.weaponSelectTime + WEAPON_SELECT_TIME) as f32;
    let fpTime = world.cg.forceSelectTime + WEAPON_SELECT_TIME as f32;

    // don't display if dead
    // no snapshot means no health to read, so nothing to draw
    let health = world
        .cg
        .snap_ref()
        .map_or(0, |snap| snap.ps.stats[STAT_HEALTH as usize]);
    if health <= 0 {
        return;
    }

    if world.cvars.cg_hudFiles.integer != 0 {
        // simple hud
        return;
    }

    if inTime > wpTime {
        world.cg.iconSelectTime = world.cg.invenSelectTime;
    } else {
        world.cg.iconSelectTime = world.cg.weaponSelectTime as f32;
    }

    if fpTime > inTime && fpTime > wpTime {
        world.cg.iconSelectTime = world.cg.forceSelectTime;
    }

    // Time is up for the HUD to display
    if (world.cg.iconSelectTime + WEAPON_SELECT_TIME as f32) < world.cg.time as f32 {
        // The time is up, but we still need to move the prongs back to their original position
        if world.cg.iconHUDActive != qfalse {
            let t = (world.cg.time as f32 - (world.cg.iconSelectTime + WEAPON_SELECT_TIME as f32))
                as c_int;
            world.cg.iconHUDPercent = t as f32 / 130.0;
            world.cg.iconHUDPercent = 1.0 - world.cg.iconHUDPercent;

            if world.cg.iconHUDPercent < 0.0 {
                world.cg.iconHUDActive = qfalse;
                world.cg.iconHUDPercent = 0.0;
            }
        }

        return;
    }

    if world.cg.iconHUDActive == qfalse {
        // Raven's `t` is an int, so the float subtraction truncates before the divide
        let t = (world.cg.time as f32 - world.cg.iconSelectTime) as c_int;
        world.cg.iconHUDPercent = t as f32 / 130.0;

        // Calc how far into opening sequence we are
        if world.cg.iconHUDPercent > 1.0 {
            world.cg.iconHUDActive = qtrue;
            world.cg.iconHUDPercent = 1.0;
        } else if world.cg.iconHUDPercent < 0.0 {
            world.cg.iconHUDPercent = 0.0;
        }
    } else {
        world.cg.iconHUDPercent = 1.0;
    }

    // The side-prong draws that closed Raven's body are commented out too.
}

/// Raven `CG_WeaponCheck` — has the snapshot player enough ammo to fire weapon
/// `weap` either way?
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1024-1033`
pub fn CG_WeaponCheck(world: &CgWorld, weap: c_int) -> bool {
    // no snapshot means no ammo we can vouch for, so the weapon isn't usable
    let Some(snap) = world.cg.snap_ref() else {
        return false;
    };
    let wd = &weaponData[weap as usize];

    if snap.ps.ammo[wd.ammoIndex as usize] < wd.energyPerShot
        && snap.ps.ammo[wd.ammoIndex as usize] < wd.altEnergyPerShot
    {
        return false;
    }

    true
}

/// Raven `CG_WeaponSelectable` — can the weapon-select HUD land on weapon `i`?
/// Ammo, the planted-det-pack exception, and actually owning it.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1040-1066`
pub fn CG_WeaponSelectable(world: &CgWorld, i: c_int) -> bool {
    // Raven's ammo-only precheck is commented out; the real ammo test is below.
    if i == 0 {
        return false;
    }

    let ps = &world.cg.predictedPlayerState;
    let wd = &weaponData[i as usize];

    if ps.ammo[wd.ammoIndex as usize] < wd.energyPerShot
        && ps.ammo[wd.ammoIndex as usize] < wd.altEnergyPerShot
    {
        return false;
    }

    if i == WP_DET_PACK && ps.ammo[wd.ammoIndex as usize] < 1 && ps.hasDetPackPlanted == qfalse {
        return false;
    }

    if (ps.stats[STAT_WEAPONS as usize] & (1 << i)) == 0 {
        return false;
    }

    true
}

/// Raven `CG_DrawWeaponSelect` — the weapon-select HUD: up to three small icons
/// either side of the big current one, then the selected weapon's name.
///
/// The concussion rifle is walked out of numeric order on both sides (Raven's
/// "*SIGH*" hack), and `drewConc` is what stops it being drawn twice.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1076-1371`
pub fn CG_DrawWeaponSelect(ctx: &mut CgContext, ds: &DisplayState) {
    // Raven's `_XBOX` block (CL_ExtendSelectTime plus a -50 yOffset) is compiled
    // out of the PC build, so the offset stays 0.
    let yOffset: c_int = 0;

    if ctx.world.cg.predictedPlayerState.emplacedIndex != 0 {
        //can't cycle when on a weapon
        ctx.world.cg.weaponSelectTime = 0;
    }

    // Time is up for the HUD to display
    if (ctx.world.cg.weaponSelectTime + WEAPON_SELECT_TIME) < ctx.world.cg.time {
        return;
    }

    // don't display if dead
    if ctx.world.cg.predictedPlayerState.stats[STAT_HEALTH as usize] <= 0 {
        return;
    }

    // showing weapon select clears pickup item display, but not the blend blob
    ctx.world.cg.itemPickupTime = 0;

    let bits = ctx.world.cg.predictedPlayerState.stats[STAT_WEAPONS as usize];

    // count the number of weapons owned
    let mut count: c_int = 0;

    let selected = ctx.world.cg.weaponSelect;
    if !CG_WeaponSelectable(ctx.world, selected)
        && (selected == WP_THERMAL || selected == WP_TRIP_MINE)
    {
        //display this weapon that we don't actually "have" as unhighlighted until it's deselected
        //since it's selected we must increase the count to display the proper number of valid selectable weapons
        count += 1;
    }

    for i in 1..WP_NUM_WEAPONS {
        if bits & (1 << i) != 0
            && (CG_WeaponSelectable(ctx.world, i) || (i != WP_THERMAL && i != WP_TRIP_MINE))
        {
            count += 1;
        }
    }

    if count == 0 {
        // If no weapons, don't display
        return;
    }

    let sideMax: c_int = 3; // Max number of icons on the side

    // Calculate how many icons will appear to either side of the center one
    let holdCount = count - 1; // -1 for the center icon
    let (sideLeftIconCnt, sideRightIconCnt) = if holdCount == 0 {
        // No icons to either side
        (0, 0)
    } else if count > (2 * sideMax) {
        // Go to the max on each side
        (sideMax, sideMax)
    } else {
        // Less than max, so do the calc
        let left = holdCount / 2;
        (left, holdCount - left)
    };

    let mut i = if ctx.world.cg.weaponSelect == WP_CONCUSSION {
        WP_FLECHETTE
    } else {
        ctx.world.cg.weaponSelect - 1
    };
    if i < 1 {
        i = LAST_USEABLE_WEAPON;
    }

    let smallIconSize: c_int = 40;
    let bigIconSize: c_int = 80;
    let pad: c_int = 12;

    let x: c_int = 320;
    let y: c_int = 410;

    // Raven's background pass (a `.35f`-alpha white `trap_R_SetColor`) is
    // commented out right here.

    // Left side ICONS
    trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
    // Work backwards from current icon
    let mut holdX = x - ((bigIconSize / 2) + pad + smallIconSize);
    // Raven's `height` is written three times across this fn and never read -
    // dead store, dropped with its `cg.iconHUDPercent` reads.
    let mut drewConc = false;

    let mut iconCnt: c_int = 1;
    while iconCnt < (sideLeftIconCnt + 1) {
        // the labelled block is Raven's `continue`, which still runs the `i--`
        'iter: {
            if i == WP_CONCUSSION {
                i -= 1;
            } else if i == WP_FLECHETTE && !drewConc && ctx.world.cg.weaponSelect != WP_CONCUSSION {
                i = WP_CONCUSSION;
            }
            if i < 1 {
                //i = 13;
                //...don't ever do this.
                i = LAST_USEABLE_WEAPON;
            }

            if bits & (1 << i) == 0 {
                // Does he have this weapon?
                if i == WP_CONCUSSION {
                    drewConc = true;
                    i = WP_ROCKET_LAUNCHER;
                }
                break 'iter;
            }

            if !CG_WeaponSelectable(ctx.world, i) && (i == WP_THERMAL || i == WP_TRIP_MINE) {
                //Don't show thermal and tripmine when out of them
                break 'iter;
            }

            iconCnt += 1; // Good icon

            if ctx.world.cgs.media.weaponIcons[i as usize] != 0 {
                CG_RegisterWeapon(ctx, i);
                // Raven's `weaponInfo` local goes unread - the icon handles it
                // would have supplied are commented out in favour of
                // `cgs.media.weaponIcons*`.

                trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
                let icon = if !CG_WeaponCheck(ctx.world, i) {
                    ctx.world.cgs.media.weaponIcons_NA[i as usize]
                } else {
                    ctx.world.cgs.media.weaponIcons[i as usize]
                };
                CG_DrawPic(
                    ctx,
                    holdX as f32,
                    (y + 10 + yOffset) as f32,
                    smallIconSize as f32,
                    smallIconSize as f32,
                    icon,
                );

                holdX -= smallIconSize + pad;
            }
            if i == WP_CONCUSSION {
                drewConc = true;
                i = WP_ROCKET_LAUNCHER;
            }
        }
        i -= 1;
    }

    // Current Center Icon
    let center = ctx.world.cg.weaponSelect;
    if ctx.world.cgs.media.weaponIcons[center as usize] != 0 {
        CG_RegisterWeapon(ctx, center);

        trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
        let icon = if !CG_WeaponCheck(ctx.world, center) {
            ctx.world.cgs.media.weaponIcons_NA[center as usize]
        } else {
            ctx.world.cgs.media.weaponIcons[center as usize]
        };
        CG_DrawPic(
            ctx,
            (x - (bigIconSize / 2)) as f32,
            ((y - ((bigIconSize - smallIconSize) / 2)) + 10 + yOffset) as f32,
            bigIconSize as f32,
            bigIconSize as f32,
            icon,
        );
    }

    i = if ctx.world.cg.weaponSelect == WP_CONCUSSION {
        WP_ROCKET_LAUNCHER
    } else {
        ctx.world.cg.weaponSelect + 1
    };
    if i > LAST_USEABLE_WEAPON {
        i = 1;
    }

    // Right side ICONS
    // Work forwards from current icon
    holdX = x + (bigIconSize / 2) + pad;
    iconCnt = 1;
    while iconCnt < (sideRightIconCnt + 1) {
        // the labelled block is Raven's `continue`, which still runs the `i++`
        'iter: {
            if i == WP_CONCUSSION {
                i += 1;
            } else if i == WP_ROCKET_LAUNCHER
                && !drewConc
                && ctx.world.cg.weaponSelect != WP_CONCUSSION
            {
                i = WP_CONCUSSION;
            }
            if i > LAST_USEABLE_WEAPON {
                i = 1;
            }

            if bits & (1 << i) == 0 {
                // Does he have this weapon?
                if i == WP_CONCUSSION {
                    drewConc = true;
                    i = WP_FLECHETTE;
                }
                break 'iter;
            }

            if !CG_WeaponSelectable(ctx.world, i) && (i == WP_THERMAL || i == WP_TRIP_MINE) {
                //Don't show thermal and tripmine when out of them
                break 'iter;
            }

            iconCnt += 1; // Good icon

            // Raven's `weaponData[i].weaponIcon[0]` test is commented out in
            // favour of the media handle
            if ctx.world.cgs.media.weaponIcons[i as usize] != 0 {
                CG_RegisterWeapon(ctx, i);

                // No ammo for this weapon?
                trap::R_SetColor(ctx.engine, Some(&colorTable[ct_table_t::CT_WHITE as usize]));
                let icon = if !CG_WeaponCheck(ctx.world, i) {
                    ctx.world.cgs.media.weaponIcons_NA[i as usize]
                } else {
                    ctx.world.cgs.media.weaponIcons[i as usize]
                };
                CG_DrawPic(
                    ctx,
                    holdX as f32,
                    (y + 10 + yOffset) as f32,
                    smallIconSize as f32,
                    smallIconSize as f32,
                    icon,
                );

                holdX += smallIconSize + pad;
            }
            if i == WP_CONCUSSION {
                drewConc = true;
                i = WP_FLECHETTE;
            }
        }
        i += 1;
    }

    // draw the selected name
    let selectedItem = ctx.world.cg_weapons[ctx.world.cg.weaponSelect as usize].item;
    if let Some(item) = selectedItem {
        let textColor: vec4_t = [0.875, 0.718, 0.121, 1.0];
        let classname = item.item().classname;

        // Raven upper-cases a scratch copy, so the fall-back below still prints
        // the classname as it sits in the item table
        let upperKey = classname.to_ascii_uppercase();

        let key = format!("SP_INGAME_{}", upperKey);
        match trap::SP_GetStringTextString(ctx.engine, &key, 1024) {
            Some(text) => UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y + 45 + yOffset,
                &text,
                UI_CENTER | UI_SMALLFONT,
                textColor,
            ),
            None => UI_DrawProportionalString(
                ctx,
                ds,
                320,
                y + 45 + yOffset,
                classname,
                UI_CENTER | UI_SMALLFONT,
                textColor,
            ),
        }
    }

    trap::R_SetColor(ctx.engine, None);
}

/// Raven `CG_NextWeapon_f` — the `+weapnext` console command: step forward to
/// the next selectable weapon, wrapping, and keep the selection where it was if
/// nothing else is usable.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1379-1438`
pub fn CG_NextWeapon_f(ctx: &mut CgContext) {
    let engine = ctx.engine;

    // no snapshot means no weapon state to cycle - Raven's `!cg.snap` guard
    let Some((pmFlags, emplacedIndex, clientNum)) = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| (snap.ps.pm_flags, snap.ps.emplacedIndex, snap.ps.clientNum))
    else {
        return;
    };

    if pmFlags & PMF_FOLLOW != 0 {
        return;
    }

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int {
        return;
    }

    if emplacedIndex != 0 {
        return;
    }

    ctx.world.cg.weaponSelectTime = ctx.world.cg.time;
    let original = ctx.world.cg.weaponSelect;

    let mut i = 0;
    while i < WP_NUM_WEAPONS {
        //*SIGH*... Hack to put concussion rifle before rocketlauncher
        if ctx.world.cg.weaponSelect == WP_FLECHETTE {
            ctx.world.cg.weaponSelect = WP_CONCUSSION;
        } else if ctx.world.cg.weaponSelect == WP_CONCUSSION {
            ctx.world.cg.weaponSelect = WP_ROCKET_LAUNCHER;
        } else if ctx.world.cg.weaponSelect == WP_DET_PACK {
            ctx.world.cg.weaponSelect = WP_BRYAR_OLD;
        } else {
            ctx.world.cg.weaponSelect += 1;
        }
        if ctx.world.cg.weaponSelect == WP_NUM_WEAPONS {
            ctx.world.cg.weaponSelect = 0;
        }
        //	if ( cg.weaponSelect == WP_STUN_BATON ) {
        //		continue;		// never cycle to gauntlet
        //	}
        let sel = ctx.world.cg.weaponSelect;
        if CG_WeaponSelectable(ctx.world, sel) {
            break;
        }
        i += 1;
    }
    if i == WP_NUM_WEAPONS {
        ctx.world.cg.weaponSelect = original;
    } else {
        trap::S_MuteSound(engine, clientNum, CHAN_WEAPON);
    }
}

/// Raven `CG_PrevWeapon_f` — [`CG_NextWeapon_f`] backwards.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1445-1504`
pub fn CG_PrevWeapon_f(ctx: &mut CgContext) {
    let engine = ctx.engine;

    // no snapshot means no weapon state to cycle - Raven's `!cg.snap` guard
    let Some((pmFlags, emplacedIndex, clientNum)) = ctx
        .world
        .cg
        .snap_ref()
        .map(|snap| (snap.ps.pm_flags, snap.ps.emplacedIndex, snap.ps.clientNum))
    else {
        return;
    };

    if pmFlags & PMF_FOLLOW != 0 {
        return;
    }

    if ctx.world.cg.predictedPlayerState.pm_type == pmtype_t::PM_SPECTATOR as c_int {
        return;
    }

    if emplacedIndex != 0 {
        return;
    }

    ctx.world.cg.weaponSelectTime = ctx.world.cg.time;
    let original = ctx.world.cg.weaponSelect;

    let mut i = 0;
    while i < WP_NUM_WEAPONS {
        //*SIGH*... Hack to put concussion rifle before rocketlauncher
        if ctx.world.cg.weaponSelect == WP_ROCKET_LAUNCHER {
            ctx.world.cg.weaponSelect = WP_CONCUSSION;
        } else if ctx.world.cg.weaponSelect == WP_CONCUSSION {
            ctx.world.cg.weaponSelect = WP_FLECHETTE;
        } else if ctx.world.cg.weaponSelect == WP_BRYAR_OLD {
            ctx.world.cg.weaponSelect = WP_DET_PACK;
        } else {
            ctx.world.cg.weaponSelect -= 1;
        }
        if ctx.world.cg.weaponSelect == -1 {
            ctx.world.cg.weaponSelect = WP_NUM_WEAPONS - 1;
        }
        //	if ( cg.weaponSelect == WP_STUN_BATON ) {
        //		continue;		// never cycle to gauntlet
        //	}
        let sel = ctx.world.cg.weaponSelect;
        if CG_WeaponSelectable(ctx.world, sel) {
            break;
        }
        i += 1;
    }
    if i == WP_NUM_WEAPONS {
        ctx.world.cg.weaponSelect = original;
    } else {
        trap::S_MuteSound(engine, clientNum, CHAN_WEAPON);
    }
}

/// Raven `CG_Weapon_f` — the `weapon <n>` console command. `n` is the single-player
/// slot number, so the body remaps it, then cycles the thermal/mine/detpack slot
/// and finally checks we actually own the thing.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1511-1629`
pub fn CG_Weapon_f(ctx: &mut CgContext) {
    let engine = ctx.engine;

    // no snapshot means no weapon state to switch - Raven's `!cg.snap` guard
    let Some((pmFlags, emplacedIndex, psWeapon, weaponTime, statWeapons, clientNum)) =
        ctx.world.cg.snap_ref().map(|snap| {
            (
                snap.ps.pm_flags,
                snap.ps.emplacedIndex,
                snap.ps.weapon,
                snap.ps.weaponTime,
                snap.ps.stats[STAT_WEAPONS as usize],
                snap.ps.clientNum,
            )
        })
    else {
        return;
    };

    if pmFlags & PMF_FOLLOW != 0 {
        return;
    }

    if emplacedIndex != 0 {
        return;
    }

    let argv = CG_Argv(ctx, 1);
    let mut num = atoi(&argv);

    if num < 1 || num > LAST_USEABLE_WEAPON {
        return;
    }

    if num == 1 && psWeapon == WP_SABER {
        if weaponTime < 1 {
            trap::SendConsoleCommand(engine, "sv_saberswitch\n");
        }
        return;
    }

    //rww - hack to make weapon numbers same as single player
    if num > WP_STUN_BATON {
        //num++;
        num += 2; //I suppose this is getting kind of crazy, what with the wp_melee in there too now.
    } else if statWeapons & (1 << WP_SABER) != 0 {
        num = WP_SABER;
    } else {
        num = WP_MELEE;
    }

    if num > LAST_USEABLE_WEAPON + 1 {
        //other weapons are off limits due to not actually being weapon weapons
        return;
    }

    if num >= WP_THERMAL && num <= WP_DET_PACK {
        let mut i = 0;
        let mut weap = if psWeapon >= WP_THERMAL && psWeapon <= WP_DET_PACK {
            // already in cycle range so start with next cycle item
            psWeapon + 1
        } else {
            // not in cycle range, so start with thermal detonator
            WP_THERMAL
        };

        // prevent an endless loop
        while i <= 4 {
            if weap > WP_DET_PACK {
                weap = WP_THERMAL;
            }

            if CG_WeaponSelectable(ctx.world, weap) {
                num = weap;
                break;
            }

            weap += 1;
            i += 1;
        }
    }

    if !CG_WeaponSelectable(ctx.world, num) {
        return;
    }

    ctx.world.cg.weaponSelectTime = ctx.world.cg.time;

    if statWeapons & (1 << num) == 0 {
        if num == WP_SABER {
            //don't have saber, try melee on the same slot
            num = WP_MELEE;

            if statWeapons & (1 << num) == 0 {
                return;
            }
        } else {
            return; // don't have the weapon
        }
    }

    if ctx.world.cg.weaponSelect != num {
        trap::S_MuteSound(engine, clientNum, CHAN_WEAPON);
    }

    ctx.world.cg.weaponSelect = num;
}

/// Raven `CG_WeaponClean_f` — the `weaponclean <n>` console command:
/// [`CG_Weapon_f`] without the single-player slot shift, so only the stun-baton
/// slot gets remapped to saber/melee.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1634-1745`
pub fn CG_WeaponClean_f(ctx: &mut CgContext) {
    let engine = ctx.engine;

    // no snapshot means no weapon state to switch - Raven's `!cg.snap` guard
    let Some((pmFlags, emplacedIndex, psWeapon, weaponTime, statWeapons, clientNum)) =
        ctx.world.cg.snap_ref().map(|snap| {
            (
                snap.ps.pm_flags,
                snap.ps.emplacedIndex,
                snap.ps.weapon,
                snap.ps.weaponTime,
                snap.ps.stats[STAT_WEAPONS as usize],
                snap.ps.clientNum,
            )
        })
    else {
        return;
    };

    if pmFlags & PMF_FOLLOW != 0 {
        return;
    }

    if emplacedIndex != 0 {
        return;
    }

    let argv = CG_Argv(ctx, 1);
    let mut num = atoi(&argv);

    if num < 1 || num > LAST_USEABLE_WEAPON {
        return;
    }

    if num == 1 && psWeapon == WP_SABER {
        if weaponTime < 1 {
            trap::SendConsoleCommand(engine, "sv_saberswitch\n");
        }
        return;
    }

    if num == WP_STUN_BATON {
        if statWeapons & (1 << WP_SABER) != 0 {
            num = WP_SABER;
        } else {
            num = WP_MELEE;
        }
    }

    if num > LAST_USEABLE_WEAPON + 1 {
        //other weapons are off limits due to not actually being weapon weapons
        return;
    }

    if num >= WP_THERMAL && num <= WP_DET_PACK {
        let mut i = 0;
        let mut weap = if psWeapon >= WP_THERMAL && psWeapon <= WP_DET_PACK {
            // already in cycle range so start with next cycle item
            psWeapon + 1
        } else {
            // not in cycle range, so start with thermal detonator
            WP_THERMAL
        };

        // prevent an endless loop
        while i <= 4 {
            if weap > WP_DET_PACK {
                weap = WP_THERMAL;
            }

            if CG_WeaponSelectable(ctx.world, weap) {
                num = weap;
                break;
            }

            weap += 1;
            i += 1;
        }
    }

    if !CG_WeaponSelectable(ctx.world, num) {
        return;
    }

    ctx.world.cg.weaponSelectTime = ctx.world.cg.time;

    if statWeapons & (1 << num) == 0 {
        if num == WP_SABER {
            //don't have saber, try melee on the same slot
            num = WP_MELEE;

            if statWeapons & (1 << num) == 0 {
                return;
            }
        } else {
            return; // don't have the weapon
        }
    }

    if ctx.world.cg.weaponSelect != num {
        trap::S_MuteSound(engine, clientNum, CHAN_WEAPON);
    }

    ctx.world.cg.weaponSelect = num;
}

/// Raven `CG_OutOfAmmoChange` — the gun ran dry, so walk down from the top
/// useable weapon to the first selectable one that isn't the one we just had.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1756-1783`
pub fn CG_OutOfAmmoChange(ctx: &mut CgContext, oldWeapon: c_int) {
    let engine = ctx.engine;

    ctx.world.cg.weaponSelectTime = ctx.world.cg.time;

    //We don't want the emplaced or turret
    let mut i = LAST_USEABLE_WEAPON;
    while i > 0 {
        if CG_WeaponSelectable(ctx.world, i) {
            /*
            if ( 1 == cg_autoswitch.integer &&
                ( i == WP_TRIP_MINE || i == WP_DET_PACK || i == WP_THERMAL || i == WP_ROCKET_LAUNCHER) ) // safe weapon switch
            */
            //rww - Don't we want to make sure i != one of these if autoswitch is 1 (safe)?
            if ctx.world.cvars.cg_autoswitch.integer != 1
                || (i != WP_TRIP_MINE
                    && i != WP_DET_PACK
                    && i != WP_THERMAL
                    && i != WP_ROCKET_LAUNCHER)
            {
                if i != oldWeapon {
                    //don't even do anything if we're just selecting the weapon we already have/had
                    ctx.world.cg.weaponSelect = i;
                    break;
                }
            }
        }
        i -= 1;
    }

    // Raven reads `cg.snap->ps.clientNum` with no null check; with no snapshot
    // there is no client to mute, so the call is skipped (§F19).
    if let Some(clientNum) = ctx.world.cg.snap_ref().map(|snap| snap.ps.clientNum) {
        trap::S_MuteSound(engine, clientNum, CHAN_WEAPON);
    }
}

/// Raven `CG_GetClientWeaponMuzzleBoltPoint` — world position of client
/// `clIndex`'s weapon muzzle bolt. `to` is left untouched when the client has
/// no ghoul2 weapon model, exactly as Raven's early returns leave it.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1795-1815`
pub fn CG_GetClientWeaponMuzzleBoltPoint(ctx: &mut CgContext, clIndex: c_int, to: &mut vec3_t) {
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    if clIndex < 0 || clIndex >= MAX_CLIENTS_I32 {
        return;
    }

    let engine = ctx.engine;
    let idx = clIndex as usize;
    let cent = ctx.world.entity(idx);
    let ghoul2 = cent.ghoul2;
    let turAngles = cent.turAngles;
    let lerpOrigin = cent.lerpOrigin;
    let modelScale = cent.modelScale;

    // Raven's null-check on `cent` itself drops — an owned array element can't
    // be null (§B5). `HasGhoul2ModelOnIndex` takes the ADDRESS of the instance
    // slot, not the token: Raven passes `&(cent->ghoul2)` and the engine casts
    // the word to `CGhoul2Info_v **` (`cl_cgame.cpp:1434`).
    if ghoul2.is_null()
        || !trap::G2_HaveWeGhoul2Models(engine, ghoul2)
        || !trap::G2API_HasGhoul2ModelOnIndex(
            engine,
            &mut ctx.world.entity_mut(idx).ghoul2 as *mut *mut c_void,
            1,
        )
    {
        return;
    }

    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        ghoul2,
        1,
        0,
        &mut boltMatrix,
        &turAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, to);
}

/// Raven `CG_FireWeapon` — the entity just fired: latch the muzzle flash, kick
/// or shake the local player's view for the weapons that do that, and play one
/// of the up-to-four flash sounds.
///
/// Raven: "Caused by an EV_FIRE_WEAPON event".
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1824-1949`
pub fn CG_FireWeapon(ctx: &mut CgContext, centNum: usize, altFire: bool) {
    let engine = ctx.engine;

    let entWeapon = ctx.world.entity(centNum).currentState.weapon;
    if entWeapon == WP_NONE {
        return;
    }
    if entWeapon >= WP_NUM_WEAPONS {
        CG_Error(ctx, "CG_FireWeapon: ent->weapon >= WP_NUM_WEAPONS");
        return;
    }

    // mark the entity as muzzle flashing, so when it is added it will
    // append the flash to the weapon model
    let time = ctx.world.cg.time;
    ctx.world.entity_mut(centNum).muzzleFlashTime = time;

    let entNumber = ctx.world.entity(centNum).currentState.number;
    if ctx.world.cg.predictedPlayerState.clientNum == entNumber {
        if (entWeapon == WP_BRYAR_PISTOL && altFire)
            || (entWeapon == WP_BRYAR_OLD && altFire)
            || (entWeapon == WP_BOWCASTER && !altFire)
            || (entWeapon == WP_DEMP2 && altFire)
        {
            // the charge-up age is an int subtraction, then scaled as a float
            let constantLight = ctx.world.entity(centNum).currentState.constantLight;
            let mut val = (time - constantLight) as f32 * 0.001;

            if val > 3.0 {
                val = 3.0;
            }
            // Raven's `0.2` literal is a double, so C promotes `val` for this test
            if (val as f64) < 0.2 {
                val = 0.2;
            }

            val *= 2.0;

            CGCam_Shake(ctx.world, val, 250);
        } else if entWeapon == WP_ROCKET_LAUNCHER
            || (entWeapon == WP_REPEATER && altFire)
            || entWeapon == WP_FLECHETTE
            || (entWeapon == WP_CONCUSSION && !altFire)
        {
            if entWeapon == WP_CONCUSSION {
                if ctx.world.cg.renderingThirdPerson == qfalse {
                    //gives an advantage to being in 3rd person, but would look silly otherwise
                    //kick the view back
                    let kick = ctx.world.bg_state.rng.flrand(-10.0, -15.0);
                    ctx.world.cg.kick_angles[PITCH] = kick;
                    ctx.world.cg.kick_time = time;
                }
            } else if entWeapon == WP_ROCKET_LAUNCHER {
                let intensity = ctx.world.bg_state.rng.flrand(2.0, 3.0);
                CGCam_Shake(ctx.world, intensity, 350);
            } else if entWeapon == WP_REPEATER {
                let intensity = ctx.world.bg_state.rng.flrand(2.0, 3.0);
                CGCam_Shake(ctx.world, intensity, 350);
            } else if entWeapon == WP_FLECHETTE {
                if altFire {
                    let intensity = ctx.world.bg_state.rng.flrand(2.0, 3.0);
                    CGCam_Shake(ctx.world, intensity, 350);
                } else {
                    CGCam_Shake(ctx.world, 1.5, 250);
                }
            }
        }
    }
    // lightning gun only does this this on initial press
    if entWeapon == WP_DEMP2 && ctx.world.entity(centNum).pe.lightningFiring != 0 {
        return;
    }

    // Raven's "play quad sound if needed" block tests
    // `powerups & (1 << PW_QUAD)` around a commented-out `trap_S_StartSound`,
    // so the test has no body and drops with it.

    // play a sound
    let weapSlot = entWeapon as usize;
    let mut c = 0usize;
    if altFire {
        // play a sound
        while c < 4 {
            if ctx.world.cg_weapons[weapSlot].altFlashSound[c] == 0 {
                break;
            }
            c += 1;
        }
        if c > 0 {
            let pick = (ctx.world.bg_state.rng.rand() % c as c_int) as usize;
            let sfx = ctx.world.cg_weapons[weapSlot].altFlashSound[pick];
            if sfx != 0 {
                trap::S_StartSound(engine, None, entNumber, CHAN_WEAPON, sfx);
            }
        }
        //		if ( weap->altFlashSnd )
        //		{
        //			trap_S_StartSound( NULL, ent->number, CHAN_WEAPON, weap->altFlashSnd );
        //		}
    } else {
        // play a sound
        while c < 4 {
            if ctx.world.cg_weapons[weapSlot].flashSound[c] == 0 {
                break;
            }
            c += 1;
        }
        if c > 0 {
            let pick = (ctx.world.bg_state.rng.rand() % c as c_int) as usize;
            let sfx = ctx.world.cg_weapons[weapSlot].flashSound[pick];
            if sfx != 0 {
                trap::S_StartSound(engine, None, entNumber, CHAN_WEAPON, sfx);
            }
        }
    }
}

/// Raven `CG_VehicleWeaponImpact` — does this missile belong to a vehicle
/// weapon with its own impact effect? If so it plays that effect instead of the
/// generic one.
///
/// Raven: "see if this is a missile entity that's owned by a vehicle and should
/// do a special, overridden impact effect".
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1951-1964`
pub fn CG_VehicleWeaponImpact(ctx: &mut CgContext, cent: &centity_t) -> bool {
    // hack so we know we're a vehicle Weapon shot
    if (cent.currentState.eFlags & EF_JETPACK_ACTIVE) != 0
        && cent.currentState.otherEntityNum2 != 0
        && ctx.world.bg_state.g_vehWeaponInfo[cent.currentState.otherEntityNum2 as usize].iImpactFX
            != 0
    {
        // missile is from a special vehWeapon
        let mut normal: vec3_t = [0.0; 3];
        ByteToDir(cent.currentState.eventParm, &mut normal);

        let iImpactFX = ctx.world.bg_state.g_vehWeaponInfo
            [cent.currentState.otherEntityNum2 as usize]
            .iImpactFX;
        trap::FX_PlayEffectID(ctx.engine, iImpactFX, &cent.lerpOrigin, &normal, -1, -1);
        return true;
    }
    false
}

/// Raven `CG_MissileHitWall` — the per-weapon wall-impact effect.
///
/// Raven: "Caused by an EV_MISSILE_MISS event, or directly by local
/// bullet tracing."
///
/// `clientNum` and `soundType` are Raven's unused parameters (the impact-sound
/// switch is gone; every arm plays an effect instead).
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:1973-2073`
#[allow(clippy::too_many_arguments)]
pub fn CG_MissileHitWall(
    ctx: &mut CgContext,
    weapon: c_int,
    _clientNum: c_int,
    origin: &vec3_t,
    dir: &vec3_t,
    _soundType: impactSound_t,
    altFire: bool,
    charge: c_int,
) {
    let up: vec3_t = [0.0, 0.0, 1.0];

    match weapon {
        WP_BRYAR_PISTOL => {
            if altFire {
                let parm = charge;
                FX_BryarAltHitWall(ctx, origin, dir, parm);
            } else {
                FX_BryarHitWall(ctx, origin, dir);
            }
        }

        WP_CONCUSSION => FX_ConcussionHitWall(ctx, origin, dir),

        WP_BRYAR_OLD => {
            if altFire {
                let parm = charge;
                FX_BryarAltHitWall(ctx, origin, dir, parm);
            } else {
                FX_BryarHitWall(ctx, origin, dir);
            }
        }

        WP_TURRET => FX_TurretHitWall(ctx, origin, dir),

        WP_BLASTER => FX_BlasterWeaponHitWall(ctx, origin, dir),

        WP_DISRUPTOR => FX_DisruptorAltMiss(ctx, origin, dir),

        WP_BOWCASTER => FX_BowcasterHitWall(ctx, origin, dir),

        WP_REPEATER => {
            if altFire {
                FX_RepeaterAltHitWall(ctx, origin, dir);
            } else {
                FX_RepeaterHitWall(ctx, origin, dir);
            }
        }

        WP_DEMP2 => {
            if altFire {
                let effect = ctx.world.cgs.effects.mAltDetonate;
                trap::FX_PlayEffectID(ctx.engine, effect, origin, dir, -1, -1);
            } else {
                FX_DEMP2_HitWall(ctx, origin, dir);
            }
        }

        WP_FLECHETTE => {
            /*if (altFire)
            {
                CG_SurfaceExplosion(origin, dir, 20.0f, 12.0f, qtrue);
            }
            else
            */
            if !altFire {
                FX_FlechetteWeaponHitWall(ctx, origin, dir);
            }
        }

        WP_ROCKET_LAUNCHER => FX_RocketHitWall(ctx, origin, dir),

        WP_THERMAL => {
            let explosion = ctx.world.cgs.effects.thermalExplosionEffect;
            trap::FX_PlayEffectID(ctx.engine, explosion, origin, dir, -1, -1);
            let shockwave = ctx.world.cgs.effects.thermalShockwaveEffect;
            trap::FX_PlayEffectID(ctx.engine, shockwave, origin, &up, -1, -1);
        }

        WP_EMPLACED_GUN => {
            FX_BlasterWeaponHitWall(ctx, origin, dir);
            //FIXME: Give it its own hit wall effect
        }

        // Raven's switch has no default arm, so every other weapon is silent.
        _ => {}
    }
}

/// Raven `CG_MissileHitPlayer` — the per-weapon flesh-impact effect.
///
/// Raven's `humanoid` local is set once and never cleared (the single-player
/// droid test above it is commented out as "Non-portable code from single
/// player"), so every call takes the humanoid effect. `entityNum` goes unused
/// with the commented-out `CG_Bleed`.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2081-2198`
pub fn CG_MissileHitPlayer(
    ctx: &mut CgContext,
    weapon: c_int,
    origin: &vec3_t,
    dir: &vec3_t,
    _entityNum: c_int,
    altFire: bool,
) {
    let humanoid = true;
    let up: vec3_t = [0.0, 0.0, 1.0];

    // NOTENOTE No bleeding in this game
    //	CG_Bleed( origin, entityNum );

    // some weapons will make an explosion with the blood, while
    // others will just make the blood
    match weapon {
        WP_BRYAR_PISTOL => {
            if altFire {
                FX_BryarAltHitPlayer(ctx, origin, dir, humanoid);
            } else {
                FX_BryarHitPlayer(ctx, origin, dir, humanoid);
            }
        }

        WP_CONCUSSION => FX_ConcussionHitPlayer(ctx, origin, dir, humanoid),

        WP_BRYAR_OLD => {
            if altFire {
                FX_BryarAltHitPlayer(ctx, origin, dir, humanoid);
            } else {
                FX_BryarHitPlayer(ctx, origin, dir, humanoid);
            }
        }

        WP_TURRET => FX_TurretHitPlayer(ctx, origin, dir, humanoid),

        WP_BLASTER => FX_BlasterWeaponHitPlayer(ctx, origin, dir, humanoid),

        WP_DISRUPTOR => FX_DisruptorAltHit(ctx, origin, dir),

        WP_BOWCASTER => FX_BowcasterHitPlayer(ctx, origin, dir, humanoid),

        WP_REPEATER => {
            if altFire {
                FX_RepeaterAltHitPlayer(ctx, origin, dir, humanoid);
            } else {
                FX_RepeaterHitPlayer(ctx, origin, dir, humanoid);
            }
        }

        WP_DEMP2 => {
            // Do a full body effect here for some more feedback
            // NOTENOTE The chaining of the demp2 is not yet implemented.
            if altFire {
                let effect = ctx.world.cgs.effects.mAltDetonate;
                trap::FX_PlayEffectID(ctx.engine, effect, origin, dir, -1, -1);
            } else {
                FX_DEMP2_HitPlayer(ctx, origin, dir, humanoid);
            }
        }

        WP_FLECHETTE => FX_FlechetteWeaponHitPlayer(ctx, origin, dir, humanoid),

        WP_ROCKET_LAUNCHER => FX_RocketHitPlayer(ctx, origin, dir, humanoid),

        WP_THERMAL => {
            let explosion = ctx.world.cgs.effects.thermalExplosionEffect;
            trap::FX_PlayEffectID(ctx.engine, explosion, origin, dir, -1, -1);
            let shockwave = ctx.world.cgs.effects.thermalShockwaveEffect;
            trap::FX_PlayEffectID(ctx.engine, shockwave, origin, &up, -1, -1);
        }

        WP_EMPLACED_GUN => {
            //FIXME: Its own effect?
            FX_BlasterWeaponHitPlayer(ctx, origin, dir, humanoid);
        }

        _ => {}
    }
}

/// Raven `CG_CalcMuzzlePoint` — where entity `entityNum`'s shots come out of,
/// for the crosshair trace. The local player gets the full viewmodel treatment
/// (per-weapon muzzle offset, third-person vs first-person, the emplaced-gun
/// override); everyone else gets their trajectory base plus a viewheight.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2215-2313`
pub fn CG_CalcMuzzlePoint(world: &CgWorld, entityNum: c_int, muzzle: &mut vec3_t) -> bool {
    let mut forward: vec3_t = [0.0; 3];
    let mut right: vec3_t = [0.0; 3];
    let mut gunpoint: vec3_t = [0.0; 3];

    // no snapshot means nothing can match our clientNum, so we fall through to
    // the generic path rather than reading a state we don't have
    let snapPs = world
        .cg
        .snap_ref()
        .map(|snap| &snap.ps)
        .filter(|ps| ps.clientNum == entityNum);

    if let Some(ps) = snapPs {
        // I'm not exactly sure why we'd be rendering someone else's crosshair, but hey.
        let weapontype = ps.weapon;
        let mut weaponMuzzle: vec3_t = [0.0; 3];
        let pEnt = world.entity(world.cg.predictedPlayerState.clientNum as usize);

        _VectorCopy(WP_MuzzlePoint[weapontype as usize], &mut weaponMuzzle);

        if weapontype == WP_DISRUPTOR
            || weapontype == WP_STUN_BATON
            || weapontype == WP_MELEE
            || weapontype == WP_SABER
        {
            VectorClear(&mut weaponMuzzle);
        }

        if world.cg.renderingThirdPerson != qfalse {
            _VectorCopy(pEnt.lerpOrigin, &mut gunpoint);
            AngleVectors(pEnt.lerpAngles, Some(&mut forward), Some(&mut right), None);
        } else {
            _VectorCopy(world.cg.refdef.vieworg, &mut gunpoint);
            AngleVectors(
                world.cg.refdef.viewangles,
                Some(&mut forward),
                Some(&mut right),
                None,
            );
        }

        if weapontype == WP_EMPLACED_GUN && ps.emplacedIndex != 0 {
            // Raven null-checks `gunEnt` here; an owned array element is never
            // null, so the check drops and the block runs unconditionally.
            let gunEnt = world.entity(ps.emplacedIndex as usize);
            let mut pitchConstraint: vec3_t = [0.0; 3];

            _VectorCopy(gunEnt.lerpOrigin, &mut gunpoint);
            gunpoint[2] += 46.0;

            if world.cg.renderingThirdPerson != qfalse {
                _VectorCopy(pEnt.lerpAngles, &mut pitchConstraint);
            } else {
                _VectorCopy(world.cg.refdef.viewangles, &mut pitchConstraint);
            }

            if pitchConstraint[PITCH] > 40.0 {
                pitchConstraint[PITCH] = 40.0;
            }
            AngleVectors(pitchConstraint, Some(&mut forward), Some(&mut right), None);
        }

        _VectorCopy(gunpoint, muzzle);

        _VectorMA(*muzzle, weaponMuzzle[0], forward, muzzle);
        _VectorMA(*muzzle, weaponMuzzle[1], right, muzzle);

        if weapontype == WP_EMPLACED_GUN && ps.emplacedIndex != 0 {
            // Do nothing
        } else if world.cg.renderingThirdPerson != qfalse {
            muzzle[2] += ps.viewheight as f32 + weaponMuzzle[2];
        } else {
            muzzle[2] += weaponMuzzle[2];
        }

        return true;
    }

    let cent = world.entity(entityNum as usize);
    if cent.currentValid == qfalse {
        return false;
    }

    _VectorCopy(cent.currentState.pos.trBase, muzzle);

    AngleVectors(
        cent.currentState.apos.trBase,
        Some(&mut forward),
        None,
        None,
    );
    let anim = cent.currentState.legsAnim;
    if anim == animNumber_t::BOTH_CROUCH1WALK as c_int
        || anim == animNumber_t::BOTH_CROUCH1IDLE as c_int
    {
        muzzle[2] += CROUCH_VIEWHEIGHT as f32;
    } else {
        muzzle[2] += DEFAULT_VIEWHEIGHT as f32;
    }

    _VectorMA(*muzzle, 14.0, forward, muzzle);

    true
}

/// Raven `CG_InitG2Weapons` — builds the one shared ghoul2 instance per weapon.
///
/// Raven: "create one instance of all the weapons we are going to use so we can
/// just copy this info into each clients gun ghoul2 object in fast way".
///
/// Raven walks `bg_itemlist + 1` to the `NULL`-classname sentinel; our
/// `bg_itemlist` dropped that sentinel, so the walk is `[1..]`.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2324-2361`
pub fn CG_InitG2Weapons(ctx: &mut CgContext) {
    let engine = ctx.engine;
    let mut i: usize = 0;

    ctx.world.weapons.g2WeaponInstances = [null_mut(); MAX_WEAPONS];

    for item in bg_itemlist[1..].iter() {
        let ItemKind::Weapon(giTag) = item.kind else {
            continue;
        };
        debug_assert!(giTag < MAX_WEAPONS as c_int);
        let slot = giTag as usize;

        // initialise model
        trap::G2API_InitGhoul2Model(
            engine,
            &mut ctx.world.weapons.g2WeaponInstances[slot] as *mut *mut c_void,
            item.world_model[0],
            0,
            0,
            0,
            0,
            0,
        );

        let instance = ctx.world.weapons.g2WeaponInstances[slot];
        if !instance.is_null() {
            // indicate we will be bolted to model 0 (ie the player) on bolt 0 (always the right hand) when we get copied
            trap::G2API_SetBoltInfo(engine, instance, 0, 0);
            // now set up the gun bolt on it
            if giTag == WP_SABER {
                trap::G2API_AddBolt(engine, instance, 0, "*blade1");
            } else {
                trap::G2API_AddBolt(engine, instance, 0, "*flash");
            }
            i += 1;
        }

        if i == MAX_WEAPONS {
            debug_assert!(false, "CG_InitG2Weapons ran out of weapon slots");
            break;
        }
    }
}

/// Raven `CG_ShutDownG2Weapons` — frees every shared weapon instance.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2364-2371`
pub fn CG_ShutDownG2Weapons(ctx: &mut CgContext) {
    let engine = ctx.engine;
    for i in 0..MAX_WEAPONS {
        trap::G2API_CleanGhoul2Models(
            engine,
            &mut ctx.world.weapons.g2WeaponInstances[i] as *mut *mut c_void,
        );
    }
}

/// Raven `CG_G2WeaponInstance` — which ghoul2 weapon instance to copy for this
/// entity: a player's custom saber hilt when there is one, otherwise the shared
/// per-weapon instance.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2373-2411`
pub fn CG_G2WeaponInstance(world: &CgWorld, cent: &centity_t, weapon: c_int) -> *mut c_void {
    let shared = world.weapons.g2WeaponInstances[weapon as usize];

    if weapon != WP_SABER {
        return shared;
    }

    if cent.currentState.eType != ET_PLAYER as c_int && cent.currentState.eType != ET_NPC as c_int {
        return shared;
    }

    let ci: Option<&clientInfo_t> = if cent.currentState.eType == ET_NPC as c_int {
        cent.npcClient.as_deref()
    } else {
        Some(&world.cgs.clientinfo[cent.currentState.number as usize])
    };

    let Some(ci) = ci else {
        return shared;
    };

    // Try to return the custom saber instance if we can.
    if ci.saber[0].model[0] != 0 && !ci.ghoul2Weapons[0].is_null() {
        return ci.ghoul2Weapons[0];
    }

    // If no custom then just use the default.
    shared
}

/// Raven `CG_CopyG2WeaponInstance` — stamps the shared weapon instance (or the
/// player's own saber hilts) onto `toGhoul2`'s model slot 1, and strips the
/// second-saber/gun models when we are switching away from sabers.
///
/// PORT-NOTE: Raven passes `&(toGhoul2)` — the address of its own parameter —
/// wherever the engine wants an instance slot, so a remove/clean can only
/// replace this function's local copy; the caller's handle never sees it. The
/// `mut` local below reproduces that exactly.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2414-2492`
pub fn CG_CopyG2WeaponInstance(
    ctx: &mut CgContext,
    centNum: usize,
    weaponNum: c_int,
    toGhoul2: *mut c_void,
) {
    let engine = ctx.engine;
    let mut toGhoul2 = toGhoul2;

    //rww - the -1 is because there is no "weapon" for WP_NONE
    debug_assert!(weaponNum < MAX_WEAPONS as c_int);

    // Raven re-calls `CG_G2WeaponInstance` inside the branches; nothing between
    // here and those uses touches what it reads, so one call covers both.
    let world = &*ctx.world;
    let from = CG_G2WeaponInstance(world, world.entity(centNum), weaponNum);
    if from.is_null() {
        return;
    }

    if weaponNum == WP_SABER {
        let isNpc = ctx.world.entity(centNum).currentState.eType == ET_NPC as c_int;
        let number = ctx.world.entity(centNum).currentState.number as usize;
        // Raven's `if (!ci)` can only fire on the NPC path - the clientinfo slot
        // is an array element and never NULL.
        let noCi = isNpc && ctx.world.entity(centNum).npcClient.is_none();

        if noCi {
            trap::G2API_CopySpecificGhoul2Model(engine, from, 0, toGhoul2, 1);
        } else {
            //Try both the left hand saber and the right hand saber
            for i in 0..MAX_SABERS {
                let (hasModel, weapInst) = if isNpc {
                    match ctx.world.entity(centNum).npcClient.as_deref() {
                        Some(ci) => (ci.saber[i].model[0] != 0, ci.ghoul2Weapons[i]),
                        None => (false, null_mut()),
                    }
                } else {
                    let ci = &ctx.world.cgs.clientinfo[number];
                    (ci.saber[i].model[0] != 0, ci.ghoul2Weapons[i])
                };

                if hasModel && !weapInst.is_null() {
                    trap::G2API_CopySpecificGhoul2Model(
                        engine,
                        weapInst,
                        0,
                        toGhoul2,
                        i as c_int + 1,
                    );
                } else if !weapInst.is_null() {
                    //if the second saber has been removed, then be sure to remove it and free the instance.
                    let g2HasSecondSaber = trap::G2API_HasGhoul2ModelOnIndex(
                        engine,
                        &mut toGhoul2 as *mut *mut c_void,
                        2,
                    );

                    if g2HasSecondSaber {
                        //remove it now since we're switching away from sabers
                        trap::G2API_RemoveGhoul2Model(engine, &mut toGhoul2 as *mut *mut c_void, 2);
                    }
                    if isNpc {
                        if let Some(ci) = ctx.world.entity_mut(centNum).npcClient.as_deref_mut() {
                            trap::G2API_CleanGhoul2Models(
                                engine,
                                &mut ci.ghoul2Weapons[i] as *mut *mut c_void,
                            );
                        }
                    } else {
                        trap::G2API_CleanGhoul2Models(
                            engine,
                            &mut ctx.world.cgs.clientinfo[number].ghoul2Weapons[i]
                                as *mut *mut c_void,
                        );
                    }
                }
            }
        }
    } else {
        let g2HasSecondSaber =
            trap::G2API_HasGhoul2ModelOnIndex(engine, &mut toGhoul2 as *mut *mut c_void, 2);

        if g2HasSecondSaber {
            //remove it now since we're switching away from sabers
            trap::G2API_RemoveGhoul2Model(engine, &mut toGhoul2 as *mut *mut c_void, 2);
        }

        if weaponNum == WP_EMPLACED_GUN {
            //a bit of a hack to remove gun model when using an emplaced weap
            if trap::G2API_HasGhoul2ModelOnIndex(engine, &mut toGhoul2 as *mut *mut c_void, 1) {
                trap::G2API_RemoveGhoul2Model(engine, &mut toGhoul2 as *mut *mut c_void, 1);
            }
        } else if weaponNum == WP_MELEE {
            //don't want a weapon on the model for this one
            if trap::G2API_HasGhoul2ModelOnIndex(engine, &mut toGhoul2 as *mut *mut c_void, 1) {
                trap::G2API_RemoveGhoul2Model(engine, &mut toGhoul2 as *mut *mut c_void, 1);
            }
        } else {
            trap::G2API_CopySpecificGhoul2Model(engine, from, 0, toGhoul2, 1);
        }
    }
}

/// Raven `CG_CheckPlayerG2Weapons` — keeps the player's ghoul2 gun model in
/// step with the weapon the playerstate says they are holding, and plays the
/// saber on/off sounds across the switch.
///
/// Raven's `if (!ps) { assert(0); return; }` guard drops — a `&playerState_t`
/// is never null. `ps` never aliases `ctx`, so a caller handing us
/// `cg.predictedPlayerState` copies it out first.
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:2494-2577`
pub fn CG_CheckPlayerG2Weapons(ctx: &mut CgContext, ps: &playerState_t, centNum: usize) {
    let engine = ctx.engine;

    if ps.pm_flags & PMF_FOLLOW != 0 {
        return;
    }

    if ctx.world.entity(centNum).currentState.eType == ET_NPC as c_int {
        debug_assert!(false, "CG_CheckPlayerG2Weapons called on an ET_NPC");
        return;
    }

    // should we change the gun model on this player?
    if ctx.world.entity(centNum).currentState.saberInFlight != qfalse {
        let world = &*ctx.world;
        let saberInstance = CG_G2WeaponInstance(world, world.entity(centNum), WP_SABER);
        ctx.world.entity_mut(centNum).ghoul2weapon = saberInstance;
    }

    if (ctx.world.entity(centNum).currentState.eFlags & EF_DEAD) != 0 {
        //no updating weapons when dead
        ctx.world.entity_mut(centNum).ghoul2weapon = null_mut();
        return;
    }

    if ctx.world.entity(centNum).torsoBolt != 0 {
        //got our limb cut off, no updating weapons until it's restored
        ctx.world.entity_mut(centNum).ghoul2weapon = null_mut();
        return;
    }

    let psClient = ps.clientNum as usize;
    if ctx.world.cgs.clientinfo[psClient].team == TEAM_SPECTATOR
        || ps.persistant[PERS_TEAM as usize] == TEAM_SPECTATOR
    {
        ctx.world.entity_mut(psClient).ghoul2weapon = null_mut();
        ctx.world.entity_mut(centNum).ghoul2weapon = null_mut();
        ctx.world.entity_mut(psClient).weapon = 0;
        ctx.world.entity_mut(centNum).weapon = 0;
        return;
    }

    let ghoul2 = ctx.world.entity(centNum).ghoul2;
    let ghoul2weapon = ctx.world.entity(centNum).ghoul2weapon;
    let number = ctx.world.entity(centNum).currentState.number;
    let wanted = {
        let world = &*ctx.world;
        CG_G2WeaponInstance(world, world.entity(centNum), ps.weapon)
    };

    //don't want spectator mode forcing one client's weapon instance over another's
    if !ghoul2.is_null() && ghoul2weapon != wanted && ps.clientNum == number {
        CG_CopyG2WeaponInstance(ctx, centNum, ps.weapon, ghoul2);
        // the copy can free a saber instance, so re-ask which one we ended up on
        let nowWanted = {
            let world = &*ctx.world;
            CG_G2WeaponInstance(world, world.entity(centNum), ps.weapon)
        };
        ctx.world.entity_mut(centNum).ghoul2weapon = nowWanted;

        let centWeapon = ctx.world.entity(centNum).weapon;
        let lerpOrigin = ctx.world.entity(centNum).lerpOrigin;

        if centWeapon == WP_SABER && centWeapon != ps.weapon && ps.saberHolstered == 0 {
            //switching away from the saber
            //trap_S_StartSound(cent->lerpOrigin, cent->currentState.number, CHAN_AUTO, trap_S_RegisterSound( "sound/weapons/saber/saberoffquick.wav" ));
            let soundOff0 = ctx.world.cgs.clientinfo[psClient].saber[0].soundOff;
            if soundOff0 != 0 && ps.saberHolstered == 0 {
                trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, soundOff0);
            }

            let soundOff1 = ctx.world.cgs.clientinfo[psClient].saber[1].soundOff;
            let hasModel1 = ctx.world.cgs.clientinfo[psClient].saber[1].model[0] != 0;
            if soundOff1 != 0 && hasModel1 && ps.saberHolstered == 0 {
                trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, soundOff1);
            }
        } else if ps.weapon == WP_SABER
            && centWeapon != ps.weapon
            && ctx.world.entity(centNum).saberWasInFlight == qfalse
        {
            //switching to the saber
            //trap_S_StartSound(cent->lerpOrigin, cent->currentState.number, CHAN_AUTO, trap_S_RegisterSound( "sound/weapons/saber/saberon.wav" ));
            let soundOn0 = ctx.world.cgs.clientinfo[psClient].saber[0].soundOn;
            if soundOn0 != 0 {
                trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, soundOn0);
            }

            let soundOn1 = ctx.world.cgs.clientinfo[psClient].saber[1].soundOn;
            if soundOn1 != 0 {
                trap::S_StartSound(engine, Some(&lerpOrigin), number, CHAN_AUTO, soundOn1);
            }

            BG_SI_SetDesiredLength(
                &mut ctx.world.cgs.clientinfo[psClient].saber[0] as *mut saberInfo_t,
                0.0,
                -1,
            );
            BG_SI_SetDesiredLength(
                &mut ctx.world.cgs.clientinfo[psClient].saber[1] as *mut saberInfo_t,
                0.0,
                -1,
            );
        }
        ctx.world.entity_mut(centNum).weapon = ps.weapon;
    }
}
