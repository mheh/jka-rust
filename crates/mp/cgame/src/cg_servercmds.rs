//! Port of `oracle/codemp/cgame/cg_servercmds.c` — server command and configstring handling. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_bg::bg_panimate::BG_InDeathAnim;
use mp_bg::local::bg_customSiegeSoundNames;
use mp_bg::public::anim_number::animNumber_t;
use mp_bg::weapons::weapon_t::WP_BRYAR_PISTOL;
use mp_engine_select::Engine;
use mp_qshared::common::mp::ghoul2::bone_flags::BONE_ANIM_OVERRIDE_FREEZE;
use mp_qshared::common::mp::qcommon::saber::saber_info::MAX_SABERS;
use mp_qshared::shared::{qfalse, sfxHandle_t, MAX_CLIENTS, MAX_CLIENTS_I32};
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::shared::display_state::DisplayState;
use mp_uishared::shared::menu_system::MenuSystem;
use native_string::{atoi, strcat_string};

use crate::cg_event::CG_ReattachLimb;
use crate::cg_main::{CG_Argv, CG_GetStringEdString, CG_SetScoreSelection};
use crate::cg_players::{
    cg_customCombatSoundNames, cg_customDuelSoundNames, cg_customExtraSoundNames,
    cg_customJediSoundNames, cg_customSoundNames, CG_DestroyNPCClient,
};
use crate::cg_weapons::CG_G2WeaponInstance;
use crate::local::client_info_t::{
    clientInfo_t, MAX_CUSTOM_COMBAT_SOUNDS, MAX_CUSTOM_EXTRA_SOUNDS, MAX_CUSTOM_JEDI_SOUNDS,
    MAX_CUSTOM_SOUNDS,
};
use crate::local::score_t::score_t;
use crate::trap;
use crate::world::cg_context::CgContext;
use crate::world::CgWorld;

/// Raven `MAX_STRINGED_SV_STRING` — this is an quake-engine limit, not a StringEd limit.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1057`
pub const MAX_STRINGED_SV_STRING: usize = 1024;

/// Raven `MAX_CLIENT_SCORE_SEND` — the score rows one `scores` command carries.
///
/// `bg_public.h` is the owning header but `mp_bg` never ported the define;
/// `mp_game`'s `g_cmds.rs` spells its own copy the same way.
/// Source: `oracle/codemp/game/bg_public.h:51`
const MAX_CLIENT_SCORE_SEND: c_int = 20;

/// Raven `GetCustomSoundForType` — looks up a custom sound name table by
/// `cg_customSoundSet_e`-ish `setType`, indexed by `index`.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:310-330`
pub fn GetCustomSoundForType(setType: i32, index: usize) -> Option<&'static str> {
    match setType {
        1 => cg_customSoundNames[index],
        2 => cg_customCombatSoundNames[index],
        3 => cg_customExtraSoundNames[index],
        4 => cg_customJediSoundNames[index],
        // Raven returns `bg_customSiegeSoundNames[index]` unconditionally
        // (`cg_servercmds.c:322-323`), no encoding check; `to_str()` maps a
        // non-UTF-8 slot to `None` instead (§F19) - unreachable today since
        // every entry is a UTF-8 string literal.
        5 => bg_customSiegeSoundNames[index].and_then(|s| s.to_str().ok()),
        6 => cg_customDuelSoundNames[index],
        _ => {
            debug_assert!(false, "GetCustomSoundForType: unhandled setType {setType}");
            None
        }
    }
}

/// Raven `SetCustomSoundForType` — writes `sfx` into the client's custom
/// sound table selected by `setType`.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:332-358`
pub fn SetCustomSoundForType(ci: &mut clientInfo_t, setType: i32, index: usize, sfx: sfxHandle_t) {
    match setType {
        1 => ci.sounds[index] = sfx,
        2 => ci.combatSounds[index] = sfx,
        3 => ci.extraSounds[index] = sfx,
        4 => ci.jediSounds[index] = sfx,
        5 => ci.siegeSounds[index] = sfx,
        6 => ci.duelSounds[index] = sfx,
        _ => debug_assert!(false, "SetCustomSoundForType: unhandled setType {setType}"),
    }
}

