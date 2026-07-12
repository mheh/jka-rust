//! `GameWorld` — the one owned module-island instance (STATE-D1/D9, FROZEN).

use core::ffi::c_int;

use mp_qshared::common::mp::gentity_t;
use mp_qshared::shared::{MAX_CLIENTS, MAX_GENTITIES};

use crate::client::gclient_t;
use crate::game_cvars::GameCvars;
use crate::level::level_locals::level_locals_t;
use crate::world::EntityId;

/// A value type owned by the module crate. NOT a global. Field types are the
/// EXISTING Raven-faithful, already-offset-asserted structs (§D12) — exactly the
/// structs the raw `LocateGameData` seam aliases into.
///
/// Source: `docs/architecture/state-ownership.md` § `GameWorld` (STATE-D1).
pub struct GameWorld {
    /// `level` (`level_locals_t`, `g_main.c:9`).
    pub level: level_locals_t,
    /// `g_entities[MAX_GENTITIES]` (`g_main.c:27`; contiguous `#[repr(C)]`,
    /// size-asserted 1832 B).
    pub g_entities: Box<[gentity_t; MAX_GENTITIES]>,
    /// `g_clients[MAX_CLIENTS]` (reached as `level.clients`, `g_main.c:28`;
    /// asserted 7344 B). MP only.
    pub clients: Box<[gclient_t; MAX_CLIENTS]>,
    /// Raven's ~136 file-scope `vmCvar_t` cvar handles, grouped as one
    /// GameWorld sub-struct (file-scope globals become GameWorld
    /// fields). Not part of the `LocateGameData` alias set.
    /// Source: `oracle/codemp/game/g_main.c:230-475`
    pub cvars: GameCvars,

    /// Raven's remaining game-tier mutable file-scope globals/statics as one
    /// owned sub-struct (grouped by owning `.c` file). Pass-2
    /// porters reach these through `ctx.world.globals`; they never add a field.
    /// Source: `crate::game_globals::GameGlobals`
    pub globals: crate::game_globals::GameGlobals,

    /// `w_force.c` file-scope loop-sound handles (file-scope
    /// mutable globals become GameWorld fields, grouped by owning .c file).
    /// Cached `G_SoundIndex` results, lazily filled in `WP_InitForcePowers`.
    /// Source: `oracle/codemp/game/w_force.c:24-34`
    pub speedLoopSound: c_int,
    pub rageLoopSound: c_int,
    pub protectLoopSound: c_int,
    pub absorbLoopSound: c_int,
    pub seeLoopSound: c_int,
    pub ysalamiriLoopSound: c_int,

    /// `NPC_utils.c` file-scope globals (file-scope mutable
    /// globals become GameWorld fields, grouped by owning .c file).
    /// Source: `oracle/codemp/game/NPC_utils.c:7-9`
    pub teamNumbers: [c_int; 4],
    pub teamStrength: [c_int; 4],
    pub teamCounter: [c_int; 4],

    /// `g_mem.c` file-scope globals (file-scope mutable
    /// globals become GameWorld fields, grouped by owning .c file).
    /// Memory pool for G_Alloc (256 KB), and current allocation point.
    /// Source: `oracle/codemp/game/g_mem.c:13-14`
    pub memoryPool: Box<[u8; 262144]>, // 256 * 1024
    pub allocPoint: c_int,

    /// The bg tier's session-lifetime state: the anim/saber/
    /// vehicle tables, the `BG_Alloc` pool, and the RNG. Game reaches the
    /// LCG as `world.bg_state.rng`; `Pmove` borrows this to build a
    /// `PmoveContext` each call.
    /// Source: `crate::bg_channel::BgState`
    pub bg_state: crate::bg_channel::BgState,

    /// `g_misc.c` file-scope `refTagOwnerMap[MAX_TAG_OWNERS]` (file-scope
    /// mutable globals become GameWorld fields, grouped by owning
    /// .c file).
    /// Source: `oracle/codemp/game/g_misc.c:2886`
    pub refTagOwnerMap:
        Box<[crate::level::tag_owner::tagOwner_t; crate::level::tag_owner::MAX_TAG_OWNERS]>,

    /// `char gSharedBuffer[MAX_G_SHARED_BUFFER_SIZE]`, the module's
    /// engine-registered shared-memory region (`trap_SV_RegisterSharedMemory`).
    /// Untyped raw bytes, same shape as `memoryPool` above.
    /// Source: `oracle/codemp/game/g_main.c:881`
    pub gSharedBuffer: Box<[u8; crate::g_local_consts::MAX_G_SHARED_BUFFER_SIZE]>,
}

impl GameWorld {
    /// Borrow entity `id` out of the owned `g_entities` arena (§B5). Safe: the
    /// world owns the arena, so this is a plain checked index, not pointer math.
    ///
    /// Source: `docs/architecture/state-ownership.md` § `EntityId` (§B5).
    #[inline]
    pub fn entity(&self, id: EntityId) -> &gentity_t {
        &self.g_entities[id.index()]
    }

    /// Mutable [`Self::entity`].
    #[inline]
    pub fn entity_mut(&mut self, id: EntityId) -> &mut gentity_t {
        &mut self.g_entities[id.index()]
    }

    /// Borrow client `i` out of the owned `clients` arena. `i` is the Raven
    /// client number (`0..MAX_CLIENTS`), the same index Raven uses for
    /// `level.clients[i]` / `g_entities[i].client`.
    #[inline]
    pub fn client(&self, i: usize) -> &gclient_t {
        &self.clients[i]
    }

    /// Mutable [`Self::client`].
    #[inline]
    pub fn client_mut(&mut self, i: usize) -> &mut gclient_t {
        &mut self.clients[i]
    }

