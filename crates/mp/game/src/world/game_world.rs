//! `GameWorld` is the one owned module-island instance.

use core::ffi::c_int;
use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};
use std::ffi::CString;

use crate::entity::gentity_t;
use mp_qshared::shared::{MAX_CLIENTS, MAX_GENTITIES};

use crate::client::gclient_t;
use crate::game_cvars::GameCvars;
use crate::level::level_locals::level_locals_t;
use crate::world::guarded_entities::GuardedEntities;
use crate::world::EntityId;

/// A value type owned by the module crate, not a global.
/// Field types are the existing, already-offset-asserted structs that match Raven's layout (§D12),
/// the same structs the raw `LocateGameData` seam aliases into.
///
/// Source: `docs/architecture/state-ownership.md` § `GameWorld`.
pub struct GameWorld {
    /// `level` (`level_locals_t`, `g_main.c:9`).
    pub level: level_locals_t,
    /// `g_entities[MAX_GENTITIES]` (`g_main.c:27`, contiguous `#[repr(C)]`, size-asserted 1832 B),
    /// plus the [`GuardedEntities`] guard slot for the engine's `SV_GentityNum(-1)` read.
    /// `Deref` keeps `g_entities[i]` and `.as_mut_ptr()` pointed at the real element 0.
    pub g_entities: Box<GuardedEntities>,
    /// `g_clients[MAX_CLIENTS]` (reached as `level.clients`, `g_main.c:28`, asserted 7344 B).
    /// MP only.
    pub clients: Box<[gclient_t; MAX_CLIENTS]>,
    /// Raven's ~136 file-scope `vmCvar_t` cvar handles, grouped as one `GameWorld` sub-struct.
    /// File-scope globals become `GameWorld` fields.
    /// Not part of the `LocateGameData` alias set.
    /// Source: `oracle/codemp/game/g_main.c:230-475`
    pub cvars: GameCvars,

    /// Raven's remaining game-tier mutable file-scope globals and statics live in one
    /// owned sub-struct, grouped by owning `.c` file.
    /// Code reaches these through `ctx.world.globals` and never adds a new field here directly.
    /// Source: `crate::game_globals::GameGlobals`
    pub globals: crate::game_globals::GameGlobals,

    /// `w_force.c` file-scope loop-sound handles.
    /// File-scope mutable globals become `GameWorld` fields, grouped by owning `.c` file.
    /// Cached `G_SoundIndex` results, lazily filled in `WP_InitForcePowers`.
    /// Source: `oracle/codemp/game/w_force.c:24-34`
    pub speedLoopSound: c_int,
    pub rageLoopSound: c_int,
    pub protectLoopSound: c_int,
    pub absorbLoopSound: c_int,
    pub seeLoopSound: c_int,
    pub ysalamiriLoopSound: c_int,

    /// `NPC_utils.c` file-scope globals.
    /// File-scope mutable globals become `GameWorld` fields, grouped by owning `.c` file.
    /// Source: `oracle/codemp/game/NPC_utils.c:7-9`
    pub teamNumbers: [c_int; 4],
    pub teamStrength: [c_int; 4],
    pub teamCounter: [c_int; 4],

    /// `g_mem.c` file-scope globals.
    /// File-scope mutable globals become `GameWorld` fields, grouped by owning `.c` file.
    /// Memory pool for `G_Alloc` (256 KB), and the current allocation point.
    /// Source: `oracle/codemp/game/g_mem.c:13-14`
    pub memoryPool: Box<[u8; 262144]>, // 256 * 1024
    pub allocPoint: c_int,

    /// Level-lifetime, append-only ownership record for the five `*mut c_char` prefix string slots
    /// and `behaviorSet`.
    /// This replaces the string half of Raven's `G_Alloc` bump pool (`memoryPool`, which now serves
    /// only ICARUS `parms_t`).
    /// The prefix slots keep their `*mut c_char` layout permanently, for the drop-in engine ABI.
    /// Their bytes live in these owned `CString`s, and each write pushes a fresh `CString`,
    /// then stores its `.as_ptr()` into the slot.
    /// Entries are never dropped or replaced on entity free or slot rewrite.
    /// `alias_from` and engine-side ICARUS `script_targetname = targetname` pointer copies alias
    /// arbitrary older entries, and Raven's pool was likewise never freed, so entries persist
    /// until `GameWorld` is torn down.
    /// `Vec` growth relocates only the `Vec`'s spine (the `CString` structs).
    /// Each `CString`'s heap buffer stays put, so a slot pointer returned by an earlier push
    /// remains valid across later pushes.
    /// Source: replaces `oracle/codemp/game/g_spawn.c:724-749` (`G_NewString`).
    pub prefixStrings: Vec<CString>,

