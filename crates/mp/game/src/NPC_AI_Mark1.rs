// PORT-COMPLETE: NPC_AI_Mark1.c
//! FAITHFUL port of `oracle/codemp/game/NPC_AI_Mark1.c`.
//!
//! Filled by the jampgame mega-pass; all bodies are live. The file-scope AI
//! globals (`NPC`, `NPCInfo`, `ucmd`, `gPainHitLoc`) are reached through
//! `ctx.world.globals`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::entity::hit_location::*;
use crate::g_missile::CreateMissile;
use crate::level::damage_flags::DAMAGE_DEATH_KNOCKBACK;
use crate::npc::spot_t::spot_t;
use crate::npc_c::NPC_SetAnim;
use crate::prelude::*;
use crate::trap;
use mp_bg::public::anim_number::animNumber_t::*;

// `DIST_MELEE`/`DIST_LONG` are the canonical `crate::ai::distance` enum variants,
// reached via the prelude glob (the former per-file duplicate `pub const`
// copies caused a glob-glob ambiguity with the canonical import at every call
// site through `crate::prelude::*`; porting-rules §E dedupe-at-import rule).

// Raven's file-scope `#define`s (`NPC_AI_Mark1.c:4-22`) — not central
// constants, ported as file-local consts matching the C values.
const MIN_MELEE_RANGE: c_int = 320;
const MIN_MELEE_RANGE_SQR: c_int = MIN_MELEE_RANGE * MIN_MELEE_RANGE;
const MIN_DISTANCE: c_int = 128;
const MIN_DISTANCE_SQR: c_int = MIN_DISTANCE * MIN_DISTANCE;
const TURN_OFF: c_int = 0x00000100;
const LEFT_ARM_HEALTH: c_int = 40;
const RIGHT_ARM_HEALTH: c_int = 40;
pub const AMMO_POD_HEALTH: c_int = 40;
pub const BOWCASTER_VELOCITY: c_int = 1300;
const BOWCASTER_NPC_DAMAGE_EASY: c_int = 12;
const BOWCASTER_NPC_DAMAGE_NORMAL: c_int = 24;
const BOWCASTER_NPC_DAMAGE_HARD: c_int = 36;
pub const BOWCASTER_SIZE: c_int = 2;
pub const BOWCASTER_SPLASH_DAMAGE: c_int = 0;
pub const BOWCASTER_SPLASH_RADIUS: c_int = 0;

// Raven's anonymous local-state `enum` (`NPC_AI_Mark1.c:25-35`) — no
// separate typedef name, so it stays a plain set of `c_int` consts per
// house rule (typedef int + anonymous enum -> consts).
const LSTATE_NONE: c_int = 0;
const LSTATE_ASLEEP: c_int = 1;
pub const LSTATE_WAKEUP: c_int = 2;
pub const LSTATE_FIRED0: c_int = 3;
pub const LSTATE_FIRED1: c_int = 4;
pub const LSTATE_FIRED2: c_int = 5;
pub const LSTATE_FIRED3: c_int = 6;
pub const LSTATE_FIRED4: c_int = 7;

// `MASK_SHOT`/`CONTENTS_LIGHTSABER` are the canonical `mp_qshared::shared::surface_flags`
// consts, reached via the prelude glob (the former per-file duplicate `pub(crate)`
// copies caused a glob-glob ambiguity with the canonical import at every call site
// through `crate::prelude::*`; porting-rules §E dedupe-at-import rule).

/// Raven `NPC_Mark1_Precache`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:50-74`
pub fn NPC_Mark1_Precache(ctx: &mut GameContext) {
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_wakeup".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/shutdown".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/walk".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/run".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/death1".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/death2".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/anger".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_fire".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_pain".as_ptr());
    crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_explo".as_ptr());

    //	G_EffectIndex( "small_chunks");
    crate::g_utils::G_EffectIndex(c"env/med_explode2".as_ptr());
    crate::g_utils::G_EffectIndex(c"explosions/probeexplosion1".as_ptr());
    crate::g_utils::G_EffectIndex(c"blaster/smoke_bolton".as_ptr());
    crate::g_utils::G_EffectIndex(c"bryar/muzzle_flash".as_ptr());
    crate::g_utils::G_EffectIndex(c"explosions/droidexplosion1".as_ptr());

    crate::g_items::RegisterItem(
        ctx,
        mp_bg::bg_misc::BG_FindItemForAmmo(ammo_t::AMMO_METAL_BOLTS),
    );
    crate::g_items::RegisterItem(
        ctx,
        mp_bg::bg_misc::BG_FindItemForAmmo(ammo_t::AMMO_BLASTER),
    );
    crate::g_items::RegisterItem(
        ctx,
        mp_bg::bg_misc::BG_FindItemForWeapon(mp_bg::weapons::weapon_t::WP_BOWCASTER),
    );
    crate::g_items::RegisterItem(
        ctx,
        mp_bg::bg_misc::BG_FindItemForWeapon(mp_bg::weapons::weapon_t::WP_BRYAR_PISTOL),
    );
}

