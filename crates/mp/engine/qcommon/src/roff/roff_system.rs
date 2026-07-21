//! `CROFFSystem` methods — the five frozen seam arms plus the private
//! parse/playback/cleanup helpers, as inherent `impl` blocks on [`RoffSystem`].
//!
//! Ported against the FROZEN design `docs/subsystems/roff.md` under the
//! **WinDed Release macro set** (`-DNDEBUG -DDEDICATED -DBOTLIB`, ROFF-D3): the
//! `#ifndef DEDICATED` client-only branches inside `ApplyROFF`
//! (`RoffSystem.cpp:835-843`), `ClearLerp` (`:981-989`), and `ProcessNote`
//! (`:951-952`) do not exist in the ported TU — noted at each method below, not
//! transcribed as separate stubs (§20). `is_client` parameters are kept for
//! signature fidelity (they compile in the oracle) but arrive `false` at every
//! live call site under DEDICATED (ROFF-D3).
//!
//! Upward services (FS reads, entity access, `svs.time`, the note-track
//! `VM_Call`) are reached through `&mut impl EngineHost` (ROFF-D2, RULING 11) —
//! never ambient state (§B3). Cache entries (`Croff`) are owned in
//! [`RoffSystem::roff_list`] by id; playback entries (`SroffEntity`) are owned
//! in [`RoffSystem::roff_ent_list`] by index — internal helpers below take an
//! id/index rather than a raw pointer (§B5).
//!
//! Type definition source: `oracle/codemp/qcommon/RoffSystem.h:35-181`;
//! method bodies: `oracle/codemp/qcommon/RoffSystem.cpp`

use std::ffi::CString;

use mp_host_interface::vm_slot::VmSlot;
use mp_host_interface::EngineHost;
use mp_qshared::common::mp::qcommon::game_export_t::gameExport_t;
use mp_qshared::shared::q_math::{_VectorMA, _VectorScale, AngleVectors};
use mp_qshared::shared::q_string::COM_StripExtension;
use mp_qshared::shared::{qfalse, qtrue, trType_t, trajectory_t, vec3_t};

use super::croff::MoveRotateEntry;
use super::header::{TROFF2Entry, TROFF2Header, TROFFEntry, TROFFHeader};
use super::sroff_entity::SroffEntity;
use super::{Croff, RoffSystem, ROFF_NEW_VERSION, ROFF_SAMPLE_RATE, ROFF_VERSION};

impl RoffSystem {
    // ---------------------------------------------------------------------
    // Seam arms (ROFF-D1: exactly five; frozen signatures, `## Seam
    // definition`). Dispatched by `SV_GameSystemCalls`
    // (`codemp/server/sv_game.cpp:714-728`) with `is_client = false`.
    // ---------------------------------------------------------------------

    /// Raven `CROFFSystem::Clean(qboolean isClient)` — seam arm
    /// `G_ROFF_CLEAN`. The live body is the `#else` branch (ROFF-V3): Unloads
    /// every cached roff, ignoring `is_client`; the `#if 0` per-client variant
    /// (`:455-496`) is dead. `is_client` is kept for signature fidelity only.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:453-510` (live branch
    /// `:497-509`)
    pub fn clean(&mut self, is_client: bool) -> bool {
        // ROFF-V3: the live `#else` Unloads everything, ignoring `is_client`
        // and never touching `mROFFEntList`. Oracle repeatedly Unloads the
        // first (smallest-id) entry until the map is empty; collecting the
        // keys ascending and Unloading each reproduces that order (ROFF-D4).
        let _ = is_client;
        let ids: Vec<i32> = self.roff_list.keys().copied().collect();
        for id in ids {
            self.unload(id);
        }
        true
    }

    /// Raven `CROFFSystem::UpdateEntities(qboolean isClient)` — seam arm
    /// `G_ROFF_UPDATE_ENTITIES`. Walks `mROFFEntList` in insertion order,
    /// skipping entries whose `mIsClient != isClient`, calling `ApplyROFF`; a
    /// `false` return or a missing roff marks `mKill`; a second pass erases
    /// killed entries.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:746-808`
    pub fn update_entities(&mut self, is_client: bool, host: &mut impl EngineHost) {
        // First pass: apply roff to every matching ent, marking finished /
        // orphaned ones for death. `ApplyROFF`/`ClearLerp` mutate fields but
        // never add or remove list entries, so the length is stable here.
        for i in 0..self.roff_ent_list.len() {
            let ent = self.roff_ent_list[i];

            if ent.m_is_client != is_client {
                continue;
            }

            // Get this entity's ROFF object.
            if self.roff_list.contains_key(&ent.m_roff_id) {
                // roff that baby!
                if !self.apply_roff(i, ent.m_roff_id, host) {
                    // done roffing, mark for death
                    self.roff_ent_list[i].m_kill = true;
                }
            } else {
                // roff not found == bad, dump an error message and purge this
                // ent (the per-ent-name print is entitySystem-commented in the
                // oracle, `:772-773`).
                host.print("^1ROFF System Error:\n");
                self.roff_ent_list[i].m_kill = true;
                self.clear_lerp(i, host);
            }
        }

        // Second pass: delete killed ROFFers from the list. Oracle resets the
        // iterator to `begin()` after each erase (`:801`); the index reset to 0
        // reproduces that.
        let mut i = 0;
        while i < self.roff_ent_list.len() {
            if self.roff_ent_list[i].m_is_client != is_client {
                i += 1;
                continue;
            }

            if self.roff_ent_list[i].m_kill {
                // trash this guy from the list
                self.roff_ent_list.remove(i);
                i = 0;
            } else {
                i += 1;
            }
        }
    }

