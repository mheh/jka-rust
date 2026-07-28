//! Port of `oracle/codemp/cgame/cg_players.c` — player models, animation, ghoul2 setup and player effects. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_char, c_int, c_short, c_void};
use core::ptr::null_mut;

use mp_bg::bg_misc::BG_GiveMeVectorFromMatrix;
use mp_bg::bg_panimate::BG_ParseAnimationFile;
use mp_bg::bg_vehicleLoad::{BG_GetVehicleModelName, BG_GetVehicleSkinName};
use mp_bg::local::{bgToggleableSurfaceDebris, bgToggleableSurfaces, bg_customSiegeSoundNames};
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::public::entity_flags::EF_DEAD;
use mp_bg::public::entity_type::entityType_t;
use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL, GT_TEAM};
use mp_bg::public::gender::gender_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::ref_entity_type_t::refEntityType_t;
use mp_qshared::common::mp::game::class_t::class_t;
use mp_qshared::common::mp::ghoul2::bone_flags::{
    BONE_ANGLES_POSTMULT, BONE_ANIM_OVERRIDE, BONE_ANIM_OVERRIDE_FREEZE,
};
use mp_qshared::common::mp::qcommon::collision_record::{G2Trace_t, MAX_G2_COLLISIONS};
use mp_qshared::common::mp::qcommon::saber::saber_colors::{
    saber_colors_t, SABER_BLUE, SABER_GREEN, SABER_ORANGE, SABER_PURPLE, SABER_RED, SABER_YELLOW,
};
use mp_qshared::common::mp::qcommon::saber::saber_flags::SFL_BOLT_TO_WRIST;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::q_math::{
    _DotProduct, _VectorAdd, _VectorCopy, _VectorMA, _VectorScale, _VectorSubtract, vec3_origin,
    vectoangles, AngleVectors, AnglesToAxis, MatrixMultiply, VectorClear, VectorInverse,
    VectorLength, VectorNormalize, VectorSet, PITCH, ROLL, YAW,
};
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{
    addbezierArgStruct_t, mdxaBone_t, orientation_t, qfalse, qhandle_t, qtrue, vec3_t,
    CollisionRecord_t, Eorientations, CONTENTS_LAVA, CONTENTS_SLIME, CONTENTS_SOLID,
    CONTENTS_WATER, ENTITYNUM_NONE, ENTITYNUM_WORLD, FP_SEE, FS_READ, MAX_CLIENTS, MAX_CLIENTS_I32,
    MAX_GENTITIES, MAX_QPATH,
};
use native_string::{atoi, buf_to_string, cstr, Q_stricmp, Q_strncpyz};

use crate::bg_channel::{CgBgTraps, CgGameCallbacks};
use crate::local::centity_s::centity_t;
use crate::local::client_info_t::{
    clientInfo_t, MAX_CUSTOM_DUEL_SOUNDS, MAX_CUSTOM_SIEGE_SOUNDS, MAX_CUSTOM_SOUNDS,
};
use crate::local::lerp_frame_t::lerpFrame_t;
use crate::local::vehicle_id::VehicleId;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::cg_world::CgWorld;

// ---------------------------------------------------------------------------
// File-scope constants and tables
// ---------------------------------------------------------------------------

/// Raven `RF_THIRD_PERSON` — don't draw through eyes, only mirrors (player
/// bodies, chat sprites). `tr_types.h`'s renderfx bits have no ported
/// cross-crate home yet, so this one lands beside its reader (the
/// `RF_DISINTEGRATE1` treatment in `cg_ents.rs`).
/// Source: `oracle/codemp/cgame/tr_types.h:19`
const RF_THIRD_PERSON: c_int = 0x00002;

/// Raven `DEFAULT_FEMALE_SOUNDPATH`.
///
/// Raven: the commented-out original was `"chars/tavion/misc"`.
/// Source: `oracle/codemp/cgame/cg_players.c:797`
const DEFAULT_FEMALE_SOUNDPATH: &str = "chars/mp_generic_female/misc";

/// Raven `DEFAULT_MALE_SOUNDPATH`.
///
/// Raven: the commented-out original was `"chars/kyle/misc"`.
/// Source: `oracle/codemp/cgame/cg_players.c:798`
const DEFAULT_MALE_SOUNDPATH: &str = "chars/mp_generic_male/misc";

/// Raven `MAX_SHIELD_TIME` — a double literal, so the `cg.time + MAX_SHIELD_TIME`
/// it feeds is a double expression that truncates back to `int`.
/// Source: `oracle/codemp/cgame/cg_players.c:5073`
const MAX_SHIELD_TIME: f64 = 2000.0;

/// Raven `MIN_SHIELD_TIME` — same double literal, the shield-alpha divisor.
/// Source: `oracle/codemp/cgame/cg_players.c:5074`
const MIN_SHIELD_TIME: f64 = 2000.0;

