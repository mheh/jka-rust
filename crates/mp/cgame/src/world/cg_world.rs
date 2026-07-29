//! `CgWorld` — the one owned cgame-module island (DEC-46.1).

#![allow(non_snake_case)]

use core::mem::MaybeUninit;
use core::ptr::{addr_of_mut, write_bytes};
use std::alloc::{alloc_zeroed, handle_alloc_error, Layout};

use mp_bg::bg_channel::{BgHost, BgState};
use mp_bg::bg_misc::MAX_POOL_SIZE_CGAME;
use mp_bg::public::bg_entity::bgEntity_t;
use mp_bg::public::max_items::MAX_ITEMS;
use mp_qshared::common::mp::qcommon::playerState_t;
use mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS;
use mp_qshared::shared::MAX_GENTITIES;

use crate::local::centity_s::centity_t;
use crate::local::cg_t::{cg_t, MAX_CG_SHARED_BUFFER_SIZE};
use crate::local::cgs_t::cgs_t;
use crate::local::item_info_t::itemInfo_t;
use crate::local::local_entity_s::localEntity_t;
use crate::local::mark_poly_s::markPoly_t;
use crate::local::weapon_info_s::weaponInfo_t;

use super::cg_cvars::CgCvars;
use super::cg_draw_state::CgDrawState;
use super::cg_effects_state::CgEffectsState;
use super::cg_ents_state::CgEntsState;
use super::cg_light_state::CgLightState;
use super::cg_main_state::CgMainState;
use super::cg_marks_state::CgMarksState;
use super::cg_players_state::CgPlayersState;
use super::cg_playerstate_state::CgPlayerstateState;
use super::cg_predict_state::CgPredictState;
use super::cg_saga_state::CgSagaState;
use super::cg_scoreboard_state::CgScoreboardState;
use super::cg_view_state::CgViewState;
use super::cg_weapons_state::CgWeaponsState;
use super::effect_pool::EffectPool;

/// Raven `#define MAX_LOCAL_ENTITIES 512`.
///
/// Source: `oracle/codemp/cgame/cg_localents.c:9`
pub const MAX_LOCAL_ENTITIES: usize = 512;

/// Raven `#define MAX_MARK_POLYS 256`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:57`
pub const MAX_MARK_POLYS: usize = 256;

/// The cgame module's one owned state island — Raven's three cgame spine
/// globals (`cg`, `cgs`, `cg_entities`) with the media/weapon/item registries,
/// the effect pools, the cvar mirrors and every per-`.c`-file static hanging
/// off them (DEC-46.1). It is a value owned by the `vmMain` shell, not a
/// global; the ABI entrypoints hold the single instance and hand it inward
/// inside a [`CgContext`](super::cg_context::CgContext) (§B3/§B4).
///
/// Transcription reads line-for-line: Raven's `cg.time` is `world.cg.time`,
/// `cgs.media.X` is `world.cgs.media.X`, `cg_entities[n]` is
/// `world.entities[n]`. The spine field types are the existing C1/C2 leaf ports
/// under [`crate::local`], unchanged.
///
/// `cgDC` and `ui_shared.c`'s menu-framework globals are NOT here: cgame links
/// `ui_shared.c` for its HUD and scoreboard (`cgame.q3asm:47`,
/// `JK2_cgame.vcproj:471`), so they are sibling fields of
/// [`CgState`](super::cg_state::CgState) and the ported fns can hold them
/// beside a live `CgContext` — the same borrow split DEC-38 ruling 1 made for
/// ui.
///
/// Source: `oracle/codemp/cgame/cg_main.c:691-699` (`cg`, `cgs`, `cg_entities`,
/// `cg_weapons`, `cg_items`), `docs/decisions.md` DEC-46 (rulings 1, 3, 6)
pub struct CgWorld {
    /// Raven `cg_t cg`.
    /// Source: `oracle/codemp/cgame/cg_main.c:691`
    pub cg: cg_t,

    /// Raven `cgs_t cgs`.
    /// Source: `oracle/codemp/cgame/cg_main.c:692`
    pub cgs: cgs_t,

