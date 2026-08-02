//! `snd_ambient.cpp` — the ambient sound system.
//!
//! `sound/sound.txt` describes named sets of ambient waves: a looping bed plus a
//! pool of one-shot subwaves with a time range between them. cgame precaches the
//! set names it needs, calls `AS_ParseSets` once, and then names a set per frame
//! for the region the listener stands in. Two sets cross-fade while a change
//! settles.
//!
//! Every `snd_ambient.cpp` file-scope global lives in `SoundSystem.ambient`
//! (porting-rules §B3). Raven's `parseFuncs` function-pointer table becomes the
//! `match` in `AS_GetSet` (porting-rules §C8), so no type carries `parseFunc_t`.
//!
//! Source: `oracle/codemp/client/snd_ambient.cpp`

#![allow(non_snake_case)]

pub mod ambient_set_s;
pub mod ambient_system;
pub mod c_set_group;
pub mod set_e;
pub mod set_keyword_e;

use core::ffi::c_int;

use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::files_common::FS_ReadFileVec;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::vec3_t;
use native_math::qmath::{VectorLength, _VectorSubtract};

use crate::snd::sound_system::SoundSystem;
use crate::snd_ambient::ambient_set_s::{ambientSet_t, MAX_SET_VOLUME, MAX_WAVES_PER_GROUP};
use crate::snd_ambient::ambient_system::AmbientSystem;
use crate::snd_ambient::c_set_group::CSetGroup;
use crate::snd_ambient::set_e::set_e::{AS_SET_BMODEL, AS_SET_LOCAL, NUM_AS_SETS};
use crate::snd_ambient::set_keyword_e::setKeyword_e::{
    NUM_AS_KEYWORDS, SET_KEYWORD_LOOPEDWAVE, SET_KEYWORD_RADIUS, SET_KEYWORD_SUBWAVES,
    SET_KEYWORD_TIMEBETWEENWAVES, SET_KEYWORD_TYPE, SET_KEYWORD_VOLRANGE,
};
use crate::snd_dma::{sfxHandle_t, S_AddAmbientLoopingSound, S_RegisterSound, S_StartAmbientSound};

/// Raven `AMBIENT_SET_FILENAME`.
///
/// Source: `oracle/codemp/client/snd_ambient.h:26`
pub const AMBIENT_SET_FILENAME: &str = "sound/sound.txt";

/// Raven `setNames` — the group keyword each `set_e` is written as.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:46-51`
const SET_NAMES: [&str; NUM_AS_SETS as usize] = ["generalSet", "localSet", "bmodelSet"];

/// Raven `keywordNames` — the keyword each `setKeyword_e` is written as.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:62-73`
const KEYWORD_NAMES: [&str; NUM_AS_KEYWORDS as usize] = [
    "timeBetweenWaves",
    "subWaves",
    "loopedWave",
    "volRange",
    "radius",
    "type",
    "amsdir",
    "outdir",
    "basedir",
];

// ===========================================================================
// File parsing
// ===========================================================================

/// Raven `AS_GetSetNameIDForString` — the `set_e` a group keyword names.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:197-210`
fn AS_GetSetNameIDForString(name: &str) -> c_int {
    if name.is_empty() {
        return -1;
    }
    for (i, setName) in SET_NAMES.iter().enumerate() {
        if name.eq_ignore_ascii_case(setName) {
            return i as c_int;
        }
    }
    -1
}

/// Raven `AS_GetKeywordIDForString` — the `setKeyword_e` a keyword names.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:218-231`
fn AS_GetKeywordIDForString(name: &str) -> c_int {
    if name.is_empty() {
        return -1;
    }
    for (i, keyword) in KEYWORD_NAMES.iter().enumerate() {
        if name.eq_ignore_ascii_case(keyword) {
            return i as c_int;
        }
    }
    -1
}