/// Raven `cg_customSoundNames[MAX_CUSTOM_SOUNDS]` — the base per-client custom
/// sound set, scanned until the `NULL` sentinel (`None`). The leading `*` is
/// stripped by every reader.
/// Source: `oracle/codemp/cgame/cg_players.c:25-41`
pub static cg_customSoundNames: [Option<&str>; MAX_CUSTOM_SOUNDS] = [
    Some("*death1"),
    Some("*death2"),
    Some("*death3"),
    Some("*jump1"),
    Some("*pain25"),
    Some("*pain50"),
    Some("*pain75"),
    Some("*pain100"),
    Some("*falling1"),
    Some("*choke1"),
    Some("*choke2"),
    Some("*choke3"),
    Some("*gasp"),
    Some("*land1"),
    Some("*taunt"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `cg_customDuelSoundNames[MAX_CUSTOM_DUEL_SOUNDS]`.
///
/// Raven: Used for DUEL taunts.
/// Source: `oracle/codemp/cgame/cg_players.c:142-158`
pub static cg_customDuelSoundNames: [Option<&str>; MAX_CUSTOM_DUEL_SOUNDS] = [
    // Raven: Say when acquire an enemy when didn't have one before
    Some("*anger1"),
    Some("*anger2"),
    Some("*anger3"),
    // Raven: Say when killed an enemy
    Some("*victory1"),
    Some("*victory2"),
    Some("*victory3"),
    Some("*taunt1"),
    Some("*taunt2"),
    Some("*taunt3"),
    Some("*deflect1"),
    Some("*deflect2"),
    Some("*deflect3"),
    Some("*gloat1"),
    Some("*gloat2"),
    Some("*gloat3"),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

/// Raven `RF_FORCE_ENT_ALPHA` — override shader alpha settings. Same
/// no-ported-home story as [`RF_THIRD_PERSON`].
/// Source: `oracle/codemp/cgame/tr_types.h:36`
const RF_FORCE_ENT_ALPHA: c_int = 0x00400;

/// Raven `RF_DISTORTION` — area distortion effect.
///
/// Raven: -rww
/// Source: `oracle/codemp/cgame/tr_types.h:41`
const RF_DISTORTION: c_int = 0x02000;

/// Raven `SHIPSURF_FRONT` — the impact-damage surface indices. `bg_vehicles.h`'s
/// `#define`s have no ported cross-crate home (mp_game keeps its own copy in
/// `g_vehicles.rs`, which cgame must not depend on), so they land beside their
/// reader.
/// Source: `oracle/codemp/game/bg_vehicles.h:426-429`
const SHIPSURF_FRONT: c_int = 0;
const SHIPSURF_BACK: c_int = 1;
const SHIPSURF_RIGHT: c_int = 2;
const SHIPSURF_LEFT: c_int = 3;

/// Raven `JETPACK_MODEL`.
/// Source: `oracle/codemp/cgame/cg_players.c:7889`
const JETPACK_MODEL: &str = "models/weapons2/jetpack/model.glm";

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// Raven `CG_ColorFromString` — decodes a 1..7 colour index string into an RGB
/// triple; anything outside that range comes back white.
/// Source: `oracle/codemp/cgame/cg_players.c:701-722`
pub fn CG_ColorFromString(v: &str, color: &mut vec3_t) {
    VectorClear(color);

    let val = atoi(v);

    if !(1..=7).contains(&val) {
        VectorSet(color, 1.0, 1.0, 1.0);
        return;
    }

    if val & 1 != 0 {
        color[2] = 1.0;
    }
    if val & 2 != 0 {
        color[1] = 1.0;
    }
    if val & 4 != 0 {
        color[0] = 1.0;
    }
}

/// Raven `CG_ColorFromInt` — the already-parsed twin of
/// [`CG_ColorFromString`].
/// Source: `oracle/codemp/cgame/cg_players.c:729-746`
pub fn CG_ColorFromInt(val: c_int, color: &mut vec3_t) {
    VectorClear(color);

    if !(1..=7).contains(&val) {
        VectorSet(color, 1.0, 1.0, 1.0);
        return;
    }

    if val & 1 != 0 {
        color[2] = 1.0;
    }
    if val & 2 != 0 {
        color[1] = 1.0;
    }
    if val & 4 != 0 {
        color[0] = 1.0;
    }
}

/// Raven `CG_G2SkelForModel` — the animation-set index for a ghoul2 instance,
/// found by swapping the `.gla` filename for its sibling `animation.cfg`.
/// A GLA name with no `/` leaves the index at -1.
/// Source: `oracle/codemp/cgame/cg_players.c:749-767`
pub fn CG_G2SkelForModel(ctx: &mut CgContext, g2: *mut c_void) -> c_int {
    let mut animIndex: c_int = -1;

    let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

    if let Some(slash) = GLAName.rfind('/') {
        GLAName.truncate(slash);
        GLAName.push_str("/animation.cfg");

        let traps = CgBgTraps::new(ctx.engine);
        let mut callbacks = CgGameCallbacks::new(ctx.engine);
        let filename = cstr(&GLAName);

        animIndex = BG_ParseAnimationFile(
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
            filename.as_ptr(),
            null_mut(),
            qfalse,
        );
    }

    animIndex
}

/// Raven `CG_G2EvIndexForModel` — the animation-event-set index for a ghoul2
/// instance, parsed from the `animevents.cfg` beside its `.gla`.
/// Source: `oracle/codemp/cgame/cg_players.c:770-795`
pub fn CG_G2EvIndexForModel(ctx: &mut CgContext, g2: *mut c_void, animIndex: c_int) -> c_int {
    let evtIndex: c_int = -1;

    if animIndex == -1 {
        debug_assert!(false, "shouldn't happen, bad animIndex");
        return -1;
    }

    let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

    if let Some(slash) = GLAName.rfind('/') {
        // Raven cuts just past the slash, leaving the directory with its
        // trailing separator.
        GLAName.truncate(slash + 1);

        // DEFERRED: BG_ParseAnimationEvtFile — `oracle/codemp/game/bg_panimate.c:1756-2328`.
        // The whole anim-event block sits inside `#ifndef QAGAME`, so `mp_bg`
        // deliberately did NOT port it (see that file's module doc); there is
        // no cgame-side home for it yet. Until it lands the index stays -1.
        //   evtIndex = BG_ParseAnimationEvtFile(&GLAName, animIndex, bgNumAnimEvents);
    }

    evtIndex
}

/// Raven `CG_LoadCISounds` — registers a client's custom sound set, following
/// the `sounds*.cfg` redirect (and its `f` gender marker) when the model ships
/// one, otherwise falling back to the generic male/female path.
/// Source: `oracle/codemp/cgame/cg_players.c:799-996`
pub fn CG_LoadCISounds(ctx: &mut CgContext, ci: &mut clientInfo_t, modelloaded: bool) {
    let mut f = 0;
    let mut isFemale = false;
    #[allow(unused_assignments)]
    let mut fLen;

    let dir = buf_to_string(&ci.modelName.iter().map(|&c| c as u8).collect::<Vec<u8>>());
    let skinName = buf_to_string(&ci.skinName.iter().map(|&c| c as u8).collect::<Vec<u8>>());

    // PORT-NOTE: Raven's `!ci->skinName` tests an array, so it is always false;
    // only the `Q_stricmp` half of the disjunction ever decides.
    if Q_stricmp("default", &skinName) == 0 {
        // try default sounds.cfg first
        fLen = trap::FS_FOpenFile(
            ctx.engine,
            &format!("models/players/{}/sounds.cfg", dir),
            &mut f,
            FS_READ,
        );
        if f == 0 {
            // no?  Look for _default sounds.cfg
            fLen = trap::FS_FOpenFile(
                ctx.engine,
                &format!("models/players/{}/sounds_default.cfg", dir),
                &mut f,
                FS_READ,
            );
        }
    } else {
        // use the .skin associated with this skin
        fLen = trap::FS_FOpenFile(
            ctx.engine,
            &format!("models/players/{}/sounds_{}.cfg", dir, skinName),
            &mut f,
            FS_READ,
        );
        if f == 0 {
            // fall back to default sounds
            fLen = trap::FS_FOpenFile(
                ctx.engine,
                &format!("models/players/{}/sounds.cfg", dir),
                &mut f,
                FS_READ,
            );
        }
    }

    let mut soundpath = [0u8; MAX_QPATH];

    if f != 0 {
        // PORT-NOTE: Raven reads `fLen` bytes into a MAX_QPATH buffer and then
        // writes `soundpath[fLen] = 0` — a file longer than 63 bytes overruns
        // the stack (§19 UB). The port clamps to the buffer instead.
        let fClamped = (fLen.max(0) as usize).min(MAX_QPATH - 1);
        trap::FS_Read(ctx.engine, &mut soundpath[..fClamped], f);
        soundpath[fClamped] = 0;

        let mut i = fClamped as isize;

        while i >= 0 && soundpath[i as usize] != b'\n' {
            if soundpath[i as usize] == b'f' {
                isFemale = true;
                soundpath[i as usize] = 0;
            }

            i -= 1;
        }

        i = 0;

        while soundpath[i as usize] != 0
            && soundpath[i as usize] != b'\r'
            && soundpath[i as usize] != b'\n'
        {
            i += 1;
        }
        soundpath[i as usize] = 0;

        trap::FS_FCloseFile(ctx.engine, f);
    }

    // the redirect line, once the gender marker and the line ending are cut off
    let soundpath = buf_to_string(&soundpath);

    ci.gender = if isFemale {
        gender_t::GENDER_FEMALE
    } else {
        gender_t::GENDER_MALE
    };

    trap::S_ShutUp(ctx.engine, true);

    for i in 0..MAX_CUSTOM_SOUNDS {
        let s = match cg_customSoundNames[i] {
            Some(s) => s,
            None => break,
        };

        // strip the extension because we might want .mp3's
        let soundName = COM_StripExtension(&s[1..]);

        ci.sounds[i] = 0;
        // if the model didn't load use the sounds of the default model
        if !soundpath.is_empty() {
            ci.sounds[i] = trap::S_RegisterSound(
                ctx.engine,
                &format!("sound/chars/{}/misc/{}", soundpath, soundName),
            );
        } else if modelloaded {
            ci.sounds[i] = trap::S_RegisterSound(
                ctx.engine,
                &format!("sound/chars/{}/misc/{}", dir, soundName),
            );
        }

        if ci.sounds[i] == 0 {
            // failed the load, try one out of the generic path
            let generic = if isFemale {
                DEFAULT_FEMALE_SOUNDPATH
            } else {
                DEFAULT_MALE_SOUNDPATH
            };
            ci.sounds[i] =
                trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
        }
    }

    if ctx.world.cgs.gametype >= GT_TEAM || ctx.world.cvars.cg_buildScript.integer != 0 {
        // load the siege sounds then
        for i in 0..MAX_CUSTOM_SIEGE_SOUNDS {
            let s = match bg_customSiegeSoundNames[i] {
                Some(s) => s.to_str().unwrap_or(""),
                None => break,
            };

            // strip the extension because we might want .mp3's
            let soundName = COM_StripExtension(&s[1..]);

            ci.siegeSounds[i] = 0;
            // if the model didn't load use the sounds of the default model
            //
            // PORT-NOTE: this arm's path has no `chars/` prefix, unlike the
            // other two sound loops — Raven's own asymmetry, kept.
            if !soundpath.is_empty() {
                ci.siegeSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/{}/{}", soundpath, soundName),
                );
            } else if modelloaded {
                ci.siegeSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", dir, soundName),
                );
            }

            if ci.siegeSounds[i] == 0 {
                // failed the load, try one out of the generic path
                let generic = if isFemale {
                    DEFAULT_FEMALE_SOUNDPATH
                } else {
                    DEFAULT_MALE_SOUNDPATH
                };
                ci.siegeSounds[i] =
                    trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
            }
        }
    }

    if ctx.world.cgs.gametype == GT_DUEL
        || ctx.world.cgs.gametype == GT_POWERDUEL
        || ctx.world.cvars.cg_buildScript.integer != 0
    {
        // load the Duel sounds then
        for i in 0..MAX_CUSTOM_DUEL_SOUNDS {
            let s = match cg_customDuelSoundNames[i] {
                Some(s) => s,
                None => break,
            };

            // strip the extension because we might want .mp3's
            let soundName = COM_StripExtension(&s[1..]);

            ci.duelSounds[i] = 0;
            // if the model didn't load use the sounds of the default model
            if !soundpath.is_empty() {
                ci.duelSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", soundpath, soundName),
                );
            } else if modelloaded {
                ci.duelSounds[i] = trap::S_RegisterSound(
                    ctx.engine,
                    &format!("sound/chars/{}/misc/{}", dir, soundName),
                );
            }

            if ci.duelSounds[i] == 0 {
                // failed the load, try one out of the generic path
                let generic = if isFemale {
                    DEFAULT_FEMALE_SOUNDPATH
                } else {
                    DEFAULT_MALE_SOUNDPATH
                };
                ci.duelSounds[i] =
                    trap::S_RegisterSound(ctx.engine, &format!("sound/{}/{}", generic, soundName));
            }
        }
    }

    trap::S_ShutUp(ctx.engine, false);
}

/// Raven `CG_InitG2SaberData` — builds the ghoul2 instance for one of a
/// client's sabers and bolts every blade tag on it.
/// Source: `oracle/codemp/cgame/cg_players.c:1151-1203`
pub fn CG_InitG2SaberData(ctx: &mut CgContext, saberNum: usize, ci: &mut clientInfo_t) {
    let model = buf_to_string(
        &ci.saber[saberNum]
            .model
            .iter()
            .map(|&c| c as u8)
            .collect::<Vec<u8>>(),
    );
    let skin = ci.saber[saberNum].skin;
    let saberFlags = ci.saber[saberNum].saberFlags;
    let numBlades = ci.saber[saberNum].numBlades;

    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut ci.ghoul2Weapons[saberNum] as *mut *mut c_void,
        &model,
        0,
        skin,
        0,
        0,
        0,
    );

    if !ci.ghoul2Weapons[saberNum].is_null() {
        let g2 = ci.ghoul2Weapons[saberNum];
        let mut k = 0;

        if skin != 0 {
            trap::G2API_SetSkin(ctx.engine, g2, 0, skin, skin);
        }

        if saberFlags & SFL_BOLT_TO_WRIST != 0 {
            trap::G2API_SetBoltInfo(ctx.engine, g2, 0, 3 + saberNum as c_int);
        } else {
            trap::G2API_SetBoltInfo(ctx.engine, g2, 0, saberNum as c_int);
        }

        while k < numBlades {
            let tagName = format!("*blade{}", k + 1);
            let tagBolt = trap::G2API_AddBolt(ctx.engine, g2, 0, &tagName);

            if tagBolt == -1 {
                if k == 0 {
                    // guess this is an 0ldsk3wl saber
                    let tagBolt = trap::G2API_AddBolt(ctx.engine, g2, 0, "*flash");

                    if tagBolt == -1 {
                        debug_assert!(false, "no *blade1 and no *flash on this saber");
                    }
                    break;
                }

                // Raven re-tests the same value it just branched on, so this
                // arm always takes the break.
                debug_assert!(false, "missing blade tag on this saber");
                break;
            }

            k += 1;
        }
    }
}

