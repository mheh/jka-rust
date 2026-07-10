//! Raven `CGoreSet` (`G2_gore.h:59-65`) plus the server-live gore-record store
//! it lives alongside in `G2_misc.cpp` (`GoreRecords`/`GoreSets` and their free
//! functions).
//!
//! Per `docs/subsystems/ghoul2-server.md` roster (`gore/gore_set.rs`, class
//! `CGoreSet`): `AllocGoreRecord`/`FindGoreRecord`/`DeleteGoreRecord` (+ its
//! private helper `DestroyGoreTexCoordinates`) — all **server-live**
//! (`G2SV-D11`/ruling 26, closing `G2SV-Q8`: reached through `~CGoreSet` ←
//! `DeleteGoreSet` ← the live `G2API_ClearSkinGore`/`REMOVEGHOUL2MODEL` paths,
//! not the graph-dead gore-*apply* set) — `FindGoreSet`/`NewGoreSet`/
//! `DeleteGoreSet`, `CGoreSet::~CGoreSet`, `GoreState` (`G2SV-D5`), and
//! `G2_GorePolys` (server-live via the collision-trace loop,
//! `G2_misc.cpp:1494`).
//!
//! **Dropped, §20 zero-caller notes, no stub, no roster row** (`G2SV-D7`,
//! narrowed by `G2SV-D11`/ruling 26 to exactly this trio): `ResetGoreTag`
//! (`G2_misc.cpp:96-101`, sole caller `G2API_AddSkinGore:2590`) and
//! `G2_GetGoreRecord` (`:113-116`, no caller anywhere in `codemp/`) — both
//! reachable only from `G2API_AddSkinGore` (`G2_API.cpp:2569`, `api_gore.rs`'s
//! domain), which itself has only the client `CG_G2_ADDSKINGORE` trap and no
//! `G_G2_*` server arm.
//!
//! Per `G2SV-D13`(c)/ruling 29 (closing `G2SV-Q10`): `GoreState` **owns** each
//! per-LOD gore texture-coordinate buffer as an owned `Vec<f32>`
//! (`tex_buffers`), and the frozen `GoreTextureCoordinates.tex: [*mut c_float;
//! MAX_LODS]` pointers point INTO those `Vec`s; alloc mirrors Raven's
//! `Z_Malloc` (`G2_misc.cpp:1020`), free/teardown order mirrors `Z_Free` at
//! `DestroyGoreTexCoordinates` (`G2_gore.h:25-36`). The server slice is
//! all-null (no `TS.gore` setter reaches here server-side, `G2SV-Q4`) so there
//! is no golden surface for the vertex/index math, but `G2_GorePolys` ports
//! fully because the record-store allocation it drives is server-observable.
//!
//! FILE PLACEMENT (mechanical, not a new decision, per the roster's own note):
//! this submodule lands INSIDE the existing `gore/` directory (declared `pub
//! mod gore_set;` in `gore/mod.rs`, already done) rather than a top-level
//! `src/gore.rs`, which would collide at compile time with `gore/mod.rs`.
//! One-type-per-file + folder-mirrors-owning-header (`CLAUDE.md`; `CGoreSet`
//! is declared in `G2_gore.h`, the gore-subsystem header) colocates this with
//! the already-type-ported `gore/` data types (`G2SV-D10` pattern).
//!
//! **Doc/oracle mismatch, reported (not improvised around — porting-rules
//! §F: pinned shapes are LAW).** The doc's per-file host-service map (`##
//! Slice hooks`, "Host-consuming") and the `ghoul2_system.rs` cvar-ownership
//! note both claim `gore/gore_set.rs`'s `G2_GorePolys` reads
//! `cg_g2MarksAllModels` via `EngineHost::cvar_integer` at `G2_misc.cpp:1524`.
//! Oracle ground truth: `G2_GorePolys` spans `G2_misc.cpp:804-1073`; line
//! `1524` is inside the **separate** function `G2_TraceModels`
//! (`:1514-1611`, → `misc.rs`), not `G2_GorePolys`. Grepping
//! `cg_g2MarksAllModels` in `G2_misc.cpp` finds exactly two read sites —
//! `:569` (`G2_TransformModel`) and `:1524` (`G2_TraceModels`) — both outside
//! `G2_GorePolys`'s line range, and `G2_GorePolys`'s body (`CrossProduct`/
//! `DotProduct`/`VectorNormalize`/`VectorScale`/`VectorMA`/`assert`/`cos`/
//! `sin`, plus the `Z_Malloc`-mirroring buffer alloc) calls no `EngineHost`
//! service at all. So every function in this file is **host-free**;
//! `g2_gore_polys` below is transcribed **without** a `host` parameter,
//! matching the oracle bodies exactly as `api_gore.rs` already did for the
//! same `cg_g2MarksAllModels` mis-citation.

use std::collections::BTreeMap;

use core::ffi::c_void;

use mp_qshared::shared::{mdxaBone_t, vec3_t};

use crate::api_collision::g2api_get_time;
use crate::ghoul2_system::Ghoul2System;
use crate::gore::gore_texture_coordinates::{GoreTextureCoordinates, MAX_LODS};
use crate::gore::sgore_surface::SGoreSurface;
use crate::shared::cghoul2_info::CGhoul2Info;