/// Raven `NPC_Mark1_Part_Explode`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:81-102`
pub fn NPC_Mark1_Part_Explode(ctx: &mut GameContext, self_: EntityId, bolt: c_int) {
    if bolt >= 0 {
        let ghoul2 = ctx.world.entity(self_).ghoul2;
        let current_angles = ctx.world.entity(self_).r.currentAngles;
        let current_origin = ctx.world.entity(self_).r.currentOrigin;
        let model_scale = ctx.world.entity(self_).modelScale;
        let level_time = ctx.world.level.time;

        let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
        let mut org: vec3_t = [0.0; 3];
        let mut dir: vec3_t = [0.0; 3];

        trap::G2API_GetBoltMatrix(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
                ghoul2,
                0,
                bolt,
                &mut boltMatrix,
                &current_angles,
                &current_origin,
                level_time,
                core::ptr::null_mut(),
                &model_scale,
            ),
        );

        BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN as c_int, &mut org);
        BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y as c_int, &mut dir);

        crate::g_utils::G_PlayEffectID(
            crate::g_utils::G_EffectIndex(c"env/med_explode2".as_ptr()),
            org,
            dir,
        );

        crate::g_utils::G_PlayEffectID(
            crate::g_utils::G_EffectIndex(c"blaster/smoke_bolton".as_ptr()),
            org,
            dir,
        );
    }
}

/// Raven `Mark1_Idle`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:109-115`
pub fn Mark1_Idle(ctx: &mut GameContext) {
    crate::NPC_AI_Default::NPC_BSIdle(ctx);
    let npc = ctx.world.globals.NPC;
    if !npc.is_null() {
        NPC_SetAnim(
            ctx,
            ctx.entity_id_of(npc).unwrap(),
            SETANIM_BOTH,
            BOTH_SLEEP1 as c_int,
            SETANIM_FLAG_NORMAL,
        );
    }
}

/// Raven `Mark1Dead_FireRocket`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:123-163`
pub fn Mark1Dead_FireRocket(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut muzzle_dir: vec3_t = [0.0; 3];

    let damage = 50;
    let ghoul2 = ctx.world.entity(npc_id).ghoul2;
    let bolt = trap::G2API_AddBolt(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(ghoul2, 0, c"*flash5".to_owned()),
    );

    let current_angles = ctx.world.entity(npc_id).r.currentAngles;
    let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let model_scale = ctx.world.entity(npc_id).modelScale;
    let level_time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &current_angles,
            &current_origin,
            level_time,
            core::ptr::null_mut(),
            &model_scale,
        ),
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN as c_int, &mut muzzle1);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y as c_int, &mut muzzle_dir);

    crate::g_utils::G_PlayEffectID(
        crate::g_utils::G_EffectIndex(c"bryar/muzzle_flash".as_ptr()),
        muzzle1,
        muzzle_dir,
    );

    crate::g_utils::G_Sound(
        ctx,
        Some(npc_id),
        CHAN_AUTO,
        crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_fire".as_ptr()),
    );

    let missile_id = CreateMissile(
        ctx,
        muzzle1,
        muzzle_dir,
        BOWCASTER_VELOCITY as f32,
        10000,
        npc_id,
        false,
    );

    ctx.world.entity_mut(missile_id).classname = c"bowcaster_proj".as_ptr().cast_mut();
    ctx.world.entity_mut(missile_id).s.weapon = WP_BOWCASTER as c_int;

    ctx.world.entity_mut(missile_id).r.maxs[0] = BOWCASTER_SIZE as f32;
    ctx.world.entity_mut(missile_id).r.maxs[1] = BOWCASTER_SIZE as f32;
    ctx.world.entity_mut(missile_id).r.maxs[2] = BOWCASTER_SIZE as f32;
    ctx.world.entity_mut(missile_id).r.mins[0] = -(BOWCASTER_SIZE as f32);
    ctx.world.entity_mut(missile_id).r.mins[1] = -(BOWCASTER_SIZE as f32);
    ctx.world.entity_mut(missile_id).r.mins[2] = -(BOWCASTER_SIZE as f32);

    ctx.world.entity_mut(missile_id).damage = damage;
    ctx.world.entity_mut(missile_id).dflags = DAMAGE_DEATH_KNOCKBACK;
    ctx.world.entity_mut(missile_id).methodOfDeath = MOD_ROCKET as c_int;
    ctx.world.entity_mut(missile_id).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    ctx.world.entity_mut(missile_id).splashDamage = BOWCASTER_SPLASH_DAMAGE;
    ctx.world.entity_mut(missile_id).splashRadius = BOWCASTER_SPLASH_RADIUS;

    ctx.world.entity_mut(missile_id).bounceCount = 0;
}