    /// Builds the zeroed island (STATE-D9), then wires `level`'s
    /// self-referencing back-pointers in the allocate-first order — the latter in
    /// `G_InitGame`'s dispatched arm, not here. Uses `native_platform::zeroed_box`
    /// for the ~1.83 MB entity array (heap-built, never transits the stack).
    ///
    /// Source: `docs/architecture/state-ownership.md` § `GameWorld::zeroed` (STATE-D9).
    pub fn zeroed() -> Self {
        // The frozen STATE-D9 sketch verbatim: zeroed heap boxes first; the
        // level.gentities/clients + entities[i].client back-pointers alias them
        // AFTER they exist, in G_InitGame's dispatched arm (g_main.c:978-988) —
        // not here.
        let g_entities = native_platform::zeroed_box::<[gentity_t; MAX_GENTITIES]>();
        // The zeroed bytes leave each entity's FnId<EntXxx> handler fields as
        // None by construction (zero == None, std-guaranteed via
        // Option<NonZeroU8>) — no post-zero fixup needed.
        let clients = native_platform::zeroed_box::<[gclient_t; MAX_CLIENTS]>();
        let level = *native_platform::zeroed_box::<level_locals_t>();
        let memoryPool = native_platform::zeroed_box::<[u8; 262144]>();
        let refTagOwnerMap = native_platform::zeroed_box::<
            [crate::level::tag_owner::tagOwner_t; crate::level::tag_owner::MAX_TAG_OWNERS],
        >();
        let gSharedBuffer =
            native_platform::zeroed_box::<[u8; crate::g_local_consts::MAX_G_SHARED_BUFFER_SIZE]>();
        // Keep in sync with `zeroed_boxed()`: every field in this (compiler-
        // exhaustive) literal needs a matching `addr_of_mut!().write()` there,
        // or its `assume_init` is UB on the missed field.
        GameWorld {
            level,
            g_entities,
            clients,
            cvars: GameCvars::default(),
            globals: crate::game_globals::GameGlobals::default(),
            speedLoopSound: 0,
            rageLoopSound: 0,
            protectLoopSound: 0,
            absorbLoopSound: 0,
            seeLoopSound: 0,
            ysalamiriLoopSound: 0,
            teamNumbers: [0; 4],
            teamStrength: [0; 4],
            teamCounter: [0; 4],
            memoryPool,
            allocPoint: 0,
            // Zeroed session state with the LCG seeded to Raven's `holdrand`.
            bg_state: crate::bg_channel::BgState::new(),
            refTagOwnerMap,
            gSharedBuffer,
        }
    }

    /// Builds the zeroed island directly on the heap, field-by-field into an
    /// uninitialized `Box`, so the ~1.4 MB `GameWorld` (and its ~0.5 MB inline
    /// `globals` temporary) never transits the caller's stack by value. This is
    /// the engine-path constructor (`vmMain(GAME_INIT)`, `lib.rs`): the engine
    /// calls `vmMain` from a deep stack where a by-value `zeroed()` — whose
    /// return slot plus the `Some(..)` temporary each hold a full world image —
    /// overflowed the guard page. Same resulting values as [`Self::zeroed`];
    /// only where the bytes land differs.
    pub fn zeroed_boxed() -> Box<Self> {
        // `Box::new_uninit` allocates the storage on the heap without ever
        // materializing a `GameWorld` (or `MaybeUninit<GameWorld>`) on the
        // stack; every field is then written in place exactly once, so the
        // final `assume_init` observes a fully-initialized value (the real
        // `Box` fields — `g_entities`, `nodetable`, `bg_state`'s `Vec`s — are
        // written as live values, never left null).
        let mut boxed: Box<core::mem::MaybeUninit<GameWorld>> = Box::new_uninit();
        let p: *mut GameWorld = boxed.as_mut_ptr();
        // SAFETY: `p` points at freshly-allocated, correctly-aligned storage for
        // one `GameWorld`; each `addr_of_mut!` write initializes a distinct
        // field exactly once before `assume_init`.
        unsafe {
            use core::ptr::addr_of_mut;
            addr_of_mut!((*p).level).write(*native_platform::zeroed_box::<level_locals_t>());
            addr_of_mut!((*p).g_entities).write(native_platform::zeroed_box());
            // The zeroed bytes leave each entity's FnId<EntXxx> handler fields as
            // None by construction (zero == None, std-guaranteed via
            // Option<NonZeroU8>) — no post-zero fixup needed.
            addr_of_mut!((*p).clients).write(native_platform::zeroed_box());
            addr_of_mut!((*p).cvars).write(GameCvars::default());
            addr_of_mut!((*p).globals).write(crate::game_globals::GameGlobals::default());
            addr_of_mut!((*p).speedLoopSound).write(0);
            addr_of_mut!((*p).rageLoopSound).write(0);
            addr_of_mut!((*p).protectLoopSound).write(0);
            addr_of_mut!((*p).absorbLoopSound).write(0);
            addr_of_mut!((*p).seeLoopSound).write(0);
            addr_of_mut!((*p).ysalamiriLoopSound).write(0);
            addr_of_mut!((*p).teamNumbers).write([0; 4]);
            addr_of_mut!((*p).teamStrength).write([0; 4]);
            addr_of_mut!((*p).teamCounter).write([0; 4]);
            addr_of_mut!((*p).memoryPool).write(native_platform::zeroed_box());
            addr_of_mut!((*p).allocPoint).write(0);
            addr_of_mut!((*p).bg_state).write(crate::bg_channel::BgState::new());
            addr_of_mut!((*p).refTagOwnerMap).write(native_platform::zeroed_box());
            addr_of_mut!((*p).gSharedBuffer).write(native_platform::zeroed_box());
            boxed.assume_init()
        }
    }
}