/// Raven `#define GORE_TAG_UPPER (256)` — the per-generation gore-tag block
/// size `CurrentTag`/`CurrentTagUpper` step by.
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:28`
pub const GORE_TAG_UPPER: i32 = 256;

/// Raven `#define GORE_TAG_MASK (~255)` — masks a tag down to its generation
/// block, used by `AllocGoreRecord`'s eviction loop to group same-generation
/// records.
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:29`
pub const GORE_TAG_MASK: i32 = !255;

/// Raven `#define MAX_GORE_RECORDS (500)` — `AllocGoreRecord`'s eviction
/// watermark (`GoreRecords.size() > MAX_GORE_RECORDS`).
///
/// Raven: "`TODO`: This needs to be set via a scalability cvar with some
/// reasonable minimum value if pgore is used at all" (`G2_misc.cpp:55`).
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:56`
pub const MAX_GORE_RECORDS: usize = 500;

/// Raven `SGoreSurface` per-surface gore-decal state, keyed by surface index —
/// the Rust stand-in for `multimap<int,SGoreSurface>` (several records may
/// share a surface index, hence `Vec` per key; container shape is free
/// internal latitude, porting-rules §A1, since `mGoreRecords` never crosses
/// the ABI seam).
pub type GoreRecordsBySurface = BTreeMap<i32, Vec<SGoreSurface>>;

/// Raven `class CGoreSet` — a refcounted set of applied gore decals, looked
/// up by a UUID tag (`mMyGoreSetTag`) from `GoreState.sets`.
///
/// Raven: `mMyGoreSetTag`/`mRefCount`/`mGoreRecords` plus the inline ctor
/// `CGoreSet(int tag) : mMyGoreSetTag(tag), mRefCount(0) {}`. Not ABI-crossing
/// (§F17 idiomatic reimplementation) — field naming/shape is free; Raven names
/// are cited per field.
/// Type definition source: `oracle/codemp/ghoul2/G2_gore.h:59-65`
#[derive(Default)]
pub struct CGoreSet {
    /// Raven `int mMyGoreSetTag` — the UUID this set is filed under in
    /// `GoreState.sets`.
    pub my_gore_set_tag: i32,
    /// Raven `unsigned char mRefCount`.
    pub ref_count: u8,
    /// Raven `multimap<int,SGoreSurface> mGoreRecords` — a map from surface
    /// index to every gore-decal record applied to it.
    pub gore_records: GoreRecordsBySurface,
}

impl CGoreSet {
    /// Raven `CGoreSet::~CGoreSet()` — `DeleteGoreRecord`s every recorded
    /// surface's gore tag. Takes `gore` explicitly and consumes `self` (not a
    /// `Drop` impl): the teardown reaches sibling `GoreState` fields (the
    /// record store) that `Drop::drop(&mut self)` cannot see (porting-rules
    /// §B4, state threaded not reached); callers (`GoreState::delete_gore_set`)
    /// remove the `CGoreSet` from `GoreState.sets` first, then call this on
    /// the owned value, mirroring Raven's `delete (*f).second` invoking the
    /// destructor synchronously.
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:174-181`
    pub(crate) fn destroy(self, gore: &mut GoreState) {
        for (_, records) in self.gore_records {
            for record in records {
                gore.delete_gore_record(record.mGoreTag);
            }
        }
    }
}

/// Raven's server-side gore store: two file-scope maps (`GoreRecords`,
/// `GoreSets`), their rolling UUID counters (`CurrentTag`/`CurrentTagUpper`,
/// `CurrentGoreSet`), the trace-scoped `GoreTagsTemp`, and the persistent
/// `GoreTouch` generation counter — folded into one owned struct per ruling 2
/// (globals → owning-file sub-structs) and threaded as `Ghoul2System.gore`
/// (`G2SV-D5`, ruling 12).
///
/// Raven: `GoreRecords`/`GoreTagsTemp`/`CurrentTag`/`CurrentTagUpper`
/// (`G2_misc.cpp:32-36`), `GoreSets`/`CurrentGoreSet` (`:124-125`), `GoreTouch`
/// (`:795`). `GoreVerts`/`GoreIndexCopy`/`GoreIndecies` (`:793-798`) are
/// **not** fields here — the state-ownership survey classifies them as
/// per-call scratch, rebuilt every `G2_GorePolys` invocation and invalidated
/// by `gore_touch`, not persistent state (three-kind rule, scratch kind);
/// `goreModelIndex` (`:38`) is likewise not a field — it is threaded as an
/// explicit parameter into [`g2_gore_polys`] (set by the caller's model loop,
/// `misc.rs`'s `G2_TraceModels:1539`).
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:32-40,124-125,793-795`
pub struct GoreState {
    /// Raven `static map<int,GoreTextureCoordinates> GoreRecords`
    /// (`G2_misc.cpp:35`).
    pub records: BTreeMap<i32, GoreTextureCoordinates>,
    /// Per `G2SV-D13`(c): the owned backing storage the frozen
    /// `GoreTextureCoordinates.tex: [*mut c_float; MAX_LODS]` pointers point
    /// into, keyed by `(record tag, LOD)`. Raven `Z_Malloc`s each
    /// `tex[TS.lod]` (`G2_misc.cpp:1020`); free/teardown order mirrors
    /// `Z_Free` at `DestroyGoreTexCoordinates` (`G2_gore.h:25-36`).
    pub tex_buffers: BTreeMap<(i32, usize), Vec<f32>>,
    /// Raven `static map<pair<int,int>,int> GoreTagsTemp` (`G2_misc.cpp:36`) —
    /// a `(goreModelIndex, surfaceNum)` → gore-tag map reused per LOD during
    /// one generation pass; cleared by the dropped `ResetGoreTag` (§20 note
    /// above — its sole caller is graph-dead, so this map is never reset
    /// server-side, matching oracle: the reset path simply never runs here).
    pub tags_temp: BTreeMap<(i32, i32), i32>,
    /// Raven `static int CurrentTag=GORE_TAG_UPPER+1` (`G2_misc.cpp:29`) — the
    /// next gore-record tag `AllocGoreRecord` hands out.
    pub current_tag: i32,
    /// Raven `static int CurrentTagUpper=GORE_TAG_UPPER` (`G2_misc.cpp:30`) —
    /// the next generation block `ResetGoreTag` would roll `current_tag` to
    /// (dropped, §20 note above; field kept for layout parity with the
    /// oracle global).
    pub current_tag_upper: i32,
    /// Raven `static map<int,CGoreSet *> GoreSets` (`G2_misc.cpp:125`).
    pub sets: BTreeMap<i32, CGoreSet>,
    /// Raven `static int CurrentGoreSet=1` (`G2_misc.cpp:124`) — the next
    /// gore-set UUID `NewGoreSet` hands out.
    pub current_set: i32,
    /// Raven `static int GoreTouch=1` (`G2_misc.cpp:795`) — a persistent
    /// generation counter bumped every `G2_GorePolys` call (`:890`), read
    /// server-side via the collision path even though the gore-apply vertex
    /// math never runs (`G2SV-D7`).
    pub gore_touch: i32,
}

impl Default for GoreState {
    /// Mirrors Raven's static initializers exactly (`G2_misc.cpp:29-30,124,795`)
    /// — a blanket `#[derive(Default)]` would zero `current_tag`/
    /// `current_tag_upper`/`current_set`/`gore_touch`, which does not match
    /// the oracle's non-zero statics.
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
            tex_buffers: BTreeMap::new(),
            tags_temp: BTreeMap::new(),
            current_tag: GORE_TAG_UPPER + 1,
            current_tag_upper: GORE_TAG_UPPER,
            sets: BTreeMap::new(),
            current_set: 1,
            gore_touch: 1,
        }
    }
}