/// Raven `Mark1Dead_FireBlaster`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:171-202`
pub fn Mark1Dead_FireBlaster(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut muzzle_dir: vec3_t = [0.0; 3];

    let ghoul2 = ctx.world.entity(npc_id).ghoul2;
    let bolt = trap::G2API_AddBolt(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(ghoul2, 0, c"*flash1".to_owned()),
    );

    let current_angles = ctx.world.entity(npc_id).r.currentAngles;
    let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let model_scale = ctx.world.entity(npc_id).modelScale;
    let level_time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &current_angles,
            &current_origin,
            level_time,
            core::ptr::null_mut(),
            &model_scale,
        ),
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN as c_int, &mut muzzle1);
    BG_GiveMeVectorFromMatrix(&boltMatrix, NEGATIVE_Y as c_int, &mut muzzle_dir);

    crate::g_utils::G_PlayEffectID(
        crate::g_utils::G_EffectIndex(c"bryar/muzzle_flash".as_ptr()),
        muzzle1,
        muzzle_dir,
    );

    let missile_id = CreateMissile(ctx, muzzle1, muzzle_dir, 1600.0, 10000, npc_id, false);

    crate::g_utils::G_Sound(
        ctx,
        Some(npc_id),
        CHAN_AUTO,
        crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_fire".as_ptr()),
    );

    ctx.world.entity_mut(missile_id).classname = c"bryar_proj".as_ptr().cast_mut();
    ctx.world.entity_mut(missile_id).s.weapon = WP_BRYAR_PISTOL as c_int;

    ctx.world.entity_mut(missile_id).damage = 1;
    ctx.world.entity_mut(missile_id).dflags = DAMAGE_DEATH_KNOCKBACK;
    ctx.world.entity_mut(missile_id).methodOfDeath = MOD_BRYAR_PISTOL as c_int;
    ctx.world.entity_mut(missile_id).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/// Raven `Mark1_die`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:209-243`
pub fn Mark1_die(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    inflictor: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
    r#mod: c_int,
    dFlags: c_int,
    hitLoc: c_int,
) {
    let sound_index = crate::g_utils::G_SoundIndex(
        cstr(&format!(
            "sound/chars/mark1/misc/death{}.wav",
            ctx.world.bg_state.rng.Q_irand(1, 2),
        ))
        .as_ptr(),
    );
    let Some(self_id) = self_ else {
        return;
    };

    crate::g_utils::G_Sound(ctx, Some(self_id), CHAN_AUTO, sound_index);

    // Choose a death anim
    if ctx.world.bg_state.rng.Q_irand(1, 10) > 5 {
        NPC_SetAnim(
            ctx,
            self_id,
            SETANIM_BOTH,
            BOTH_DEATH2 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    } else {
        NPC_SetAnim(
            ctx,
            self_id,
            SETANIM_BOTH,
            BOTH_DEATH1 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
    }
}

/// Raven `Mark1_dying`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:250-312`
pub fn Mark1_dying(ctx: &mut GameContext, self_: Option<EntityId>) {
    let Some(self_id) = self_ else {
        return;
    };

    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw
    // via the safe entity borrow, per trap 2b.
    let client = ctx.world.entity(self_id).client;
    if unsafe { (*client).ps.torsoTimer } > 0 {
        if crate::g_timer::TIMER_Done(ctx, Some(self_id), c"dyingExplosion".as_ptr()) != 0 {
            let num = ctx.world.bg_state.rng.Q_irand(1, 3);

            // Find place to generate explosion
            if num == 1 {
                let random_num = ctx.world.bg_state.rng.Q_irand(8, 10);
                let ghoul2 = ctx.world.entity(self_id).ghoul2;
                let newBolt = trap::G2API_AddBolt(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                        ghoul2,
                        0,
                        std::ffi::CString::new(format!("*flash{}", random_num)).unwrap(),
                    ),
                );
                NPC_Mark1_Part_Explode(ctx, self_id, newBolt);
            } else {
                let random_num = ctx.world.bg_state.rng.Q_irand(1, 6);
                let ghoul2 = ctx.world.entity(self_id).ghoul2;
                let newBolt = trap::G2API_AddBolt(
                    ctx.engine,
                    mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                        ghoul2,
                        0,
                        std::ffi::CString::new(format!("*torso_tube{}", random_num)).unwrap(),
                    ),
                );
                NPC_Mark1_Part_Explode(ctx, self_id, newBolt);
                crate::NPC_utils::NPC_SetSurfaceOnOff(
                    ctx,
                    self_id,
                    cstr(&format!("torso_tube{}", random_num)).as_ptr(),
                    TURN_OFF,
                );
            }

            // Oracle draws Q_irand(300,1000) LAST, after the branch draw.
            // Source: oracle/codemp/game/NPC_AI_Mark1.c:273
            let delay = ctx.world.bg_state.rng.Q_irand(300, 1000);
            crate::g_timer::TIMER_Set(ctx, Some(self_id), c"dyingExplosion".as_ptr(), delay);
        }

        // See which weapons are there
        // Randomly fire blaster
        let ghoul2 = ctx.world.entity(self_id).ghoul2;
        if trap::G2API_GetSurfaceRenderStatus(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                ghoul2,
                0,
                c"l_arm".to_owned(),
            ),
        ) == 0
        {
            if ctx.world.bg_state.rng.Q_irand(1, 5) == 1 {
                crate::npc_c::SaveNPCGlobals(ctx);
                crate::npc_c::SetNPCGlobals(ctx, self_id);
                Mark1Dead_FireBlaster(ctx);
                crate::npc_c::RestoreNPCGlobals(ctx);
            }
        }

        // Randomly fire rocket
        let ghoul2 = ctx.world.entity(self_id).ghoul2;
        if trap::G2API_GetSurfaceRenderStatus(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                ghoul2,
                0,
                c"r_arm".to_owned(),
            ),
        ) == 0
        {
            if ctx.world.bg_state.rng.Q_irand(1, 10) == 1 {
                crate::npc_c::SaveNPCGlobals(ctx);
                crate::npc_c::SetNPCGlobals(ctx, self_id);
                Mark1Dead_FireRocket(ctx);
                crate::npc_c::RestoreNPCGlobals(ctx);
            }
        }
    }
}