/// Raven `SetDuelistHealthsFromConfigString` — parses the `|`-delimited
/// duelist-health configstring into `cgs.duelist{1,2,3}health`.
///
/// Raven: we only have 2 duelists, apparently (a leading `!` on the third
/// field means power-duel is off and `duelist3health` is forced to -1).
///
/// PORT-NOTE: Raven copies each field into a fixed `char buf[64]` with no
/// bounds check, so a field over 64 bytes overflows the stack (UB); the port's
/// `String` grows instead, so there's no cap to replicate (§F19).
/// Source: `oracle/codemp/cgame/cg_servercmds.c:612-656`
pub fn SetDuelistHealthsFromConfigString(world: &mut CgWorld, s: &str) {
    let bytes = s.as_bytes();
    let mut buf = String::new();
    let mut i = 0usize;

    while i < bytes.len() && bytes[i] != b'|' {
        buf.push(bytes[i] as char);
        i += 1;
    }
    world.cgs.duelist1health = atoi(&buf);

    buf.clear();
    i += 1;
    while i < bytes.len() && bytes[i] != b'|' {
        buf.push(bytes[i] as char);
        i += 1;
    }
    world.cgs.duelist2health = atoi(&buf);

    buf.clear();
    i += 1;
    if i < bytes.len() && bytes[i] == b'!' {
        // we only have 2 duelists, apparently.
        world.cgs.duelist3health = -1;
        return;
    }

    while i < bytes.len() && bytes[i] != b'|' {
        buf.push(bytes[i] as char);
        i += 1;
    }
    world.cgs.duelist3health = atoi(&buf);
}

/// Raven `CG_RemoveChatEscapeChar` — strips the `\x19` chat-escape marker out
/// of `text`, returning the compacted copy (Raven mutates the buffer in
/// place; the out-param becomes a return per porting-rules #7).
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1045-1055`
pub fn CG_RemoveChatEscapeChar(text: &str) -> String {
    text.chars().filter(|&c| c != '\u{19}').collect()
}

/// The zero fill Raven gets from its `memset(cg.scores, 0, sizeof(cg.scores))`.
/// `score_t` is plain ints with no `Default`, so the fill is spelled out once.
fn zeroed_score() -> score_t {
    score_t {
        client: 0,
        score: 0,
        ping: 0,
        time: 0,
        scoreFlags: 0,
        powerUps: 0,
        accuracy: 0,
        impressiveCount: 0,
        excellentCount: 0,
        guantletCount: 0,
        defendCount: 0,
        assistCount: 0,
        captures: 0,
        perfect: 0,
        team: 0,
    }
}

/// Raven `CG_ParseScores` — decodes the `scores` server command into
/// `cg.scores` and mirrors each row's score/powerups back onto `cgs.clientinfo`.
///
/// PORT-NOTE: Raven's `cg.numScores > MAX_CLIENTS` clamp is dead - the very
/// next line overwrites `cg.numScores` with `readScores`. Kept as written.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:21-69`
pub fn CG_ParseScores(
    ctx: &mut CgContext,
    menus: &mut MenuSystem,
    ds: &DisplayState,
    dc: &mut dyn DisplayContext,
) {
    let arg = CG_Argv(ctx, 1);
    ctx.world.cg.numScores = atoi(&arg);

    let mut readScores = ctx.world.cg.numScores;

    if readScores > MAX_CLIENT_SCORE_SEND {
        readScores = MAX_CLIENT_SCORE_SEND;
    }

    if ctx.world.cg.numScores > MAX_CLIENTS_I32 {
        ctx.world.cg.numScores = MAX_CLIENTS_I32;
    }

    ctx.world.cg.numScores = readScores;

    let arg = CG_Argv(ctx, 2);
    ctx.world.cg.teamScores[0] = atoi(&arg);
    let arg = CG_Argv(ctx, 3);
    ctx.world.cg.teamScores[1] = atoi(&arg);

    for score in ctx.world.cg.scores.iter_mut() {
        *score = zeroed_score();
    }

    let mut i: c_int = 0;
    while i < readScores {
        let idx = i as usize;

        let arg = CG_Argv(ctx, i * 14 + 4);
        ctx.world.cg.scores[idx].client = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 5);
        ctx.world.cg.scores[idx].score = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 6);
        ctx.world.cg.scores[idx].ping = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 7);
        ctx.world.cg.scores[idx].time = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 8);
        ctx.world.cg.scores[idx].scoreFlags = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 9);
        let powerups = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 10);
        ctx.world.cg.scores[idx].accuracy = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 11);
        ctx.world.cg.scores[idx].impressiveCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 12);
        ctx.world.cg.scores[idx].excellentCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 13);
        ctx.world.cg.scores[idx].guantletCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 14);
        ctx.world.cg.scores[idx].defendCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 15);
        ctx.world.cg.scores[idx].assistCount = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 16);
        ctx.world.cg.scores[idx].perfect = atoi(&arg);
        let arg = CG_Argv(ctx, i * 14 + 17);
        ctx.world.cg.scores[idx].captures = atoi(&arg);

        if ctx.world.cg.scores[idx].client < 0 || ctx.world.cg.scores[idx].client >= MAX_CLIENTS_I32
        {
            ctx.world.cg.scores[idx].client = 0;
        }

        let client = ctx.world.cg.scores[idx].client as usize;
        let score = ctx.world.cg.scores[idx].score;
        ctx.world.cgs.clientinfo[client].score = score;
        ctx.world.cgs.clientinfo[client].powerups = powerups;

        ctx.world.cg.scores[idx].team = ctx.world.cgs.clientinfo[client].team;

        i += 1;
    }

    CG_SetScoreSelection(ctx.world, menus, ds, dc, None);
}

