//! Port of `oracle/codemp/cgame/cg_servercmds.c` — server command and configstring handling. Functions land via the C5
//! transcription waves.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use mp_bg::local::bg_customSiegeSoundNames;
use mp_qshared::shared::sfxHandle_t;
use native_string::atoi;

use crate::cg_players::{cg_customDuelSoundNames, cg_customSoundNames};
use crate::local::client_info_t::clientInfo_t;
use crate::world::CgWorld;

/// Raven `MAX_STRINGED_SV_STRING` — this is an quake-engine limit, not a StringEd limit.
///
/// Source: `oracle/codemp/cgame/cg_servercmds.c:1057`
pub const MAX_STRINGED_SV_STRING: usize = 1024;

/// Raven `GetCustomSoundForType` — looks up a custom sound name table by
/// `cg_customSoundSet_e`-ish `setType`, indexed by `index`.
///
/// PORT-NOTE: `cg_customCombatSoundNames`/`cg_customExtraSoundNames`/
/// `cg_customJediSoundNames` (`oracle/codemp/cgame/cg_players.c:47,71,114`)
/// are not yet ported to `cg_players.rs` — see `// DEFERRED:` below.
/// Source: `oracle/codemp/cgame/cg_servercmds.c:310-330`
pub fn GetCustomSoundForType(setType: i32, index: usize) -> Option<&'static str> {
    match setType {
        1 => cg_customSoundNames[index],
        2 => {
            // DEFERRED: cg_customCombatSoundNames — oracle/codemp/cgame/cg_players.c:47
            todo!("cg_customCombatSoundNames — oracle/codemp/cgame/cg_players.c:47")
        }
        3 => {
            // DEFERRED: cg_customExtraSoundNames — oracle/codemp/cgame/cg_players.c:71
            todo!("cg_customExtraSoundNames — oracle/codemp/cgame/cg_players.c:71")
        }
        4 => {
            // DEFERRED: cg_customJediSoundNames — oracle/codemp/cgame/cg_players.c:114
            todo!("cg_customJediSoundNames — oracle/codemp/cgame/cg_players.c:114")
        }
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