    /// Raven `CROFFSystem::Cache(const char *file, qboolean isClient)` — seam
    /// arm `G_ROFF_CACHE`. `GetID` short-circuits an already-cached path; else
    /// `FS_ReadFile`s it (falling back to `scripts/%s.rof` on a stripped-name
    /// miss), validates via `IsROFF`, mints a `NewID`, and `InitROFF`s it. On
    /// `InitROFF` failure this guards-and-returns (ROFF-D5/ROFF-V7): `Unload`s
    /// the roff, frees the file buffer, and returns 0 — it never reaches the
    /// oracle's `mROFFList.find(0)` end-iterator deref. On success sets
    /// `mUsedByClient`/`mUsedByServer` from `is_client` and returns the id.
    /// Also called internally from ICARUS (`GameInterface.cpp:491,505`, both
    /// `qfalse`) via the RULING 11 split-borrow view.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:298-365`
    pub fn cache(&mut self, file: &str, is_client: bool, host: &mut impl EngineHost) -> i32 {
        // See if this item is already cached.
        let mut id = self.get_id(file);

        if id != 0 {
            // Already cached. Oracle's `_DEBUG` "Ignoring." print is compiled
            // out under NDEBUG (ROFF-D3); fall through to the used-by flag.
        } else {
            // Read the file in one fell swoop. FS_ReadFile's `len <= 0` (a
            // missing file returns -1/NULL, an empty one returns 0) becomes a
            // missing/empty `Option`; both trigger the `scripts/%s.rof`
            // fallback (`:314-326`).
            let data = match host.fs_read_file(file) {
                Some(d) if !d.is_empty() => d,
                _ => {
                    let other_path = COM_StripExtension(file);
                    let fallback = format!("scripts/{other_path}.rof");
                    match host.fs_read_file(&fallback) {
                        Some(d) if !d.is_empty() => d,
                        _ => {
                            host.print(&format!("^1Could not open .ROF file '{file}'\n"));
                            return 0;
                        }
                    }
                }
            };

            // Make sure that the file is roff.
            if !self.is_roff(&data) {
                host.print(&format!(
                    "^1cache failed: roff <{file}> does not exist or is not a valid roff\n"
                ));
                host.fs_free_file(data);
                return 0;
            }

            // Things are looking good so far, so create a new CROFF object.
            id = self.new_id();

            let mut croff = Croff {
                id,
                roff_file_path: file.to_string(),
                ..Default::default()
            };

            // Decode into the object before inserting; `InitROFF` has no map
            // side effects, so this is equivalent to the oracle's decode-in-map
            // (`mROFFList[id] = cROFF; InitROFF(data, cROFF)`).
            let ok = self.init_roff(&data, &mut croff);
            self.roff_list.insert(id, croff);

            if !ok {
                // Something failed. ROFF-D5/ROFF-V7: guard and return — Unload
                // the just-inserted roff, free the buffer, and return 0. Never
                // fall through to the oracle's `mROFFList.find(0)` end()-deref.
                self.unload(id);
                host.fs_free_file(data);
                return 0;
            }

            host.fs_free_file(data);
        }

        // Success: flag the roff as used by whichever side requested it. Only
        // reached with a live `id` (ROFF-D5 returned early on failure).
        if let Some(croff) = self.roff_list.get_mut(&id) {
            if is_client {
                croff.used_by_client = true;
            } else {
                croff.used_by_server = true;
            }
        }

        // If we haven't requested a new ID, we'll just be returning the ID of
        // the existing roff.
        id
    }

    /// Raven `CROFFSystem::Play(int entID, int roffID, qboolean doTranslation,
    /// qboolean isClient)` — seam arm `G_ROFF_PLAY`. Resolves
    /// `SV_GentityNum(entID)` and sets `ent->r.mIsRoffing = qtrue`
    /// **before** the NULL check (ROFF-V6, preserve write-then-check order
    /// faithfully); allocates an `SROFFEntity`, seeds `mNextROFFTime =
    /// svs.time`, `mROFFFrame = 0`, copies `ent->s.apos.trBase` into
    /// `mStartAngles`, and pushes onto `mROFFEntList`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:592-624`
    pub fn play(
        &mut self,
        ent_id: i32,
        roff_id: i32,
        do_translation: bool,
        is_client: bool,
        host: &mut impl EngineHost,
    ) -> bool {
        let ent = host.gentity(ent_id);

        // ROFF-V6: Raven writes `ent->r.mIsRoffing = qtrue` BEFORE the
        // `if (ent == 0)` NULL check; reproduce the write-then-check order
        // faithfully (SV_GentityNum never returns NULL for a valid ent, so the
        // trailing check is dead and the write is safe in practice).
        // SAFETY: `SV_GentityNum` marshals a live entity pointer at the ABI
        // seam (§D11); the deref is confined here.
        unsafe {
            (*ent).r.mIsRoffing = qtrue;
        }

        if ent.is_null() {
            // shame on you..
            return false;
        }

        // SAFETY: non-NULL per the check above; §D11 seam deref.
        let start_angles = unsafe { (*ent).s.apos.trBase };

        let roffing_ent = SroffEntity {
            m_ent_id: ent_id,
            m_roff_id: roff_id,
            m_next_roff_time: host.sv_time(),
            m_roff_frame: 0,
            m_kill: false,
            // Raven sets `mSignal = qtrue` with a note that the real signal
            // code was never hooked up; nothing reads it (see `SroffEntity`).
            m_signal: true,
            m_translated: do_translation,
            m_is_client: is_client,
            m_start_angles: start_angles,
        };

        self.roff_ent_list.push(roffing_ent);

        true
    }