/// The byte at one parse offset. Raven reads a zero-terminated buffer, so the
/// byte one past the file is the terminator.
fn parse_byte(ambient: &AmbientSystem, at: c_int) -> u8 {
    match usize::try_from(at) {
        Ok(index) => ambient.parseBuffer.get(index).copied().unwrap_or(0),
        Err(_) => 0,
    }
}

/// Raven `AS_SkipLine` — step the cursor past the end of this line.
///
/// Raven guards the entry with `parsePos > parseSize`, which he notes is needed
/// to avoid a crash from an out-of-range access.
/// Source: `oracle/codemp/client/snd_ambient.cpp:241-255`
fn AS_SkipLine(ambient: &mut AmbientSystem) {
    if ambient.parsePos > ambient.parseSize {
        return;
    }

    while parse_byte(ambient, ambient.parsePos) != b'\n'
        && parse_byte(ambient, ambient.parsePos) != b'\r'
    {
        ambient.parsePos += 1;

        if ambient.parsePos > ambient.parseSize {
            return;
        }
    }

    ambient.parsePos += 1;
}

/// One whitespace-separated token at `at`, the way `sscanf("%s")` reads it.
///
/// An empty answer is Raven's zero-field return.
fn scan_token(ambient: &AmbientSystem, at: c_int) -> String {
    scan_token_n(ambient, at, 0)
}

/// The `n`th whitespace-separated token at `at`, counting from zero.
fn scan_token_n(ambient: &AmbientSystem, at: c_int, n: usize) -> String {
    let mut cursor = at;
    let mut token = String::new();

    for _ in 0..=n {
        token.clear();
        while parse_byte(ambient, cursor).is_ascii_whitespace() {
            cursor += 1;
            if cursor > ambient.parseSize {
                return token;
            }
        }
        loop {
            let byte = parse_byte(ambient, cursor);
            if byte == 0 || byte.is_ascii_whitespace() {
                break;
            }
            token.push(byte as char);
            cursor += 1;
        }
    }

    token
}

/// C's `atoi` over a token.
fn atoi(text: &str) -> c_int {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !c.is_ascii_digit() && c != '-' && c != '+')
        .unwrap_or(text.len());
    text[..end].parse().unwrap_or(0)
}

/// Raven `AS_GetTimeBetweenWaves` — `timeBetweenWaves <start> <end>`.
///
/// A swapped pair is corrected. Raven warns about it outside `FINAL_BUILD`,
/// which the retail build drops (DEC-62.6).
/// Source: `oracle/codemp/client/snd_ambient.cpp:265-289`
fn AS_GetTimeBetweenWaves(ambient: &mut AmbientSystem, set: &mut ambientSet_t) {
    ambient.tempBuffer = scan_token_n(ambient, ambient.parsePos, 0);
    let mut startTime = atoi(&scan_token_n(ambient, ambient.parsePos, 1));
    let mut endTime = atoi(&scan_token_n(ambient, ambient.parsePos, 2));

    if startTime > endTime {
        core::mem::swap(&mut startTime, &mut endTime);
    }

    set.time_start = startTime as u32;
    set.time_end = endTime as u32;

    AS_SkipLine(ambient);
}

