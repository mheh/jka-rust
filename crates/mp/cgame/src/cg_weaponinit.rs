//! Port of `oracle/codemp/cgame/cg_weaponinit.c` — per-weapon registration data. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::c_int;

use mp_bg::public::bg_itemlist::bg_itemlist;
use mp_bg::public::item_id::ItemId;
use mp_bg::public::item_kind::ItemKind;
use mp_bg::weapons::weapon_t::{
    WP_BLASTER, WP_BOWCASTER, WP_BRYAR_OLD, WP_BRYAR_PISTOL, WP_CONCUSSION, WP_DEMP2, WP_DET_PACK,
    WP_DISRUPTOR, WP_EMPLACED_GUN, WP_FLECHETTE, WP_MELEE, WP_REPEATER, WP_ROCKET_LAUNCHER,
    WP_SABER, WP_STUN_BATON, WP_THERMAL, WP_TRIP_MINE, WP_TURRET,
};
use mp_qshared::shared::q_math::VectorSet;
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{fxHandle_t, qfalse, qhandle_t, qtrue, sfxHandle_t, vec3_t};

use crate::cg_main::CG_Error;
use crate::cg_weapons::CG_RegisterItemVisuals;
use crate::local::trail_fn::TrailFn;
use crate::local::weapon_info_s::weaponInfo_t;
use crate::trap;
use crate::world::cg_context::CgContext;

// ---------------------------------------------------------------------------
// FILE-SCOPE CONSTANTS
// `cg_local.h`'s three null-handle spellings have no ported cross-crate home
// yet, so they land beside their reader — the treatment `cg_weapons.rs` gave
// `LAND_DEFLECT_TIME`.
// ---------------------------------------------------------------------------

/// Raven `NULL_HANDLE`.
/// Source: `oracle/codemp/cgame/cg_local.h:19`
pub const NULL_HANDLE: qhandle_t = 0;

/// Raven `NULL_SOUND`.
/// Source: `oracle/codemp/cgame/cg_local.h:20`
pub const NULL_SOUND: sfxHandle_t = 0;