/// Raven `CG_CopyClientInfoModel` — moves one client's registered model, bolts
/// and sound handles onto another, duplicating (never sharing) the ghoul2
/// instance.
/// Source: `oracle/codemp/cgame/cg_players.c:1211-1281`
pub fn CG_CopyClientInfoModel(ctx: &mut CgContext, from: &clientInfo_t, to: &mut clientInfo_t) {
    _VectorCopy(from.headOffset, &mut to.headOffset);
    // to->footsteps = from->footsteps;
    to.gender = from.gender;

    to.legsModel = from.legsModel;
    to.legsSkin = from.legsSkin;
    to.torsoModel = from.torsoModel;
    to.torsoSkin = from.torsoSkin;
    //to->headModel = from->headModel;
    //to->headSkin = from->headSkin;
    to.modelIcon = from.modelIcon;

    to.newAnims = from.newAnims;

    //to->ghoul2Model = from->ghoul2Model;
    //rww - Trying to use the same ghoul2 pointer for two seperate clients == DISASTER
    //
    // PORT-NOTE: Raven's `assert(to->ghoul2Model != from->ghoul2Model)` fires
    // whenever both are NULL (the ordinary unregistered case), so it is not
    // ported as a runtime assert — retail compiles it out.

    if !to.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, to.ghoul2Model) {
        trap::G2API_CleanGhoul2Models(ctx.engine, &mut to.ghoul2Model as *mut *mut c_void);
    }
    if !from.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(ctx.engine, from.ghoul2Model) {
        trap::G2API_DuplicateGhoul2Instance(
            ctx.engine,
            from.ghoul2Model,
            &mut to.ghoul2Model as *mut *mut c_void,
        );
    }

    // Raven: Don't do this, I guess. Just leave the saber info in the original,
    // so it will be properly initialized. (The saber-copy block that used to
    // live here is commented out in the oracle, `cg_players.c:1242-1268`.)

    to.bolt_head = from.bolt_head;
    to.bolt_lhand = from.bolt_lhand;
    to.bolt_rhand = from.bolt_rhand;
    to.bolt_motion = from.bolt_motion;
    to.bolt_llumbar = from.bolt_llumbar;

    to.siegeIndex = from.siegeIndex;

    to.sounds = from.sounds;
    to.siegeSounds = from.siegeSounds;
    to.duelSounds = from.duelSounds;
}

/// Raven `CG_LoadDeferredPlayers` — arms the queue flag; the actual load
/// happens at the top of the next `CG_Player`.
/// Source: `oracle/codemp/cgame/cg_players.c:1976-1978`
pub fn CG_LoadDeferredPlayers(world: &mut CgWorld) {
    world.players.cgQueueLoad = true;
}

/// Raven `CG_InRoll` — is this entity mid-roll (either a getup-roll or a plain
/// one) with time left on the animation?
/// Source: `oracle/codemp/cgame/cg_players.c:2721-2744`
pub fn CG_InRoll(world: &CgWorld, cent: &centity_t) -> bool {
    let legsAnim = cent.currentState.legsAnim;

    if legsAnim == animNumber_t::BOTH_GETUP_BROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_L as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_BROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_L as c_int
        || legsAnim == animNumber_t::BOTH_GETUP_FROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_L as c_int
    {
        if cent.pe.legs.animationTime > world.cg.time {
            return true;
        }
    }

    false
}

/// Raven `CG_InRollAnim` — the plain-roll half of [`CG_InRoll`], with no
/// animation-time test.
/// Source: `oracle/codemp/cgame/cg_players.c:2746-2757`
pub fn CG_InRollAnim(cent: &centity_t) -> bool {
    let legsAnim = cent.currentState.legsAnim;

    legsAnim == animNumber_t::BOTH_ROLL_F as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_B as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_R as c_int
        || legsAnim == animNumber_t::BOTH_ROLL_L as c_int
}

/// Raven `CG_FirstAnimFrame` — false only when the speed scale is unchanged;
/// the frame-position tests it used to make are commented out in the oracle
/// ("I don't care where it is in the anim now, I am going to pick up from the
/// same bone frame").
/// Source: `oracle/codemp/cgame/cg_players.c:3144-3175`
pub fn CG_FirstAnimFrame(lf: &lerpFrame_t, torsoOnly: bool, speedScale: f32) -> bool {
    if torsoOnly {
        if lf.animationTorsoSpeed == speedScale {
            return false;
        }
    } else if lf.animationSpeed == speedScale {
        return false;
    }

    true
}

/// Raven `CG_G2SetBoneAngles` — straight pass-through to the trap.
///
/// Raven: we want to hold off on setting the bone angles until the end of the
/// frame, because every time we set them the entire skeleton has to be
/// reconstructed... We don't want to go with the delayed approach, we want our
/// bolt points and everything to be updated in realtime. We'll just take the
/// reconstructs and live with them. (The deferred `cgBoneAnglePostSet` path is
/// `#if 0`'d out in the oracle, `cg_players.c:3370-3391`.)
/// Source: `oracle/codemp/cgame/cg_players.c:3365-3396`
#[allow(clippy::too_many_arguments)]
pub fn CG_G2SetBoneAngles(
    ctx: &mut CgContext,
    ghoul2: *mut c_void,
    modelIndex: c_int,
    boneName: &str,
    angles: &vec3_t,
    flags: c_int,
    up: c_int,
    right: c_int,
    forward: c_int,
    modelList: Option<&mut qhandle_t>,
    blendTime: c_int,
    currentTime: c_int,
) {
    trap::G2API_SetBoneAngles(
        ctx.engine,
        ghoul2,
        modelIndex,
        boneName,
        angles,
        flags,
        up,
        right,
        forward,
        modelList,
        blendTime,
        currentTime,
    );
}

/// Raven `CG_Rag_Trace` — the ragdoll trace callback: world brushes only, so
/// the hit is reported as the world or as nothing at all.
/// Source: `oracle/codemp/cgame/cg_players.c:3407-3411`
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn CG_Rag_Trace(
    ctx: &mut CgContext,
    result: &mut trace_t,
    start: &vec3_t,
    mins: &vec3_t,
    maxs: &vec3_t,
    end: &vec3_t,
    skipNumber: c_int,
    mask: c_int,
) {
    trap::CM_BoxTrace(ctx.engine, result, start, end, mins, maxs, 0, mask);
    result.entityNum = if result.fraction != 1.0 {
        ENTITYNUM_WORLD as c_short
    } else {
        ENTITYNUM_NONE as c_short
    };
}

/// Raven `CG_RagAnimForPositioning` — picks the deadflop pose that matches
/// which way the ragdoll's pelvis is facing.
/// Source: `oracle/codemp/cgame/cg_players.c:3460-3482`
pub fn CG_RagAnimForPositioning(ctx: &mut CgContext, cent: &centity_t) -> c_int {
    let mut dir: vec3_t = [0.0; 3];
    let mut matrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;
    let turAngles = cent.turAngles;
    let lerpOrigin = cent.lerpOrigin;
    let modelScale = cent.modelScale;
    let time = ctx.world.cg.time;

    debug_assert!(!ghoul2.is_null());
    let bolt = trap::G2API_AddBolt(engine, ghoul2, 0, "pelvis");
    debug_assert!(bolt > -1);

    trap::G2API_GetBoltMatrix(
        engine,
        ghoul2,
        0,
        bolt,
        &mut matrix,
        &turAngles,
        &lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &modelScale,
    );
    BG_GiveMeVectorFromMatrix(&matrix, Eorientations::NEGATIVE_Z as c_int, &mut dir);

    if dir[2] > 0.0 {
        // facing up
        animNumber_t::BOTH_DEADFLOP2 as c_int
    } else {
        // facing down
        animNumber_t::BOTH_DEADFLOP1 as c_int
    }
}

/// Raven `CG_G2SetHeadBlink` — closes (or reopens) the eyes; one blink in
/// twenty is a wink, which shuts only the left eye and runs faster.
/// Source: `oracle/codemp/cgame/cg_players.c:3969-4006`
pub fn CG_G2SetHeadBlink(ctx: &mut CgContext, cent: &centity_t, bStart: bool) {
    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;

    let mut desiredAngles: vec3_t = [0.0; 3];
    let mut blendTime: c_int = 80;
    let mut bWink = false;
    let hReye = trap::G2API_AddBolt(engine, ghoul2, 0, "reye");
    let hLeye = trap::G2API_AddBolt(engine, ghoul2, 0, "leye");

    if hLeye == -1 {
        return;
    }

    VectorClear(&mut desiredAngles);

    if bStart {
        desiredAngles[YAW] = -50.0;
        if ctx.world.bg_state.rng.random() > 0.95 {
            bWink = true;
            blendTime /= 3;
        }
    }

    let time = ctx.world.cg.time;
    trap::G2API_SetBoneAngles(
        engine,
        ghoul2,
        0,
        "leye",
        &desiredAngles,
        BONE_ANGLES_POSTMULT,
        Eorientations::POSITIVE_Y as c_int,
        Eorientations::POSITIVE_Z as c_int,
        Eorientations::POSITIVE_X as c_int,
        None,
        blendTime,
        time,
    );

    if hReye == -1 {
        return;
    }

    if !bWink {
        trap::G2API_SetBoneAngles(
            engine,
            ghoul2,
            0,
            "reye",
            &desiredAngles,
            BONE_ANGLES_POSTMULT,
            Eorientations::POSITIVE_Y as c_int,
            Eorientations::POSITIVE_Z as c_int,
            Eorientations::POSITIVE_X as c_int,
            None,
            blendTime,
            time,
        );
    }
}