/// Raven `AS_GetSubWaves` — `subWaves <directory> <wave1> <wave2> ...`.
///
/// Raven's cap test is `numSubWaves > MAX_WAVES_PER_GROUP`, so a ninth wave
/// writes one slot past `subWaves` and lands on `loopedWave`, which sits right
/// after it. The port writes `loopedWave` for that one case, which is the
/// defined behaviour of Raven's overrun (porting-rules §19).
/// Source: `oracle/codemp/client/snd_ambient.cpp:299-349`
fn AS_GetSubWaves(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    ambient: &mut AmbientSystem,
    set: &mut ambientSet_t,
) {
    // Get the directory for these sets.
    ambient.tempBuffer = scan_token_n(ambient, ambient.parsePos, 0);
    let dirBuffer = scan_token_n(ambient, ambient.parsePos, 1);

    // Move the pointer past these two strings. Raven steps by the keyword
    // table's length, not by the token it just scanned.
    ambient.parsePos += (KEYWORD_NAMES[SET_KEYWORD_SUBWAVES as usize].len()
        + 1
        + dirBuffer.len()
        + 1) as c_int;

    while ambient.parsePos <= ambient.parseSize {
        let waveBuffer = scan_token(ambient, ambient.parsePos);

        if set.numSubWaves as usize > MAX_WAVES_PER_GROUP {
            // Raven's "too many subwaves" warning is a `!FINAL_BUILD` print.
        } else {
            // Construct the wave name (pretty, huh?)
            let waveName = format!("sound/{dirBuffer}/{waveBuffer}.wav");

            // Precache the file now and store the handle instead of the name.
            let handle = S_RegisterSound(view, snd, &waveName);
            let slot = set.numSubWaves as usize;
            set.numSubWaves += 1;
            if slot < MAX_WAVES_PER_GROUP {
                set.subWaves[slot] = handle;
            } else {
                set.loopedWave = handle;
            }
            // Raven's "unable to load" warning is a `!FINAL_BUILD` print.
        }

        // Move the pointer past this string.
        ambient.parsePos += (waveBuffer.len() + 1) as c_int;

        let byte = parse_byte(ambient, ambient.parsePos);
        if byte == b'\n' || byte == b'\r' {
            break;
        }
    }

    AS_SkipLine(ambient);
}

/// Raven `AS_GetLoopedWave` — `loopedWave <name>`.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:359-380`
fn AS_GetLoopedWave(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    ambient: &mut AmbientSystem,
    set: &mut ambientSet_t,
) {
    ambient.tempBuffer = scan_token_n(ambient, ambient.parsePos, 0);
    let waveBuffer = scan_token_n(ambient, ambient.parsePos, 1);

    let waveName = format!("sound/{waveBuffer}.wav");

    set.loopedWave = S_RegisterSound(view, snd, &waveName);
    // Raven's "unable to load" warning is a `!FINAL_BUILD` print.

    AS_SkipLine(ambient);
}

/// Raven `AS_GetVolumeRange` — `volRange <min> <max>`.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:388-412`
fn AS_GetVolumeRange(ambient: &mut AmbientSystem, set: &mut ambientSet_t) {
    ambient.tempBuffer = scan_token_n(ambient, ambient.parsePos, 0);
    let mut min = atoi(&scan_token_n(ambient, ambient.parsePos, 1));
    let mut max = atoi(&scan_token_n(ambient, ambient.parsePos, 2));

    if min > max {
        core::mem::swap(&mut min, &mut max);
    }

    set.volRange_start = min as u32;
    set.volRange_end = max as u32;

    AS_SkipLine(ambient);
}

/// Raven `AS_GetRadius` — `radius <value>`.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:420-426`
fn AS_GetRadius(ambient: &mut AmbientSystem, set: &mut ambientSet_t) {
    ambient.tempBuffer = scan_token_n(ambient, ambient.parsePos, 0);
    set.radius = atoi(&scan_token_n(ambient, ambient.parsePos, 1));

    AS_SkipLine(ambient);
}