/// Raven `NULL_FX`.
/// Source: `oracle/codemp/cgame/cg_local.h:21`
pub const NULL_FX: fxHandle_t = 0;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Raven `CG_RegisterWeapon` — loads every model, icon, sound and effect one
/// weapon needs, once.
///
/// The sixteen `missileTrailFunc`/`altMissileTrailFunc` stores are held: see
/// the DEFERRED note on the first one. Every other line transcribes in Raven's
/// exact registration order, which is the syscall order the referee compares.
///
/// Source: `oracle/codemp/cgame/cg_weaponinit.c:14-592`
pub fn CG_RegisterWeapon(ctx: &mut CgContext, weaponNum: c_int) {
    let engine = ctx.engine;
    let slot = weaponNum as usize;

    if weaponNum == 0 {
        return;
    }

    if ctx.world.cg_weapons[slot].registered != qfalse {
        return;
    }

    // Raven's `memset( weaponInfo, 0, sizeof( *weaponInfo ) )` — a whole fresh
    // record, spelled out because `weaponInfo_t` has no `zeroed()`.
    ctx.world.cg_weapons[slot] = weaponInfo_t {
        registered: qfalse,
        item: None,
        handsModel: 0,
        weaponModel: 0,
        viewModel: 0,
        barrelModel: 0,
        flashModel: 0,
        weaponMidpoint: [0.0; 3],
        flashDlight: 0.0,
        flashDlightColor: [0.0; 3],
        weaponIcon: 0,
        ammoIcon: 0,
        ammoModel: 0,
        flashSound: [0; 4],
        firingSound: 0,
        chargeSound: 0,
        muzzleEffect: 0,
        missileModel: 0,
        missileSound: 0,
        missileTrailFunc: TrailFn::None,
        missileDlight: 0.0,
        missileDlightColor: [0.0; 3],
        missileRenderfx: 0,
        missileHitSound: 0,
        altFlashSound: [0; 4],
        altFiringSound: 0,
        altChargeSound: 0,
        altMuzzleEffect: 0,
        altMissileModel: 0,
        altMissileSound: 0,
        altMissileTrailFunc: TrailFn::None,
        altMissileDlight: 0.0,
        altMissileDlightColor: [0.0; 3],
        altMissileRenderfx: 0,
        altMissileHitSound: 0,
        selectSound: 0,
        readySound: 0,
        trailRadius: 0.0,
        wiTrailTime: 0.0,
    };
    ctx.world.cg_weapons[slot].registered = qtrue;

    // Raven walks `bg_itemlist + 1` to the NULL-classname sentinel; our
    // `bg_itemlist` dropped that sentinel, so the walk is `[1..]` and running
    // off the end is Raven's `!item->classname`.
    let mut itemNum: Option<usize> = None;
    for (i, entry) in bg_itemlist.iter().enumerate().skip(1) {
        if entry.kind == ItemKind::Weapon(weaponNum) {
            ctx.world.cg_weapons[slot].item = ItemId::from_modelindex(i as c_int);
            itemNum = Some(i);
            break;
        }
    }
    let Some(itemNum) = itemNum else {
        // Raven's `CG_Error` longjmps out of the module, so nothing below runs;
        // the explicit return is that non-return spelled in Rust.
        CG_Error(ctx, &format!("Couldn't find weapon {}", weaponNum));
        return;
    };

    CG_RegisterItemVisuals(ctx, itemNum as c_int);

    let world = &mut *ctx.world;
    let weaponInfo = &mut world.cg_weapons[slot];
    let item = &bg_itemlist[itemNum];

    // Raven hands `gitem_t::view_model` (a `char *`) straight to the traps and
    // to `strcpy`; ours is `Option<&str>`, and a weapon item with no view model
    // registers the empty name rather than dereferencing NULL.
    let view_model = item.view_model.unwrap_or("");

    // load cmodel before model so filecache works
    weaponInfo.weaponModel = trap::R_RegisterModel(engine, item.world_model[0]);
    // load in-view model also
    weaponInfo.viewModel = trap::R_RegisterModel(engine, view_model);

    // calc midpoint for rotation
    let mut mins: vec3_t = [0.0; 3];
    let mut maxs: vec3_t = [0.0; 3];
    trap::R_ModelBounds(engine, weaponInfo.weaponModel, &mut mins, &mut maxs);
    for i in 0..3 {
        weaponInfo.weaponMidpoint[i] = (mins[i] as f64 + 0.5 * (maxs[i] - mins[i]) as f64) as f32;
    }

    // Raven's `item->icon` is a `char *` too — same empty-name treatment.
    let icon = item.icon.unwrap_or("");
    weaponInfo.weaponIcon = trap::R_RegisterShader(engine, icon);
    weaponInfo.ammoIcon = trap::R_RegisterShader(engine, icon);

    let mut ammo = None;
    for entry in bg_itemlist[1..].iter() {
        if entry.kind == ItemKind::Ammo(weaponNum) {
            ammo = Some(entry);
            break;
        }
    }
    // Raven's paired `ammo->classname && ammo->world_model[0]` test: the found
    // half is the `Some`, the model half is a non-empty model list.
    if let Some(ammo) = ammo {
        if !ammo.world_model.is_empty() {
            weaponInfo.ammoModel = trap::R_RegisterModel(engine, ammo.world_model[0]);
        }
    }

    //	strcpy( path, item->view_model );
    //	COM_StripExtension( path, path );
    //	strcat( path, "_flash.md3" );
    weaponInfo.flashModel = 0; //trap_R_RegisterModel( path );

    if weaponNum == WP_DISRUPTOR
        || weaponNum == WP_FLECHETTE
        || weaponNum == WP_REPEATER
        || weaponNum == WP_ROCKET_LAUNCHER
    {
        let path = format!("{}_barrel.md3", COM_StripExtension(view_model));
        weaponInfo.barrelModel = trap::R_RegisterModel(engine, &path);
    } else if weaponNum == WP_STUN_BATON {
        //only weapon with more than 1 barrel..
        trap::R_RegisterModel(engine, "models/weapons2/stun_baton/baton_barrel.md3");
        trap::R_RegisterModel(engine, "models/weapons2/stun_baton/baton_barrel2.md3");
        trap::R_RegisterModel(engine, "models/weapons2/stun_baton/baton_barrel3.md3");
    } else {
        weaponInfo.barrelModel = 0;
    }

    if weaponNum != WP_SABER {
        let path = format!("{}_hand.md3", COM_StripExtension(view_model));
        weaponInfo.handsModel = trap::R_RegisterModel(engine, &path);
    } else {
        weaponInfo.handsModel = 0;
    }

    //	if ( !weaponInfo->handsModel ) {
    //		weaponInfo->handsModel = trap_R_RegisterModel( "models/weapons2/shotgun/shotgun_hand.md3" );
    //	}

    match weaponNum {
        WP_STUN_BATON | WP_MELEE => {
            /*		MAKERGB( weaponInfo->flashDlightColor, 0.6f, 0.6f, 1.0f );
                    weaponInfo->firingSound = trap_S_RegisterSound( "sound/weapons/saber/saberhum.wav" );
            //		weaponInfo->flashSound[0] = trap_S_RegisterSound( "sound/weapons/melee/fstatck.wav" );
            */
            //trap_R_RegisterShader( "gfx/effects/stunPass" );
            trap::FX_RegisterEffect(engine, "stunBaton/flesh_impact");

            if weaponNum == WP_STUN_BATON {
                trap::S_RegisterSound(engine, "sound/weapons/baton/idle.wav");
                weaponInfo.flashSound[0] =
                    trap::S_RegisterSound(engine, "sound/weapons/baton/fire.mp3");
                weaponInfo.altFlashSound[0] =
                    trap::S_RegisterSound(engine, "sound/weapons/baton/fire.mp3");
            } else {
                /*
                int j = 0;

                while (j < 4)
                {
                    weaponInfo->flashSound[j] = trap_S_RegisterSound( va("sound/weapons/melee/swing%i", j+1) );
                    weaponInfo->altFlashSound[j] = weaponInfo->flashSound[j];
                    j++;
                }
                */
                //No longer needed, animsound config plays them for us
            }
        }

        WP_SABER => {
            // Raven's `MAKERGB` macro is the same three stores `VectorSet` does.
            VectorSet(&mut weaponInfo.flashDlightColor, 0.6, 0.6, 1.0);
            weaponInfo.firingSound =
                trap::S_RegisterSound(engine, "sound/weapons/saber/saberhum1.wav");
            weaponInfo.missileModel =
                trap::R_RegisterModel(engine, "models/weapons2/saber/saber_w.glm");
        }

        WP_CONCUSSION => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/concussion/select.wav");

            weaponInfo.flashSound[0] = NULL_SOUND;
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "concussion/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //weaponInfo->missileDlightColor= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:159`
            weaponInfo.missileTrailFunc = TrailFn::Concussion;

            weaponInfo.altFlashSound[0] = NULL_SOUND;
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/bryar/altcharge.wav");
            weaponInfo.altMuzzleEffect =
                trap::FX_RegisterEffect(engine, "concussion/altmuzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:170`
            weaponInfo.altMissileTrailFunc = TrailFn::Concussion;

            // the concussion case really does register the disruptor's alt-miss
            // effect - Raven's own crossover, kept
            world.cgs.effects.disruptorAltMissEffect =
                trap::FX_RegisterEffect(engine, "disruptor/alt_miss");

            world.cgs.effects.concussionShotEffect =
                trap::FX_RegisterEffect(engine, "concussion/shot");
            world.cgs.effects.concussionImpactEffect =
                trap::FX_RegisterEffect(engine, "concussion/explosion");
            trap::R_RegisterShader(engine, "gfx/effects/blueLine");
            trap::R_RegisterShader(engine, "gfx/misc/whiteline2");
        }

        WP_BRYAR_PISTOL | WP_BRYAR_OLD => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/bryar/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/bryar/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "bryar/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //weaponInfo->missileDlightColor= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:193`
            weaponInfo.missileTrailFunc = TrailFn::Bryar;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/bryar/alt_fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/bryar/altcharge.wav");
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "bryar/muzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:204`
            weaponInfo.altMissileTrailFunc = TrailFn::BryarAlt;

            world.cgs.effects.bryarShotEffect = trap::FX_RegisterEffect(engine, "bryar/shot");
            world.cgs.effects.bryarPowerupShotEffect =
                trap::FX_RegisterEffect(engine, "bryar/crackleShot");
            world.cgs.effects.bryarWallImpactEffect =
                trap::FX_RegisterEffect(engine, "bryar/wall_impact");
            world.cgs.effects.bryarWallImpactEffect2 =
                trap::FX_RegisterEffect(engine, "bryar/wall_impact2");
            world.cgs.effects.bryarWallImpactEffect3 =
                trap::FX_RegisterEffect(engine, "bryar/wall_impact3");
            world.cgs.effects.bryarFleshImpactEffect =
                trap::FX_RegisterEffect(engine, "bryar/flesh_impact");
            world.cgs.effects.bryarDroidImpactEffect =
                trap::FX_RegisterEffect(engine, "bryar/droid_impact");

            world.cgs.media.bryarFrontFlash =
                trap::R_RegisterShader(engine, "gfx/effects/bryarFrontFlash");

            // Note these are temp shared effects
            trap::FX_RegisterEffect(engine, "blaster/wall_impact.efx");
            trap::FX_RegisterEffect(engine, "blaster/flesh_impact.efx");
        }

        //rww - just use the same as this for now..
        WP_BLASTER | WP_EMPLACED_GUN => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/blaster/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/blaster/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "blaster/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:235`
            weaponInfo.missileTrailFunc = TrailFn::Blaster;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/blaster/alt_fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "blaster/muzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:246`
            weaponInfo.altMissileTrailFunc = TrailFn::Blaster;

            trap::FX_RegisterEffect(engine, "blaster/deflect");
            world.cgs.effects.blasterShotEffect = trap::FX_RegisterEffect(engine, "blaster/shot");
            world.cgs.effects.blasterWallImpactEffect =
                trap::FX_RegisterEffect(engine, "blaster/wall_impact");
            world.cgs.effects.blasterFleshImpactEffect =
                trap::FX_RegisterEffect(engine, "blaster/flesh_impact");
            world.cgs.effects.blasterDroidImpactEffect =
                trap::FX_RegisterEffect(engine, "blaster/droid_impact");
        }

        WP_DISRUPTOR => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/disruptor/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/disruptor/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "disruptor/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            weaponInfo.missileTrailFunc = TrailFn::None;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/disruptor/alt_fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/disruptor/altCharge.wav");
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "disruptor/muzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            weaponInfo.altMissileTrailFunc = TrailFn::None;

            world.cgs.effects.disruptorRingsEffect =
                trap::FX_RegisterEffect(engine, "disruptor/rings");
            world.cgs.effects.disruptorProjectileEffect =
                trap::FX_RegisterEffect(engine, "disruptor/projectile");
            world.cgs.effects.disruptorWallImpactEffect =
                trap::FX_RegisterEffect(engine, "disruptor/wall_impact");
            world.cgs.effects.disruptorFleshImpactEffect =
                trap::FX_RegisterEffect(engine, "disruptor/flesh_impact");
            world.cgs.effects.disruptorAltMissEffect =
                trap::FX_RegisterEffect(engine, "disruptor/alt_miss");
            world.cgs.effects.disruptorAltHitEffect =
                trap::FX_RegisterEffect(engine, "disruptor/alt_hit");

            trap::R_RegisterShader(engine, "gfx/effects/redLine");
            trap::R_RegisterShader(engine, "gfx/misc/whiteline2");
            trap::R_RegisterShader(engine, "gfx/effects/smokeTrail");

            trap::S_RegisterSound(engine, "sound/weapons/disruptor/zoomstart.wav");
            trap::S_RegisterSound(engine, "sound/weapons/disruptor/zoomend.wav");

            // Disruptor gun zoom interface
            world.cgs.media.disruptorMask = trap::R_RegisterShader(engine, "gfx/2d/cropCircle2");
            world.cgs.media.disruptorInsert = trap::R_RegisterShader(engine, "gfx/2d/cropCircle");
            world.cgs.media.disruptorLight =
                trap::R_RegisterShader(engine, "gfx/2d/cropCircleGlow");
            world.cgs.media.disruptorInsertTick =
                trap::R_RegisterShader(engine, "gfx/2d/insertTick");
            world.cgs.media.disruptorChargeShader =
                trap::R_RegisterShaderNoMip(engine, "gfx/2d/crop_charge");

            world.cgs.media.disruptorZoomLoop =
                trap::S_RegisterSound(engine, "sound/weapons/disruptor/zoomloop.wav");
        }

        WP_BOWCASTER => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/bowcaster/select.wav");

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/bowcaster/fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "bowcaster/muzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor	= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:316`
            weaponInfo.altMissileTrailFunc = TrailFn::Bowcaster;

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/bowcaster/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/bowcaster/altcharge.wav");
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "bowcaster/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:327`
            weaponInfo.missileTrailFunc = TrailFn::BowcasterAlt;

            world.cgs.effects.bowcasterShotEffect =
                trap::FX_RegisterEffect(engine, "bowcaster/shot");
            world.cgs.effects.bowcasterImpactEffect =
                trap::FX_RegisterEffect(engine, "bowcaster/explosion");

            trap::FX_RegisterEffect(engine, "bowcaster/deflect");

            world.cgs.media.greenFrontFlash =
                trap::R_RegisterShader(engine, "gfx/effects/greenFrontFlash");
        }

        WP_REPEATER => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/repeater/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/repeater/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "repeater/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:349`
            weaponInfo.missileTrailFunc = TrailFn::Repeater;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/repeater/alt_fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "repeater/muzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:360`
            weaponInfo.altMissileTrailFunc = TrailFn::RepeaterAlt;

            world.cgs.effects.repeaterProjectileEffect =
                trap::FX_RegisterEffect(engine, "repeater/projectile");
            world.cgs.effects.repeaterAltProjectileEffect =
                trap::FX_RegisterEffect(engine, "repeater/alt_projectile");
            world.cgs.effects.repeaterWallImpactEffect =
                trap::FX_RegisterEffect(engine, "repeater/wall_impact");
            world.cgs.effects.repeaterFleshImpactEffect =
                trap::FX_RegisterEffect(engine, "repeater/flesh_impact");
            //cgs.effects.repeaterAltWallImpactEffect	= trap_FX_RegisterEffect( "repeater/alt_wall_impact" );
            world.cgs.effects.repeaterAltWallImpactEffect =
                trap::FX_RegisterEffect(engine, "repeater/concussion");
        }

        WP_DEMP2 => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/demp2/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/demp2/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "demp2/muzzle_flash");
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:382`
            weaponInfo.missileTrailFunc = TrailFn::Demp2;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/demp2/altfire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/demp2/altCharge.wav");
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "demp2/muzzle_flash");
            weaponInfo.altMissileModel = NULL_HANDLE;
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            weaponInfo.altMissileTrailFunc = TrailFn::None;

            world.cgs.effects.demp2ProjectileEffect =
                trap::FX_RegisterEffect(engine, "demp2/projectile");
            world.cgs.effects.demp2WallImpactEffect =
                trap::FX_RegisterEffect(engine, "demp2/wall_impact");
            world.cgs.effects.demp2FleshImpactEffect =
                trap::FX_RegisterEffect(engine, "demp2/flesh_impact");

            world.cgs.media.demp2Shell = trap::R_RegisterModel(engine, "models/items/sphere.md3");
            world.cgs.media.demp2ShellShader =
                trap::R_RegisterShader(engine, "gfx/effects/demp2shell");

            world.cgs.media.lightningFlash =
                trap::R_RegisterShader(engine, "gfx/misc/lightningFlash");
        }

        WP_FLECHETTE => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/flechette/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/flechette/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "flechette/muzzle_flash");
            weaponInfo.missileModel =
                trap::R_RegisterModel(engine, "models/weapons2/golan_arms/projectileMain.md3");
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:417`
            weaponInfo.missileTrailFunc = TrailFn::Flechette;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/flechette/alt_fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "flechette/muzzle_flash");
            weaponInfo.altMissileModel =
                trap::R_RegisterModel(engine, "models/weapons2/golan_arms/projectile.md3");
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:428`
            weaponInfo.altMissileTrailFunc = TrailFn::FlechetteAlt;

            world.cgs.effects.flechetteShotEffect =
                trap::FX_RegisterEffect(engine, "flechette/shot");
            world.cgs.effects.flechetteAltShotEffect =
                trap::FX_RegisterEffect(engine, "flechette/alt_shot");
            world.cgs.effects.flechetteWallImpactEffect =
                trap::FX_RegisterEffect(engine, "flechette/wall_impact");
            world.cgs.effects.flechetteFleshImpactEffect =
                trap::FX_RegisterEffect(engine, "flechette/flesh_impact");
        }

        WP_ROCKET_LAUNCHER => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/rocket/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/rocket/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = trap::FX_RegisterEffect(engine, "rocket/muzzle_flash"); //trap_FX_RegisterEffect( "rocket/muzzle_flash2" );
                                                                                              //flash2 still looks crappy with the fx bolt stuff. Because the fx bolt stuff doesn't work entirely right.
            weaponInfo.missileModel =
                trap::R_RegisterModel(engine, "models/weapons2/merr_sonn/projectile.md3");
            weaponInfo.missileSound =
                trap::S_RegisterSound(engine, "sound/weapons/rocket/missleloop.wav");
            weaponInfo.missileDlight = 125.0;
            VectorSet(&mut weaponInfo.missileDlightColor, 1.0, 1.0, 0.5);
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:449`
            weaponInfo.missileTrailFunc = TrailFn::Rocket;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/rocket/alt_fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = trap::FX_RegisterEffect(engine, "rocket/altmuzzle_flash");
            weaponInfo.altMissileModel =
                trap::R_RegisterModel(engine, "models/weapons2/merr_sonn/projectile.md3");
            weaponInfo.altMissileSound =
                trap::S_RegisterSound(engine, "sound/weapons/rocket/missleloop.wav");
            weaponInfo.altMissileDlight = 125.0;
            VectorSet(&mut weaponInfo.altMissileDlightColor, 1.0, 1.0, 0.5);
            weaponInfo.altMissileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:460`
            weaponInfo.altMissileTrailFunc = TrailFn::RocketAlt;

            world.cgs.effects.rocketShotEffect = trap::FX_RegisterEffect(engine, "rocket/shot");
            world.cgs.effects.rocketExplosionEffect =
                trap::FX_RegisterEffect(engine, "rocket/explosion");

            trap::R_RegisterShaderNoMip(engine, "gfx/2d/wedge");
            trap::R_RegisterShaderNoMip(engine, "gfx/2d/lock");

            trap::S_RegisterSound(engine, "sound/weapons/rocket/lock.wav");
            trap::S_RegisterSound(engine, "sound/weapons/rocket/tick.wav");
        }

        WP_THERMAL => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/charge.wav");
            weaponInfo.muzzleEffect = NULL_FX;
            weaponInfo.missileModel =
                trap::R_RegisterModel(engine, "models/weapons2/thermal/thermal_proj.md3");
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            weaponInfo.missileTrailFunc = TrailFn::None;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/charge.wav");
            weaponInfo.altMuzzleEffect = NULL_FX;
            weaponInfo.altMissileModel =
                trap::R_RegisterModel(engine, "models/weapons2/thermal/thermal_proj.md3");
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            weaponInfo.altMissileTrailFunc = TrailFn::None;

            world.cgs.effects.thermalExplosionEffect =
                trap::FX_RegisterEffect(engine, "thermal/explosion");
            world.cgs.effects.thermalShockwaveEffect =
                trap::FX_RegisterEffect(engine, "thermal/shockwave");

            world.cgs.media.grenadeBounce1 =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/bounce1.wav");
            world.cgs.media.grenadeBounce2 =
                trap::S_RegisterSound(engine, "sound/weapons/thermal/bounce2.wav");

            trap::S_RegisterSound(engine, "sound/weapons/thermal/thermloop.wav");
            trap::S_RegisterSound(engine, "sound/weapons/thermal/warning.wav");
        }

        WP_TRIP_MINE => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/detpack/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/laser_trap/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = NULL_FX;
            weaponInfo.missileModel = 0; //trap_R_RegisterModel( "models/weapons2/laser_trap/laser_trap_w.md3" );
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            weaponInfo.missileTrailFunc = TrailFn::None;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/laser_trap/fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = NULL_FX;
            weaponInfo.altMissileModel = 0; //trap_R_RegisterModel( "models/weapons2/laser_trap/laser_trap_w.md3" );
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            weaponInfo.altMissileTrailFunc = TrailFn::None;

            world.cgs.effects.tripmineLaserFX =
                trap::FX_RegisterEffect(engine, "tripMine/laserMP.efx");
            world.cgs.effects.tripmineGlowFX =
                trap::FX_RegisterEffect(engine, "tripMine/glowbit.efx");

            trap::FX_RegisterEffect(engine, "tripMine/explosion");
            // NOTENOTE temp stuff
            trap::S_RegisterSound(engine, "sound/weapons/laser_trap/stick.wav");
            trap::S_RegisterSound(engine, "sound/weapons/laser_trap/warning.wav");
        }

        WP_DET_PACK => {
            weaponInfo.selectSound =
                trap::S_RegisterSound(engine, "sound/weapons/detpack/select.wav");

            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/detpack/fire.wav");
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = NULL_FX;
            weaponInfo.missileModel =
                trap::R_RegisterModel(engine, "models/weapons2/detpack/det_pack.md3");
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            //		weaponInfo->missileDlightColor	= {0,0,0};
            weaponInfo.missileHitSound = NULL_SOUND;
            weaponInfo.missileTrailFunc = TrailFn::None;

            weaponInfo.altFlashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/detpack/fire.wav");
            weaponInfo.altFiringSound = NULL_SOUND;
            weaponInfo.altChargeSound = NULL_SOUND;
            weaponInfo.altMuzzleEffect = NULL_FX;
            weaponInfo.altMissileModel =
                trap::R_RegisterModel(engine, "models/weapons2/detpack/det_pack.md3");
            weaponInfo.altMissileSound = NULL_SOUND;
            weaponInfo.altMissileDlight = 0.0;
            //		weaponInfo->altMissileDlightColor= {0,0,0};
            weaponInfo.altMissileHitSound = NULL_SOUND;
            weaponInfo.altMissileTrailFunc = TrailFn::None;

            trap::R_RegisterModel(engine, "models/weapons2/detpack/det_pack.md3");
            trap::S_RegisterSound(engine, "sound/weapons/detpack/stick.wav");
            trap::S_RegisterSound(engine, "sound/weapons/detpack/warning.wav");
            trap::S_RegisterSound(engine, "sound/weapons/explosions/explode5.wav");
        }

        WP_TURRET => {
            weaponInfo.flashSound[0] = NULL_SOUND;
            weaponInfo.firingSound = NULL_SOUND;
            weaponInfo.chargeSound = NULL_SOUND;
            weaponInfo.muzzleEffect = NULL_HANDLE;
            weaponInfo.missileModel = NULL_HANDLE;
            weaponInfo.missileSound = NULL_SOUND;
            weaponInfo.missileDlight = 0.0;
            weaponInfo.missileHitSound = NULL_SOUND;
            // Source: `oracle/codemp/cgame/cg_weaponinit.c:581`
            weaponInfo.missileTrailFunc = TrailFn::Turret;

            trap::FX_RegisterEffect(engine, "effects/blaster/wall_impact.efx");
            trap::FX_RegisterEffect(engine, "effects/blaster/flesh_impact.efx");
        }

        _ => {
            VectorSet(&mut weaponInfo.flashDlightColor, 1.0, 1.0, 1.0);
            weaponInfo.flashSound[0] =
                trap::S_RegisterSound(engine, "sound/weapons/rocket/rocklf1a.wav");
        }
    }
}