/// Raven `CG_G2SetHeadAnim` — plays a face animation on the head bone. A
/// negative `frameLerp` in the table means the animation runs backwards.
///
/// §F19: a model whose `animation.cfg` never parsed leaves `localAnimIndex` at
/// -1, which Raven indexes `bgAllAnims[]` with (a defined-nothing read one
/// entry before the table); `-1 as usize` would panic here, so we early-out.
/// Source: `oracle/codemp/cgame/cg_players.c:4013-4057`
pub fn CG_G2SetHeadAnim(ctx: &mut CgContext, cent: &centity_t, anim: c_int) {
    let engine = ctx.engine;
    let ghoul2 = cent.ghoul2;

    let blendTime: c_int = 50;
    // no skeleton parsed yet, nothing to look the animation up in
    if cent.localAnimIndex < 0 {
        return;
    }
    // `bgAllAnims[..].anims` is still `mp_bg`'s raw `animation_t*` table; the
    // whole workspace reads it through a pointer (see `g_vehicles.rs`,
    // `NPC_reactions.rs`, ui's `UI_ParseAnimationFile`).
    let animations = ctx.world.bg_state.bgAllAnims[cent.localAnimIndex as usize].anims;
    let mut animFlags: c_int = BONE_ANIM_OVERRIDE; //| BONE_ANIM_BLEND;

    // animSpeed is 1.0 if the frameLerp (ms/frame) is 50 (20 fps).
    let cg_timescale = ctx.world.cvars.cg_timescale.value;
    let timeScaleMod: f32 = if cg_timescale != 0.0 {
        (1.0f64 / cg_timescale as f64) as f32
    } else {
        1.0
    };

    // SAFETY: `anims` is the animation table `BG_ParseAnimationFile` allocated
    // for this skeleton; Raven indexes it unchecked and so do we.
    let (numFrames, firstFrame, frameLerp) = unsafe {
        let a = &*animations.offset(anim as isize);
        (a.numFrames as c_int, a.firstFrame as c_int, a.frameLerp)
    };

    let animSpeed = 50.0f32 / frameLerp as f32 * timeScaleMod;
    let firstFrameOut;
    let lastFrame;

    if numFrames <= 0 {
        return;
    }
    if anim == animNumber_t::FACE_DEAD as c_int {
        animFlags |= BONE_ANIM_OVERRIDE_FREEZE;
    }
    // animSpeed is 1.0 if the frameLerp (ms/frame) is 50 (20 fps).
    if animSpeed < 0.0 {
        // play anim backwards
        lastFrame = firstFrame - 1;
        firstFrameOut = (numFrames - 1) + firstFrame;
    } else {
        firstFrameOut = firstFrame;
        lastFrame = numFrames + firstFrame;
    }

    // Raven's "are we already animating the head" early-out is commented out
    // (`cg_players.c:4046-4051`), so the anim is always set.
    trap::G2API_SetBoneAnim(
        engine,
        ghoul2,
        0,
        "face",
        firstFrameOut,
        lastFrame,
        animFlags,
        animSpeed,
        ctx.world.cg.time,
        -1.0,
        blendTime,
    );
}