/// Raven's three per-group parse functions, which differ only in the keywords
/// they take: general takes four, local adds `radius`, and bmodel takes
/// `subWaves` alone. Anything else ends the group.
///
/// Raven reaches them through the `parseFuncs` table; the port matches on the
/// group id (porting-rules §C8).
/// Source: `oracle/codemp/client/snd_ambient.cpp:434-595`
fn AS_GetSet(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    ambient: &mut AmbientSystem,
    set: &mut ambientSet_t,
    setID: c_int,
) {
    let bmodel = setID == AS_SET_BMODEL as c_int;
    let local = setID == AS_SET_LOCAL as c_int;

    while ambient.parsePos <= ambient.parseSize {
        let token = scan_token(ambient, ambient.parsePos);
        if token.is_empty() {
            return;
        }
        ambient.tempBuffer = token;

        let keywordID = AS_GetKeywordIDForString(&ambient.tempBuffer);

        if keywordID == SET_KEYWORD_SUBWAVES as c_int {
            AS_GetSubWaves(view, snd, ambient, set);
            continue;
        }
        if !bmodel {
            if keywordID == SET_KEYWORD_TIMEBETWEENWAVES as c_int {
                AS_GetTimeBetweenWaves(ambient, set);
                continue;
            }
            if keywordID == SET_KEYWORD_LOOPEDWAVE as c_int {
                AS_GetLoopedWave(view, snd, ambient, set);
                continue;
            }
            if keywordID == SET_KEYWORD_VOLRANGE as c_int {
                AS_GetVolumeRange(ambient, set);
                continue;
            }
            if local && keywordID == SET_KEYWORD_RADIUS as c_int {
                AS_GetRadius(ambient, set);
                continue;
            }
        }

        // The group has finished. Raven's unknown-keyword warning is a
        // `!FINAL_BUILD` print, and `AS_GetSetNameIDForString` only decides
        // whether to print it.
        let _ = AS_GetSetNameIDForString(&ambient.tempBuffer);
        return;
    }
}

/// Raven `AS_ParseSet` — pull every set of one group out of the file.
///
/// Only a set the precache list names is kept.
/// Source: `oracle/codemp/client/snd_ambient.cpp:605-656`
fn AS_ParseSet(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    ambient: &mut AmbientSystem,
    setID: c_int,
) {
    // Make sure we're not overstepping the name array.
    if setID > NUM_AS_SETS as c_int {
        return;
    }

    // Reset the pointers for this run through.
    ambient.parsePos = 0;

    let name = SET_NAMES[setID as usize];

    while ambient.parsePos <= ambient.parseSize {
        // Check for a valid set group.
        let at = ambient.parsePos.max(0) as usize;
        let matches = ambient
            .parseBuffer
            .get(at..at + name.len())
            .is_some_and(|slice| slice == name.as_bytes());

        if !matches {
            // If not found on this line, go down another and check again.
            AS_SkipLine(ambient);
            continue;
        }

        // Update the debug info.
        ambient.numSets += 1;

        // Push past the set specifier and on to the name, the following space
        // included.
        ambient.parsePos += (name.len() + 1) as c_int;

        // Get the set name (this MUST be first).
        ambient.tempBuffer = scan_token(ambient, ambient.parsePos);
        AS_SkipLine(ambient);

        // Test the string against the precaches.
        if !ambient.tempBuffer.is_empty() && !ambient.pMap.contains_key(&ambient.tempBuffer) {
            // Not in our precache listings, so skip it.
            continue;
        }

        // Create a new set, parse into it, and store it back.
        let setName = ambient.tempBuffer.clone();
        let slot = ambient
            .aSets
            .as_mut()
            .expect("AS_Init seated the set group")
            .AddSet(&setName);
        let mut set = ambient
            .aSets
            .as_ref()
            .expect("AS_Init seated the set group")
            .set(slot)
            .clone();
        AS_GetSet(view, snd, ambient, &mut set, setID);
        *ambient
            .aSets
            .as_mut()
            .expect("AS_Init seated the set group")
            .set_mut(slot) = set;
    }
}

/// Raven `AS_ParseHeader` — read the directory information off the file head.
///
/// A `type` that is not `ambientSet` is fatal. Raven leaves the three directory
/// keywords unimplemented, with his own TODO on each.
/// Source: `oracle/codemp/client/snd_ambient.cpp:666-705`
fn AS_ParseHeader(ambient: &mut AmbientSystem) {
    while ambient.parsePos <= ambient.parseSize {
        ambient.tempBuffer = scan_token(ambient, ambient.parsePos);

        if AS_GetKeywordIDForString(&ambient.tempBuffer) == SET_KEYWORD_TYPE as c_int
        {
            let typeBuffer = scan_token_n(ambient, ambient.parsePos, 1);

            if typeBuffer.eq_ignore_ascii_case("ambientSet") {
                return;
            }
            com_error(
                errorParm_t::ERR_DROP,
                format!("AS_ParseHeader: Set type \"{typeBuffer}\" is not a valid set type!\n"),
            );
        }
        // Raven's amsdir, outdir, and basedir arms carry his own TODO and no body.

        AS_SkipLine(ambient);
    }
}