    /// Raven `centity_t cg_entities[MAX_GENTITIES]` — fixed and
    /// entity-number-indexed, no arena (DEC-46.1). Boxed because the array is
    /// ~2 MB and must never transit the stack.
    /// Source: `oracle/codemp/cgame/cg_main.c:693`
    pub entities: Box<[centity_t; MAX_GENTITIES]>,

    /// Raven `playerState_t cgSendPSPool[MAX_GENTITIES]` - the per-entity
    /// snapshot playerStates bg logic reads through `centity_t.playerState`;
    /// `PlayerStateRef::Snap` resolves to row `n` here (DEC-47.2).
    /// Source: `oracle/codemp/cgame/cg_predict.c:853`
    pub cgSendPSPool: Box<[playerState_t; MAX_GENTITIES]>,

    /// The bg tier's `bgEntity_t` view of `cg_entities`. Raven's
    /// `cg_pmove.baseEnt = (bgEntity_t *)cg_entities` head-overlay pun cannot
    /// read the DEC-46.2 reshaped `centity_t`, so the port owns real rows:
    /// `CG_PmoveClientPointerUpdate` wires each row's `playerState` at the
    /// matching `cgSendPSPool` row, and `CG_PredictPlayerState` syncs the
    /// entity-state fields before running `Pmove` over them (DEC-47.2).
    /// Source: `oracle/codemp/cgame/cg_predict.c:912-914`
    pub bg_ents: Box<[bgEntity_t; MAX_GENTITIES]>,

    /// Raven `weaponInfo_t cg_weapons[MAX_WEAPONS]` — the per-weapon
    /// model/sound/effect registry.
    /// Source: `oracle/codemp/cgame/cg_main.c:698`
    pub cg_weapons: Box<[weaponInfo_t; MAX_WEAPONS]>,

    /// Raven `itemInfo_t cg_items[MAX_ITEMS]` — the per-item model/icon cache.
    /// Source: `oracle/codemp/cgame/cg_main.c:699`
    pub cg_items: Box<[itemInfo_t; MAX_ITEMS]>,

    /// Raven `localEntity_t cg_localEntities[MAX_LOCAL_ENTITIES]` plus its
    /// `cg_activeLocalEntities`/`cg_freeLocalEntities` chain heads, as the
    /// DEC-46.3 gen-counted slab.
    /// Source: `oracle/codemp/cgame/cg_localents.c:10-12`
    pub cg_localEntities: EffectPool<localEntity_t>,

    /// Raven `markPoly_t cg_markPolys[MAX_MARK_POLYS]` plus its
    /// `cg_activeMarkPolys`/`cg_freeMarkPolys` chain heads, as the DEC-46.3
    /// gen-counted slab.
    /// Source: `oracle/codemp/cgame/cg_marks.c:16-18`
    pub cg_markPolys: EffectPool<markPoly_t>,

    /// Raven `char cg.sharedBuffer[MAX_CG_SHARED_BUFFER_SIZE]`, the module's
    /// engine-registered shared-memory region — the census's one Class-A block
    /// (DEC-46.6). It is pinned: registered once with `CG_SET_SHARED_BUFFER` at
    /// init, and the engine writes into it behind our back. Consuming vmcalls
    /// therefore COPY OUT of it at entry and decode through the `mp_abi` `TCG*`
    /// types — no Rust reference into these bytes outlives a call into the
    /// engine.
    ///
    /// It lives on `CgWorld` rather than inside `cg` (where Raven put it) so
    /// the address stays put for the module's whole life; `cg_t.sharedBuffer`
    /// is the layout remnant and is not the live buffer.
    /// Source: `oracle/codemp/cgame/cg_local.h:997`,
    /// `oracle/codemp/cgame/cg_public.h:593`
    pub shared_buffer: Box<[u8; MAX_CG_SHARED_BUFFER_SIZE]>,

    /// Raven's 127 file-scope `vmCvar_t` handles.
    /// Source: `super::cg_cvars::CgCvars`
    pub cvars: CgCvars,

    /// `cg_draw.c`'s mutable file-scope globals.
    /// Source: `super::cg_draw_state::CgDrawState`
    pub draw: CgDrawState,

    /// `cg_effects.c`'s mutable file-scope globals.
    /// Source: `super::cg_effects_state::CgEffectsState`
    pub effects: CgEffectsState,

    /// `cg_ents.c`'s mutable file-scope globals.
    /// Source: `super::cg_ents_state::CgEntsState`
    pub ents: CgEntsState,