    /// Raven `CROFFSystem::PurgeEnt(int entID, qboolean isClient)` — seam arm
    /// `G_ROFF_PURGE_ENT`. Linear-scans `mROFFEntList` for the first entry
    /// matching `(is_client, ent_id)`, `ClearLerp`s it so it doesn't stay
    /// lerping, then erases it.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:684-705`
    pub fn purge_ent(&mut self, ent_id: i32, is_client: bool, host: &mut impl EngineHost) -> bool {
        for i in 0..self.roff_ent_list.len() {
            let ent = self.roff_ent_list[i];
            if ent.m_is_client == is_client && ent.m_ent_id == ent_id {
                // Make sure it won't stay lerping.
                self.clear_lerp(i, host);
                self.roff_ent_list.remove(i);
                return true;
            }
        }

        host.print(&format!("^1Purge failed:  Entity <{ent_id}> not found\n"));

        false
    }

    // ---------------------------------------------------------------------
    // Private parse helpers (Golden A). Signatures are free (§A1); operate on
    // raw file bytes and a `Croff` under construction.
    // ---------------------------------------------------------------------

    /// Raven `CROFFSystem::IsROFF(unsigned char *data)` — validates the
    /// header string, version (1 or 2), and a positive count.
    ///
    /// Raven's `!strcmp(hdr->mHeader, ROFF_STRING)` reads the 4-byte
    /// `mHeader` (no NUL) as a C-string that runs into `mVersion`'s low byte
    /// (ROFF-V1); reproduce that 5-byte-with-trailing-version compare
    /// faithfully, not a "fixed" 4-byte `memcmp`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:96-122`
    fn is_roff(&self, data: &[u8]) -> bool {
        // ROFF-V1: `strcmp(hdr->mHeader, "ROFF")` compares the file's C-string
        // (mHeader is `char[4]` with no NUL, so it runs into mVersion's low
        // byte) against `"ROFF\0"`. It returns 0 (equal) iff the first five
        // bytes are exactly `R O F F \0`; `!strcmp` then rejects that as a "bad
        // header". A valid file's mVersion low byte is 1 (nonzero), so the
        // strings differ at byte 4 and the file passes — Raven's accidental
        // pass, reproduced, not fixed.
        let header_equals_roff_nul = data.len() >= 5 && &data[0..4] == b"ROFF" && data[4] == 0;
        if header_equals_roff_nul {
            // bad header
            return false;
        }

        let version = read_i32_le(data, 4);
        if version != ROFF_VERSION && version != ROFF_NEW_VERSION {
            // bad version
            return false;
        }

        if version == ROFF_VERSION && read_f32_le(data, 8) <= 0.0 {
            // bad count (v1 `mCount` is a float)
            return false;
        }

        if version == ROFF_NEW_VERSION && read_i32_le(data, 8) <= 0 {
            // bad count (v2 `mCount` is an int)
            return false;
        }

        true
    }

    /// Raven `CROFFSystem::InitROFF(unsigned char *data, CROFF *obj)` —
    /// stuffs version-1 roff data into `obj`: defaults `mFrameTime =
    /// 1000/ROFF_SAMPLE_RATE` (100ms), `mLerp = ROFF_SAMPLE_RATE`, no note
    /// tracks; delegates to `init_roff2` when `mVersion == ROFF_NEW_VERSION`.
    /// Runs `fix_bad_angles` over the decoded entries.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:135-174`
    fn init_roff(&self, data: &[u8], obj: &mut Croff) -> bool {
        let version = read_i32_le(data, 4);
        if version == ROFF_NEW_VERSION {
            return self.init_roff2(data, obj);
        }

        // v1 `mCount` is a float, truncated to the int entry count.
        let count = read_f32_le(data, 8) as i32;
        obj.frame_time = 1000 / ROFF_SAMPLE_RATE; // default 10 hz
        obj.lerp = ROFF_SAMPLE_RATE;
        // No note tracks in v1.
        obj.note_track_blob.clear();
        obj.note_track_offsets.clear();

        // The `new TROFF2Entry[count]` allocation never fails in Rust, so the
        // oracle's `else { return qfalse }` allocation-failure branch (`:169`)
        // has no reachable path (§C9); always take the copy path.
        // Step past the header (`TROFFHeader`) to get to the goods.
        let base = core::mem::size_of::<TROFFHeader>();
        let entry_size = core::mem::size_of::<TROFFEntry>();
        let mut list = Vec::with_capacity(count.max(0) as usize);
        for i in 0..count {
            let off = base + (i as usize) * entry_size;
            list.push(MoveRotateEntry {
                origin_offset: read_vec3_le(data, off),
                rotate_offset: read_vec3_le(data, off + 12),
                // v1 synthesizes empty note info.
                start_note: -1,
                num_notes: 0,
            });
        }
        obj.move_rotate_list = list;

        self.fix_bad_angles(obj);

        true
    }

