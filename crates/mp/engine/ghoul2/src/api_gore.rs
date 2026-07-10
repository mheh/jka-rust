//! `G2API` gore — the two server-live `G2API_*` gore-store entry points.
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`api_gore.rs`, class "G2API
//! gore"): `G2API_ClearSkinGore` (server-live via the `G_G2_CLEANMODELS`
//! syscall arm, `G2_API.cpp:545`, and the save/load destruct path, `:2493`;
//! its `DeleteGoreSet` call is also reached directly from the
//! `G_G2_REMOVEGHOUL2MODEL`/`G_G2_REMOVEGHOUL2MODELS` removal paths,
//! `:814,901` — ports fully, `G2SV-D7`) and `G2API_GetNumGoreMarks`.
//! `_G2_GORE` is ON (`G2SV-D5` gore state).
//!
//! **`G2API_AddSkinGore` is graph-dead server-side** (`G2SV-D7`/ruling 22:
//! only the client `CG_G2_ADDSKINGORE` trap, `cg_public.h:280`, calls it — no
//! `G_G2_*` server arm exists) → a §20 zero-caller note, not a live seam fn;
//! no stub is emitted for it (its vert-buffer/`GoreTouch` goldens are not
//! M3-gating, `G2SV-Q4`). `ResetGoreTag` (`G2_misc.cpp:96`, `AddSkinGore`'s
//! sole caller) and `G2_GetGoreRecord` (`:113`, no caller anywhere in
//! `codemp/`) are dropped alongside it for the same reason and carry no
//! roster row at all (Method transcription table, `G2_API.cpp:2569`,
//! `G2_misc.cpp:96,113`).
//!
//! Every `G2API_*` entry keeps its 1:1 signature (`G2SV-D6`) and threads
//! `g2: &mut Ghoul2System`/`&Ghoul2System` (ruling 4/11, state threaded not
//! reached). **Doc/oracle mismatch, reported (not improvised around):** the
//! `## Slice hooks` per-file host-service map lists `api_gore.rs` as
//! host-consuming via `cvar_integer` (doc line ~1354), but neither
//! `G2API_ClearSkinGore` (`G2_API.cpp:2549-2559`) nor `G2API_GetNumGoreMarks`
//! (`:2534-2545`) — nor the record-store fns they call, `FindGoreSet`/
//! `DeleteGoreSet` (`G2_misc.cpp:127,153`) — reads any cvar or calls any other
//! `EngineHost` service; the `cg_g2MarksAllModels` reads the doc is tracking
//! live in `G2_TransformModel` (`G2_misc.cpp:569`, → `misc.rs`) and
//! `G2_GorePolys` (`:1524`, → `gore/gore_set.rs`), not in this file. Both
//! signatures below are therefore kept host-free, matching the oracle bodies;
//! flagged upstream rather than adding an unused `host` parameter.
//!
//! `G2API_GetNumGoreMarks` takes a single `CGhoul2Info *g2` (not the
//! `CGhoul2Info_v` wrapper) — its parameter is named `info` here, not `g2`,
//! to avoid colliding with the `g2: &Ghoul2System` state-threading convention
//! (mirroring `g2api_override_server_with_client_data`'s `server_instance`
//! rename, `api_collision.rs`/Seam definition).

use crate::ghoul2_system::Ghoul2System;
use crate::shared::cghoul2_info::CGhoul2Info;
use crate::shared::cghoul2_info_v::CGhoul2Info_v;

/// Raven `void G2API_ClearSkinGore(CGhoul2Info_v &ghoul2)` — for every
/// instance with a nonzero `mGoreSetTag`, `DeleteGoreSet` it and reset the
/// tag to `0`. Server-live: reached from `G2API_CleanGhoul2Models`
/// (`G2_API.cpp:496`, its call at `:545`, behind the `G_G2_CLEANMODELS`
/// syscall) and from `G2API_LoadSaveCodeDestructGhoul2Info` (`:2493`).
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2549-2559`
pub fn g2api_clear_skin_gore(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v) {
    // Raven: `for (i=0;i<ghoul2.size();i++) { if (ghoul2[i].mGoreSetTag) {
    // DeleteGoreSet(ghoul2[i].mGoreSetTag); ghoul2[i].mGoreSetTag=0; } }`.
    // Split into a read (tag) then a mutate (`DeleteGoreSet` + zero) per
    // instance so no borrow of `ghoul2`'s arena slot overlaps the `g2.gore`
    // borrow `delete_gore_set` needs.
    let count = ghoul2.size(g2);
    for i in 0..count {
        let tag = ghoul2.get(g2, i).gore_set_tag;
        if tag != 0 {
            g2.gore.delete_gore_set(tag);
            ghoul2.get_mut(g2, i).gore_set_tag = 0;
        }
    }
}

/// Raven `int G2API_GetNumGoreMarks(CGhoul2Info *g2)` — `0` when
/// `mGoreSetTag` is unset or `FindGoreSet` misses, else the found
/// `CGoreSet`'s `mGoreRecords.size()`.
///
/// Source: `oracle/codemp/ghoul2/G2_API.cpp:2534-2545`
pub fn g2api_get_num_gore_marks(g2: &Ghoul2System, info: &CGhoul2Info) -> i32 {
    // Raven: `if (g2->mGoreSetTag) { CGoreSet *goreSet=FindGoreSet(g2->mGoreSetTag);
    // if (goreSet) { return goreSet->mGoreRecords.size(); } } return 0;`.
    //
    // `FindGoreSet` (`gore/gore_set.rs`) is a plain map lookup in Raven too —
    // its Rust sibling takes `&mut GoreState` only because it colocates with
    // mutating siblings in the same `impl` block, not because the lookup
    // itself mutates. This fn's own frozen signature takes `g2: &Ghoul2System`
    // (Raven's `FindGoreSet` truly reads only), so the lookup goes straight at
    // the public `sets` map rather than through that `&mut`-only method.
    //
    // `mGoreRecords` is a `multimap<int,SGoreSurface>`; its port
    // (`GoreRecordsBySurface = BTreeMap<i32, Vec<SGoreSurface>>`) keys by
    // surface index with a `Vec` of same-key records, so `.size()` is the sum
    // of every key's `Vec` length, not the key count.
    if info.gore_set_tag != 0 {
        if let Some(gore_set) = g2.gore.sets.get(&info.gore_set_tag) {
            return gore_set
                .gore_records
                .values()
                .map(|records| records.len() as i32)
                .sum();
        }
    }
    0
}