/// Raven `AS_ParseFile` — open one sound-set file and parse every group in it.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:715-734`
fn AS_ParseFile(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    ambient: &mut AmbientSystem,
    filename: &str,
) -> bool {
    // Open the file and read the information from it.
    let Some(mut data) = FS_ReadFileVec(view, filename) else {
        ambient.parseSize = -1;
        return false;
    };
    ambient.parseSize = data.len() as c_int;
    if ambient.parseSize <= 0 {
        return false;
    }
    // Raven's file system zero-terminates every loaded file, and the parse
    // cursor reads that terminator.
    data.push(0);
    ambient.parseBuffer = data;

    // Parse the directory information out of the file.
    AS_ParseHeader(ambient);

    // Parse all the relevant sets out of it.
    for i in 0..NUM_AS_SETS as c_int {
        AS_ParseSet(view, snd, ambient, i);
    }

    // Free the memory and close the file.
    ambient.parseBuffer = Vec::new();

    true
}

// ===========================================================================
// Main code
// ===========================================================================

/// Raven `AS_Init` — seat the set container and the precache list.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:752-764`
pub fn AS_Init(ambient: &mut AmbientSystem) {
    if ambient.aSets.is_none() {
        ambient.numSets = 0;
        ambient.pMap.clear();
        ambient.aSets = Some(CSetGroup::new());
    }
}

/// Raven `AS_AddPrecacheEntry` — name one set cgame wants, or clear the list.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:772-782`
pub fn AS_AddPrecacheEntry(ambient: &mut AmbientSystem, name: &str) {
    if name.eq_ignore_ascii_case("#clear") {
        ambient.pMap.clear();
    } else {
        ambient.pMap.insert(name.to_string(), 1);
    }
}

/// Raven `AS_ParseSets` — load and precache every ambient set cgame asked for.
///
/// A named set the file does not carry is fatal, because the caller would
/// otherwise play silence for it.
/// Source: `oracle/codemp/client/snd_ambient.cpp:792-824`
pub fn AS_ParseSets(view: &mut EngineHostView, snd: &mut SoundSystem) {
    let mut ambient = core::mem::take(&mut snd.ambient);

    AS_Init(&mut ambient);

    // Parse all the sets.
    let parsed = AS_ParseFile(view, snd, &mut ambient, AMBIENT_SET_FILENAME);

    let mut iErrorsOccured = 0;
    if parsed {
        let names: Vec<String> = ambient.pMap.keys().cloned().collect();
        for str in names {
            let found = ambient
                .aSets
                .as_ref()
                .is_some_and(|sets| sets.GetSetByName(&str).is_some());
            if !found {
                // Raven prints these red instead of yellow because they cause
                // an ERR_DROP if they occur.
                com_printf(
                    view.common,
                    &format!("^1ERROR: AS_ParseSets: Unable to find ambient soundset \"{str}\"!\n"),
                );
                iErrorsOccured += 1;
            }
        }
    }

    snd.ambient = ambient;

    if !parsed {
        com_error(
            errorParm_t::ERR_FATAL,
            format!("^1ERROR: Couldn't load ambient sound sets from {AMBIENT_SET_FILENAME}"),
        );
    }

    if iErrorsOccured != 0 {
        com_error(
            errorParm_t::ERR_DROP,
            format!("....{iErrorsOccured} missing sound sets! (see above)\n"),
        );
    }
}

/// Raven `AS_Free` — drop the whole ambient system.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:834-850`
pub fn AS_Free(ambient: &mut AmbientSystem) {
    if ambient.aSets.is_some() {
        ambient.aSets = None;

        ambient.currentSet = -1;
        ambient.oldSet = -1;

        ambient.currentSetTime = 0;
        ambient.oldSetTime = 0;

        ambient.numSets = 0;
    }
}