    /// Raven `CROFFSystem::InitROFF2(unsigned char *data, CROFF *obj)` —
    /// stuffs version-2 roff data into `obj`: reads `mFrameRate`
    /// (`mLerp = 1000/mFrameRate`), copies the packed NUL-terminated
    /// note-track strings into one blob with per-track pointers
    /// (ROFF-V5: Rust owns this as one buffer plus offset indices, no
    /// interior raw pointers). Runs `fix_bad_angles` over the decoded
    /// entries.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:187-245`
    fn init_roff2(&self, data: &[u8], obj: &mut Croff) -> bool {
        let count = read_i32_le(data, 8); // mCount
        let frame_rate = read_i32_le(data, 12); // mFrameRate
        let num_notes = read_i32_le(data, 16); // mNumNotes

        obj.frame_time = frame_rate;
        obj.lerp = 1000 / frame_rate;

        // Step past the header (`TROFF2Header`) to the entries.
        let base = core::mem::size_of::<TROFF2Header>();
        let entry_size = core::mem::size_of::<TROFF2Entry>();
        let mut list = Vec::with_capacity(count.max(0) as usize);
        for i in 0..count {
            let off = base + (i as usize) * entry_size;
            list.push(MoveRotateEntry {
                origin_offset: read_vec3_le(data, off),
                rotate_offset: read_vec3_le(data, off + 12),
                start_note: read_i32_le(data, off + 24),
                num_notes: read_i32_le(data, off + 28),
            });
        }
        obj.move_rotate_list = list;

        self.fix_bad_angles(obj);

        obj.note_track_blob.clear();
        obj.note_track_offsets.clear();
        if num_notes != 0 {
            // Note-track strings are packed NUL-terminated right after the roff
            // entries (`&roff_data[count]`). Measure the total blob size, copy
            // it whole, then record each string's start offset (ROFF-V5: offset
            // indices replace the oracle's interior `char*` pointers).
            let notes_start = base + (count.max(0) as usize) * entry_size;

            let mut ptr = notes_start;
            let mut size = 0usize;
            for _ in 0..num_notes {
                let slen = c_strlen(data, ptr) + 1;
                size += slen;
                ptr += slen;
            }

            obj.note_track_blob = data[notes_start..notes_start + size].to_vec();

            // mNoteTrackIndexes[0] = blob start; each subsequent index steps
            // past the previous string's NUL.
            obj.note_track_offsets.push(0);
            let mut poff = 0usize;
            for _ in 1..num_notes {
                poff += c_strlen(&obj.note_track_blob, poff) + 1;
                obj.note_track_offsets.push(poff);
            }
        }

        true
    }

    /// Raven `CROFFSystem::FixBadAngles(CROFF *obj)` — wraps any rotate
    /// component `> 180` or `< -180` by ∓360, in place. Gated on
    /// `ROFF_AUTO_FIX_BAD_ANGLES` in the oracle, which is always defined;
    /// runs on every load. Parity-visible (Golden A pins the post-fix
    /// entries).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:258-285`
    fn fix_bad_angles(&self, obj: &mut Croff) {
        for entry in &mut obj.move_rotate_list {
            for t in 0..3 {
                if entry.rotate_offset[t] > 180.0 {
                    // found a bad angle
                    entry.rotate_offset[t] -= 360.0;
                } else if entry.rotate_offset[t] < -180.0 {
                    // found a bad angle
                    entry.rotate_offset[t] += 360.0;
                }
            }
        }
    }

    // ---------------------------------------------------------------------
    // Private cache/list helpers.
    // ---------------------------------------------------------------------

    /// Raven `CROFFSystem::NewID()` — `++mID`; increments before returning,
    /// so it never mints 0 (a zero id signals cache failure at the `cache`
    /// seam, ROFF-D5).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:146`
    fn new_id(&mut self) -> i32 {
        // Increment before return so we can use zero as a failed return val.
        self.id += 1;
        self.id
    }

    /// Raven `CROFFSystem::GetID(const char *file)` — linear scan of
    /// `mROFFList` for a cached roff with a matching file path; 0 if not
    /// found.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:378-393`
    fn get_id(&self, file: &str) -> i32 {
        // Attempt to find the requested roff.
        for (id, croff) in &self.roff_list {
            if croff.roff_file_path == file {
                // return the ID to this roff
                return *id;
            }
        }
        // Not found
        0
    }

    /// Raven `CROFFSystem::Unload(int id)` — deletes the cached roff and
    /// erases it from `mROFFList`; `true` if the id was present, `false`
    /// otherwise.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:407-441`
    fn unload(&mut self, id: i32) -> bool {
        // The owned `Croff` frees itself on removal (ROFF-V5, §9); the oracle's
        // `_DEBUG` success/failure prints are compiled out under NDEBUG.
        self.roff_list.remove(&id).is_some()
    }

    /// Raven `CROFFSystem::Restart()` — Unloads every cached roff and resets
    /// `mID = 0`. Called from the (idiomatic-Rust-superseded) dtor;
    /// exposed here as the same instance method Raven names, for any live
    /// caller that invokes it directly.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:66-83`
    // Faithful Raven surface with no ported live caller yet (the dtor is
    // superseded by Rust ownership; a direct `Restart` caller lands with the
    // server spine, wave 25).
    #[allow(dead_code)]
    fn restart(&mut self) -> bool {
        // Remove everything from the list (owned `Croff`s drop themselves,
        // ROFF-V5) and clear the unique-ID counter. Restart does not touch
        // `mROFFEntList`.
        self.roff_list.clear();
        self.id = 0;
        true
    }

    /// Raven `CROFFSystem::List(void)` — dumps every cached roff's id and
    /// file path to the console via `Com_Printf`, plus the total count.
    /// Debug-only; ascending-id iteration order is behaviour-visible
    /// (ROFF-D4).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:522-535`
    // Debug/console `List` surface; its live caller is the `roff` console
    // command, not yet ported.
    #[allow(dead_code)]
    fn list_all(&self, host: &mut impl EngineHost) {
        host.print("^2\n--Cached ROFF files--\n");
        host.print("^2ID   FILE\n");

        for (id, croff) in &self.roff_list {
            host.print(&format!("^2{:2} - {}\n", id, croff.roff_file_path));
        }

        host.print(&format!("^2\nFiles: {}\n", self.roff_list.len()));
    }