/// Raven `CG_PlayerFloatSprite` — the sprite that floats above a player's head
/// (chat bubble, flag icon). On your own body it renders in mirrors only.
/// Source: `oracle/codemp/cgame/cg_players.c:4505-4527`
pub fn CG_PlayerFloatSprite(ctx: &mut CgContext, cent: &centity_t, shader: qhandle_t) {
    let rf;

    // PORT-NOTE: Raven derefs `cg.snap` unguarded; `cg_t.snap` is still the C1
    // a missing snapshot reads as "not us"
    let ourClientNum = ctx.world.cg.snap_ref().map_or(-1, |snap| snap.ps.clientNum);

    if cent.currentState.number == ourClientNum && ctx.world.cg.renderingThirdPerson == 0 {
        rf = RF_THIRD_PERSON; // only show in mirrors
    } else {
        rf = 0;
    }

    let mut ent = refEntity_t::zeroed();
    _VectorCopy(cent.lerpOrigin, &mut ent.origin);
    ent.origin[2] += 48.0;
    ent.reType = refEntityType_t::RT_SPRITE;
    ent.customShader = shader;
    ent.radius = 10.0;
    ent.renderfx = rf;
    ent.shaderRGBA[0] = 255;
    ent.shaderRGBA[1] = 255;
    ent.shaderRGBA[2] = 255;
    ent.shaderRGBA[3] = 255;
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_PlayerSplash` — the water-wake quad under a player whose feet are
/// in liquid and whose head is not.
/// Source: `oracle/codemp/cgame/cg_players.c:4717-4795`
pub fn CG_PlayerSplash(ctx: &mut CgContext, cent: &centity_t) {
    let engine = ctx.engine;
    let mut start: vec3_t = [0.0; 3];
    let mut end: vec3_t = [0.0; 3];
    let mut trace = trace_t::zeroed();
    let mut verts = [polyVert_t {
        xyz: [0.0; 3],
        st: [0.0; 2],
        modulate: [0; 4],
    }; 4];

    if ctx.world.cvars.cg_shadows.integer == 0 {
        return;
    }

    _VectorCopy(cent.lerpOrigin, &mut end);
    end[2] -= 24.0;

    // if the feet aren't in liquid, don't make a mark
    // this won't handle moving water brushes, but they wouldn't draw right anyway...
    let contents = trap::CM_PointContents(engine, &end, 0);
    if contents & (CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) == 0 {
        return;
    }

    _VectorCopy(cent.lerpOrigin, &mut start);
    start[2] += 32.0;

    // if the head isn't out of liquid, don't make a mark
    let contents = trap::CM_PointContents(engine, &start, 0);
    if contents & (CONTENTS_SOLID | CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA) != 0 {
        return;
    }

    // trace down to find the surface
    //
    // PORT-NOTE: Raven passes NULL mins/maxs; `CM_Trace` substitutes
    // `vec3_origin` for a NULL bound (`cm_trace.cpp:1603-1610`), so the zero
    // vectors below are the same point trace.
    trap::CM_BoxTrace(
        engine,
        &mut trace,
        &start,
        &end,
        &vec3_origin,
        &vec3_origin,
        0,
        CONTENTS_WATER | CONTENTS_SLIME | CONTENTS_LAVA,
    );

    if trace.fraction == 1.0 {
        return;
    }

    // create a mark polygon
    _VectorCopy(trace.endpos, &mut verts[0].xyz);
    verts[0].xyz[0] -= 32.0;
    verts[0].xyz[1] -= 32.0;
    verts[0].st[0] = 0.0;
    verts[0].st[1] = 0.0;
    verts[0].modulate[0] = 255;
    verts[0].modulate[1] = 255;
    verts[0].modulate[2] = 255;
    verts[0].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[1].xyz);
    verts[1].xyz[0] -= 32.0;
    verts[1].xyz[1] += 32.0;
    verts[1].st[0] = 0.0;
    verts[1].st[1] = 1.0;
    verts[1].modulate[0] = 255;
    verts[1].modulate[1] = 255;
    verts[1].modulate[2] = 255;
    verts[1].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[2].xyz);
    verts[2].xyz[0] += 32.0;
    verts[2].xyz[1] += 32.0;
    verts[2].st[0] = 1.0;
    verts[2].st[1] = 1.0;
    verts[2].modulate[0] = 255;
    verts[2].modulate[1] = 255;
    verts[2].modulate[2] = 255;
    verts[2].modulate[3] = 255;

    _VectorCopy(trace.endpos, &mut verts[3].xyz);
    verts[3].xyz[0] += 32.0;
    verts[3].xyz[1] -= 32.0;
    verts[3].st[0] = 1.0;
    verts[3].st[1] = 0.0;
    verts[3].modulate[0] = 255;
    verts[3].modulate[1] = 255;
    verts[3].modulate[2] = 255;
    verts[3].modulate[3] = 255;

    trap::R_AddPolyToScene(engine, ctx.world.cgs.media.wakeMarkShader, &verts);
}

/// Raven `CG_PlayerShieldHit` — arms the personal-shield flash on an entity and
/// points it back down the incoming damage direction. Bigger hits hold longer,
/// capped at two seconds.
/// Source: `oracle/codemp/cgame/cg_players.c:5077-5104`
pub fn CG_PlayerShieldHit(world: &mut CgWorld, entitynum: c_int, dir: &mut vec3_t, amount: c_int) {
    // PORT-NOTE: Raven guards with `tr_types.h`'s MAX_ENTITIES (2048), twice
    // the 1024-entry `cg_entities` array, so 1024..2047 is an out-of-bounds
    // read (§19). The port guards at the array bound instead.
    if entitynum < 0 || entitynum >= MAX_GENTITIES as c_int {
        return;
    }

    let time = if amount > 100 {
        (world.cg.time as f64 + MAX_SHIELD_TIME) as c_int // 2 sec.
    } else {
        world.cg.time + 500 + amount * 15
    };

    let cent = world.entity_mut(entitynum as usize);

    if time > cent.damageTime {
        cent.damageTime = time;
        let incoming = *dir;
        _VectorScale(incoming, -1.0, dir);
        vectoangles(*dir, &mut cent.damageAngles);
    }
}

/// Raven `CG_DrawPlayerShield` — the half-shield shell over the last hit
/// direction, fading out and swelling as it goes. Never drawn on a corpse.
/// Source: `oracle/codemp/cgame/cg_players.c:5107-5142`
pub fn CG_DrawPlayerShield(ctx: &mut CgContext, cent: &centity_t, origin: &vec3_t) {
    // Don't draw the shield when the player is dead.
    if cent.currentState.eFlags & EF_DEAD != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();

    _VectorCopy(*origin, &mut ent.origin);
    ent.origin[2] += 10.0;
    AnglesToAxis(cent.damageAngles, ent.axis.as_mut_ptr());

    let jitter = ctx.world.bg_state.rng.random() * 16.0;
    let mut alpha = (255.0f64 * ((cent.damageTime - ctx.world.cg.time) as f64 / MIN_SHIELD_TIME)
        + jitter as f64) as c_int;
    if alpha > 255 {
        alpha = 255;
    }

    // Make it bigger, but tighter if more solid
    let scale = (1.4f64 - (alpha as f32 as f64 * (0.4f64 / 255.0))) as f32; // Range from 1.0 to 1.4
    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale, &mut ent.axis[2]);

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = ctx.world.cgs.media.halfShieldShader;
    ent.shaderRGBA[0] = alpha as u8;
    ent.shaderRGBA[1] = alpha as u8;
    ent.shaderRGBA[2] = alpha as u8;
    ent.shaderRGBA[3] = 255;
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_LightVerts` — lights a poly's vertices from the world lightgrid;
/// a face pointing away from the light gets the ambient term only.
///
/// Raven's `numVerts` is dropped in favour of the slice length — callers hand
/// over exactly the vertices they mean to light.
/// Source: `oracle/codemp/cgame/cg_players.c:5167-5207`
pub fn CG_LightVerts(ctx: &mut CgContext, normal: &vec3_t, verts: &mut [polyVert_t]) -> bool {
    let mut ambientLight: vec3_t = [0.0; 3];
    let mut lightDir: vec3_t = [0.0; 3];
    let mut directedLight: vec3_t = [0.0; 3];

    let point = verts[0].xyz;
    trap::R_LightForPoint(
        ctx.engine,
        &point,
        &mut ambientLight,
        &mut directedLight,
        &mut lightDir,
    );

    for vert in verts.iter_mut() {
        let incoming = _DotProduct(*normal, lightDir);
        if incoming <= 0.0 {
            vert.modulate[0] = ambientLight[0] as u8;
            vert.modulate[1] = ambientLight[1] as u8;
            vert.modulate[2] = ambientLight[2] as u8;
            vert.modulate[3] = 255;
            continue;
        }

        let mut j = (ambientLight[0] + incoming * directedLight[0]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[0] = j as u8;

        let mut j = (ambientLight[1] + incoming * directedLight[1]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[1] = j as u8;

        let mut j = (ambientLight[2] + incoming * directedLight[2]) as c_int;
        if j > 255 {
            j = 255;
        }
        vert.modulate[2] = j as u8;

        vert.modulate[3] = 255;
    }

    true
}

/// Raven `CG_RGBForSaberColor` — the blade tint for one of the six saber
/// colours. Anything outside the six leaves `rgb` exactly as the caller had it.
/// Source: `oracle/codemp/cgame/cg_players.c:5209-5232`
pub fn CG_RGBForSaberColor(color: saber_colors_t, rgb: &mut vec3_t) {
    match color {
        SABER_RED => VectorSet(rgb, 1.0, 0.2, 0.2),
        SABER_ORANGE => VectorSet(rgb, 1.0, 0.5, 0.1),
        SABER_YELLOW => VectorSet(rgb, 1.0, 1.0, 0.2),
        SABER_GREEN => VectorSet(rgb, 0.2, 1.0, 0.2),
        SABER_BLUE => VectorSet(rgb, 0.2, 0.4, 1.0),
        SABER_PURPLE => VectorSet(rgb, 0.9, 0.2, 1.0),
        _ => {}
    }
}

/// Raven `CG_GetTagWorldPosition` — where a model's tag ends up once the
/// model's own origin and axis are applied.
///
/// Raven: Can pass in NULL for the axis.
/// Source: `oracle/codemp/cgame/cg_players.c:5437-5456`
pub fn CG_GetTagWorldPosition(
    ctx: &mut CgContext,
    model: &refEntity_t,
    tag: &str,
    pos: &mut vec3_t,
    axis: Option<&mut [vec3_t; 3]>,
) {
    let mut orientation = orientation_t {
        origin: [0.0; 3],
        axis: [[0.0; 3]; 3],
    };

    // Get the requested tag
    trap::R_LerpTag(
        ctx.engine,
        &mut orientation,
        model.hModel,
        model.oldframe,
        model.frame,
        1.0 - model.backlerp,
        tag,
    );

    _VectorCopy(model.origin, pos);
    for i in 0..3 {
        let running = *pos;
        _VectorMA(running, orientation.origin[i], model.axis[i], pos);
    }

    if let Some(axis) = axis {
        MatrixMultiply(&orientation.axis, &model.axis, axis);
    }
}

/// Raven `CG_G2TraceCollide` — re-runs a landed trace against the hit entity's
/// ghoul2 skeleton. A miss rewrites the trace as "hit nothing"; a hit moves the
/// endpoint and normal onto the collided polygon.
/// Source: `oracle/codemp/cgame/cg_players.c:5632-5689`
pub fn CG_G2TraceCollide(
    ctx: &mut CgContext,
    tr: &mut trace_t,
    mins: Option<&vec3_t>,
    maxs: Option<&vec3_t>,
    lastValidStart: &vec3_t,
    lastValidEnd: &vec3_t,
) -> bool {
    let mut angles: vec3_t = [0.0; 3];
    let mut fRadius: f32 = 0.0;

    if let (Some(mins), Some(maxs)) = (mins, maxs) {
        if mins[0] != 0.0 || maxs[0] != 0.0 {
            fRadius = (maxs[0] - mins[0]) / 2.0;
        }
    }

    // Raven's `memset(&G2Trace, 0, sizeof(G2Trace))` then the -1 entity-number
    // sweep, in one go.
    let mut G2Trace: G2Trace_t = [CollisionRecord_t {
        mDistance: 0.0,
        mEntityNum: -1,
        mModelIndex: 0,
        mPolyIndex: 0,
        mSurfaceIndex: 0,
        mCollisionPosition: [0.0; 3],
        mCollisionNormal: [0.0; 3],
        mFlags: 0,
        mMaterial: 0,
        mLocation: 0,
        mBarycentricI: 0.0,
        mBarycentricJ: 0.0,
    }; MAX_G2_COLLISIONS];

    let engine = ctx.engine;
    let time = ctx.world.cg.time;
    let g2TraceLod = ctx.world.cvars.cg_g2TraceLod.integer;
    let optvehtrace = ctx.world.cvars.cg_optvehtrace.integer;

    let g2Hit = ctx.world.entity(tr.entityNum as usize);
    let ghoul2 = g2Hit.ghoul2;
    let number = g2Hit.currentState.number;
    let eType = g2Hit.currentState.eType;
    let NPC_class = g2Hit.currentState.NPC_class;
    let hasVehicle = g2Hit.m_pVehicle.is_some();
    let lerpOrigin = g2Hit.lerpOrigin;
    let modelScale = g2Hit.modelScale;
    let yaw = g2Hit.lerpAngles[YAW];

    if ghoul2.is_null() {
        return false;
    }

    angles[ROLL] = 0.0;
    angles[PITCH] = 0.0;
    angles[YAW] = yaw;

    if optvehtrace != 0
        && eType == entityType_t::ET_NPC as c_int
        && NPC_class == class_t::CLASS_VEHICLE as c_int
        && hasVehicle
    {
        trap::G2API_CollisionDetectCache(
            engine,
            &mut G2Trace,
            ghoul2,
            &angles,
            &lerpOrigin,
            time,
            number,
            lastValidStart,
            lastValidEnd,
            &modelScale,
            0,
            g2TraceLod,
            fRadius,
        );
    } else {
        trap::G2API_CollisionDetect(
            engine,
            &mut G2Trace,
            ghoul2,
            &angles,
            &lerpOrigin,
            time,
            number,
            lastValidStart,
            lastValidEnd,
            &modelScale,
            0,
            g2TraceLod,
            fRadius,
        );
    }

    if G2Trace[0].mEntityNum != number {
        tr.fraction = 1.0;
        tr.entityNum = ENTITYNUM_NONE as c_short;
        tr.startsolid = 0;
        tr.allsolid = 0;
        false
    } else {
        // Yay!
        _VectorCopy(G2Trace[0].mCollisionPosition, &mut tr.endpos);
        _VectorCopy(G2Trace[0].mCollisionNormal, &mut tr.plane.normal);
        true
    }
}

/// Raven `CG_AddGhoul2Mark` — puts a gore decal on a ghoul2 skin along the
/// start→end ray. Bails when the instance is already carrying `cg_ghoul2Marks`
/// decals, or when the ray is too short to have a direction.
///
/// DEFERRED: the `SSkinGoreData` fill and the `trap_G2API_AddSkinGore` call —
/// `SSkinGoreData` (`oracle/codemp/game/q_shared.h:3111-3144`) has no
/// mp_cgame-reachable Rust home: the only MP definition lives in the engine
/// ghoul2 crate, which this module must not depend on, and `trap.rs`'s wrapper
/// therefore takes the gore block as an opaque `*mut c_void`. Everything with a
/// side effect (the mark-count early-out, the `flrand` draw, Raven's argument-
/// swapped scale copy, the ray-length early-out) is transcribed; only the
/// engine hand-off is missing, so no mark is added yet.
/// Source: `oracle/codemp/cgame/cg_players.c:5738-5796`
#[allow(clippy::too_many_arguments)]
pub fn CG_AddGhoul2Mark(
    ctx: &mut CgContext,
    shader: c_int,
    size: f32,
    start: &vec3_t,
    end: &vec3_t,
    entnum: c_int,
    entposition: &vec3_t,
    entangle: f32,
    ghoul2: *mut c_void,
    scale: &mut vec3_t,
    lifeTime: c_int,
) {
    debug_assert!(!ghoul2.is_null());

    if trap::G2API_GetNumGoreMarks(ctx.engine, ghoul2, 0) >= ctx.world.cvars.cg_ghoul2Marks.integer
    {
        // you've got too many marks already
        return;
    }

    // Raven's field fill, in its order — the `theta` draw moves the shared RNG
    // and so has to happen even while the block itself can't be handed over.
    let theta = ctx.world.bg_state.rng.flrand(0.0, 6.28);

    // PORT-NOTE: Raven's else arm is `VectorCopy(goreSkin.scale, scale)` — the
    // arguments are the wrong way round, so it copies the memset-zeroed
    // `goreSkin.scale` OVER the caller's vector and leaves the gore scale at
    // zero. Kept: the caller really does get stomped.
    let goreScale: vec3_t = if scale[0] == 0.0 && scale[1] == 0.0 && scale[2] == 0.0 {
        [1.0, 1.0, 1.0]
    } else {
        _VectorCopy([0.0; 3], scale);
        [0.0; 3]
    };

    let mut rayDirection: vec3_t = [0.0; 3];
    _VectorSubtract(*end, *start, &mut rayDirection);
    if VectorNormalize(&mut rayDirection) < 0.1 {
        return;
    }

    // DEFERRED: SSkinGoreData — `oracle/codemp/game/q_shared.h:3111-3144`.
    // The remaining fields Raven sets are growDuration -1, goreScaleStartFraction
    // 1.0, frontFaces/backFaces true, baseModelOnly false, lifeTime, currentTime
    // cg.time, entNum, SSize/TSize size, theta, shader, scale, hitLocation start,
    // rayDirection, position entposition and angles[YAW] entangle; then
    // `trap_G2API_AddSkinGore(ghoul2, &goreSkin)`.
    let _ = (
        shader,
        size,
        start,
        entnum,
        entposition,
        entangle,
        lifeTime,
        theta,
        goreScale,
        rayDirection,
    );
}

/// Raven `CG_IsMindTricked` — is this client hidden from us by mind trick? The
/// four bitmask args are the four 16-client blocks of `trickedentindex`; force
/// sight beats the trick outright.
/// Source: `oracle/codemp/cgame/cg_players.c:6482-6518`
pub fn CG_IsMindTricked(
    world: &CgWorld,
    trickIndex1: c_int,
    trickIndex2: c_int,
    trickIndex3: c_int,
    trickIndex4: c_int,
    client: c_int,
) -> bool {
    let checkIn;
    let mut sub = 0;

    if world.entity(client as usize).currentState.forcePowersActive & (1 << FP_SEE) != 0 {
        return false;
    }

    if client > 47 {
        checkIn = trickIndex4;
        sub = 48;
    } else if client > 31 {
        checkIn = trickIndex3;
        sub = 32;
    } else if client > 15 {
        checkIn = trickIndex2;
        sub = 16;
    } else {
        checkIn = trickIndex1;
    }

    checkIn & (1 << (client - sub)) != 0
}

/// Raven `CG_DrawPlayerSphere` — the force-effect shell (invulnerability,
/// ysalamiri, enlightenment…) drawn around a player. Never on a corpse, and
/// never on your own body in first person past the first pass.
/// Source: `oracle/codemp/cgame/cg_players.c:6522-6626`
pub fn CG_DrawPlayerSphere(
    ctx: &mut CgContext,
    cent: &centity_t,
    origin: &vec3_t,
    scale: f32,
    shader: c_int,
) {
    // Don't draw the shield when the player is dead.
    if cent.currentState.eFlags & EF_DEAD != 0 {
        return;
    }

    let mut ent = refEntity_t::zeroed();
    let mut ang: vec3_t = [0.0; 3];
    let mut viewDir: vec3_t = [0.0; 3];

    _VectorCopy(*origin, &mut ent.origin);
    ent.origin[2] += 9.0;

    let vieworg = ctx.world.cg.refdef.vieworg;
    _VectorSubtract(ent.origin, vieworg, &mut ent.axis[0]);
    let vLen = VectorLength(ent.axis[0]);
    if vLen <= 0.1 {
        // Entity is right on vieworg.  quit.
        return;
    }

    _VectorCopy(ent.axis[0], &mut viewDir);
    VectorInverse(&mut viewDir);
    VectorNormalize(&mut viewDir);

    vectoangles(ent.axis[0], &mut ang);
    ang[ROLL] += 180.0;
    ang[PITCH] += 180.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale, &mut ent.axis[2]);

    ent.nonNormalizedAxes = qtrue;

    ent.hModel = ctx.world.cgs.media.halfShieldModel;
    ent.customShader = shader;

    trap::R_AddRefEntityToScene(ctx.engine, &ent);

    if ctx.world.cg.renderingThirdPerson == 0
        && cent.currentState.number == ctx.world.cg.predictedPlayerState.clientNum
    {
        // don't do the rest then
        return;
    }
    if ctx.world.cvars.cg_renderToTextureFX.integer == 0 {
        return;
    }

    ang[PITCH] -= 180.0;
    AnglesToAxis(ang, ent.axis.as_mut_ptr());

    let axis0 = ent.axis[0];
    _VectorScale(axis0, scale * 0.5, &mut ent.axis[0]);
    let axis1 = ent.axis[1];
    _VectorScale(axis1, scale * 0.5, &mut ent.axis[1]);
    let axis2 = ent.axis[2];
    _VectorScale(axis2, scale * 0.5, &mut ent.axis[2]);

    ent.renderfx = RF_DISTORTION | RF_FORCE_ENT_ALPHA;
    if shader == ctx.world.cgs.media.invulnerabilityShader {
        // ok, ok, this is a little hacky. sorry!
        ent.shaderRGBA[0] = 0;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 100;
    } else if shader == ctx.world.cgs.media.ysalimariShader {
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 100;
    } else if shader == ctx.world.cgs.media.endarkenmentShader {
        ent.shaderRGBA[0] = 100;
        ent.shaderRGBA[1] = 0;
        ent.shaderRGBA[2] = 0;
        ent.shaderRGBA[3] = 20;
    } else if shader == ctx.world.cgs.media.enlightenmentShader {
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;
        ent.shaderRGBA[3] = 20;
    } else {
        // ysal red/blue, boon
        ent.shaderRGBA[0] = 255;
        ent.shaderRGBA[1] = 255;
        ent.shaderRGBA[2] = 255;
        ent.shaderRGBA[3] = 20;
    }

    ent.radius = 256.0;

    let entOrigin = ent.origin;
    _VectorMA(entOrigin, 40.0, viewDir, &mut ent.origin);

    ent.customShader = trap::R_RegisterShader(ctx.engine, "effects/refract_2");
    trap::R_AddRefEntityToScene(ctx.engine, &ent);
}

/// Raven `CG_AddLightningBeam` — the wobbling force-lightning bezier between
/// two points; the "chaos" that bends the control points is three sine waves off
/// `cg.time`, so it is barely chaotic at all.
/// Source: `oracle/codemp/cgame/cg_players.c:6628-6697`
pub fn CG_AddLightningBeam(ctx: &mut CgContext, start: &vec3_t, end: &vec3_t) {
    let mut dir: vec3_t = [0.0; 3];
    let mut chaos: vec3_t = [0.0; 3];
    let mut c1: vec3_t = [0.0; 3];
    let mut c2: vec3_t = [0.0; 3];
    let mut v1: vec3_t = [0.0; 3];
    let mut v2: vec3_t = [0.0; 3];

    _VectorSubtract(*end, *start, &mut dir);
    let len = VectorNormalize(&mut dir);

    // Get the base control points, we'll work from there
    _VectorMA(*start, 0.3333 * len, dir, &mut c1);
    _VectorMA(*start, 0.6666 * len, dir, &mut c2);

    // get some chaos values that really aren't very chaotic :)
    let time = ctx.world.cg.time;
    let s1 = (((time as f32 * 0.005) as f64).sin() * 2.0
        + ctx.world.bg_state.rng.crandom() * 0.2f32 as f64) as f32;
    let s2 = ((time as f32 * 0.001) as f64).sin() as f32;
    let s3 = ((time as f32 * 0.011) as f64).sin() as f32;

    VectorSet(
        &mut chaos,
        len * 0.01 * s1,
        len * 0.02 * s2,
        len * 0.04 * (s1 + s2 + s3),
    );

    let c1in = c1;
    _VectorAdd(c1in, chaos, &mut c1);
    _VectorScale(chaos, 4.0, &mut v1);

    VectorSet(
        &mut chaos,
        -len * 0.02 * s3,
        len * 0.01 * (s1 * s2),
        -len * 0.02 * (s1 + s2 * s3),
    );

    let c2in = c2;
    _VectorAdd(c2in, chaos, &mut c2);
    _VectorScale(chaos, 2.0, &mut v2);

    VectorSet(&mut chaos, 1.0, 1.0, 1.0);

    // PORT-NOTE: `v1`, `v2` and that last `chaos` set are never read again in
    // Raven — dead stores, kept so the arithmetic reads like the oracle.
    let _ = (v1, v2, chaos);

    let sRGB: vec3_t = [255.0, 255.0, 255.0];
    let shader = trap::R_RegisterShader(ctx.engine, "gfx/misc/electric2");

    let mut b = addbezierArgStruct_t {
        start: *start,
        end: *end,
        control1: c1,
        control1Vel: vec3_origin,
        control2: c2,
        control2Vel: vec3_origin,
        size1: 6.0,
        size2: 6.0,
        sizeParm: 0.0,
        alpha1: 0.0,
        alpha2: 0.2,
        alphaParm: 0.5,
        sRGB,
        eRGB: sRGB,
        rgbParm: 0.0,
        killTime: 50,
        shader,
        flags: 0x00000001, // FX_ALPHA_LINEAR
    };

    trap::FX_AddBezier(ctx.engine, &mut b);
}

/// Raven `CG_ThereIsAMaster` — is anyone carrying the Jedi Master mantle right
/// now?
/// Source: `oracle/codemp/cgame/cg_players.c:6744-6762`
pub fn CG_ThereIsAMaster(world: &CgWorld) -> bool {
    for i in 0..MAX_CLIENTS {
        if world.entity(i).currentState.isJediMaster != 0 {
            return true;
        }
    }

    false
}

/// Raven `CG_HandleAppendedSkin` — splits a `model*skin` string in place and
/// registers the skin out of the model's own folder. `modelName` comes back cut
/// at the `*`; a `|` in the skin name means a three-part skin.
/// Source: `oracle/codemp/cgame/cg_players.c:6817-6868`
pub fn CG_HandleAppendedSkin(ctx: &mut CgContext, modelName: &mut String) -> qhandle_t {
    let mut skinID: qhandle_t = 0;

    // see if it has a skin name
    let p = modelName.rfind('*');

    if let Some(p) = p {
        // found a *, we should have a model name before it and a skin name after it.
        let skinName = modelName[p + 1..].to_string();
        modelName.truncate(p);

        if !skinName.is_empty() {
            // got it, register the skin under the model path.
            let mut baseFolder = modelName.clone();

            // go back to the first /, should be the path point
            if let Some(slash) = baseFolder.rfind('/') {
                // got it.. terminate at the slash and register.
                baseFolder.truncate(slash);

                let useSkinName = if skinName.contains('|') {
                    // three part skin
                    format!("{}/|{}", baseFolder, skinName)
                } else {
                    format!("{}/model_{}.skin", baseFolder, skinName)
                };

                skinID = trap::R_RegisterSkin(ctx.engine, &useSkinName);
            }
        }
    }

    skinID
}

/// Raven `CG_CacheG2AnimInfo` — precaches a model's skeleton and animation set
/// by building a throwaway ghoul2 instance, parsing the `animation.cfg` beside
/// its `.gla`, and cleaning the instance up again.
/// Source: `oracle/codemp/cgame/cg_players.c:6876-6939`
pub fn CG_CacheG2AnimInfo(ctx: &mut CgContext, modelName: &str) {
    let mut useModel = modelName.to_string();

    if modelName.starts_with('$') {
        // it's a vehicle name actually, let's precache the whole vehicle
        let traps = CgBgTraps::new(ctx.engine);
        let mut callbacks = CgGameCallbacks::new(ctx.engine);

        // both bg fns stomp the `$name` they were handed with their answer, so
        // each gets its own copy of it the way Raven's two stack buffers do
        let mut modelBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        let mut skinBuf: [c_char; MAX_QPATH] = [0; MAX_QPATH];
        Q_strncpyz(&mut modelBuf, modelName, MAX_QPATH);
        Q_strncpyz(&mut skinBuf, modelName, MAX_QPATH);

        BG_GetVehicleModelName(
            modelBuf.as_mut_ptr(),
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );
        BG_GetVehicleSkinName(
            skinBuf.as_mut_ptr(),
            &mut ctx.world.bg_state,
            &traps,
            &mut callbacks,
        );

        let vehModel = buf_to_string(&modelBuf.map(|c| c as u8));
        let useSkin = buf_to_string(&skinBuf.map(|c| c as u8));

        if !useSkin.is_empty() {
            // use a custom skin
            trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{vehModel}/model_{useSkin}.skin"),
            );
        } else {
            trap::R_RegisterSkin(
                ctx.engine,
                &format!("models/players/{vehModel}/model_default.skin"),
            );
        }
        useModel = format!("models/players/{vehModel}/model.glm");
    }

    let mut g2: *mut c_void = null_mut();
    trap::G2API_InitGhoul2Model(
        ctx.engine,
        &mut g2 as *mut *mut c_void,
        &useModel,
        0,
        0,
        0,
        0,
        0,
    );

    if !g2.is_null() {
        let mut animIndex: c_int = -1;
        let mut GLAName = trap::G2API_GetGLAName(ctx.engine, g2, 0, MAX_QPATH);

        if let Some(slash) = GLAName.rfind('/') {
            GLAName.truncate(slash);
            GLAName.push_str("/animation.cfg");

            let traps = CgBgTraps::new(ctx.engine);
            let mut callbacks = CgGameCallbacks::new(ctx.engine);
            let filename = cstr(&GLAName);

            animIndex = BG_ParseAnimationFile(
                &mut ctx.world.bg_state,
                &traps,
                &mut callbacks,
                filename.as_ptr(),
                null_mut(),
                qfalse,
            );
        }

        if animIndex != -1 {
            // DEFERRED: BG_ParseAnimationEvtFile —
            // `oracle/codemp/game/bg_panimate.c:2169-2328`. Raven cuts
            // `originalModelName` (its pre-GLA copy of `useModel`) back to its
            // directory and registers the model's animation events against
            // `bgNumAnimEvents`. The whole event-file block sits inside
            // `#ifndef QAGAME`, so `mp_bg` deliberately did NOT port it and
            // there is no cgame home for it yet.
            //   BG_ParseAnimationEvtFile(originalModelName, animIndex, bgNumAnimEvents);
        }

        // Now free the temp instance
        trap::G2API_CleanGhoul2Models(ctx.engine, &mut g2 as *mut *mut c_void);
    }
}

/// Raven `CG_RegisterVehicleAssets` — precaches a vehicle's effect set.
///
/// The whole body is commented out in the oracle (every `trap_FX_RegisterEffect`
/// line), so this really is a no-op; it is kept because `CG_Player`'s vehicle
/// path still calls it (`cg_players.c:7051`).
/// Source: `oracle/codemp/cgame/cg_players.c:6941-6986`
pub fn CG_RegisterVehicleAssets(_pVeh: VehicleId) {}

/// Raven `CG_CreateSurfaceDebris` — blows the named ghoul2 surface off a ship,
/// playing the debris effect at the surface's bolt (or at the ship's origin when
/// the surface has no bolt).
///
/// DEFERRED past the vehicle presence test: the five wing/nose arms also read
/// `m_pVehicle->m_pVehicleInfo->i{R,L}WingFX`/`iNoseFX` for the thrown-part
/// effect, and DEC-46.2's `Option<VehicleId>` carries only the vehicle cent's
/// entity number until the `Vehicle_t` referent pool lands. `lostPartFX` stays
/// 0, so the thrown ship part at the end never fires; the debris effect itself
/// plays normally.
/// Source: `oracle/codemp/cgame/cg_players.c:7266-7361`
pub fn CG_CreateSurfaceDebris(
    ctx: &mut CgContext,
    cent: &centity_t,
    surfNum: c_int,
    fxID: c_int,
    throwPart: bool,
) {
    let lostPartFX: c_int = 0;
    let mut b: c_int = -1;
    let mut v: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };
    let mut surfName: Option<&str> = None;

    if surfNum > 0 {
        surfName = bgToggleableSurfaces
            .get(surfNum as usize)
            .and_then(|s| *s)
            .map(|s| s.to_str().unwrap_or(""));
    }

    if cent.ghoul2.is_null() {
        // oh no
        return;
    }

    let engine = ctx.engine;

    // PORT-NOTE: Raven indexes `bgToggleableSurfaceDebris[surfNum]` before it
    // tests `surfNum == -1` further down, so a -1 surface reads one slot before
    // the table (§19 UB). The port reads through a bounds check: an
    // out-of-range surface matches no arm and falls into the origin path, which
    // is exactly where Raven's own `surfNum == -1` test sends it.
    let debris = bgToggleableSurfaceDebris
        .get(surfNum as usize)
        .copied()
        .unwrap_or(-1);

    // let's add the surface as a bolt so we can get the base point of it
    if debris == 3 {
        // right wing flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*r_wingdamage");
    } else if debris == 4 {
        // left wing flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*l_wingdamage");
    } else if debris == 5 {
        // right wing flame 2
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*r_wingdamage");
    } else if debris == 6 {
        // left wing flame 2
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*l_wingdamage");
    } else if debris == 7 {
        // nose flame
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, "*nosedamage");
    } else if let Some(surfName) = surfName {
        b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, surfName);
    }

    if b == -1 || surfNum == -1 {
        // couldn't find this surface apparently, so play on origin?
        _VectorCopy(cent.lerpOrigin, &mut v);
        AngleVectors(cent.lerpAngles, Some(&mut d), None, None);
        VectorNormalize(&mut d);
    } else {
        // now let's get the position and direction of this surface and make a big explosion
        let time = ctx.world.cg.time;
        trap::G2API_GetBoltMatrix(
            engine,
            cent.ghoul2,
            0,
            b,
            &mut boltMatrix,
            &cent.lerpAngles,
            &cent.lerpOrigin,
            time,
            Some(&mut ctx.world.cgs.gameModels[0]),
            &cent.modelScale,
        );
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut v);
        BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_Z as c_int, &mut d);
    }

    trap::FX_PlayEffectID(engine, fxID, &v, &d, -1, -1);
    if throwPart && lostPartFX != 0 {
        // throw off a ship part, too
        let mut fxFwd: vec3_t = [0.0; 3];
        AngleVectors(cent.lerpAngles, Some(&mut fxFwd), None, None);
        trap::FX_PlayEffectID(engine, lostPartFX, &v, &fxFwd, -1, -1);
    }
}