/// Raven `NPC_Mark1_Pain`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:320-396`
pub fn NPC_Mark1_Pain(
    ctx: &mut GameContext,
    self_: Option<EntityId>,
    attacker: Option<EntityId>,
    damage: c_int,
) {
    let Some(self_id) = self_ else {
        return;
    };

    // Oracle reads gPainHitLoc BEFORE NPC_Pain + G_Sound.
    // Source: oracle/codemp/game/NPC_AI_Mark1.c:322-326
    let hitLoc = ctx.world.globals.gPainHitLoc;

    crate::NPC_reactions::NPC_Pain(ctx, self_id, attacker, damage);

    crate::g_utils::G_Sound(
        ctx,
        Some(self_id),
        CHAN_AUTO,
        crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_pain".as_ptr()),
    );

    // Hit in the CHEST???
    if hitLoc == HL_CHEST {
        let chance = ctx.world.bg_state.rng.Q_irand(1, 4);

        if chance == 1 && damage > 5 {
            NPC_SetAnim(
                ctx,
                self_id,
                SETANIM_BOTH,
                BOTH_PAIN1 as c_int,
                SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
            );
        }
    }
    // Hit in the left arm?
    else if hitLoc == HL_ARM_LT
        && ctx.world.entity(self_id).locationDamage[HL_ARM_LT as usize] > LEFT_ARM_HEALTH
    {
        if ctx.world.entity(self_id).locationDamage[hitLoc as usize] >= LEFT_ARM_HEALTH {
            let ghoul2 = ctx.world.entity(self_id).ghoul2;
            let newBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    ghoul2,
                    0,
                    c"*flash3".to_owned(),
                ),
            );
            if newBolt != -1 {
                NPC_Mark1_Part_Explode(ctx, self_id, newBolt);
            }

            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_id, c"l_arm".as_ptr(), TURN_OFF);
        }
    }
    // Hit in the right arm?
    else if hitLoc == HL_ARM_RT
        && ctx.world.entity(self_id).locationDamage[HL_ARM_RT as usize] > RIGHT_ARM_HEALTH
    {
        if ctx.world.entity(self_id).locationDamage[hitLoc as usize] >= RIGHT_ARM_HEALTH {
            let ghoul2 = ctx.world.entity(self_id).ghoul2;
            let newBolt = trap::G2API_AddBolt(
                ctx.engine,
                mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                    ghoul2,
                    0,
                    c"*flash4".to_owned(),
                ),
            );
            if newBolt != -1 {
                NPC_Mark1_Part_Explode(ctx, self_id, newBolt);
            }

            crate::NPC_utils::NPC_SetSurfaceOnOff(ctx, self_id, c"r_arm".as_ptr(), TURN_OFF);
        }
    }
    // Check ammo pods
    else {
        for i in 0..6 {
            let location_idx = HL_GENERIC1 as usize + i;
            if hitLoc == HL_GENERIC1 + i as c_int
                && ctx.world.entity(self_id).locationDamage[location_idx] > AMMO_POD_HEALTH
            {
                if ctx.world.entity(self_id).locationDamage[hitLoc as usize] >= AMMO_POD_HEALTH {
                    let ghoul2 = ctx.world.entity(self_id).ghoul2;
                    let newBolt = trap::G2API_AddBolt(
                        ctx.engine,
                        mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                            ghoul2,
                            0,
                            std::ffi::CString::new(format!("*torso_tube{}", (i + 1) as c_int))
                                .unwrap(),
                        ),
                    );
                    if newBolt != -1 {
                        NPC_Mark1_Part_Explode(ctx, self_id, newBolt);
                    }
                    crate::NPC_utils::NPC_SetSurfaceOnOff(
                        ctx,
                        self_id,
                        cstr(&format!("torso_tube{}", (i + 1) as c_int)).as_ptr(),
                        TURN_OFF,
                    );
                    NPC_SetAnim(
                        ctx,
                        self_id,
                        SETANIM_BOTH,
                        BOTH_PAIN1 as c_int,
                        SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                    );
                    break;
                }
            }
        }
    }

    // Are both guns shot off?
    let ghoul2 = ctx.world.entity(self_id).ghoul2;
    if trap::G2API_GetSurfaceRenderStatus(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
            ghoul2,
            0,
            c"l_arm".to_owned(),
        ),
    ) > 0
        && trap::G2API_GetSurfaceRenderStatus(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
                ghoul2,
                0,
                c"r_arm".to_owned(),
            ),
        ) > 0
    {
        let health = ctx.world.entity(self_id).health;
        crate::g_combat::G_Damage(
            ctx,
            Some(self_id),
            None,
            None,
            None,
            [0.0, 0.0, 0.0],
            health,
            0,
            MOD_UNKNOWN as c_int,
        );
    }
}