    /// Raven `CROFFSystem::List(int id)` — overload of `List` that dumps one
    /// roff's file path, id, entry count, and every move/rotate entry to the
    /// console; `false` (with a not-found print) if `id` isn't cached.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:548-578`
    // Debug/console `List(id)` surface; live caller is the `roff` console
    // command, not yet ported.
    #[allow(dead_code)]
    fn list_one(&self, id: i32, host: &mut impl EngineHost) -> bool {
        if let Some(obj) = self.roff_list.get(&id) {
            // requested item found in the list
            host.print(&format!("^2File: {}\n", obj.roff_file_path));
            host.print(&format!("^2ID: {id}\n"));
            host.print(&format!("^2Entries: {}\n\n", obj.move_rotate_list.len()));

            host.print("^2MOVE                 ROTATE\n");

            for e in &obj.move_rotate_list {
                host.print(&format!(
                    "^2{:6.2} {:6.2} {:6.2}   {:6.2} {:6.2} {:6.2}\n",
                    e.origin_offset[0],
                    e.origin_offset[1],
                    e.origin_offset[2],
                    e.rotate_offset[0],
                    e.rotate_offset[1],
                    e.rotate_offset[2]
                ));
            }

            return true;
        }

        host.print(&format!("^3ROFF not found: id <{id}>\n"));

        false
    }

    /// Raven `CROFFSystem::ListEnts()` — lists the currently-roffing
    /// entities. The whole body is commented out (entitySystem-stubbed,
    /// ROFF-V4): ports as a faithful no-op (§20 zero-live-caller note).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:637-671`
    // ROFF-V4 faithful no-op; also has no ported live caller (the `roff`
    // console command, not yet ported).
    #[allow(dead_code)]
    fn list_ents(&self, host: &mut impl EngineHost) {
        // ROFF-V4: the whole body is entitySystem-commented in the oracle;
        // faithful no-op (§20 zero-live-caller).
        let _ = host;
    }

    /// Raven `CROFFSystem::PurgeEnt(char *name)` — the by-name overload; its
    /// real body is entitySystem-stubbed (commented out) and always returns
    /// `qfalse` (ROFF-V4): ports as a faithful no-op (§20 zero-live-caller
    /// note). Distinct Rust name from the by-id seam arm [`Self::purge_ent`]
    /// (Rust has no overloading).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:719-734`
    // ROFF-V4 faithful no-op; the oracle's only caller is entitySystem-
    // commented, so there is no live caller to wire.
    #[allow(dead_code)]
    fn purge_ent_by_name(&mut self, name: &str, host: &mut impl EngineHost) -> bool {
        // ROFF-V4: entitySystem-commented in the oracle; always `qfalse`
        // (§20 zero-live-caller).
        let _ = (name, host);
        false
    }

    // ---------------------------------------------------------------------
    // Private playback helpers (Golden B). `ent_index` names a live index
    // into `roff_ent_list` (§B5: by index, never raw pointer).
    // ---------------------------------------------------------------------

    /// Raven `CROFFSystem::ApplyROFF(SROFFEntity *roff_ent, CROFF *roff)` —
    /// applies one frame of roff playback. Returns early (`true`, "not done
    /// yet") if `svs.time < mNextROFFTime`.
    ///
    /// The `if (mIsClient)` branch (`:833-844`) is `#ifndef DEDICATED` —
    /// compiled out under WinDed (ROFF-D3, ROFF-V2): its whole body is
    /// empty, so `ent` is only ever set in the server `else` (`:845-859`)
    /// and the `:907` `ent->next_roff_time` deref never sees a NULL. That
    /// empty branch is a §20 zero-caller drop, not transcribed here.
    ///
    /// On the server path: NULL entity → `false`. When the frame index
    /// reaches `mROFFEntries`, `SetLerp`s both trajectories to
    /// `TR_STATIONARY`, clears `mIsRoffing`, returns `false` (done).
    /// Otherwise optionally rotates the origin offset by `mStartAngles`
    /// (translated playback), `SetLerp`s origin (`TR_LINEAR`) and angles,
    /// fires any notes via `process_note`, advances the frame, and writes
    /// `mNextROFFTime`/`ent->next_roff_time`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:820-911`
    fn apply_roff(&mut self, ent_index: usize, roff_id: i32, host: &mut impl EngineHost) -> bool {
        let sv_time = host.sv_time();
        let roff_ent = self.roff_ent_list[ent_index];

        if sv_time < roff_ent.m_next_roff_time {
            // Not time to roff yet.
            return true;
        }

        // Server path only (the `mIsClient` branch is a §20 DEDICATED drop,
        // ROFF-V2/ROFF-D3). Find the entity to apply the roff to.
        let ent = host.gentity(roff_ent.m_ent_id);
        if ent.is_null() {
            // bad stuff
            return false;
        }
        // SAFETY: non-NULL per the check; `SV_GentityNum` marshals a live
        // entity pointer at the ABI seam (§D11). `ent` aliases shared memory
        // owned by the server, disjoint from `self`.
        let ent = unsafe { &mut *ent };

        let origin = ent.r.currentOrigin;
        let angle = ent.r.currentAngles;

        let roff_entries = self.roff_list[&roff_id].move_rotate_list.len() as i32;
        let lerp = self.roff_list[&roff_id].lerp;
        let frame_time = self.roff_list[&roff_id].frame_time;

        if roff_ent.m_roff_frame >= roff_entries {
            // We are done roffing, so stop moving and flag this ent to be
            // removed.
            self.set_lerp(
                &mut ent.s.pos,
                trType_t::TR_STATIONARY,
                origin,
                None,
                sv_time,
                lerp,
            );
            self.set_lerp(
                &mut ent.s.apos,
                trType_t::TR_STATIONARY,
                angle,
                None,
                sv_time,
                lerp,
            );
            // Server path: !mIsClient, so clear the flag.
            ent.r.mIsRoffing = qfalse;
            return false;
        }

        let entry = self.roff_list[&roff_id].move_rotate_list[roff_ent.m_roff_frame as usize];

        let result = if roff_ent.m_translated {
            let mut f = [0.0f32; 3];
            let mut r = [0.0f32; 3];
            let mut u = [0.0f32; 3];
            AngleVectors(
                roff_ent.m_start_angles,
                Some(&mut f),
                Some(&mut r),
                Some(&mut u),
            );
            // result = f*offset[0]; result += -offset[1]*r; result += offset[2]*u
            let mut result = [0.0f32; 3];
            _VectorScale(f, entry.origin_offset[0], &mut result);
            _VectorMA(result, -entry.origin_offset[1], r, &mut result);
            _VectorMA(result, entry.origin_offset[2], u, &mut result);
            result
        } else {
            entry.origin_offset
        };

        // Set up our origin interpolation.
        self.set_lerp(
            &mut ent.s.pos,
            trType_t::TR_LINEAR,
            origin,
            Some(result),
            sv_time,
            lerp,
        );

        // Set up our angle interpolation.
        self.set_lerp(
            &mut ent.s.apos,
            trType_t::TR_LINEAR,
            angle,
            Some(entry.rotate_offset),
            sv_time,
            lerp,
        );

        if entry.start_note >= 0 {
            for i in 0..entry.num_notes {
                let note = self.note_track_string(roff_id, (entry.start_note + i) as usize);
                self.process_note(ent_index, &note, host);
            }
        }

        // Advance ROFF frames and lock to a 10hz cycle.
        let next = sv_time + frame_time;
        self.roff_ent_list[ent_index].m_roff_frame += 1;
        self.roff_ent_list[ent_index].m_next_roff_time = next;

        // rww - npcs need to know when they're getting roff'd.
        ent.next_roff_time = next;

        true
    }