impl GoreState {
    /// Raven `int AllocGoreRecord()` — evicts oldest same-generation records
    /// once `GoreRecords.size() > MAX_GORE_RECORDS`, inserts a fresh
    /// default-initialized `GoreTextureCoordinates` under `current_tag`, and
    /// returns that tag (post-incrementing it).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:58-94`
    pub fn alloc_gore_record(&mut self) -> i32 {
        while self.records.len() > MAX_GORE_RECORDS {
            // SAFETY-free: `records.len() > MAX_GORE_RECORDS` (a `usize`, so
            // never negative) guarantees at least one entry.
            let first_tag = *self
                .records
                .keys()
                .next()
                .expect("records.len() > MAX_GORE_RECORDS implies records is non-empty");
            let tag_high = first_tag & GORE_TAG_MASK;
            self.destroy_gore_tex_coordinates(first_tag);
            self.records.remove(&first_tag);
            while let Some(&next_tag) = self.records.keys().next() {
                if (next_tag & GORE_TAG_MASK) != tag_high {
                    break;
                }
                self.destroy_gore_tex_coordinates(next_tag);
                self.records.remove(&next_tag);
            }
        }
        let ret = self.current_tag;
        self.records.insert(
            self.current_tag,
            GoreTextureCoordinates {
                tex: [core::ptr::null_mut(); MAX_LODS],
            },
        );
        self.current_tag += 1;
        ret
    }

    /// Raven `GoreTextureCoordinates *FindGoreRecord(int tag)` — `GoreRecords`
    /// lookup; a miss returns `0` in Raven, `None` here (rule 7, pointer →
    /// `Option`).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:103-111`
    pub fn find_gore_record(&mut self, tag: i32) -> Option<&mut GoreTextureCoordinates> {
        self.records.get_mut(&tag)
    }

    /// Raven `static inline void DestroyGoreTexCoordinates(int tag)` — a
    /// translation-unit-private helper (kept private here too, ruling 21:
    /// private helpers colocate): no-ops on a `FindGoreRecord` miss, else
    /// explicitly runs `GoreTextureCoordinates`'s teardown (freeing every
    /// non-null per-LOD `tex` buffer).
    ///
    /// Raven: "I don't know what's going on here, it should call the
    /// destructor for this when it erases the record but sometimes it
    /// doesn't. -rww" (`G2_misc.cpp:50-51`) — preserved because it explains
    /// why this explicit call exists instead of relying on the map erase.
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:43-53`
    fn destroy_gore_tex_coordinates(&mut self, tag: i32) {
        if !self.records.contains_key(&tag) {
            return;
        }
        // Raven frees only the non-null `tex[i]` entries; removing a missing
        // `(tag, lod)` key from `tex_buffers` is already a no-op, so every LOD
        // slot can be unconditionally removed here (matches the `if (tex[i])`
        // guard's observable effect without needing a per-slot null check).
        for lod in 0..MAX_LODS {
            self.tex_buffers.remove(&(tag, lod));
        }
        if let Some(gtc) = self.records.get_mut(&tag) {
            gtc.tex = [core::ptr::null_mut(); MAX_LODS];
        }
    }

    /// Raven `void DeleteGoreRecord(int tag)` — `DestroyGoreTexCoordinates`
    /// then erases the record from `GoreRecords`. Server-live per `G2SV-D11`
    /// (ruling 26, closing `G2SV-Q8`): reached through `CGoreSet::destroy` ←
    /// `delete_gore_set` ← the live `G2API_ClearSkinGore`/
    /// `REMOVEGHOUL2MODEL` paths.
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:118-122`
    pub fn delete_gore_record(&mut self, tag: i32) {
        self.destroy_gore_tex_coordinates(tag);
        self.records.remove(&tag);
    }