/// Raven `Mark1_Hunt`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:404-416`
pub fn Mark1_Hunt(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    if !npc_info.is_null() {
        if unsafe { (*npc_info).goalEntity }.is_none() {
            let npc_id = ctx.entity_id_of(npc).unwrap();
            let enemy = ctx.world.entity(npc_id).enemy;
            unsafe {
                (*npc_info).goalEntity = enemy;
            }
        }
    }

    crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

    if !npc_info.is_null() {
        unsafe {
            (*npc_info).combatMove = qtrue;
        }
    }
    crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
}

/// Raven `Mark1_FireBlaster`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:424-488`
pub fn Mark1_FireBlaster(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];
    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };

    let ghoul2 = ctx.world.entity(npc_id).ghoul2;

    // Which muzzle to fire from?
    let localState = unsafe { (*npc_info).localState };
    let bolt = if localState <= LSTATE_FIRED0 || localState == LSTATE_FIRED4 {
        unsafe {
            (*npc_info).localState = LSTATE_FIRED1;
        }
        trap::G2API_AddBolt(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                ghoul2,
                0,
                c"*flash1".to_owned(),
            ),
        )
    } else if localState == LSTATE_FIRED1 {
        unsafe {
            (*npc_info).localState = LSTATE_FIRED2;
        }
        trap::G2API_AddBolt(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                ghoul2,
                0,
                c"*flash2".to_owned(),
            ),
        )
    } else if localState == LSTATE_FIRED2 {
        unsafe {
            (*npc_info).localState = LSTATE_FIRED3;
        }
        trap::G2API_AddBolt(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                ghoul2,
                0,
                c"*flash3".to_owned(),
            ),
        )
    } else {
        unsafe {
            (*npc_info).localState = LSTATE_FIRED4;
        }
        trap::G2API_AddBolt(
            ctx.engine,
            mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(
                ghoul2,
                0,
                c"*flash4".to_owned(),
            ),
        )
    };

    let current_angles = ctx.world.entity(npc_id).r.currentAngles;
    let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let model_scale = ctx.world.entity(npc_id).modelScale;
    let level_time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &current_angles,
            &current_origin,
            level_time,
            core::ptr::null_mut(),
            &model_scale,
        ),
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN as c_int, &mut muzzle1);

    if ctx.world.entity(npc_id).health != 0 {
        let enemy_id = ctx.world.entity(npc_id).enemy;
        crate::NPC_utils::CalcEntitySpot(ctx, enemy_id, spot_t::SPOT_HEAD, &mut enemy_org1);
        delta1[0] = enemy_org1[0] - muzzle1[0];
        delta1[1] = enemy_org1[1] - muzzle1[1];
        delta1[2] = enemy_org1[2] - muzzle1[2];
        crate::q_math::vectoangles(delta1, &mut angleToEnemy1);
        crate::q_math::AngleVectors(
            angleToEnemy1,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    } else {
        crate::q_math::AngleVectors(
            current_angles,
            Some(&mut forward),
            Some(&mut vright),
            Some(&mut up),
        );
    }

    crate::g_utils::G_PlayEffectID(
        crate::g_utils::G_EffectIndex(c"bryar/muzzle_flash".as_ptr()),
        muzzle1,
        forward,
    );

    crate::g_utils::G_Sound(
        ctx,
        Some(npc_id),
        CHAN_AUTO,
        crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_fire".as_ptr()),
    );

    let missile_id = CreateMissile(ctx, muzzle1, forward, 1600.0, 10000, npc_id, false);

    ctx.world.entity_mut(missile_id).classname = c"bryar_proj".as_ptr().cast_mut();
    ctx.world.entity_mut(missile_id).s.weapon = WP_BRYAR_PISTOL as c_int;

    ctx.world.entity_mut(missile_id).damage = 1;
    ctx.world.entity_mut(missile_id).dflags = DAMAGE_DEATH_KNOCKBACK;
    ctx.world.entity_mut(missile_id).methodOfDeath = MOD_BRYAR_PISTOL as c_int;
    ctx.world.entity_mut(missile_id).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
}