    /// The bg tier's session-lifetime state: the anim, saber, and vehicle tables, the
    /// `BG_Alloc` pool, and the RNG.
    /// Game code reaches the LCG as `world.bg_state.rng`.
    /// `Pmove` borrows this to build a `PmoveContext` on each call.
    /// Source: `crate::bg_channel::BgState`
    pub bg_state: crate::bg_channel::BgState,

    /// `g_misc.c` file-scope `refTagOwnerMap[MAX_TAG_OWNERS]`.
    /// File-scope mutable globals become `GameWorld` fields, grouped by owning `.c` file.
    /// Source: `oracle/codemp/game/g_misc.c:2886`
    pub refTagOwnerMap:
        Box<[crate::level::tag_owner::tagOwner_t; crate::level::tag_owner::MAX_TAG_OWNERS]>,

    /// `char gSharedBuffer[MAX_G_SHARED_BUFFER_SIZE]` is the module's engine-registered
    /// shared-memory region (`trap_SV_RegisterSharedMemory`).
    /// The `SharedBuffer` newtype owns the bytes and exposes one typed overlay accessor
    /// per `T_G_ICARUS_*` command.
    /// Source: `oracle/codemp/game/g_main.c:881`
    pub gSharedBuffer: Box<crate::world::shared_buffer::SharedBuffer>,

    /// Game-tier function-local persistent and rotating scratch (§B3): the `g_*`/`w_*`/`NPC_*`
    /// function-local `static` return buffers.
    /// Source: `crate::world::game_scratch::GameScratch`
    pub scratch: crate::world::game_scratch::GameScratch,
}

/// Zeroed `g_clients` array, built directly on the heap.
/// It never touches the stack: the by-value array is ~230 KB, and the engine calls `vmMain`
/// from a deep stack.
/// `gclient_t` stopped being `ZeroValid` when its `String` fields landed, so this mirrors
/// `native_platform::zeroed_box` and then installs a valid empty `String` into each client's
/// owned-`String` slots before the array is ever read.
/// Raven's `memset(g_clients, 0, ...)` sets every scalar to 0 and every name to an empty string.
fn zeroed_clients() -> Box<[gclient_t; MAX_CLIENTS]> {
    let layout = Layout::new::<[gclient_t; MAX_CLIENTS]>();
    // SAFETY: `alloc_zeroed` yields storage that is all-zero-valid for every `gclient_t` field
    // except the owned `String`s.
    // Each `ptr::write` overwrites one such slot with a valid empty `String`, its zeroed bytes
    // never dropped, before ownership passes to the `Box`, so the whole array is initialized.
    unsafe {
        let base = alloc_zeroed(layout) as *mut gclient_t;
        if base.is_null() {
            handle_alloc_error(layout);
        }
        for i in 0..MAX_CLIENTS {
            let c = base.add(i);
            core::ptr::write(core::ptr::addr_of_mut!((*c).pers.netname), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*c).sess.siegeClass), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*c).sess.saberType), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*c).sess.saber2Type), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*c).sess.IPstring), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*c).modelname), String::new());
        }
        Box::from_raw(base as *mut [gclient_t; MAX_CLIENTS])
    }
}

/// Zeroed `g_entities` storage with its guard slot, built directly on the heap.
/// It never touches the stack: the by-value array is ~1.83 MB.
/// `gentity_t` stopped being `ZeroValid` when its owned-`String` tail fields landed, so this
/// mirrors `native_platform::zeroed_box` and then seats a valid empty `String` into each
/// entity's owned-`String` slots ([`gentity_t::seat_owned_strings`]) before the array is
/// ever read.
/// This matches Raven's `memset(g_entities, 0, ...)`: every scalar 0, every pointer null,
/// every owned string empty.
/// The zeroed bytes leave each entity's `FnId<EntXxx>` handler fields as `None` by construction.
/// The guard slot gets the same string seating for drop safety, plus its
/// [`GuardedEntities::seat_guard`] contract fields.
fn zeroed_entities() -> Box<GuardedEntities> {
    let layout = Layout::new::<GuardedEntities>();
    // SAFETY: `alloc_zeroed` yields storage that is all-zero-valid for every `gentity_t` field
    // except the owned `String`s.
    // `seat_owned_strings` overwrites each such slot with a valid empty `String`, its zeroed
    // bytes never dropped, before ownership passes to the `Box`, so the whole value is
    // initialized.
    // The guard is entity slot 0 of the allocation, and the real array follows.
    unsafe {
        let base = alloc_zeroed(layout) as *mut gentity_t;
        if base.is_null() {
            handle_alloc_error(layout);
        }
        for i in 0..(MAX_GENTITIES + 1) {
            gentity_t::seat_owned_strings(base.add(i));
        }
        let mut boxed = Box::from_raw(base as *mut GuardedEntities);
        boxed.seat_guard();
        boxed
    }
}