/// Raven `CG_ParseTeamInfo` — decodes the `tinfo` server command into the team
/// overlay's sorted client list and the per-teammate HUD fields.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:77-94`
pub fn CG_ParseTeamInfo(ctx: &mut CgContext) {
    let arg = CG_Argv(ctx, 1);
    ctx.world.draw.numSortedTeamPlayers = atoi(&arg);

    let mut i: c_int = 0;
    while i < ctx.world.draw.numSortedTeamPlayers {
        let client = atoi(&CG_Argv(ctx, i * 6 + 2));

        // §F19: Raven bounds neither `i` against `TEAM_MAXOVERLAY` nor `client`
        // against `MAX_CLIENTS` and smashes memory past either end; we take the
        // five `CG_Argv` reads either way (same syscall trace) and drop the
        // out-of-range stores.
        if (i as usize) < ctx.world.draw.sortedTeamPlayers.len() {
            ctx.world.draw.sortedTeamPlayers[i as usize] = client;
        }

        let location = atoi(&CG_Argv(ctx, i * 6 + 3));
        let health = atoi(&CG_Argv(ctx, i * 6 + 4));
        let armor = atoi(&CG_Argv(ctx, i * 6 + 5));
        let curWeapon = atoi(&CG_Argv(ctx, i * 6 + 6));
        let powerups = atoi(&CG_Argv(ctx, i * 6 + 7));

        if client >= 0 && client < MAX_CLIENTS_I32 {
            let ci = &mut ctx.world.cgs.clientinfo[client as usize];
            ci.location = location;
            ci.health = health;
            ci.armor = armor;
            ci.curWeapon = curWeapon;
            ci.powerups = powerups;
        }

        i += 1;
    }
}