// Raven's `AS_FreePartial` has one caller, `CL_ClearLastLevel`, and that whole
// function is `#ifdef _XBOX` (`oracle/codemp/client/cl_main.cpp:685-722`). It is
// therefore zero-caller surface on every target this tree builds for, so it does
// not port (porting-rules §20).
// Source: `oracle/codemp/client/snd_ambient.cpp:853-869`

// ===========================================================================
// Sound code
// ===========================================================================

/// Raven `AS_UpdateSetVolumes` — step the cross-fade between the two live sets.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:887-931`
fn AS_UpdateSetVolumes(ambient: &mut AmbientSystem, realtime: c_int) {
    let crossDelay = ambient.crossDelay;

    // Get the sets and validate them.
    let Some(currentSlot) = ambient
        .aSets
        .as_ref()
        .and_then(|sets| sets.GetSetById(ambient.currentSet))
    else {
        return;
    };

    {
        let current = ambient
            .aSets
            .as_mut()
            .expect("current set resolved")
            .set_mut(currentSlot);
        if current.masterVolume < MAX_SET_VOLUME {
            let deltaTime = realtime - current.fadeTime;
            let scale = deltaTime as f32 / crossDelay as f32;
            current.masterVolume = (scale * MAX_SET_VOLUME as f32) as c_int;
        }

        if current.masterVolume > MAX_SET_VOLUME {
            current.masterVolume = MAX_SET_VOLUME;
        }
    }

    // Only update the old set if it's still valid.
    if ambient.oldSet == -1 {
        return;
    }

    let Some(oldSlot) = ambient
        .aSets
        .as_ref()
        .and_then(|sets| sets.GetSetById(ambient.oldSet))
    else {
        return;
    };

    let dead = {
        let old = ambient
            .aSets
            .as_mut()
            .expect("old set resolved")
            .set_mut(oldSlot);
        if old.masterVolume > 0 {
            let deltaTime = realtime - old.fadeTime;
            let scale = deltaTime as f32 / crossDelay as f32;
            old.masterVolume = MAX_SET_VOLUME - (scale * MAX_SET_VOLUME as f32) as c_int;
        }

        if old.masterVolume <= 0 {
            old.masterVolume = 0;
            true
        } else {
            false
        }
    };
    if dead {
        ambient.oldSet = -1;
    }
}

/// Raven `AS_UpdateCurrentSet` — start a cross-fade when the named set changes.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:941-973`
fn AS_UpdateCurrentSet(ambient: &mut AmbientSystem, id: c_int, realtime: c_int) {
    // Check for a change.
    if id != ambient.currentSet {
        // This is new, so start the fading.
        ambient.oldSet = ambient.currentSet;
        ambient.currentSet = id;

        let oldSlot = ambient
            .aSets
            .as_ref()
            .and_then(|sets| sets.GetSetById(ambient.oldSet));
        let currentSlot = ambient
            .aSets
            .as_ref()
            .and_then(|sets| sets.GetSetById(ambient.currentSet));

        // Raven: a null check for now, not sure if there is a more graceful way
        // to exit this function - dmv
        let Some(currentSlot) = currentSlot else {
            return;
        };

        if let Some(oldSlot) = oldSlot {
            let old = ambient
                .aSets
                .as_mut()
                .expect("old set resolved")
                .set_mut(oldSlot);
            old.masterVolume = MAX_SET_VOLUME;
            old.fadeTime = realtime;
        }

        let current = ambient
            .aSets
            .as_mut()
            .expect("current set resolved")
            .set_mut(currentSlot);
        current.masterVolume = 0;

        // Set the fading starts.
        current.fadeTime = realtime;
    }

    // Update their volumes if fading.
    AS_UpdateSetVolumes(ambient, realtime);
}