/// Raven `Mark1_BlasterAttack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:495-548`
pub fn Mark1_BlasterAttack(ctx: &mut GameContext, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();
    // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients); deref raw
    // via the safe entity borrow, per trap 2b.
    let client = ctx.world.entity(npc_id).client;

    if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay".as_ptr()) != 0 {
        let mut chance = ctx.world.bg_state.rng.Q_irand(1, 5);

        unsafe {
            (*npc_info).burstCount += 1;

            if (*npc_info).burstCount < 3 {
                // Too few shots this burst?
                chance = 2; // Force it to keep firing.
            } else if (*npc_info).burstCount > 12 {
                // Too many shots fired this burst?
                (*npc_info).burstCount = 0;
                chance = 1; // Force it to stop firing.
            }
        }

        // Stop firing.
        if chance == 1 {
            unsafe {
                (*npc_info).burstCount = 0;
            }
            let delay = ctx.world.bg_state.rng.Q_irand(1000, 3000);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attackDelay".as_ptr(), delay);
            unsafe {
                (*client).ps.torsoTimer = 0;
            }
        } else {
            if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"attackDelay2".as_ptr()) != 0 {
                let delay = ctx.world.bg_state.rng.Q_irand(50, 50);
                crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"attackDelay2".as_ptr(), delay);
                Mark1_FireBlaster(ctx);
                NPC_SetAnim(
                    ctx,
                    npc_id,
                    SETANIM_BOTH,
                    BOTH_ATTACK1 as c_int,
                    SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
                );
            }
            return;
        }
    } else if advance != 0 {
        unsafe {
            if (*client).ps.torsoAnim == BOTH_ATTACK1 as c_int {
                (*client).ps.torsoTimer = 0;
            }
        }
        Mark1_Hunt(ctx);
    } else {
        // Make sure he's not firing.
        unsafe {
            if (*client).ps.torsoAnim == BOTH_ATTACK1 as c_int {
                (*client).ps.torsoTimer = 0;
            }
        }
    }
}