    /// Raven `CROFFSystem::ProcessNote(SROFFEntity *roff_ent, char *note)` —
    /// splits `note` on control characters and, per non-empty token, fires
    /// the note-track vmcall.
    ///
    /// The client twin (`cgvm, CG_ROFF_NOTETRACK_CALLBACK`, `:951-952`) is
    /// `#ifndef DEDICATED` — compiled out under WinDed (ROFF-D3): a §20
    /// zero-caller drop, not transcribed here. The server path calls
    /// [`EngineHost::vm_call`] with [`VmSlot::Gvm`] and
    /// `gameExport_t::GAME_ROFF_NOTETRACK_CALLBACK`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:927-961`
    fn process_note(&mut self, ent_index: usize, note: &str, host: &mut impl EngineHost) {
        let ent_id = self.roff_ent_list[ent_index].m_ent_id;
        let bytes = note.as_bytes();

        let mut pos = 0;
        while pos < bytes.len() && bytes[pos] != 0 {
            // Skip leading control/white-space (Raven's signed-char `< ' '`).
            while pos < bytes.len() && bytes[pos] != 0 && (bytes[pos] as i8) < b' ' as i8 {
                pos += 1;
            }

            // Collect the printable token.
            let mut temp: Vec<u8> = Vec::new();
            while pos < bytes.len() && bytes[pos] != 0 && (bytes[pos] as i8) >= b' ' as i8 {
                temp.push(bytes[pos]);
                pos += 1;
            }

            if !temp.is_empty() {
                // Server path (client twin dropped, ROFF-D3). The token has no
                // interior NUL (all bytes >= ' '), so `CString::new` succeeds;
                // its pointer is the `char *notetrack` arg. Held alive across
                // the call.
                let cstr = CString::new(temp).expect("note token has no interior NUL");
                let args = [ent_id as isize, cstr.as_ptr() as isize];
                host.vm_call(
                    VmSlot::Gvm,
                    gameExport_t::GAME_ROFF_NOTETRACK_CALLBACK as i32,
                    &args,
                );
            }
        }
    }

    /// Raven `CROFFSystem::SetLerp(trajectory_t *tr, trType_t type, vec3_t
    /// origin, vec3_t delta, int time, int rate)` — writes `trType`/`trTime`/
    /// `trBase`; `trDelta = delta*rate` when `delta` is non-NULL, else
    /// clears `trDelta`. `delta: Option<vec3_t>` mirrors the NULL check
    /// (`:1031`).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:1024-1039`
    fn set_lerp(
        &self,
        tr: &mut trajectory_t,
        tr_type: trType_t,
        origin: vec3_t,
        delta: Option<vec3_t>,
        time: i32,
        rate: i32,
    ) {
        tr.trType = tr_type;
        tr.trTime = time;
        tr.trBase = origin; // VectorCopy( origin, tr->trBase )

        // Check for a NULL delta.
        match delta {
            Some(d) => {
                // VectorScale( delta, rate, tr->trDelta )
                _VectorScale(d, rate as f32, &mut tr.trDelta);
            }
            None => {
                // VectorClear( tr->trDelta )
                tr.trDelta = [0.0, 0.0, 0.0];
            }
        }
    }