    /// `cg_light.c`'s mutable file-scope globals.
    /// Source: `super::cg_light_state::CgLightState`
    pub light: CgLightState,

    /// `cg_main.c`'s mutable file-scope globals, minus the spine and the cvars.
    /// Source: `super::cg_main_state::CgMainState`
    pub main: CgMainState,

    /// `cg_marks.c`'s mutable file-scope globals, minus the pool.
    /// Source: `super::cg_marks_state::CgMarksState`
    pub marks: CgMarksState,

    /// `cg_players.c`'s mutable file-scope globals.
    /// Source: `super::cg_players_state::CgPlayersState`
    pub players: CgPlayersState,

    /// `cg_playerstate.c`'s mutable file-scope globals.
    /// Source: `super::cg_playerstate_state::CgPlayerstateState`
    pub playerstate: CgPlayerstateState,

    /// `cg_predict.c`'s mutable file-scope globals.
    /// Source: `super::cg_predict_state::CgPredictState`
    pub predict: CgPredictState,

    /// `cg_saga.c`'s mutable file-scope globals.
    /// Source: `super::cg_saga_state::CgSagaState`
    pub saga: CgSagaState,

    /// `cg_scoreboard.c`'s mutable file-scope globals.
    /// Source: `super::cg_scoreboard_state::CgScoreboardState`
    pub scoreboard: CgScoreboardState,

    /// `cg_view.c`'s mutable file-scope globals.
    /// Source: `super::cg_view_state::CgViewState`
    pub view: CgViewState,

    /// `cg_weapons.c`'s mutable file-scope globals.
    /// Source: `super::cg_weapons_state::CgWeaponsState`
    pub weapons: CgWeaponsState,

    /// The cgame module's own bg-tier state — Raven compiles the bg files into
    /// the cgame link unit (`CGAME`), giving cgame its own copies of the bg
    /// globals (rand state, siege class tables, parse scratch, `BG_Alloc` pool
    /// at the 2 MB cgame arm). Same second-implementor story ui got in DEC-36
    /// addendum 11.
    /// Source: `oracle/codemp/game/bg_misc.c:3311-3316`
    pub bg_state: BgState,
}