/// Raven `CG_RegisterCustomSounds` — registers one of a client's custom sound
/// sets out of `sound/chars/<psDir>/misc/`, retrying a missing numbered variant
/// as its `…1.wav` form.
///
/// PORT-NOTE: Raven's `case 5:` has no `break`, so it falls into
/// `default: assert(0); return;` — the siege set never registers through here.
/// Quirk preserved.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:360-430`
pub fn CG_RegisterCustomSounds(
    ctx: &mut CgContext,
    ci: &mut clientInfo_t,
    setType: c_int,
    psDir: &str,
) {
    let iTableEntries = match setType {
        1 => MAX_CUSTOM_SOUNDS,
        2 => MAX_CUSTOM_COMBAT_SOUNDS,
        3 => MAX_CUSTOM_EXTRA_SOUNDS,
        4 => MAX_CUSTOM_JEDI_SOUNDS,
        _ => {
            debug_assert!(
                false,
                "CG_RegisterCustomSounds: unhandled setType {setType}"
            );
            return;
        }
    };

    for i in 0..iTableEntries {
        let Some(s) = GetCustomSoundForType(setType, i) else {
            break;
        };

        // step past the leading `*`
        let s = &s[1..];
        let mut hSFX = trap::S_RegisterSound(ctx.engine, &format!("sound/chars/{psDir}/misc/{s}"));

        if hSFX == 0 {
            let mut modifiedSound = String::from(s);

            // §F19: Raven backs the cursor up one char from the `.` without
            // checking, reading before the buffer when the name starts with it;
            // a leading `.` just skips the retry here.
            if let Some(p) = modifiedSound.find('.').filter(|&p| p > 0) {
                let p = p - 1;

                // before we destroy it.. we want to see if this is actually a number.
                // If it isn't a number then don't try decrementing and registering as
                // it will only cause a disk hit (we don't try precaching such files)
                let testNumber = &modifiedSound[p..p + 1];
                if atoi(testNumber) != 0 {
                    modifiedSound.truncate(p);
                    modifiedSound.push_str("1.wav");

                    hSFX = trap::S_RegisterSound(
                        ctx.engine,
                        &format!("sound/chars/{psDir}/misc/{modifiedSound}"),
                    );
                }
            }
        }

        SetCustomSoundForType(ci, setType, i, hSFX);
    }
}

/// Raven `CG_PrecacheNPCSounds` — blind-registers every base custom sound name
/// under an NPC's sound folder, muting the sound system's missing-file spam
/// while it does.
///
/// Raven takes the folder off the configstring at `str[2]`, past the leading
/// index marker.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:432-482`
pub fn CG_PrecacheNPCSounds(ctx: &mut CgContext, s: &str) {
    // §F19: Raven walks from `str[2]` without a length check and reads past the
    // end of a shorter string; a string under 2 chars gives an empty folder here.
    let pEnd: String = s.as_bytes().iter().skip(2).map(|&b| b as char).collect();

    let mut i: c_int = 0;
    while i < 4 {
        // 4 types
        // It would be better if we knew what type this actually was (extra, combat, jedi, etc).
        // But that would require extra configstring indexing and that is a bad thing.
        let mut j = 0usize;
        while j < MAX_CUSTOM_SOUNDS {
            let sound = GetCustomSoundForType(i + 1, j);

            match sound.filter(|name| !name.is_empty()) {
                // whatever it is, try registering it under this folder.
                Some(sound) => {
                    let sEnd: String = sound
                        .as_bytes()
                        .iter()
                        .skip(1)
                        .map(|&b| b as char)
                        .collect();

                    trap::S_ShutUp(ctx.engine, true);
                    trap::S_RegisterSound(ctx.engine, &format!("sound/chars/{pEnd}/misc/{sEnd}"));
                    trap::S_ShutUp(ctx.engine, false);
                }
                // move onto the next set
                None => break,
            }

            j += 1;
        }

        i += 1;
    }
}

/// Frees a client's custom-saber weapon instances — Raven's `MAX_SABERS` loop
/// inside `CG_KillCEntityG2`, spelled once for both of that fn's `ci` arms.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:894-905`
fn CG_CleanCIWeaponInstances(engine: &Engine, ci: &mut clientInfo_t) {
    // Clean up any weapon instances for custom saber stuff
    let mut j = 0;
    while j < MAX_SABERS {
        if !ci.ghoul2Weapons[j].is_null()
            && trap::G2_HaveWeGhoul2Models(engine, ci.ghoul2Weapons[j])
        {
            trap::G2API_CleanGhoul2Models(engine, &mut ci.ghoul2Weapons[j]);
            ci.ghoul2Weapons[j] = null_mut();
        }

        j += 1;
    }
}