/// Raven `Mark1_FireRocket`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:555-599`
pub fn Mark1_FireRocket(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let mut boltMatrix: mdxaBone_t = unsafe { core::mem::zeroed() };
    let mut muzzle1: vec3_t = [0.0; 3];
    let mut enemy_org1: vec3_t = [0.0; 3];
    let mut delta1: vec3_t = [0.0; 3];
    let mut angleToEnemy1: vec3_t = [0.0; 3];
    let mut forward: vec3_t = [0.0; 3];
    let mut vright: vec3_t = [0.0; 3];
    let mut up: vec3_t = [0.0; 3];

    let damage = 50;
    let ghoul2 = ctx.world.entity(npc_id).ghoul2;
    let bolt = trap::G2API_AddBolt(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_ADDBOLT::GG2AddboltArgs::new(ghoul2, 0, c"*flash5".to_owned()),
    );

    let current_angles = ctx.world.entity(npc_id).r.currentAngles;
    let current_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let model_scale = ctx.world.entity(npc_id).modelScale;
    let level_time = ctx.world.level.time;
    trap::G2API_GetBoltMatrix(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
            ghoul2,
            0,
            bolt,
            &mut boltMatrix,
            &current_angles,
            &current_origin,
            level_time,
            core::ptr::null_mut(),
            &model_scale,
        ),
    );

    BG_GiveMeVectorFromMatrix(&boltMatrix, ORIGIN as c_int, &mut muzzle1);

    let enemy_id = ctx.world.entity(npc_id).enemy;
    crate::NPC_utils::CalcEntitySpot(ctx, enemy_id, spot_t::SPOT_HEAD, &mut enemy_org1);
    delta1[0] = enemy_org1[0] - muzzle1[0];
    delta1[1] = enemy_org1[1] - muzzle1[1];
    delta1[2] = enemy_org1[2] - muzzle1[2];
    crate::q_math::vectoangles(delta1, &mut angleToEnemy1);
    crate::q_math::AngleVectors(
        angleToEnemy1,
        Some(&mut forward),
        Some(&mut vright),
        Some(&mut up),
    );

    crate::g_utils::G_Sound(
        ctx,
        Some(npc_id),
        CHAN_AUTO,
        crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_fire".as_ptr()),
    );

    let missile_id = CreateMissile(
        ctx,
        muzzle1,
        forward,
        BOWCASTER_VELOCITY as f32,
        10000,
        npc_id,
        false,
    );

    ctx.world.entity_mut(missile_id).classname = c"bowcaster_proj".as_ptr().cast_mut();
    ctx.world.entity_mut(missile_id).s.weapon = WP_BOWCASTER as c_int;

    ctx.world.entity_mut(missile_id).r.maxs[0] = BOWCASTER_SIZE as f32;
    ctx.world.entity_mut(missile_id).r.maxs[1] = BOWCASTER_SIZE as f32;
    ctx.world.entity_mut(missile_id).r.maxs[2] = BOWCASTER_SIZE as f32;
    ctx.world.entity_mut(missile_id).r.mins[0] = -(BOWCASTER_SIZE as f32);
    ctx.world.entity_mut(missile_id).r.mins[1] = -(BOWCASTER_SIZE as f32);
    ctx.world.entity_mut(missile_id).r.mins[2] = -(BOWCASTER_SIZE as f32);

    ctx.world.entity_mut(missile_id).damage = damage;
    ctx.world.entity_mut(missile_id).dflags = DAMAGE_DEATH_KNOCKBACK;
    ctx.world.entity_mut(missile_id).methodOfDeath = MOD_ROCKET as c_int;
    ctx.world.entity_mut(missile_id).clipmask = MASK_SHOT | CONTENTS_LIGHTSABER;
    ctx.world.entity_mut(missile_id).splashDamage = BOWCASTER_SPLASH_DAMAGE;
    ctx.world.entity_mut(missile_id).splashRadius = BOWCASTER_SPLASH_RADIUS;

    ctx.world.entity_mut(missile_id).bounceCount = 0;
}

/// Raven `Mark1_RocketAttack`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:606-618`
pub fn Mark1_RocketAttack(ctx: &mut GameContext, advance: qboolean) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }

    if crate::g_timer::TIMER_Done(ctx, ctx.entity_id_of(npc), c"attackDelay".as_ptr()) != 0 {
        let npc_id = ctx.entity_id_of(npc);
        let delay = ctx.world.bg_state.rng.Q_irand(1000, 3000);
        crate::g_timer::TIMER_Set(ctx, npc_id, c"attackDelay".as_ptr(), delay);
        NPC_SetAnim(
            ctx,
            ctx.entity_id_of(npc).unwrap(),
            SETANIM_TORSO,
            BOTH_ATTACK2 as c_int,
            SETANIM_FLAG_OVERRIDE | SETANIM_FLAG_HOLD,
        );
        Mark1_FireRocket(ctx);
    } else if advance != 0 {
        Mark1_Hunt(ctx);
    }
}