impl GameWorld {
    /// Borrow entity `id` out of the owned `g_entities` arena (§B5).
    /// This is a plain checked index, not pointer math, because the world owns the arena.
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

    /// Borrow client `i` out of the owned `clients` arena.
    /// `i` is the Raven client number (`0..MAX_CLIENTS`), the same index Raven uses for
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

    /// Builds the zeroed island, then wires `level`'s self-referencing back-pointers in
    /// allocate-first order.
    /// The back-pointer wiring happens in `G_InitGame`'s dispatched arm, not here.
    /// Uses `native_platform::zeroed_box` for the ~1.83 MB entity array, built on the heap
    /// and never on the stack.
    ///
    /// Source: `docs/architecture/state-ownership.md` § `GameWorld::zeroed`.
    pub fn zeroed() -> Self {
        // Zeroed heap boxes come first.
        // The `level.gentities`/`clients` and `entities[i].client` back-pointers alias them
        // after they exist, in `G_InitGame`'s dispatched arm (`g_main.c:978-988`), not here.
        let g_entities = zeroed_entities();
        // The zeroed bytes leave each entity's `FnId<EntXxx>` handler fields as `None` by
        // construction.
        // Zero equals `None` here, guaranteed by std through `Option<NonZeroU8>`, so no fixup
        // is needed after zeroing.
        let clients = zeroed_clients();
        let level = level_locals_t::default();
        let memoryPool = native_platform::zeroed_box::<[u8; 262144]>();
        let refTagOwnerMap = native_platform::zeroed_box::<
            [crate::level::tag_owner::tagOwner_t; crate::level::tag_owner::MAX_TAG_OWNERS],
        >();
        let gSharedBuffer =
            native_platform::zeroed_box::<crate::world::shared_buffer::SharedBuffer>();
        // Keep this in sync with `zeroed_boxed()`.
        // Every field in this compiler-exhaustive literal needs a matching
        // `addr_of_mut!().write()` there, or `assume_init` is UB on the missed field.
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
            prefixStrings: Vec::new(),
            // Zeroed session state with the LCG seeded to Raven's `holdrand`.
            bg_state: crate::bg_channel::BgState::new(),
            refTagOwnerMap,
            gSharedBuffer,
            scratch: crate::world::game_scratch::GameScratch::zeroed(),
        }
    }

    /// Builds the zeroed island directly on the heap, field by field, into an uninitialized `Box`.
    /// The ~1.4 MB `GameWorld`, and its ~0.5 MB inline `globals` temporary, never transit the
    /// caller's stack by value.
    /// This is the engine-path constructor (`vmMain(GAME_INIT)`, `lib.rs`).
    /// The engine calls `vmMain` from a deep stack, and a by-value `zeroed()` overflowed the
    /// guard page there.
    /// Its return slot plus the `Some(..)` temporary each hold a full world image.
    /// This produces the same values as [`Self::zeroed`].
    /// Only where the bytes land differs.
    pub fn zeroed_boxed() -> Box<Self> {
        // `Box::new_uninit` allocates the storage on the heap without ever materializing a
        // `GameWorld` (or `MaybeUninit<GameWorld>`) on the stack.
        // Every field is then written in place exactly once, so the final `assume_init`
        // observes a fully-initialized value.
        // The real `Box` fields, for example `g_entities`, `nodetable`, and `bg_state`'s `Vec`s,
        // are written as live values and never left null.
        let mut boxed: Box<core::mem::MaybeUninit<GameWorld>> = Box::new_uninit();
        let p: *mut GameWorld = boxed.as_mut_ptr();
        // SAFETY: `p` points at freshly-allocated, correctly-aligned storage for one `GameWorld`.
        // Each `addr_of_mut!` write initializes a distinct field exactly once before
        // `assume_init`.
        unsafe {
            use core::ptr::addr_of_mut;
            addr_of_mut!((*p).level).write(level_locals_t::default());
            addr_of_mut!((*p).g_entities).write(zeroed_entities());
            // The zeroed bytes leave each entity's `FnId<EntXxx>` handler fields as `None` by
            // construction.
            // Zero equals `None` here, guaranteed by std through `Option<NonZeroU8>`, so no
            // fixup is needed after zeroing.
            addr_of_mut!((*p).clients).write(zeroed_clients());
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
            addr_of_mut!((*p).prefixStrings).write(Vec::new());
            addr_of_mut!((*p).bg_state).write(crate::bg_channel::BgState::new());
            addr_of_mut!((*p).refTagOwnerMap).write(native_platform::zeroed_box());
            addr_of_mut!((*p).gSharedBuffer).write(native_platform::zeroed_box());
            addr_of_mut!((*p).scratch).write(crate::world::game_scratch::GameScratch::zeroed());
            boxed.assume_init()
        }
    }
}