    /// Raven `CROFFSystem::ClearLerp(SROFFEntity *roff_ent)` — forces both
    /// trajectories `TR_STATIONARY` at `ROFF_SAMPLE_RATE`.
    ///
    /// The `if (mIsClient)` branch (`:979-990`) is `#ifndef DEDICATED` —
    /// compiled out under WinDed (ROFF-D3): a §20 zero-caller drop, not
    /// transcribed here. The server path resolves `SV_GentityNum`; NULL
    /// entity → `false`.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.cpp:973-1011`
    fn clear_lerp(&mut self, ent_index: usize, host: &mut impl EngineHost) -> bool {
        // Server path only (the `mIsClient` branch is a §20 DEDICATED drop,
        // ROFF-D3). Find the entity to apply the roff to.
        let ent_id = self.roff_ent_list[ent_index].m_ent_id;
        let ent = host.gentity(ent_id);
        if ent.is_null() {
            // bad stuff
            return false;
        }
        // SAFETY: non-NULL per the check; §D11 seam deref, disjoint from `self`.
        let ent = unsafe { &mut *ent };

        let origin = ent.r.currentOrigin;
        let angle = ent.r.currentAngles;
        let time = host.sv_time();

        self.set_lerp(
            &mut ent.s.pos,
            trType_t::TR_STATIONARY,
            origin,
            None,
            time,
            ROFF_SAMPLE_RATE,
        );
        self.set_lerp(
            &mut ent.s.apos,
            trType_t::TR_STATIONARY,
            angle,
            None,
            time,
            ROFF_SAMPLE_RATE,
        );

        true
    }

    /// Reads the `index`-th cached note-track string out of the roff's packed
    /// blob (ROFF-V5) as an owned `String`, mirroring the oracle's
    /// `mNoteTrackIndexes[index]` `char*` deref. Non-UTF-8 bytes are replaced
    /// (note tracks are ASCII in practice); the value is consumed byte-wise by
    /// [`Self::process_note`].
    fn note_track_string(&self, roff_id: i32, index: usize) -> String {
        let croff = &self.roff_list[&roff_id];
        let start = croff.note_track_offsets[index];
        let len = c_strlen(&croff.note_track_blob, start);
        String::from_utf8_lossy(&croff.note_track_blob[start..start + len]).into_owned()
    }
}

// -------------------------------------------------------------------------
// Free helpers — the raw-byte readers and the vector math the oracle reaches
// through `q_math` macros/functions (inlined here: `mp_engine_qcommon` cannot
// depend on `mp_game`, which owns `AngleVectors`/`VectorScale`/`VectorMA`).
// -------------------------------------------------------------------------