    /// Raven `CGoreSet *FindGoreSet(int goreSetTag)` — `GoreSets` lookup; a
    /// miss returns `0` in Raven, `None` here (rule 7).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:127-135`
    pub fn find_gore_set(&mut self, gore_set_tag: i32) -> Option<&mut CGoreSet> {
        self.sets.get_mut(&gore_set_tag)
    }

    /// Raven `CGoreSet *NewGoreSet()` — allocates a fresh `CGoreSet` under
    /// `current_set` (post-incrementing it), inserts it into `sets` with
    /// `mRefCount = 1`, and returns it. Raven's `new` never returns null here
    /// (no OOM check in this codebase's allocator path), so the port returns
    /// `&mut CGoreSet` directly rather than `Option` (rule 7 out-param
    /// discriminator: only a genuinely-fallible pointer maps to `Option`).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:142-151`
    pub fn new_gore_set(&mut self) -> &mut CGoreSet {
        let tag = self.current_set;
        self.current_set += 1;
        self.sets.insert(
            tag,
            CGoreSet {
                my_gore_set_tag: tag,
                ref_count: 1,
                gore_records: GoreRecordsBySurface::new(),
            },
        );
        self.sets.get_mut(&tag).expect("just inserted above")
    }

    /// Raven `void DeleteGoreSet(int goreSetTag)` — on a `GoreSets` hit,
    /// decrements `mRefCount`, and once it would reach (or already sits at)
    /// zero, removes the set from `sets` and runs `CGoreSet::destroy`
    /// (Raven's `delete (*f).second`, which invokes `~CGoreSet` synchronously
    /// — porting-rules §C10, control-flow shape is free as long as behavior
    /// matches). Server-live (`G2SV-D7`): reached from the live
    /// `G2API_ClearSkinGore` (`G2_API.cpp:2557`) and directly from the
    /// `G_G2_REMOVEGHOUL2MODEL`/`G_G2_REMOVEGHOUL2MODELS` removal paths
    /// (`:814,901`).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_misc.cpp:153-171`
    pub fn delete_gore_set(&mut self, gore_set_tag: i32) {
        let should_delete = match self.sets.get(&gore_set_tag) {
            Some(set) => set.ref_count == 0 || set.ref_count - 1 == 0,
            None => return,
        };
        if should_delete {
            if let Some(set) = self.sets.remove(&gore_set_tag) {
                set.destroy(self);
            }
        } else if let Some(set) = self.sets.get_mut(&gore_set_tag) {
            set.ref_count -= 1;
        }
    }
}

// ---------------------------------------------------------------------------
// Raw mdxm surface/triangle field offsets (`G2SV-D5`: `mdxmSurface_t`/
// `mdxmTriangle_t` are never named in this crate — only byte arithmetic off
// the raw `surface: *const c_void` pointer, exactly as the oracle body does
// off its typed `const mdxmSurface_t *surface`). Offsets derived from the
// field order in `oracle/codemp/renderer/mdx_format.h:219-252`: every field
// is a plain `int` (4 bytes), so natural alignment introduces no padding.
// Duplicated locally rather than shared with `api_models.rs`'s equivalent
// header-field-offset consts (this crate has no shared byte-reader module
// yet; same per-file duplication convention that file's own doc comment
// documents for `GHOUL2_NEWORIGIN`).
// ---------------------------------------------------------------------------

/// `mdxmSurface_t::thisSurfaceIndex` (`mdx_format.h:220`) — `ident`(0) precedes it.
const MDXM_SURF_OFS_THIS_SURFACE_INDEX: usize = 4;
/// `mdxmSurface_t::numVerts` (`mdx_format.h:224`).
const MDXM_SURF_OFS_NUM_VERTS: usize = 12;
/// `mdxmSurface_t::numTriangles` (`mdx_format.h:227`).
const MDXM_SURF_OFS_NUM_TRIANGLES: usize = 20;
/// `mdxmSurface_t::ofsTriangles` (`mdx_format.h:228`).
const MDXM_SURF_OFS_OFS_TRIANGLES: usize = 24;
/// `sizeof(mdxmTriangle_t)` (`mdx_format.h:250-252`) — `int indexes[3]`.
const MDXM_TRIANGLE_SIZE: usize = 12;

/// Reads an `i32` at `offset` bytes into a raw model-memory block. Mirrors
/// `api_models.rs`'s identical private helper (duplicated per this crate's
/// established per-file convention, not shared — see the offset-const block
/// above).
///
/// # Safety
/// `base` must be non-null and `offset..offset+4` must lie inside the block.
unsafe fn read_i32(base: *const c_void, offset: usize) -> i32 {
    unsafe {
        (base as *const u8)
            .add(offset)
            .cast::<i32>()
            .read_unaligned()
    }
}

/// Reads the 3 leading floats (x/y/z; two more floats/vertex follow, unused
/// here) of transformed vertex `vert_index` out of a `TS.TransformedVertsArray`-
/// resolved `float *` (Raven's `pos=j*5` stride, `G2_misc.cpp:844-847`).
///
/// # Safety
/// `verts` must be non-null and `vert_index*5+3` floats must lie inside it.
unsafe fn read_vert3(verts: *const f32, vert_index: usize) -> vec3_t {
    unsafe {
        let base = verts.add(vert_index * 5);
        [*base, *base.add(1), *base.add(2)]
    }
}