/// Raven `CG_KillCEntityG2` — tears down every ghoul2 instance an entity slot
/// owns (its own, the grip arm, the frame hold, its client's model and saber
/// instances) and resets the rag/ik/anim latches.
///
/// Raven picks `ci` as the real client table below `MAX_CLIENTS` and the
/// entity's own `npcClient` above it, then compares the two pointers; the two
/// arms are spelled separately here because the owned `npcClient` makes that
/// comparison a fact of which branch we are in, not a runtime test.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:863-935`
pub fn CG_KillCEntityG2(ctx: &mut CgContext, entNum: usize) {
    let engine = ctx.engine;
    let ghoul2 = ctx.world.entity(entNum).ghoul2;

    if entNum < MAX_CLIENTS {
        let ci = &mut ctx.world.cgs.clientinfo[entNum];

        if ci.ghoul2Model == ghoul2 {
            ci.ghoul2Model = null_mut();
        } else if !ci.ghoul2Model.is_null() && trap::G2_HaveWeGhoul2Models(engine, ci.ghoul2Model) {
            trap::G2API_CleanGhoul2Models(engine, &mut ci.ghoul2Model);
            ci.ghoul2Model = null_mut();
        }

        CG_CleanCIWeaponInstances(engine, ci);
    } else if let Some(ci) = ctx.world.entity_mut(entNum).npcClient.as_deref_mut() {
        // never going to be != cent->ghoul2, unless cent->ghoul2 has already
        // been removed (and then this ptr is not valid)
        ci.ghoul2Model = null_mut();

        CG_CleanCIWeaponInstances(engine, ci);
    }

    if !ghoul2.is_null() && trap::G2_HaveWeGhoul2Models(engine, ghoul2) {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(entNum).ghoul2);
        ctx.world.entity_mut(entNum).ghoul2 = null_mut();
    }

    let grip_arm = ctx.world.entity(entNum).grip_arm;
    if !grip_arm.is_null() && trap::G2_HaveWeGhoul2Models(engine, grip_arm) {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(entNum).grip_arm);
        ctx.world.entity_mut(entNum).grip_arm = null_mut();
    }

    let frame_hold = ctx.world.entity(entNum).frame_hold;
    if !frame_hold.is_null() && trap::G2_HaveWeGhoul2Models(engine, frame_hold) {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(entNum).frame_hold);
        ctx.world.entity_mut(entNum).frame_hold = null_mut();
    }

    if ctx.world.entity(entNum).npcClient.is_some() {
        CG_DestroyNPCClient(&mut ctx.world.entity_mut(entNum).npcClient);
    }

    // just in case.
    ctx.world.entity_mut(entNum).isRagging = qfalse;
    ctx.world.entity_mut(entNum).ikStatus = qfalse;

    ctx.world.entity_mut(entNum).localAnimIndex = 0;
}