/// Raven `AS_PlayLocalSet` — play a regional set, attenuated by distance.
///
/// `lastTime` is Raven's in-out parameter for the subwave clock.
/// Source: `oracle/codemp/client/snd_ambient.cpp:984-1020`
#[allow(clippy::too_many_arguments)]
fn AS_PlayLocalSet(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    set: &ambientSet_t,
    listener_origin: vec3_t,
    origin: vec3_t,
    entID: c_int,
    lastTime: &mut c_int,
    realtime: c_int,
) {
    let time = realtime;

    let mut dir: vec3_t = [0.0; 3];
    _VectorSubtract(origin, listener_origin, &mut dir);
    let dist = VectorLength(dir);

    // Determine the volume based on distance. This sits on top of what
    // SpatializeOrigin does.
    let distScale = if dist < (set.radius as f32 * 0.5) {
        1.0
    } else {
        (set.radius as f32 - dist) / (set.radius as f32 * 0.5)
    };
    let mut volume: u8 = if distScale > 1.0 || distScale < 0.0 {
        0
    } else {
        (set.masterVolume as f32 * distScale) as u8
    };

    // Add the looping sound.
    if set.loopedWave != 0 {
        S_AddAmbientLoopingSound(view, snd, origin, volume, set.loopedWave);
    }

    // Check the time to start another one-shot subwave.
    let gap = view
        .common
        .qrand
        .Q_irand(set.time_start as c_int, set.time_end as c_int);
    if (time - *lastTime) < gap * 1000 {
        return;
    }

    // Update the time.
    *lastTime = time;

    // Scale the volume ranges for the subwaves by the overall master volume.
    let volScale = f32::from(volume) / MAX_SET_VOLUME as f32;
    volume = view.common.qrand.Q_irand(
        (volScale * set.volRange_start as f32) as c_int,
        (volScale * set.volRange_end as f32) as c_int,
    ) as u8;

    // Add the random subwave.
    if set.numSubWaves != 0 {
        let pick = view
            .common
            .qrand
            .Q_irand(0, c_int::from(set.numSubWaves) - 1);
        let handle = set.subWaves[(pick.max(0) as usize).min(MAX_WAVES_PER_GROUP - 1)];
        S_StartAmbientSound(view, snd, Some(origin), entID, volume, handle);
    }
}

/// Raven `AS_PlayAmbientSet` — play a general set at its own master volume.
///
/// Source: `oracle/codemp/client/snd_ambient.cpp:1031-1063`
fn AS_PlayAmbientSet(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    set: &ambientSet_t,
    origin: vec3_t,
    lastTime: &mut c_int,
    realtime: c_int,
) {
    let time = realtime;

    // Add the looping sound.
    if set.loopedWave != 0 {
        S_AddAmbientLoopingSound(view, snd, origin, set.masterVolume as u8, set.loopedWave);
    }

    // Check the time to start another one-shot subwave.
    let gap = view
        .common
        .qrand
        .Q_irand(set.time_start as c_int, set.time_end as c_int);
    if (time - *lastTime) < gap * 1000 {
        return;
    }

    // Update the time.
    *lastTime = time;

    // Scale the volume ranges for the subwaves by the overall master volume.
    let volScale = set.masterVolume as f32 / MAX_SET_VOLUME as f32;
    let mut volume = view.common.qrand.Q_irand(
        (volScale * set.volRange_start as f32) as c_int,
        (volScale * set.volRange_end as f32) as c_int,
    ) as u8;

    // Allow for softer noises than the masterVolume, but not louder.
    if c_int::from(volume) > set.masterVolume {
        volume = set.masterVolume as u8;
    }

    // Add the random subwave.
    if set.numSubWaves != 0 {
        let pick = view
            .common
            .qrand
            .Q_irand(0, c_int::from(set.numSubWaves) - 1);
        let handle = set.subWaves[(pick.max(0) as usize).min(MAX_WAVES_PER_GROUP - 1)];
        S_StartAmbientSound(view, snd, Some(origin), 0, volume, handle);
    }
}