/// Raven `CG_CreateSurfaceSmoke` — the flame/smoke trail off one of a ship's
/// four damage surfaces. An unknown surface, or one with no bolt on the model,
/// draws nothing.
/// Source: `oracle/codemp/cgame/cg_players.c:7365-7411`
pub fn CG_CreateSurfaceSmoke(ctx: &mut CgContext, cent: &centity_t, shipSurf: c_int, fxID: c_int) {
    let mut v: vec3_t = [0.0; 3];
    let mut d: vec3_t = [0.0; 3];
    let mut boltMatrix = mdxaBone_t {
        matrix: [[0.0; 4]; 3],
    };

    if cent.ghoul2.is_null() {
        // oh no
        return;
    }

    // let's add the surface as a bolt so we can get the base point of it
    let surfName = if shipSurf == SHIPSURF_FRONT {
        // front flame/smoke
        "*nosedamage"
    } else if shipSurf == SHIPSURF_BACK {
        // back flame/smoke
        //
        // Raven: FIXME: random?  Some point in-between?
        "*exhaust1"
    } else if shipSurf == SHIPSURF_RIGHT {
        // right wing flame/smoke
        "*r_wingdamage"
    } else if shipSurf == SHIPSURF_LEFT {
        // left wing flame/smoke
        "*l_wingdamage"
    } else {
        // unknown surf!
        return;
    };

    let engine = ctx.engine;
    let b = trap::G2API_AddBolt(engine, cent.ghoul2, 0, surfName);
    if b == -1 {
        // couldn't find this surface apparently
        return;
    }

    // now let's get the position and direction of this surface and make a big explosion
    let time = ctx.world.cg.time;
    trap::G2API_GetBoltMatrix(
        engine,
        cent.ghoul2,
        0,
        b,
        &mut boltMatrix,
        &cent.lerpAngles,
        &cent.lerpOrigin,
        time,
        Some(&mut ctx.world.cgs.gameModels[0]),
        &cent.modelScale,
    );
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::ORIGIN as c_int, &mut v);
    BG_GiveMeVectorFromMatrix(&boltMatrix, Eorientations::POSITIVE_Z as c_int, &mut d);

    trap::FX_PlayEffectID(engine, fxID, &v, &d, -1, -1);
}