/// Raven `CG_CheckSVStringEdRef` — expands every `@@@KEY` reference in a
/// server-supplied string through the `MP_SVGAME` StringEd package. Raven's
/// `buf` out-param is the return value (porting-rules #7).
///
/// Raven: I don't really like doing this. But it utilizes the system that was
/// already in place.
///
/// PORT-NOTE: Raven's `gotStrip` is set once and never raised, so the copy-out
/// arm it guards always runs. The caller's `buf` is a fixed 1024 bytes that a
/// StringEd expansion can overrun (UB); the `String` grows instead, so only the
/// `Q_strcat` bound is replicated (§F19).
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1059-1126`
pub fn CG_CheckSVStringEdRef(ctx: &mut CgContext, s: &str) -> String {
    let bytes = s.as_bytes();
    let strLen = bytes.len();

    if strLen == 0 {
        return String::new();
    }

    if strLen >= MAX_STRINGED_SV_STRING {
        return s.to_string();
    }

    let mut buf = String::new();
    let mut i = 0usize;

    while i < strLen && bytes[i] != 0 {
        if bytes[i] == b'@'
            && (i + 1) < strLen
            && bytes[i + 1] == b'@'
            && (i + 2) < strLen
            && bytes[i + 2] == b'@'
            && (i + 3) < strLen
        {
            // @@@ should mean to insert a StringEd reference here, so insert it
            // into buf at the current place
            let mut stringRef = String::new();

            while i < strLen && bytes[i] == b'@' {
                i += 1;
            }

            while i < strLen
                && bytes[i] != 0
                && bytes[i] != b' '
                && bytes[i] != b':'
                && bytes[i] != b'.'
                && bytes[i] != b'\n'
            {
                stringRef.push(bytes[i] as char);
                i += 1;
            }

            let stringEd = CG_GetStringEdString(ctx, "MP_SVGAME", &stringRef);
            strcat_string(&mut buf, MAX_STRINGED_SV_STRING, &stringEd);
        }

        // Raven copies `str[i]` here even when the ref scan ran off the end,
        // which appends the terminator and ends the C string; we just stop.
        if i < strLen {
            buf.push(bytes[i] as char);
        }
        i += 1;
    }

    buf
}

/// Raven `CG_BodyQueueCopy` — clones a dying client's ghoul2 instance into a
/// body-queue entity, freezes it on the last frame of its death animation and
/// hands the weapon model (or drops it) per `knownWeapon`.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1128-1244`
pub fn CG_BodyQueueCopy(ctx: &mut CgContext, centNum: usize, clientNum: c_int, knownWeapon: c_int) {
    let engine = ctx.engine;
    let flags: c_int = BONE_ANIM_OVERRIDE_FREEZE;

    if !ctx.world.entity(centNum).ghoul2.is_null() {
        trap::G2API_CleanGhoul2Models(engine, &mut ctx.world.entity_mut(centNum).ghoul2);
    }

    if clientNum < 0 || clientNum >= MAX_CLIENTS_I32 {
        return;
    }
    let sourceNum = clientNum as usize;

    // Raven's `if (!source)` null-check on an array element drops — an owned
    // array slot can't be null (§B5).
    let source_ghoul2 = ctx.world.entity(sourceNum).ghoul2;
    if source_ghoul2.is_null() {
        return;
    }

    // reset in case it's still set from another body that was in this cent slot.
    ctx.world.entity_mut(centNum).isRagging = qfalse;
    // if the owner was in ragdoll state, then we want to go into it too right away.
    let source_isRagging = ctx.world.entity(sourceNum).isRagging;
    ctx.world.entity_mut(centNum).ownerRagging = source_isRagging;

    ctx.world.entity_mut(centNum).bodyFadeTime = 0;
    ctx.world.entity_mut(centNum).bodyHeight = 0.0;

    let source_dustTrailTime = ctx.world.entity(sourceNum).dustTrailTime;
    ctx.world.entity_mut(centNum).dustTrailTime = source_dustTrailTime;

    trap::G2API_DuplicateGhoul2Instance(
        engine,
        source_ghoul2,
        &mut ctx.world.entity_mut(centNum).ghoul2,
    );

    if source_isRagging != qfalse {
        // just reset it now.
        ctx.world.entity_mut(sourceNum).isRagging = qfalse;
        // NULL params is the engine's "reset to no ragdoll" arm
        // (`oracle/codemp/client/cl_cgame.cpp:1539-1542`)
        // Source: `oracle/codemp/cgame/cg_servercmds.c:1176`
        let source_ghoul2 = ctx.world.entity(sourceNum).ghoul2;
        trap::G2API_SetRagDoll(ctx.engine, source_ghoul2, None);
    }

    // either force the weapon from when we died or remove it if it was a dropped weapon
    // `HasGhoul2ModelOnIndex`/`RemoveGhoul2Model` take the ADDRESS of the
    // instance slot, not the token: Raven passes `&(cent->ghoul2)` and the
    // engine casts the word to `CGhoul2Info_v **` (`cl_cgame.cpp:1434`).
    if knownWeapon > WP_BRYAR_PISTOL
        && trap::G2API_HasGhoul2ModelOnIndex(
            engine,
            &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void as *mut c_void,
            1,
        )
    {
        trap::G2API_RemoveGhoul2Model(
            engine,
            &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void as *mut c_void,
            1,
        );
    } else if trap::G2API_HasGhoul2ModelOnIndex(
        engine,
        &mut ctx.world.entity_mut(centNum).ghoul2 as *mut *mut c_void as *mut c_void,
        1,
    ) {
        let world = &*ctx.world;
        let instance = CG_G2WeaponInstance(world, world.entity(centNum), knownWeapon);
        let cent_ghoul2 = world.entity(centNum).ghoul2;
        trap::G2API_CopySpecificGhoul2Model(engine, instance, 0, cent_ghoul2, 1);
    }

    if ctx.world.entity(centNum).ownerRagging == qfalse {
        let source_localAnimIndex = ctx.world.entity(sourceNum).localAnimIndex;
        let source_torsoAnim = ctx.world.entity(sourceNum).currentState.torsoAnim;

        //anim = &bgAllAnims[cent->localAnimIndex].anims[ cent->currentState.torsoAnim ];
        let mut fallBack = false;
        let animNum = if BG_InDeathAnim(source_torsoAnim) == qfalse {
            // then just snap the corpse into a default
            fallBack = true;
            animNumber_t::BOTH_DEAD1 as c_int
        } else {
            source_torsoAnim
        };

        // §F19: a skeleton whose animation.cfg never parsed leaves
        // `localAnimIndex` at -1 with a null table, which Raven indexes one
        // entry before `bgAllAnims`; we skip the anim block instead.
        let animations = if source_localAnimIndex >= 0
            && (source_localAnimIndex as usize) < ctx.world.bg_state.bgAllAnims.len()
        {
            ctx.world.bg_state.bgAllAnims[source_localAnimIndex as usize].anims
        } else {
            null_mut()
        };

        if !animations.is_null() {
            // SAFETY: `anims` is the animation table `BG_ParseAnimationFile`
            // allocated for this skeleton; Raven indexes it unchecked and so do we
            // (same read `cg_players::CG_G2SetHeadAnim` does).
            let (firstFrame, numFrames, frameLerp) = unsafe {
                let a = &*animations.offset(animNum as isize);
                (a.firstFrame as c_int, a.numFrames as c_int, a.frameLerp)
            };

            let animSpeed = 50.0f32 / frameLerp as f32;

            let mut aNum;
            if !fallBack {
                // this will just set us to the last frame of the animation, in theory
                let source_number = ctx.world.entity(sourceNum).currentState.number;
                aNum = ctx.world.cgs.clientinfo[source_number as usize].frame + 1;

                while aNum >= firstFrame + numFrames {
                    aNum -= 1;
                }

                if aNum < firstFrame - 1 {
                    // wrong animation...?
                    aNum = (firstFrame + numFrames) - 1;
                }
            } else {
                aNum = firstFrame;
            }

            let eFrame = firstFrame + numFrames;

            //if (!cgs.clientinfo[source->currentState.number].frame || (cent->currentState.torsoAnim) != (source->currentState.torsoAnim) )
            //{
            //	aNum = (anim->firstFrame+anim->numFrames)-1;
            //}

            let cent_ghoul2 = ctx.world.entity(centNum).ghoul2;
            let time = ctx.world.cg.time;
            for bone in ["upper_lumbar", "model_root", "Motion"] {
                trap::G2API_SetBoneAnim(
                    engine,
                    cent_ghoul2,
                    0,
                    bone,
                    aNum,
                    eFrame,
                    flags,
                    animSpeed,
                    time,
                    -1.0,
                    150,
                );
            }
        }
    }

    // After we create the bodyqueue, regenerate any limbs on the real instance
    if ctx.world.entity(sourceNum).torsoBolt != 0 {
        CG_ReattachLimb(ctx, sourceNum);
    }
}