impl CgWorld {
    /// Builds the zeroed island directly on the heap, field by field into an
    /// uninitialized `Box`, so the ~2.5 MB world (and its 295 KB `cg` / 224 KB
    /// `cgs` inline temporaries) never transits the caller's stack by value.
    ///
    /// There is deliberately no by-value `new()`: the engine calls `vmMain`
    /// from a deep stack, where `mp_game`'s equivalent by-value constructor
    /// overflowed the guard page (`mp_game::world::game_world`'s
    /// `zeroed_boxed` doc records the incident).
    ///
    /// The starting values are Raven's — `cg`, `cgs` and `cg_entities` are
    /// zeroed BSS that `CG_Init` `memset`s again on every map load, the
    /// registries start unregistered, and the effect pools start empty.
    ///
    /// Source: `oracle/codemp/cgame/cg_main.c:3164-3167` (`CG_Init`'s memsets)
    pub fn new_boxed() -> Box<Self> {
        // `Box::new_uninit` allocates the storage on the heap without ever
        // materializing a `CgWorld` on the stack; every field is written in
        // place exactly once, so the final `assume_init` observes a fully
        // initialized value.
        let mut boxed: Box<MaybeUninit<CgWorld>> = Box::new_uninit();
        let p: *mut CgWorld = boxed.as_mut_ptr();
        // SAFETY: `p` points at freshly allocated, correctly aligned storage
        // for one `CgWorld`; each write below initializes a distinct field
        // exactly once before `assume_init`.
        unsafe {
            // `cg_t`/`cgs_t` are `#[repr(C)]` PODs whose every enum member has
            // a 0 discriminant, so the in-place zero fill is Raven's
            // `memset( &cg, 0, sizeof( cg ) )` verbatim.
            write_bytes(addr_of_mut!((*p).cg), 0u8, 1);
            write_bytes(addr_of_mut!((*p).cgs), 0u8, 1);
            // SAFETY: `centity_t` is scalars, arrays, raw ghoul2 tokens (null
            // when zeroed) and the DEC-46.2 resolution fields, whose all-zero
            // patterns are `PlayerStateRef::None` / `None::<VehicleId>` (the
            // `NonZeroU32` niche) / `None::<Box<clientInfo_t>>` (the null
            // niche).
            addr_of_mut!((*p).entities).write(zeroed_box::<[centity_t; MAX_GENTITIES]>());
            // SAFETY: `playerState_t` is `#[repr(C)]` scalars all the way down
            // (`playerState_t::zeroed` documents it).
            addr_of_mut!((*p).cgSendPSPool).write(zeroed_box::<[playerState_t; MAX_GENTITIES]>());
            // SAFETY: `bgEntity_t` is an entityState POD, raw pointers (null
            // when zeroed) and scalars.
            addr_of_mut!((*p).bg_ents).write(zeroed_box::<[bgEntity_t; MAX_GENTITIES]>());
            // SAFETY: `weaponInfo_t` is scalars, arrays, an `Option<ItemId>`
            // (zero == `None`) and `Option<fn>` trail hooks (zero == `None`).
            addr_of_mut!((*p).cg_weapons).write(zeroed_box::<[weaponInfo_t; MAX_WEAPONS]>());
            // SAFETY: `itemInfo_t` is a `#[repr(C)]` POD — handles, floats and
            // ghoul2 `*mut c_void` tokens, all null/0 when zeroed.
            addr_of_mut!((*p).cg_items).write(zeroed_box::<[itemInfo_t; MAX_ITEMS]>());
            // SAFETY: `u8` is all-zero-valid.
            addr_of_mut!((*p).shared_buffer).write(zeroed_box::<[u8; MAX_CG_SHARED_BUFFER_SIZE]>());
            addr_of_mut!((*p).cg_localEntities)
                .write(EffectPool::new(MAX_LOCAL_ENTITIES, localEntity_t::zeroed));
            addr_of_mut!((*p).cg_markPolys)
                .write(EffectPool::new(MAX_MARK_POLYS, markPoly_t::zeroed));
            addr_of_mut!((*p).cvars).write(CgCvars::default());
            addr_of_mut!((*p).draw).write(CgDrawState::default());
            addr_of_mut!((*p).effects).write(CgEffectsState::default());
            addr_of_mut!((*p).ents).write(CgEntsState::default());
            addr_of_mut!((*p).light).write(CgLightState::default());
            addr_of_mut!((*p).main).write(CgMainState::default());
            addr_of_mut!((*p).marks).write(CgMarksState::default());
            addr_of_mut!((*p).players).write(CgPlayersState::default());
            addr_of_mut!((*p).predict).write(CgPredictState::default());
            addr_of_mut!((*p).saga).write(CgSagaState::default());
            addr_of_mut!((*p).playerstate).write(CgPlayerstateState::default());
            addr_of_mut!((*p).scoreboard).write(CgScoreboardState::default());
            addr_of_mut!((*p).view).write(CgViewState::default());
            addr_of_mut!((*p).weapons).write(CgWeaponsState::default());
            addr_of_mut!((*p).bg_state)
                .write(BgState::with_pool_size(MAX_POOL_SIZE_CGAME, BgHost::Cgame));
            boxed.assume_init()
        }
    }

    /// Borrow entity `n` out of the owned `cg_entities` array. Safe: the world
    /// owns the array, so this is a plain checked index, not pointer math
    /// (§B5).
    #[inline]
    pub fn entity(&self, n: usize) -> &centity_t {
        &self.entities[n]
    }

    /// Mutable [`Self::entity`].
    #[inline]
    pub fn entity_mut(&mut self, n: usize) -> &mut centity_t {
        &mut self.entities[n]
    }
}

/// Zero-filled `T` built straight on the heap, never on the stack — the
/// multi-megabyte spine arrays would blow the guard page otherwise.
///
/// SAFETY (caller): `T` must be all-zero-valid, i.e. every byte pattern of
/// zeros is a legal value of every field. Each call site above names why its
/// `T` qualifies.
unsafe fn zeroed_box<T>() -> Box<T> {
    let layout = Layout::new::<T>();
    let p = alloc_zeroed(layout) as *mut T;
    if p.is_null() {
        handle_alloc_error(layout);
    }
    Box::from_raw(p)
}