/// Raven `S_UpdateAmbientSet` — play the named set, and the fading one beside it.
///
/// Raven plays the *named* set for the current slot rather than the current set
/// itself, and only tests that a current set exists.
/// Source: `oracle/codemp/client/snd_ambient.cpp:1073-1092`
pub fn S_UpdateAmbientSet(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    name: &str,
    origin: vec3_t,
    realtime: c_int,
) {
    let mut ambient = core::mem::take(&mut snd.ambient);

    let setSlot = ambient
        .aSets
        .as_ref()
        .and_then(|sets| sets.GetSetByName(name));
    let Some(setSlot) = setSlot else {
        snd.ambient = ambient;
        return;
    };

    // Update the current and old set for crossfading.
    let id = ambient
        .aSets
        .as_ref()
        .expect("named set resolved")
        .set(setSlot)
        .id;
    AS_UpdateCurrentSet(&mut ambient, id, realtime);

    let currentSlot = ambient
        .aSets
        .as_ref()
        .and_then(|sets| sets.GetSetById(ambient.currentSet));
    let oldSlot = ambient
        .aSets
        .as_ref()
        .and_then(|sets| sets.GetSetById(ambient.oldSet));

    if currentSlot.is_some() {
        let set = ambient
            .aSets
            .as_ref()
            .expect("named set resolved")
            .set(setSlot)
            .clone();
        let mut lastTime = ambient.currentSetTime;
        AS_PlayAmbientSet(view, snd, &set, origin, &mut lastTime, realtime);
        ambient.currentSetTime = lastTime;
    }

    if let Some(oldSlot) = oldSlot {
        let set = ambient
            .aSets
            .as_ref()
            .expect("old set resolved")
            .set(oldSlot)
            .clone();
        let mut lastTime = ambient.oldSetTime;
        AS_PlayAmbientSet(view, snd, &set, origin, &mut lastTime, realtime);
        ambient.oldSetTime = lastTime;
    }

    snd.ambient = ambient;
}

/// Raven `S_AddLocalSet` — play one regional set and answer its new subwave clock.
///
/// A set the file does not carry answers `realtime`, so the caller keeps asking.
/// Source: `oracle/codemp/client/snd_ambient.cpp:1100-1115`
#[allow(clippy::too_many_arguments)]
pub fn S_AddLocalSet(
    view: &mut EngineHostView,
    snd: &mut SoundSystem,
    name: &str,
    listener_origin: vec3_t,
    origin: vec3_t,
    entID: c_int,
    time: c_int,
    realtime: c_int,
) -> c_int {
    let ambient = core::mem::take(&mut snd.ambient);

    let setSlot = ambient
        .aSets
        .as_ref()
        .and_then(|sets| sets.GetSetByName(name));
    let Some(setSlot) = setSlot else {
        snd.ambient = ambient;
        return realtime;
    };

    let set = ambient
        .aSets
        .as_ref()
        .expect("named set resolved")
        .set(setSlot)
        .clone();

    let mut currentTime = time;
    AS_PlayLocalSet(
        view,
        snd,
        &set,
        listener_origin,
        origin,
        entID,
        &mut currentTime,
        realtime,
    );

    snd.ambient = ambient;

    currentTime
}

/// Raven `AS_GetBModelSound` — the handle of one stage of a brush-model set.
///
/// Answers -1 for a missing set or a stage outside its subwave list.
/// Source: `oracle/codemp/client/snd_ambient.cpp:1123-1137`
pub fn AS_GetBModelSound(ambient: &AmbientSystem, name: &str, stage: c_int) -> sfxHandle_t {
    let Some(sets) = ambient.aSets.as_ref() else {
        return -1;
    };
    let Some(slot) = sets.GetSetByName(name) else {
        return -1;
    };
    let set = sets.set(slot);

    // Stage must be within a valid range.
    if stage > c_int::from(set.numSubWaves) - 1 || stage < 0 {
        return -1;
    }

    set.subWaves[(stage as usize).min(MAX_WAVES_PER_GROUP - 1)]
}
