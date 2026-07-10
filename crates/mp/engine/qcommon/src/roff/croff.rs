//! `CROFFSystem::CROFF` — one cached `.rof` file.
//!
//! Ported against the FROZEN design `docs/subsystems/roff.md` (roster row for
//! this class, State ownership, ROFF-D4/ROFF-V5). Pure state: id, path,
//! decoded move/rotate list, playback timing, note-track blob, and
//! client/server-used flags. This class carries **no methods of its own** —
//! see the doc note on [`Croff`] below.
//!
//! Type definition source: `oracle/codemp/qcommon/RoffSystem.h:91-118`

/// Raven `CROFFSystem::CROFF::mMoveRotateList` entry — one decoded
/// move/rotate command.
///
/// Always `TROFF2Entry`-shaped in memory regardless of source file version:
/// `InitROFF` (v1) synthesizes `start_note = -1, num_notes = 0` when copying
/// from the on-disk `TROFFEntry` (`RoffSystem.cpp:160-163`), and `InitROFF2`
/// (v2) copies `TROFF2Entry`'s note fields straight through
/// (`RoffSystem.cpp:206-209`). This is the decoded, owned form the parse
/// helpers build — not the on-disk `#[repr(C)]` layout (see
/// [`super::header::Troff2Entry`] for that; ROFF-D4).
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:82-89`
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MoveRotateEntry {
    /// Raven `float mOriginOffset[3]`.
    pub origin_offset: [f32; 3],
    /// Raven `float mRotateOffset[3]`. `FixBadAngles` wraps any component
    /// `> 180.0` or `< -180.0` by ∓360 in place (`RoffSystem.cpp:258-285`).
    pub rotate_offset: [f32; 3],
    /// Raven `int mStartNote` — note track info.
    pub start_note: i32,
    /// Raven `int mNumNotes` — note track info.
    pub num_notes: i32,
}

/// Raven `CROFFSystem::CROFF` — one cached `.rof` file.
///
/// Raven: "An individual ROFF object, contains actual rotation/offset
/// information".
///
/// **No methods port for this class.** The `CROFF(const char*, int)` ctor
/// (`RoffSystem.cpp:20-28`, sets path/id and clears the list pointers) is
/// absorbed into `RoffSystem`'s `cache` seam method / `InitROFF`/`InitROFF2`
/// parse helpers (`roff_system.rs`), which build a `Croff` value directly; the
/// default ctor `CROFF()` (`RoffSystem.h:111-114`) has zero live callers
/// (only `new CROFF(file, id)` is ever constructed, `RoffSystem.cpp:340`) and
/// is a §20 drop. The dtor (`RoffSystem.cpp:41-53`, scalar-deleting
/// `mMoveRotateList` and the packed `mNoteTrackIndexes[0]` blob) is superseded
/// by Rust ownership — `Vec`/`String` drop themselves; no manual free
/// (ROFF-V5, porting-rules §9).
///
/// Type definition source: `oracle/codemp/qcommon/RoffSystem.h:94-118`
#[derive(Debug, Clone, Default)]
pub struct Croff {
    /// Raven `int mID` — id for this roff file.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:100`
    pub id: i32,

    /// Raven `char mROFFFilePath[MAX_QPATH]` — roff file path.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:101`
    pub roff_file_path: String,

    /// Raven `TROFF2Entry *mMoveRotateList`, sized by `int mROFFEntries` — the
    /// decoded move/rotate command list. `.len()` supersedes the separate
    /// `mROFFEntries` counter (owned `Vec`, not a raw pointer + count; §B5).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:102,105`
    pub move_rotate_list: Vec<MoveRotateEntry>,

    /// Raven `int mFrameTime` — frame rate in ms/frame (v1 default:
    /// `1000/ROFF_SAMPLE_RATE` = 100; v2: read from file's `mFrameRate`).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:103`
    pub frame_time: i32,

    /// Raven `int mLerp` — lerp rate in FPS (v1 default: `ROFF_SAMPLE_RATE` =
    /// 10; v2: `1000/mFrameRate`).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:104`
    pub lerp: i32,

    /// Raven `int mNumNoteTracks` + `char **mNoteTrackIndexes` — the packed,
    /// NUL-terminated note-track strings copied out of the file
    /// (`InitROFF2`, `RoffSystem.cpp:214-236`). ROFF-V5: Rust owns the blob as
    /// one packed buffer plus offset indices instead of the C++ scalar-deleted
    /// interior-pointer array (free internal representation, §F/§A1 — Golden
    /// A pins only the decoded strings, not this in-memory form). `.len()` of
    /// [`Self::note_track_offsets`] supersedes the separate `mNumNoteTracks`
    /// counter.
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:106-107`
    pub note_track_blob: Vec<u8>,

    /// Byte offsets into [`Self::note_track_blob`] where each NUL-terminated
    /// note-track string starts — mirrors `mNoteTrackIndexes[i]` (ROFF-V5).
    pub note_track_offsets: Vec<usize>,

    /// Raven `qboolean mUsedByClient` — set from `isClient` on a successful
    /// `Cache` (`RoffSystem.cpp:354-361`).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:108`
    pub used_by_client: bool,

    /// Raven `qboolean mUsedByServer` — set from `!isClient` on a successful
    /// `Cache` (`RoffSystem.cpp:354-361`).
    ///
    /// Source: `oracle/codemp/qcommon/RoffSystem.h:109`
    pub used_by_server: bool,
}