/// Reads a little-endian `i32` at byte `off` (the fixed on-disk width, ROFF-D4).
fn read_i32_le(data: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

/// Reads a little-endian `f32` at byte `off` (the fixed on-disk width, ROFF-D4).
fn read_f32_le(data: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

/// Reads three consecutive little-endian `f32`s at byte `off` — one
/// `vec3_t`-worth of the on-disk `mOriginOffset`/`mRotateOffset`.
fn read_vec3_le(data: &[u8], off: usize) -> vec3_t {
    [
        read_f32_le(data, off),
        read_f32_le(data, off + 4),
        read_f32_le(data, off + 8),
    ]
}

/// C `strlen` over a byte buffer from `start` — counts bytes up to the NUL
/// terminator (or the buffer end), mirroring the note-track blob walk.
fn c_strlen(data: &[u8], start: usize) -> usize {
    let mut n = 0;
    while start + n < data.len() && data[start + n] != 0 {
        n += 1;
    }
    n
}

/// Raven `VectorScale( v, s, out )` — `out[i] = v[i] * s`.
/// Source: `oracle/codemp/game/q_shared.h` (macro)
#[cfg(test)]
mod tests {
    use super::*;

    // Helpers to author `.rof` bytes to the on-disk layout (ROFF-D4).
    fn push_f32(v: &mut Vec<u8>, x: f32) {
        v.extend_from_slice(&x.to_le_bytes());
    }
    fn push_i32(v: &mut Vec<u8>, x: i32) {
        v.extend_from_slice(&x.to_le_bytes());
    }

    /// ROFF-V1: the header compare passes a valid file (mVersion low byte is
    /// nonzero) and rejects an exact `"ROFF\0"` header (version 0).
    #[test]
    fn is_roff_v1_header_quirk() {
        let sys = RoffSystem::default();

        // Valid v1: "ROFF", version 1, count 1.0 → passes.
        let mut good = b"ROFF".to_vec();
        push_i32(&mut good, 1);
        push_f32(&mut good, 1.0);
        assert!(sys.is_roff(&good));

        // "ROFF" + version 0 → header bytes are exactly `R O F F \0`, so the
        // strcmp matches ("equal") and `!strcmp` rejects it as a "bad header".
        // This is the ONLY input the header check actually rejects.
        let mut zero_ver = b"ROFF".to_vec();
        push_i32(&mut zero_ver, 0);
        push_f32(&mut zero_ver, 1.0);
        assert!(!sys.is_roff(&zero_ver));

        // ROFF-V1 quirk: a wrong magic ("NOPE") still PASSES the header check —
        // `strcmp("NOPE…","ROFF")` is nonzero, so `!strcmp` is false and the
        // (broken) header gate never fires. With a valid version/count the file
        // is accepted. Reproduced faithfully, not "fixed".
        let mut bad_magic = b"NOPE".to_vec();
        push_i32(&mut bad_magic, 1);
        push_f32(&mut bad_magic, 1.0);
        assert!(sys.is_roff(&bad_magic));

        // Valid magic/version but non-positive count → rejected.
        let mut bad_count = b"ROFF".to_vec();
        push_i32(&mut bad_count, 1);
        push_f32(&mut bad_count, 0.0);
        assert!(!sys.is_roff(&bad_count));
    }

    /// InitROFF (v1) synthesizes empty note info and applies default timing.
    #[test]
    fn init_roff_v1_decode_and_fix_bad_angles() {
        let sys = RoffSystem::default();

        let mut data = b"ROFF".to_vec();
        push_i32(&mut data, 1); // version
        push_f32(&mut data, 1.0); // count = 1
                                  // one TROFFEntry: origin, rotate (a bad angle 200 -> wraps to -160)
        push_f32(&mut data, 1.0);
        push_f32(&mut data, 2.0);
        push_f32(&mut data, 3.0);
        push_f32(&mut data, 200.0); // > 180 -> -160
        push_f32(&mut data, -190.0); // < -180 -> 170
        push_f32(&mut data, 45.0); // in range

        let mut obj = Croff::default();
        assert!(sys.init_roff(&data, &mut obj));

        assert_eq!(obj.frame_time, 100); // 1000 / ROFF_SAMPLE_RATE
        assert_eq!(obj.lerp, ROFF_SAMPLE_RATE);
        assert_eq!(obj.move_rotate_list.len(), 1);
        let e = obj.move_rotate_list[0];
        assert_eq!(e.origin_offset, [1.0, 2.0, 3.0]);
        assert_eq!(e.rotate_offset, [-160.0, 170.0, 45.0]);
        assert_eq!(e.start_note, -1);
        assert_eq!(e.num_notes, 0);
        assert!(obj.note_track_offsets.is_empty());
    }

    /// InitROFF2 (v2) reads the frame rate and copies packed note-track
    /// strings into one blob with per-string offsets (ROFF-V5).
    #[test]
    fn init_roff2_note_track_blob() {
        let sys = RoffSystem::default();

        let mut data = b"ROFF".to_vec();
        push_i32(&mut data, 2); // version = ROFF_NEW_VERSION
        push_i32(&mut data, 1); // count = 1
        push_i32(&mut data, 20); // frame rate
        push_i32(&mut data, 2); // num notes
                                // one TROFF2Entry
        push_f32(&mut data, 0.0);
        push_f32(&mut data, 0.0);
        push_f32(&mut data, 0.0);
        push_f32(&mut data, 0.0);
        push_f32(&mut data, 0.0);
        push_f32(&mut data, 0.0);
        push_i32(&mut data, 0); // start note
        push_i32(&mut data, 2); // num notes
                                // two packed NUL-terminated note strings
        data.extend_from_slice(b"alpha\0");
        data.extend_from_slice(b"beta\0");

        let mut obj = Croff::default();
        assert!(sys.init_roff2(&data, &mut obj));

        assert_eq!(obj.frame_time, 20);
        assert_eq!(obj.lerp, 50); // 1000 / 20
        assert_eq!(obj.note_track_offsets.len(), 2);
        assert_eq!(obj.note_track_offsets[0], 0);
        assert_eq!(obj.note_track_offsets[1], 6); // "alpha\0".len()

        // Read both strings back through the same path apply_roff uses (a
        // system with the roff cached).
        let mut sys2 = RoffSystem::default();
        sys2.roff_list.insert(7, obj);
        assert_eq!(sys2.note_track_string(7, 0), "alpha");
        assert_eq!(sys2.note_track_string(7, 1), "beta");
    }

    /// SetLerp writes trType/trTime/trBase and scales (or clears) trDelta.
    #[test]
    fn set_lerp_delta_and_null() {
        let sys = RoffSystem::default();

        let mut tr = trajectory_t {
            trType: trType_t::TR_STATIONARY,
            trTime: 0,
            trDuration: 5,
            trBase: [0.0, 0.0, 0.0],
            trDelta: [9.0, 9.0, 9.0],
        };

        // Non-NULL delta: trDelta = delta * rate.
        sys.set_lerp(
            &mut tr,
            trType_t::TR_LINEAR,
            [1.0, 2.0, 3.0],
            Some([1.0, -2.0, 0.5]),
            100,
            10,
        );
        assert_eq!(tr.trType, trType_t::TR_LINEAR);
        assert_eq!(tr.trTime, 100);
        assert_eq!(tr.trBase, [1.0, 2.0, 3.0]);
        assert_eq!(tr.trDelta, [10.0, -20.0, 5.0]);
        // trDuration untouched.
        assert_eq!(tr.trDuration, 5);

        // NULL delta clears trDelta.
        sys.set_lerp(
            &mut tr,
            trType_t::TR_STATIONARY,
            [4.0, 5.0, 6.0],
            None,
            200,
            10,
        );
        assert_eq!(tr.trDelta, [0.0, 0.0, 0.0]);
    }

    /// NewID never mints 0; GetID/Unload round-trip on the cache map.
    #[test]
    fn new_id_get_id_unload() {
        let mut sys = RoffSystem::default();
        assert_eq!(sys.new_id(), 1);
        assert_eq!(sys.new_id(), 2);

        sys.roff_list.insert(
            3,
            Croff {
                id: 3,
                roff_file_path: "models/x.rof".to_string(),
                ..Default::default()
            },
        );
        assert_eq!(sys.get_id("models/x.rof"), 3);
        assert_eq!(sys.get_id("nope"), 0);
        assert!(sys.unload(3));
        assert!(!sys.unload(3));
        assert_eq!(sys.get_id("models/x.rof"), 0);
    }
}