/// Raven `Mark1_AttackDecision`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:625-704`
pub fn Mark1_AttackDecision(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    let npc_info = ctx.world.globals.NPCInfo;

    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    // Randomly talk
    if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"patrolNoise".as_ptr()) != 0 {
        if crate::g_timer::TIMER_Done(ctx, Some(npc_id), c"angerNoise".as_ptr()) != 0 {
            let delay = ctx.world.bg_state.rng.Q_irand(4000, 10000);
            crate::g_timer::TIMER_Set(ctx, Some(npc_id), c"patrolNoise".as_ptr(), delay);
        }
    }

    // Enemy is dead or he has no enemy.
    let enemy_id = ctx.world.entity(npc_id).enemy;
    let enemy_uid = enemy_id.unwrap();
    if ctx.world.entity(enemy_uid).health < 1
        || crate::NPC_utils::NPC_CheckEnemyExt(ctx, qfalse) == qfalse
    {
        ctx.world.entity_mut(npc_id).enemy = None;
        return;
    }

    // Rate our distance to the target and visibility
    let npc_origin = ctx.world.entity(npc_id).r.currentOrigin;
    let enemy_origin = ctx.world.entity(enemy_uid).r.currentOrigin;
    let distance = crate::q_math::DistanceHorizontalSquared(npc_origin, enemy_origin) as c_int;
    let distRate = if distance > MIN_MELEE_RANGE_SQR {
        DIST_LONG
    } else {
        DIST_MELEE
    };
    let visible = crate::NPC_utils::NPC_ClearLOS4(ctx, enemy_id);
    let advance = if distance > MIN_DISTANCE_SQR {
        qtrue
    } else {
        qfalse
    };

    // If we cannot see our target, move to see it
    if visible == qfalse || crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue) == qfalse {
        Mark1_Hunt(ctx);
        return;
    }

    // See if the side weapons are there
    let ghoul2 = ctx.world.entity(npc_id).ghoul2;
    let blasterTest = trap::G2API_GetSurfaceRenderStatus(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
            ghoul2,
            0,
            c"l_arm".to_owned(),
        ),
    );
    let rocketTest = trap::G2API_GetSurfaceRenderStatus(
        ctx.engine,
        mp_abi::game::syscalls::G_G2_GETSURFACERENDERSTATUS::GG2GetsurfacerenderstatusArgs::new(
            ghoul2,
            0,
            c"r_arm".to_owned(),
        ),
    );

    let final_distRate =
            // It has both side weapons
            if blasterTest == 0 && rocketTest == 0 {
                distRate
            }
            else if blasterTest != -1 && blasterTest != 0 {
                DIST_LONG
            }
            else if rocketTest != -1 && rocketTest != 0 {
                DIST_MELEE
            }
            else {
                // It should never get here, but just in case
                ctx.world.entity_mut(npc_id).health = 0;
                // FLAG: NPC carries a BG_Alloc'd pool client (not level.clients);
                // deref raw via the safe entity borrow, per trap 2b.
                let client = ctx.world.entity(npc_id).client;
                unsafe {
                    (*client).ps.stats[STAT_HEALTH as usize] = 0;
                }
                if let Some(die_fn) = ctx.world.entity(npc_id).die.get() {
                    crate::ent_fn_enums::dispatch_die(ctx, die_fn, npc, npc, npc, 100, MOD_UNKNOWN as c_int);
                }
                // C does not return here: it falls through to NPC_FaceEnemy and the
                // attack dispatch with the unchanged distRate from the distance check.
                distRate
            };

    // We can see enemy so shoot him if timers let you.
    crate::NPC_utils::NPC_FaceEnemy(ctx, qtrue);

    if final_distRate == DIST_MELEE {
        Mark1_BlasterAttack(ctx, advance);
    } else if final_distRate == DIST_LONG {
        Mark1_RocketAttack(ctx, advance);
    }
}

/// Raven `Mark1_Patrol`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:711-739`
pub fn Mark1_Patrol(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    if npc.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    if crate::NPC_AI_Stormtrooper::NPC_CheckPlayerTeamStealth(ctx) != 0 {
        crate::g_utils::G_Sound(
            ctx,
            Some(npc_id),
            CHAN_AUTO,
            crate::g_utils::G_SoundIndex(c"sound/chars/mark1/misc/mark1_wakeup".as_ptr()),
        );
        crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
        return;
    }

    // If we have somewhere to go, then do that
    if ctx.world.entity(npc_id).enemy.is_none() {
        let goal = crate::NPC_goal::UpdateGoal(ctx);
        if !goal.is_null() {
            ctx.world.globals.ucmd.buttons |= BUTTON_WALKING;
            crate::NPC_move::NPC_MoveToGoal(ctx, qtrue);
            crate::NPC_utils::NPC_UpdateAngles(ctx, qtrue, qtrue);
        }
    }
}

/// Raven `NPC_BSMark1_Default`.
///
/// Source: `oracle/codemp/game/NPC_AI_Mark1.c:747-764`
pub fn NPC_BSMark1_Default(ctx: &mut GameContext) {
    let npc = ctx.world.globals.NPC;
    // FLAG: gNPC_t (NPCInfo) has no accessor; derefs stay raw (recipe 2c).
    let npc_info = ctx.world.globals.NPCInfo;

    if npc.is_null() || npc_info.is_null() {
        return;
    }
    let npc_id = ctx.entity_id_of(npc).unwrap();

    let enemy = ctx.world.entity(npc_id).enemy;
    if enemy.is_some() {
        unsafe {
            (*npc_info).goalEntity = enemy;
        }
        Mark1_AttackDecision(ctx);
    } else if unsafe { (*npc_info).scriptFlags } & SCF_LOOK_FOR_ENEMIES != 0 {
        Mark1_Patrol(ctx);
    } else {
        Mark1_Idle(ctx);
    }
}