/// Raven `CG_VehicleShouldDrawShields` — are this ship's shields currently
/// taking damage, so the shell should be drawn?
///
/// DEFERRED: Raven's last clause is `m_pVehicle->m_pVehicleInfo` non-null;
/// DEC-46.2's `Option<VehicleId>` answers only the presence half until the
/// `Vehicle_t` referent pool lands, so a vehicle with no vehicle info would
/// wrongly draw shields here.
/// Source: `oracle/codemp/cgame/cg_players.c:7415-7425`
pub fn CG_VehicleShouldDrawShields(world: &CgWorld, vehCent: &centity_t) -> bool {
    // ship shields currently taking damage
    vehCent.damageTime > world.cg.time
        && vehCent.currentState.NPC_class == class_t::CLASS_VEHICLE as c_int
        && vehCent.m_pVehicle.is_some()
}

/// Raven `CG_VehicleAttachDroidUnit` — snaps an astromech onto its ship's droid
/// tag, taking the droid's whole origin and orientation off the bolt.
///
/// ESCALATION: blocked past the presence guard — the bolt index is
/// `m_pVehicle->m_iDroidUnitTag`, and DEC-46.2's `Option<VehicleId>` carries
/// only the vehicle cent's entity number until the `Vehicle_t` referent pool
/// lands ("ported code only tests presence"). There is no honest bolt index to
/// pass, so this answers Raven's own "not attached" (`qfalse`) and the droid
/// keeps its snapshot position — a visual miss, not a crash.
/// Source: `oracle/codemp/cgame/cg_players.c:7433-7461`
#[allow(unused_variables)]
pub fn CG_VehicleAttachDroidUnit(
    ctx: &mut CgContext,
    droidCent: &mut centity_t,
    legs: &refEntity_t,
) -> bool {
    if droidCent.currentState.owner != 0 && droidCent.currentState.clientNum >= MAX_CLIENTS_I32 {
        // the only NPCs that can ride a vehicle are droids...???
        let vehCent = ctx.world.entity(droidCent.currentState.owner as usize);

        if vehCent.m_pVehicle.is_some() && !vehCent.ghoul2.is_null() {
            // DEFERRED: Vehicle_t::m_iDroidUnitTag — see the fn doc. Raven bolts
            // `droidCent->lerpOrigin` to that tag's matrix and rebuilds
            // `lerpAngles` from its POSITIVE_X/NEGATIVE_Y vectors ("WTF???" is
            // Raven's own comment on the axis choice).
        }
    }

    false
}