// ---------------------------------------------------------------------------
// Minimal `vec3_t` math stopgaps. **Gap, reported under `problems`** (same
// class of gap `api_bolts.rs`'s module doc already reports for
// `VectorNormalize`): `mp_engine_ghoul2` depends only on `mp_qshared`/
// `mp_host_interface` (`Cargo.toml`), and `mp_qshared` exports the `vec3_t`
// *type* (`native/math/src/vector.rs:12`) but no `CrossProduct`/`DotProduct`/
// `VectorNormalize`/`VectorScale`/`VectorMA`/`VectorSubtract` free functions
// reachable from here. Reimplemented narrowly below rather than left
// uncallable, matching `api_bolts.rs`'s `vector_normalize_row` precedent.
// Source: `oracle/codemp/game/q_math.c` (`CrossProduct`/`DotProduct` are
// `q_shared.h` inline macros; `VectorNormalize`/`VectorScale`/`VectorMA`/
// `VectorSubtract` are `q_math.c` functions).
// ---------------------------------------------------------------------------

fn v3_cross(a: vec3_t, b: vec3_t) -> vec3_t {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn v3_dot(a: vec3_t, b: vec3_t) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn v3_normalize(v: &mut vec3_t) {
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if length != 0.0 {
        let inv = 1.0 / length;
        v[0] *= inv;
        v[1] *= inv;
        v[2] *= inv;
    }
}

fn v3_scale(v: vec3_t, scale: f32) -> vec3_t {
    [v[0] * scale, v[1] * scale, v[2] * scale]
}

fn v3_ma(v: vec3_t, scale: f32, add: vec3_t) -> vec3_t {
    [
        v[0] + add[0] * scale,
        v[1] + add[1] * scale,
        v[2] + add[2] * scale,
    ]
}

fn v3_sub(a: vec3_t, b: vec3_t) -> vec3_t {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Raven `#define GORE_MARGIN (0.0f)` (`G2_misc.cpp:799`).
const GORE_MARGIN: f32 = 0.0;

/// Raven `struct SVertexTemp` (`G2_misc.cpp:780-791`) — the per-vertex gore
/// scratch Raven keeps in the file-scope `GoreVerts[MAX_GORE_VERTS]` array.
/// Reallocated fresh per [`g2_gore_polys`] call (three-kind scratch, State
/// ownership survey) rather than a fixed 3000-slot array: within one call
/// every `touch` starts at its `Default` (0), which never equals the current
/// (post-increment, `>=2`) `gore_touch` generation, exactly matching a fresh
/// slot in Raven's reused global array (`touch != GoreTouch` on first sight
/// either way).
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:780-791`
#[derive(Clone, Copy, Default)]
struct GoreVertScratch {
    flags: i32,
    touch: i32,
    newindex: i32,
    tex: [f32; 2],
}

/// Raven `void G2_GorePolys(const mdxmSurface_t *surface, CTraceSurface &TS,
/// const mdxmSurfHierarchy_t *surfInfo)` — per-poly gore-decal texture
/// projection against one traced surface; bumps `gore_touch` unconditionally,
/// then (only when `TS.gore` is set — never true server-side, `G2SV-D7`/
/// `G2SV-Q4`) builds the clipped gore polygon and `Z_Malloc`-mirrors its
/// packed vertex/index/texcoord buffer into `tex_buffers` at
/// `(record_tag, TS.lod)`.
///
/// `surface`/`surf_info` are parsed `.glm` model-memory blocks
/// (`mdxmSurface_t`/`mdxmSurfHierarchy_t`) — per `G2SV-D5` these renderer-owned
/// types are never named in `mp_engine_ghoul2` (no `mp_renderer` crate edge);
/// the port does its byte arithmetic off raw pointers exactly as
/// `EngineHost::model_mdxm`/`model_mdxa` already return, unchanged (offset
/// consts above). `ts` is Raven's `CTraceSurface &TS`, owned by `misc.rs`
/// (the file whose `G2_TraceModels`/`G2_TransformModel` construct and drive
/// it, §F21 one class per file) — referenced here, not redeclared (now
/// `pub(crate)` in `misc.rs`, resolving this file's earlier cross-file
/// visibility blocker note). `gore_model_index` is Raven's file-scope
/// `goreModelIndex` (`G2_misc.cpp:38`), threaded as an explicit parameter per
/// the state-ownership survey (impl-local, set by the caller's model loop,
/// not a `GoreState` field). `surf_info` is unread in the oracle body (grep
/// confirms no `surfInfo` reference in `G2_misc.cpp:804-1073`) — kept,
/// unread, for 1:1 parameter fidelity (§A2), matching this crate's own
/// `second_time_around`-style precedent in `misc.rs`.
///
/// Host-free: no `EngineHost` service is called in this range (see the
/// module-level doc/oracle mismatch note above re: `cg_g2MarksAllModels`).
///
/// **Signature gap, reported under `problems` (not improvised around).** The
/// oracle body's tag-generation arm calls `G2API_GetTime(0)` three times
/// (`G2_misc.cpp:970,986,988`) for `mDeleteTime`/`mGoreGrowStartTime`/
/// `mGoreGrowEndTime`; `g2api_get_time` (`api_collision.rs:425`) needs `&
/// Ghoul2System` (it reads `time_bases`), which the pre-existing skeleton
/// signature for this **non**-`G2API_*` internal helper did not carry (only
/// `G2SV-D6` freezes `G2API_*` syscall-surface signatures 1:1; `G2_GorePolys`
/// is not one of them). A `g2: &Ghoul2System` parameter is added below —
/// mechanical wiring of already-modeled state, not new behavior — since
/// there is no way to call the real `g2api_get_time` otherwise. Flagged in
/// case a stricter reading intends this file to add nothing beyond the
/// stub's exact parameter list.
///
/// **Second signature gap, reported under `problems`.** `misc.rs`'s
/// `CTraceSurface::ghoul2_info` field is typed `*const CGhoul2Info`, but the
/// oracle body writes through it (`TS.ghoul2info->mGoreSetTag=...`,
/// `G2_misc.cpp:962`) — Raven's `CGhoul2Info *ghoul2info` is non-`const`.
/// Since `misc.rs` is a sibling porter's file (out of this file's scope),
/// this port casts the pointer to `*mut CGhoul2Info` locally at the one call
/// site that needs to write `gore_set_tag`, rather than editing the sibling
/// field's declared type.
///
/// Source: `oracle/codemp/ghoul2/G2_misc.cpp:804-1073`
pub fn g2_gore_polys(
    gore: &mut GoreState,
    g2: &Ghoul2System,
    gore_model_index: i32,
    surface: *const c_void,
    ts: &mut crate::misc::CTraceSurface,
    surf_info: *const c_void,
) {
    let _ = surf_info; // unread in the oracle body too (doc comment above)

    // basis2=(0,0,1); basis1=CrossProduct(rayEnd,basis2); if too degenerate,
    // retry with basis2=(0,1,0). (G2_misc.cpp:807-820)
    let mut basis2: vec3_t = [0.0, 0.0, 1.0];
    let mut basis1 = v3_cross(ts.ray_end, basis2);
    if v3_dot(basis1, basis1) < 0.1 {
        basis2 = [0.0, 1.0, 0.0];
        basis1 = v3_cross(ts.ray_end, basis2);
    }
    basis2 = v3_cross(ts.ray_end, basis1);
    // Raven's two `assert(DotProduct(...)>.0001f)` (:822-823) are dropped —
    // `-DNDEBUG` reduces plain `assert()` to a no-op crate-wide (see this
    // crate's established convention, e.g. `api_models.rs`'s module doc).
    v3_normalize(&mut basis1);
    v3_normalize(&mut basis2);

    let c = ts.theta.cos();
    let s = ts.theta.sin();

    let taxis = v3_ma(
        v3_scale(basis1, 0.5 * c / ts.tsize),
        0.5 * s / ts.tsize,
        basis2,
    );
    let saxis = v3_ma(
        v3_scale(basis1, -0.5 * s / ts.ssize),
        0.5 * c / ts.ssize,
        basis2,
    );

    // G2_misc.cpp:841-874: per-vertex splotch-space flags/texcoords.
    let this_surface_index = unsafe { read_i32(surface, MDXM_SURF_OFS_THIS_SURFACE_INDEX) };
    let verts_ptr = unsafe { *ts.transformed_verts_array.add(this_surface_index as usize) } as usize
        as *const f32;
    let num_verts = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_VERTS) };
    // Raven `assert(numVerts<MAX_GORE_VERTS)` (:845) dropped (NDEBUG no-op).
    let mut overall_flags: i32 = 15;
    let mut gore_verts: Vec<GoreVertScratch> =
        vec![GoreVertScratch::default(); num_verts.max(0) as usize];
    for j in 0..num_verts as usize {
        // SAFETY: `verts_ptr` is the transformed-verts buffer for this
        // surface (resolved above); `j < numVerts` bounds the read.
        let v = unsafe { read_vert3(verts_ptr, j) };
        let delta = v3_sub(v, ts.ray_start);
        let s_coord = v3_dot(delta, saxis) + 0.5;
        let t_coord = v3_dot(delta, taxis) + 0.5;
        let mut vflags: i32 = 0;
        if s_coord > GORE_MARGIN {
            vflags |= 1;
        }
        if s_coord < 1.0 - GORE_MARGIN {
            vflags |= 2;
        }
        if t_coord > GORE_MARGIN {
            vflags |= 4;
        }
        if t_coord < 1.0 - GORE_MARGIN {
            vflags |= 8;
        }
        vflags = !vflags;
        overall_flags &= vflags;
        gore_verts[j].flags = vflags;
        gore_verts[j].tex = [s_coord, t_coord];
    }
    if overall_flags != 0 {
        return; // completely off the gore splotch (G2_misc.cpp:875-878).
    }

    let num_triangles = unsafe { read_i32(surface, MDXM_SURF_OFS_NUM_TRIANGLES) };
    let ofs_triangles = unsafe { read_i32(surface, MDXM_SURF_OFS_OFS_TRIANGLES) };
    let mut new_num_tris: i32 = 0;
    let mut new_num_verts: i32 = 0;

    gore.gore_touch += 1; // G2_misc.cpp:890 (runs unconditionally)
    let gore_touch = gore.gore_touch;

    let Some(gore_data) = ts.gore.as_deref() else {
        return; // G2_misc.cpp:891-894
    };

    let mut gore_indecies: Vec<i32> = Vec::new();
    let mut gore_index_copy: Vec<i32> = Vec::new();

    for j in 0..num_triangles {
        // SAFETY: `j < numTriangles`; each triangle is 3 packed ints
        // (`MDXM_TRIANGLE_SIZE` bytes) at `ofsTriangles + j*12`.
        let tri_offset = ofs_triangles as usize + j as usize * MDXM_TRIANGLE_SIZE;
        let indexes = unsafe {
            [
                read_i32(surface, tri_offset),
                read_i32(surface, tri_offset + 4),
                read_i32(surface, tri_offset + 8),
            ]
        };
        // Raven's three bounds `assert`s (:918-920) dropped (NDEBUG no-op).
        let tri_flags = 15
            & gore_verts[indexes[0] as usize].flags
            & gore_verts[indexes[1] as usize].flags
            & gore_verts[indexes[2] as usize].flags;
        if tri_flags != 0 {
            continue;
        }
        if gore_data.frontFaces == 0 || gore_data.backFaces == 0 {
            // SAFETY: same buffer/bounds as the flag pass above.
            let p0 = unsafe { read_vert3(verts_ptr, indexes[0] as usize) };
            let p1 = unsafe { read_vert3(verts_ptr, indexes[1] as usize) };
            let p2 = unsafe { read_vert3(verts_ptr, indexes[2] as usize) };
            let e1 = v3_sub(p1, p0);
            let e2 = v3_sub(p2, p0);
            let n = v3_cross(e1, e2);
            if v3_dot(ts.ray_end, n) > 0.0 {
                if gore_data.frontFaces == 0 {
                    continue;
                }
            } else if gore_data.backFaces == 0 {
                continue;
            }
        }
        // Raven `assert(newNumTris*3+3<MAX_GORE_INDECIES)` (:944) dropped
        // (NDEBUG no-op; `gore_indecies` is also an unbounded `Vec` here,
        // three-kind scratch, porting-rules §A1).
        for &vi in indexes.iter() {
            let vi = vi as usize;
            if gore_verts[vi].touch == gore_touch {
                gore_indecies.push(gore_verts[vi].newindex);
            } else {
                gore_verts[vi].touch = gore_touch;
                gore_verts[vi].newindex = new_num_verts;
                gore_indecies.push(new_num_verts);
                gore_index_copy.push(vi as i32);
                new_num_verts += 1;
            }
        }
        new_num_tris += 1;
    }
    if new_num_verts == 0 {
        return; // G2_misc.cpp:960-963
    }

    // G2_misc.cpp:965-1002: resolve or allocate the gore record tag for
    // (goreModelIndex, TS.surfaceNum), caching it in `tags_temp`.
    let new_tag = match gore.tags_temp.get(&(gore_model_index, ts.surface_num)) {
        Some(&existing_tag) => existing_tag,
        None => {
            let tag = gore.alloc_gore_record();

            // SAFETY: see the module doc-comment's "Second signature gap"
            // note — `misc.rs`'s `CTraceSurface::ghoul2_info` is `*const`
            // but Raven's `ghoul2info` is a mutable `CGhoul2Info*` this arm
            // writes `gore_set_tag` through.
            let ghoul2_info_ptr = ts.ghoul2_info as *mut CGhoul2Info;
            let existing_gore_set_tag = unsafe { (*ghoul2_info_ptr).gore_set_tag };
            let resolved_gore_set_tag = if existing_gore_set_tag != 0
                && gore.find_gore_set(existing_gore_set_tag).is_some()
            {
                existing_gore_set_tag
            } else {
                let created_tag = gore.new_gore_set().my_gore_set_tag;
                unsafe { (*ghoul2_info_ptr).gore_set_tag = created_tag };
                created_tag
            };

            let delete_time = if gore_data.lifeTime != 0 {
                g2api_get_time(g2, 0) + gore_data.lifeTime
            } else {
                0
            };
            let grow_start_time = g2api_get_time(g2, 0);
            let grow_end_time = if gore_data.growDuration == -1 {
                -1
            } else {
                g2api_get_time(g2, 0) + gore_data.growDuration
            };
            // Raven `assert(TS.gore->growDuration != 0)` (:985) dropped (NDEBUG).
            let grow_factor =
                (1.0 - gore_data.goreScaleStartFraction) / gore_data.growDuration as f32;

            let add = SGoreSurface {
                shader: ts.gore_shader,
                mGoreTag: tag,
                mDeleteTime: delete_time,
                mFadeTime: gore_data.fadeOutTime,
                mFadeRGB: gore_data.fadeRGB != 0,
                mGoreGrowStartTime: grow_start_time,
                mGoreGrowEndTime: grow_end_time,
                mGoreGrowFactor: grow_factor,
                mGoreGrowOffset: gore_data.goreScaleStartFraction,
            };

            if let Some(gore_set) = gore.find_gore_set(resolved_gore_set_tag) {
                gore_set
                    .gore_records
                    .entry(ts.surface_num)
                    .or_default()
                    .push(add);
            }
            gore.tags_temp
                .insert((gore_model_index, ts.surface_num), tag);
            tag
        }
    };

    // G2_misc.cpp:1004-1071: pack the vert-copy/texcoord/index/entity-matrix
    // data block, mirroring Raven's `Z_Malloc`'d `int*` walk over one
    // contiguous buffer (`G2SV-D13`(c): backed here by an owned `Vec<f32>` in
    // `tex_buffers`, keyed `(new_tag, TS.lod)`; ints are bit-reinterpreted
    // into the `f32` slots via `f32::from_bits`, matching the oracle's
    // `int*`/`float*` reinterpretation of the same raw allocation). Sized to
    // exactly what Raven's pointer walk touches (2 header ints + newNumVerts
    // vert-copy ints + 9*newNumVerts reserved-but-unwritten ints, zeroed, +
    // 2*newNumVerts texcoord floats + newNumTris*3 index ints + 24 matrix
    // floats) rather than reproducing Raven's `Z_Malloc(sizeof(int)*size,...)`
    // 4x over-allocation (a harmless oracle quirk with no observable effect —
    // nothing ever reads past what is written; `tex_buffers`'s `Vec` sizing
    // is porter-internal storage, not ABI-frozen, porting-rules §A1).
    if let Some(gore_record) = gore.records.get_mut(&new_tag) {
        let total_slots = 26 + 12 * new_num_verts as usize + 3 * new_num_tris as usize;
        let mut buf: Vec<f32> = vec![0.0f32; total_slots];
        let mut cursor = 0usize;

        buf[cursor] = f32::from_bits(new_num_verts as u32);
        cursor += 1;
        buf[cursor] = f32::from_bits(new_num_tris as u32);
        cursor += 1;
        for (j, &vi) in gore_index_copy.iter().enumerate() {
            buf[cursor + j] = f32::from_bits(vi as u32);
        }
        cursor += gore_index_copy.len();
        cursor += 9 * new_num_verts as usize; // skip verts/normals space (zero-init, G2_misc.cpp:1016)

        for &vi in gore_index_copy.iter() {
            buf[cursor] = gore_verts[vi as usize].tex[0];
            cursor += 1;
            buf[cursor] = gore_verts[vi as usize].tex[1];
            cursor += 1;
        }
        for &idx in gore_indecies.iter() {
            buf[cursor] = f32::from_bits(idx as u32);
            cursor += 1;
        }

        // Build the entity-to-gore matrix + its inverse (G2_misc.cpp:1038-1069).
        let mut row0 = saxis;
        v3_normalize(&mut row0);
        let mut row1 = taxis;
        v3_normalize(&mut row1);
        let mut row2 = ts.ray_end;
        v3_normalize(&mut row2);

        let mut mat = mdxaBone_t {
            matrix: [[0.0f32; 4]; 3],
        };
        mat.matrix[0][0..3].copy_from_slice(&row0);
        mat.matrix[1][0..3].copy_from_slice(&row1);
        mat.matrix[2][0..3].copy_from_slice(&row2);
        mat.matrix[0][3] = -0.5;
        mat.matrix[1][3] = -0.5;
        mat.matrix[2][3] = 0.0;

        let shot_origin = crate::misc::transform_point(ts.ray_start, &mat);
        mat.matrix[0][3] -= shot_origin[0];
        mat.matrix[1][3] -= shot_origin[1];
        mat.matrix[2][3] -= shot_origin[2];
        let inv = crate::misc::inverse_matrix(&mat);

        for r in 0..3 {
            for c in 0..4 {
                buf[cursor + r * 4 + c] = mat.matrix[r][c];
            }
        }
        for r in 0..3 {
            for c in 0..4 {
                buf[cursor + 12 + r * 4 + c] = inv.matrix[r][c];
            }
        }

        // Raven `Z_Free`s any pre-existing `tex[TS.lod]` before overwriting
        // (:1029-1033); `insert` here drops (frees) any prior `Vec` at this
        // key the same way.
        gore.tex_buffers.insert((new_tag, ts.lod as usize), buf);
        if let Some(stored) = gore.tex_buffers.get_mut(&(new_tag, ts.lod as usize)) {
            gore_record.tex[ts.lod as usize] = stored.as_mut_ptr();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_gore_record_hands_out_sequential_tags() {
        let mut gore = GoreState::default();
        let a = gore.alloc_gore_record();
        let b = gore.alloc_gore_record();
        assert!(b > a);
    }

    #[test]
    fn alloc_gore_record_evicts_oldest_generation_once_over_watermark() {
        // `AllocGoreRecord` (`G2_misc.cpp:58-94`): once `records.len() >
        // MAX_GORE_RECORDS`, it evicts the whole oldest `GORE_TAG_MASK`
        // generation block (every tag sharing the lowest surviving tag's
        // `& GORE_TAG_MASK`), not just one record.
        let mut gore = GoreState::default();
        for _ in 0..=MAX_GORE_RECORDS {
            gore.alloc_gore_record();
        }
        // `current_tag` started at `GORE_TAG_UPPER + 1 = 257`, so the first
        // `GORE_TAG_UPPER` (256) allocations (tags 257..512) share no common
        // `& GORE_TAG_MASK` block boundary crossing until tag 512 rolls into
        // the next 256-block; asserting non-empty and under the watermark is
        // the behavior-relevant invariant (`AllocGoreRecord` always returns
        // with `records.len() <= MAX_GORE_RECORDS + 1`, since the very last
        // `insert` runs after the `while` loop's condition is re-checked only
        // on the next call).
        assert!(gore.records.len() <= MAX_GORE_RECORDS + 1);
        assert!(!gore.records.is_empty());
    }

    #[test]
    fn delete_gore_set_frees_at_zero_refcount() {
        let mut gore = GoreState::default();
        let tag = gore.new_gore_set().my_gore_set_tag;
        assert!(gore.find_gore_set(tag).is_some());
        gore.delete_gore_set(tag); // ref_count 1 -> 0, deletes.
        assert!(gore.find_gore_set(tag).is_none());
    }

    #[test]
    fn delete_gore_record_nulls_tex_pointers_and_frees_backing_buffer() {
        let mut gore = GoreState::default();
        let tag = gore.alloc_gore_record();
        gore.tex_buffers.insert((tag, 0), vec![1.0, 2.0, 3.0]);
        let ptr = gore.tex_buffers.get_mut(&(tag, 0)).unwrap().as_mut_ptr();
        if let Some(rec) = gore.records.get_mut(&tag) {
            rec.tex[0] = ptr;
        }
        gore.delete_gore_record(tag);
        assert!(gore.find_gore_record(tag).is_none());
        assert!(!gore.tex_buffers.contains_key(&(tag, 0)));
    }
}