/// Raven `CG_CreateNPCClient` — the per-NPC `clientInfo_t` an entity carries
/// beside the real client table. Raven pool-allocates it and every caller
/// memsets it straight after, so the block is handed back zeroed (DEC-46.2 makes
/// it owned, which is what retires Raven's "always free it, never stomp over
/// it" discipline).
/// Source: `oracle/codemp/cgame/cg_players.c:7695-7699`
pub fn CG_CreateNPCClient() -> Box<clientInfo_t> {
    Box::new(zeroed_client_info())
}

/// Raven `CG_DestroyNPCClient` — zeroes the NPC's client info and keeps the
/// block. The `trap_TrueFree` beside it is commented out in the oracle, so the
/// allocation deliberately stays alive and non-NULL: the caller's next
/// `if (!cent->npcClient)` must still be false.
/// Source: `oracle/codemp/cgame/cg_players.c:7701-7705`
pub fn CG_DestroyNPCClient(ci: &mut Option<Box<clientInfo_t>>) {
    if let Some(ci) = ci {
        **ci = zeroed_client_info();
    }
}

/// The zero fill Raven gets from its `memset(*ci, 0, sizeof(clientInfo_t))`.
/// `clientInfo_t` is too wide for a derived `Default` (its `char[64]` name
/// buffers), so the fill is spelled once here for both NPC-client fns.
fn zeroed_client_info() -> clientInfo_t {
    // SAFETY: `clientInfo_t` is `#[repr(C)]` scalars, arrays, `qhandle_t`s and
    // opaque ghoul2 `*mut c_void` tokens; its two enum members (`team_t`,
    // `gender_t`) both have a 0 discriminant, so all-zero is a legal value.
    unsafe { core::mem::zeroed() }
}

/// Raven `CG_InitJetpackGhoul2` — builds the one shared jetpack instance and
/// bolts it to the player's `*chestg` tag, plus the two jet-effect bolts.
/// Source: `oracle/codemp/cgame/cg_players.c:7891-7911`
pub fn CG_InitJetpackGhoul2(ctx: &mut CgContext) {
    if !ctx.world.players.cg_g2JetpackInstance.is_null() {
        debug_assert!(false, "Tried to init jetpack inst, already init'd");
        return;
    }

    let engine = ctx.engine;
    trap::G2API_InitGhoul2Model(
        engine,
        &mut ctx.world.players.cg_g2JetpackInstance as *mut *mut c_void,
        JETPACK_MODEL,
        0,
        0,
        0,
        0,
        0,
    );

    debug_assert!(!ctx.world.players.cg_g2JetpackInstance.is_null());

    // Indicate which bolt on the player we will be attached to
    // In this case bolt 0 is rhand, 1 is lhand, and 2 is the bolt
    // for the jetpack (*chestg)
    let g2 = ctx.world.players.cg_g2JetpackInstance;
    trap::G2API_SetBoltInfo(engine, g2, 0, 2);

    // Add the bolts jet effects will be played from
    trap::G2API_AddBolt(engine, g2, 0, "torso_ljet");
    trap::G2API_AddBolt(engine, g2, 0, "torso_rjet");
}

/// Raven `CG_CleanJetpackGhoul2` — drops the shared jetpack instance at
/// shutdown.
/// Source: `oracle/codemp/cgame/cg_players.c:7913-7920`
pub fn CG_CleanJetpackGhoul2(ctx: &mut CgContext) {
    if !ctx.world.players.cg_g2JetpackInstance.is_null() {
        trap::G2API_CleanGhoul2Models(
            ctx.engine,
            &mut ctx.world.players.cg_g2JetpackInstance as *mut *mut c_void,
        );
        ctx.world.players.cg_g2JetpackInstance = null_mut();
    }
}

/// Raven `CG_RadiusForCent` — the ghoul2 cull radius for an entity: its own
/// `g2radius` when it has one, otherwise 64.
///
/// DEFERRED: an NPC vehicle first checks `m_pVehicle->m_pVehicleInfo->g2radius`
/// for an override, which DEC-46.2's `Option<VehicleId>` can't reach until the
/// `Vehicle_t` referent pool lands; a vehicle with an override falls through to
/// its entity-state radius here.
/// Source: `oracle/codemp/cgame/cg_players.c:8316-8336`
pub fn CG_RadiusForCent(cent: &centity_t) -> f32 {
    if cent.currentState.eType == entityType_t::ET_NPC as c_int {
        if cent.currentState.g2radius != 0 {
            return cent.currentState.g2radius as f32;
        }
    } else if cent.currentState.g2radius != 0 {
        return cent.currentState.g2radius as f32;
    }

    64.0
}
