//! Boot: the engine subset, the renderer CPU frontend, and `_UI_Init`'s
//! equivalent prefix.
//!
//! The engine sequence is the ordered FS/cvar/cmd head of `Com_Init`
//! (`mp_engine_qcommon::common_fns::Com_Init`) and nothing after it:
//! `Cvar_Init` → `Cbuf_Init` → `Com_InitZoneMemory` (FS pack loads
//! `Z_Malloc`) → `Cmd_Init` (`FS_Startup` registers commands) → the
//! `fs_basepath`/`fs_game` seed → `FS_InitFilesystem` → `Com_InitHunkMemory`.
//! `Com_Init`'s tail (`SE_Init`, `Netchan_Init`, `VM_Init`, the mandatory
//! `SV_Init` hook, the `CL_*` hooks) is skipped: the menus need a filesystem,
//! a cvar table and a command buffer, and nothing else the engine provides.

use core::ffi::c_int;
use core::ptr::null_mut;
use std::sync::Arc;

use mp_engine_botlib::l_precomp_fns::PC_SetBaseFolder;
use mp_engine_core::Engine;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cm_terrain::CmLandScape;
use mp_engine_qcommon::cmd_common::{Cbuf_Init, Cmd_Init};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::opaque_slots::{
    BotLib as SlotBotLib, Client as SlotClient, FxSystem as SlotFxSystem,
    Ghoul2System as SlotGhoul2,
    RenderModels as SlotRenderModels, Renderer as SlotRenderer, RmManager as SlotRmManager,
    Server as SlotServer, SoundSystem as SlotSoundSystem,
};
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Init};
use mp_engine_qcommon::files_common::FS_InitFilesystem;
use mp_engine_qcommon::stringed::api::SE_Init;
use mp_engine_qcommon::z_memman_pc::{Com_InitHunkMemory, Com_InitZoneMemory};
use mp_engine_server::botlib_import::{arm_botlib_slot, botlib_import_table};
use mp_engine_server::Server;
use mp_qshared::shared::com_parse::QSharedScratch;
use mp_qshared::shared::cvar::CVAR_INIT;
use mp_qshared::shared::qfalse;
use mp_qshared::shared::qhandle_t;
use mp_renderer::render_state::frame_data::FrameData;
use mp_renderer::render_state::light_style_table::LightStyleTable;
use mp_renderer::render_state::render_assets::RenderAssets;
use mp_renderer::render_state::render_assets_sim::RenderAssetsSim;
use mp_renderer::render_state::renderer_cvars::RendererCvars;
use mp_renderer::render_state::world_walk_scratch::WorldWalkScratch;
use mp_renderer::renderer_frontend::{
    empty_render_assets, empty_sky_state, zeroed_frame_state, zeroed_view_parms,
};
use mp_renderer::tr_bsp::RE_LoadWorldMap;
use mp_renderer::tr_font::FontState;
use mp_renderer::tr_image::TrImageState;
use mp_renderer::tr_init::R_Init;
use mp_renderer::tr_local::dlight_s::dlight_t;
use mp_renderer::tr_local::fog_t::fog_t;
use mp_renderer::tr_local::srf_terrain_s::srfTerrain_t;
use mp_renderer::tr_local::tr_ref_entity_t::trRefEntity_t;
use mp_renderer::tr_local::tr_refdef_t::trRefdef_t;
use mp_renderer::tr_main::{
    DrawSurf, R_RenderView, SurfaceGeometry, TrMainScratch, WorldSurfaceRef,
};
use mp_renderer::tr_model::frontend::RE_RegisterModel;
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_noise::NoiseState;
use mp_renderer::tr_scene::SceneState;
use mp_renderer::tr_sky::SkyState;
use mp_renderer::tr_terrain::R_TerrainInit;
use mp_renderer::tr_worldeffects::world_effects::WorldEffectsState;
use mp_ui::world::ui_state::UiState;
use mp_uishared::shared::display_context::DisplayContext;
use mp_uishared::ui_shared::{Menu_Count, Menus_ActivateByName, Menus_CloseAll, String_Init};
use native_math::rng::Rng;

use crate::pipeline2d::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::ui_host::display::HarnessDc;
use crate::ui_host::state::{InputState, StubLog, UiHost};

/// Where the retail assets live and which menu set to open.
pub struct BootConfig {
    /// `fs_basepath` — the directory holding `base/assets*.pk3`.
    pub basepath: String,
    /// `fs_homepath` - empty keeps the platform default. A synthetic boot sets
    /// it, or `FS_Startup` also mounts the user's real install directory.
    pub homepath: String,
    /// `fs_game` — "" for stock `base`.
    pub fs_game: String,
    /// The menu-set file (`ui_menuFilesMP`'s default).
    pub menu_file: String,
    /// The menu opened after load (`UIMENU_MAIN`'s target).
    pub start_menu: String,
}

impl Default for BootConfig {
    fn default() -> Self {
        BootConfig {
            basepath: String::from("/Users/milohehmsoth/Developer/jka/jka_server"),
            homepath: String::new(),
            fs_game: String::new(),
            menu_file: String::from("ui/jampmenus.txt"),
            start_menu: String::from("main"),
        }
    }
}

/// Boots the engine subset, the renderer, and the menus; returns a host ready
/// to be painted every frame.
pub fn boot(cfg: &BootConfig) -> UiHost {
    let mut host = boot_renderer(cfg);
    ui_init_equivalent(&mut host, cfg);
    host
}

/// Boots the engine subset and `R_Init`, and stops there. No menu set, no
/// fonts, no cursor shaders.
///
/// This is what a renderer test needs: `R_Init` builds its own images and its
/// default shader procedurally, so a host booted this way is usable against an
/// empty `fs_basepath` and draws no retail content.
pub fn boot_renderer(cfg: &BootConfig) -> UiHost {
    let mut engine = Engine::new();

    // The hook tables `Com_Init` would have installed — Raven's link-time
    // symbol resolution (DEC-23). Installing the server table is NOT booting a
    // server: `Hunk_Clear` calls the `SV_ShutdownGameProgs` hook
    // unconditionally (`z_memman_pc.rs:816`), and with no `SV_Init` ever run
    // that hook finds a null game VM and returns. `SV_Init` itself — the step
    // that would actually start a server — is never called.
    mp_engine_server::hook_install::install_engine_hooks(&mut engine.common.hooks);
    mp_renderer::hook_install::install_engine_hooks(&mut engine.common.hooks);

    // The renderer's model pool, built before the engine subset because
    // `Com_InitHunkMemory` -> `Hunk_Clear` calls the `R_HunkClearCrap` hook,
    // which casts the view's `rm` slot before doing anything else.
    let mut models = RenderModels::default();

    // ---- engine subset -------------------------------------------------
    {
        let models_ptr: *mut RenderModels = &mut models;
        let Engine { common, cm, sv, .. } = &mut *engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = host_view(common, cm, sv_ptr, models_ptr);
        Cvar_Init(&mut view);
        Cbuf_Init(view.common);
        Com_InitZoneMemory(&mut view);
        Cmd_Init(&mut view);

        // Seed before `FS_Startup` re-registers them with platform defaults:
        // `Cvar_Get` keeps an already-registered value (`cvar_fns.rs`).
        Cvar_Get(&mut view, "fs_basepath", &cfg.basepath, CVAR_INIT);
        if !cfg.homepath.is_empty() {
            Cvar_Get(&mut view, "fs_homepath", &cfg.homepath, CVAR_INIT);
        }
        Cvar_Get(&mut view, "fs_game", &cfg.fs_game, CVAR_INIT);
        let ded = Cvar_Get(&mut view, "dedicated", "0", 0);
        view.common.com_dedicated = Some(ded);
        // `Com_Init` registers `journal` and stores the handle
        // (`common_fns.rs:667`); `Com_GetRealEvent` (under `Com_Milliseconds`,
        // reached by `R_InitWorldEffects`) reads it unconditionally.
        let journal = Cvar_Get(&mut view, "journal", "0", CVAR_INIT);
        view.common.com_journal = Some(journal);

        FS_InitFilesystem(&mut view);
        Com_InitHunkMemory(&mut view);
        // `Com_Init` runs `SE_Init` after FS is up (language load; menu `@KEY`
        // references resolve through the packages it manages).
        SE_Init(&mut view);
        println!(
            "ui_harness: FS up — {} files in pk3 files under {}",
            view.common.fs_packFiles, cfg.basepath
        );
    }

    // The precompiler's `#include` search root, as `SV_BotInitBotLib` sets it.
    PC_SetBaseFolder(&mut engine.bot, "base");

    // ---- renderer carrier bundle ---------------------------------------
    let mut host = UiHost {
        engine,
        models,
        cvars: RendererCvars::default(),
        sim: RenderAssetsSim {
            published: Arc::new(empty_assets()),
            light_styles: LightStyleTable {
                colors: [[0u8; 4]; mp_engine_qcommon::qfiles::light_style_limits::MAX_LIGHT_STYLES],
            },
        },
        img_state: TrImageState::default(),
        frame: zeroed_frame_state(),
        scene: SceneState::default(),
        noise: NoiseState::default(),
        rng: Rng::new(),
        font: FontState::default(),
        world_effects: WorldEffectsState::default(),
        qs: QSharedScratch::zeroed(),
        sky_view: zeroed_view_parms(),
        sky: empty_sky(),
        ui: UiState::default(),
        input: InputState::default(),
        stubs: StubLog::default(),
        start: std::time::Instant::now(),
    };

    // ---- R_Init (the real one) -----------------------------------------
    {
        let UiHost {
            engine,
            models,
            cvars,
            sim,
            img_state,
            frame,
            scene,
            noise,
            rng,
            font,
            world_effects,
            qs,
            sky_view,
            sky,
            ..
        } = &mut host;
        let mut frame_data = FrameData { events: Vec::new() };
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = host_view(common, cm, sv_ptr, models_ptr);
        R_Init(
            &mut view,
            cvars,
            sim,
            img_state,
            models,
            frame,
            scene,
            &mut frame_data,
            noise,
            rng,
            font,
            world_effects,
            qs,
            sky_view,
            sky,
        );
    }
    println!(
        "ui_harness: R_Init done — {} shaders, {} images registered",
        host.sim.published.shaders.iter().count(),
        host.sim.published.images.iter().count()
    );

    host
}

/// `_UI_Init`'s prefix, restricted to what the framework half needs: the
/// virtual-screen scale, the menu fonts, the cursor/white shaders, the menu
/// set, and the opening menu.
///
/// Raven order (`ui_main.c:_UI_Init`): `GetGlconfig` → `String_Init` →
/// cursor/white shader registration → `AssetCache` → menu load →
/// `Menus_CloseAll`. The module-owned steps in between (siege class load,
/// player-model list, force config, bot list, cached servers) are `mp_ui`'s
/// and need `UiContext`; they are not reachable here and are not needed to
/// paint a menu.
fn ui_init_equivalent(host: &mut UiHost, cfg: &BootConfig) {
    // The PC_* precompiler reads menu files through the botlib import table
    // (`botimport.FS_FOpenFile`, `l_script_fns.rs:1155`) — `SV_BotInitBotLib`
    // wires it in the live engine; the harness wires the identical table and
    // arms the ambient slot its trampolines read. Armed HERE, not in `boot()`:
    // the captured pointers must come from `UiHost`'s settled location
    // (`engine` is `Box`-pinned, `models` is not).
    host.engine.bot.botimport = botlib_import_table();
    {
        let models_ptr: *mut RenderModels = &mut host.models;
        let Engine { common, cm, sv, .. } = &mut *host.engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = host_view(common, cm, sv_ptr, models_ptr);
        arm_botlib_slot(&mut view, sv);
    }
    // `trap_GetGlconfig` — the harness IS the renderer, so the virtual screen
    // is the one `pipeline2d` rasterises into.
    host.ui.uiDC.glconfig.vidWidth = SCREEN_WIDTH as c_int;
    host.ui.uiDC.glconfig.vidHeight = SCREEN_HEIGHT as c_int;
    host.ui.uiDC.glconfig.isFullscreen = qfalse;
    // `_UI_Init`'s scale computation (`ui_main.c`): 640x480 reference.
    host.ui.uiDC.xscale = 1.0;
    host.ui.uiDC.yscale = 1.0;
    host.ui.uiDC.bias = 0.0;
    host.ui.uiDC.cursorx = SCREEN_WIDTH as c_int / 2;
    host.ui.uiDC.cursory = SCREEN_HEIGHT as c_int / 2;

    with_dc(host, |dc, ui| {
        // The four menu fonts `MenuFontToHandle` selects between, plus the
        // cursor/white/gradient shaders. Retail `ui/jamp/main.menu`'s
        // `assetGlobalDef` block names exactly these; the harness's menu
        // loader skips that block (`Asset_Parse` is `mp_ui`-owned and
        // `UiContext`-bound), so the same names are registered here.
        String_Init(&mut ui.menus, dc);
        ui.uiDC.Assets.qhMediumFont = dc.RegisterFont("ergoec");
        ui.uiDC.Assets.qhSmallFont = dc.RegisterFont("aurabesh");
        ui.uiDC.Assets.qhBigFont = dc.RegisterFont("anewhope");
        ui.uiDC.Assets.qhSmall2Font = dc.RegisterFont("arialnb");
        ui.uiDC.cursor = dc.registerShaderNoMip("cursor");
        ui.uiDC.whiteShader = dc.registerShaderNoMip("white");
        ui.uiDC.gradientImage = dc.registerShaderNoMip("ui/assets/gradientbar2.tga");
        ui.uiDC.Assets.gradientBar = ui.uiDC.gradientImage;
        ui.uiDC.Assets.cursor = ui.uiDC.cursor;
        // `assetGlobalDef`'s fade block, same source, same values.
        ui.uiDC.Assets.fadeClamp = 1.0;
        ui.uiDC.Assets.fadeCycle = 1;
        ui.uiDC.Assets.fadeAmount = 0.1;
        ui.uiDC.Assets.shadowColor = [0.1, 0.1, 0.1, 0.25];
        ui.uiDC.Assets.fontRegistered = true;
    });
    println!(
        "ui_harness: fonts registered — small={} medium={} big={}, cursor shader={}",
        host.ui.uiDC.Assets.qhSmallFont,
        host.ui.uiDC.Assets.qhMediumFont,
        host.ui.uiDC.Assets.qhBigFont,
        host.ui.uiDC.cursor
    );

    let menu_file = cfg.menu_file.clone();
    let start_menu = cfg.start_menu.clone();
    with_dc(host, |dc, ui| {
        dc.load_menus(&mut ui.menus, &mut ui.uiDC, &menu_file);
        println!("ui_harness: {} menus parsed", Menu_Count(&ui.menus));
        let ds = &ui.uiDC;
        // `UI_SetActiveMenu(UIMENU_MAIN)` closes everything BEFORE activating
        // (`oracle/codemp/ui/ui_main.c:1890-1891`) — without this, every menu
        // whose file declares `visible 1` paints stacked over main.
        Menus_CloseAll(&mut ui.menus, ds, dc);
        let opened = Menus_ActivateByName(&mut ui.menus, ds, dc, &start_menu);
        println!(
            "ui_harness: Menus_ActivateByName(\"{start_menu}\") -> {}",
            if opened.is_some() {
                "open"
            } else {
                "NOT FOUND"
            }
        );
    });
}

/// Runs `body` with a live [`HarnessDc`] borrowed out of `host`, plus the ui
/// state it paints. The two are split-borrowed disjointly: the `dc` owns the
/// engine/renderer carriers, `ui` owns `menus`/`uiDC`.
pub fn with_dc<R>(host: &mut UiHost, body: impl FnOnce(&mut HarnessDc, &mut UiState) -> R) -> R {
    let UiHost {
        engine,
        models,
        cvars,
        sim,
        img_state,
        frame,
        font,
        qs,
        sky_view,
        sky,
        ui,
        input,
        stubs,
        start,
        ..
    } = host;
    let millis = start.elapsed().as_millis() as c_int;
    // Split-borrowed disjointly: the view takes `common`/`cm`, the tokenizer
    // takes `bot`. The view's own `bot` slot stays null (see `host_view`).
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine {
        common,
        cm,
        bot,
        sv,
        ..
    } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let view = host_view(common, cm, sv_ptr, models_ptr);
    let mut dc = HarnessDc {
        view,
        bot,
        cvars,
        assets: Arc::make_mut(&mut sim.published),
        models,
        img_state,
        frame,
        qs,
        sky_view,
        sky,
        font,
        frame_data: FrameData { events: Vec::new() },
        input,
        stubs,
        millis,
    };
    body(&mut dc, ui)
}

/// Builds an [`EngineHostView`] over the harness's islands.
///
/// Takes `common`/`cm` as already-split field borrows rather than the whole
/// `Engine`, so a caller can hold `engine.bot` (the precompiler) at the same
/// time without aliasing.
///
/// `sv` points at the engine's own (never-initialised) `Server`: `Hunk_Clear`
/// calls the `SV_ShutdownGameProgs` hook unconditionally, and that hook casts
/// this slot before testing the game VM, so a null slot is a hard crash rather
/// than a no-op.
///
/// `rm` points at the harness's own [`RenderModels`] (the renderer's model
/// pool), not `engine.render_models` — the engine's copy belongs to the
/// headless server subset and is not what `R_Init` initialised. Every other
/// opaque slot is null: no path this harness runs reads them, and a null slot
/// makes that a loud crash rather than a silent wrong-island read.
pub fn host_view<'a>(
    common: &'a mut Common,
    cm: &'a mut CollisionWorld,
    sv: *mut (),
    rm: *mut RenderModels,
) -> EngineHostView<'a> {
    EngineHostView {
        common,
        cm,
        sv: SlotServer::from_raw(sv),
        cl: SlotClient::from_raw(null_mut()),
        snd: SlotSoundSystem::from_raw(null_mut()),
        bot: SlotBotLib::from_raw(null_mut()),
        rm: SlotRenderModels::from_raw(rm as *mut ()),
        // The harness holds its carriers as `UiHost` fields and splits them at
        // each call, so it never reaches the renderer through this slot.
        re: SlotRenderer::from_raw(null_mut()),
        rmg: SlotRmManager::from_raw(null_mut()),
        g2: SlotGhoul2::from_raw(null_mut()),
        fx: SlotFxSystem::from_raw(null_mut()),
    }
}

/// The renderer's own seed for an empty `RenderAssets` (DEC-32: one home).
pub fn empty_assets() -> RenderAssets {
    empty_render_assets()
}

/// What the world-render feasibility spike observed for one map load and one
/// `R_RenderView`.
pub struct WorldSpikeReport {
    /// The map loaded and rendered without a panic.
    pub loaded: bool,
    /// The eye point the refdef was built at (a spawn origin, z-bumped to eye
    /// height).
    pub eye: [f32; 3],
    /// Every draw surface the sorted list holds after `R_RenderView`.
    pub total_draw_surfs: usize,
    /// `SurfaceGeometry::World` entries, then their world-surface kind split.
    pub world: usize,
    pub world_face: usize,
    pub world_grid: usize,
    pub world_triangles: usize,
    pub world_flare: usize,
    pub world_skip: usize,
    /// Non-world arms.
    pub face: usize,
    pub triangles: usize,
    pub poly: usize,
    pub other: usize,
    /// World leaves the walk left marked in this view (an approximation of the
    /// oracle's `c_leafs`, which `R_AddWorldSurfaces` owns as private scratch).
    pub visible_leaves: usize,
}

/// Loads a BSP through `RE_LoadWorldMap` and initializes the null-landscape
/// terrain surface. Returns whether the world loaded, plus the terrain surface
/// the per-frame world pass reuses.
///
/// The window harness calls this once, then feeds the returned terrain surface
/// into every `WorldFrame` it builds. `R_TerrainInit` also registers the
/// terrain cvars and sets `RenderAssets::distance_cull`, which the world pass
/// reads.
pub fn load_world(host: &mut UiHost, map: &str) -> (bool, srfTerrain_t) {
    // ---- load the BSP --------------------------------------------------
    {
        let UiHost {
            engine,
            models,
            cvars,
            sim,
            img_state,
            frame,
            qs,
            sky_view,
            sky,
            world_effects,
            ..
        } = &mut *host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = host_view(common, cm, sv_ptr, models_ptr);
        RE_LoadWorldMap(
            qs,
            frame,
            Arc::make_mut(&mut sim.published),
            &mut view,
            cvars,
            models,
            img_state,
            sky_view,
            sky,
            world_effects,
            map,
        );
    }

    let loaded = host.sim.published.world.is_some();
    println!(
        "world_harness: RE_LoadWorldMap(\"{map}\") -> {}",
        if loaded { "loaded" } else { "NOT LOADED" }
    );

    (loaded, init_terrain(host))
}

/// Runs `R_TerrainInit` and returns the null-landscape terrain surface every
/// `WorldFrame` carries.
///
/// Every scene needs this, map or no map: `R_TerrainInit` registers the terrain
/// cvars `R_AddTerrainSurfaces` reads on the world walk, and sets
/// `RenderAssets::distance_cull`.
pub fn init_terrain(host: &mut UiHost) -> srfTerrain_t {
    // SAFETY: `srfTerrain_t` is a frozen `#[repr(C)]` POD (scalars, fixed
    // arrays, and raw pointers whose all-zero value is null). `R_TerrainInit`
    // overwrites both its fields with the null-landscape terrain surface, which
    // makes `R_AddTerrainSurfaces` return early.
    let mut land_scape: srfTerrain_t = unsafe { core::mem::zeroed() };
    let UiHost {
        engine,
        models,
        cvars,
        sim,
        ..
    } = &mut *host;
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine { common, cm, sv, .. } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let mut view = host_view(common, cm, sv_ptr, models_ptr);
    R_TerrainInit(&mut view, cvars, Arc::make_mut(&mut sim.published), &mut land_scape);
    land_scape
}

/// Registers one model through the real `RE_RegisterModel` chain, split-borrowing
/// the host bundle the loader needs, and returns its handle (0 when the file is
/// absent). The world harness registers a map object this way.
pub fn register_model(host: &mut UiHost, name: &str) -> qhandle_t {
    let UiHost {
        engine,
        models,
        cvars,
        sim,
        img_state,
        frame,
        qs,
        sky_view,
        sky,
        world_effects,
        ..
    } = &mut *host;
    let models_ptr: *mut RenderModels = &mut *models;
    let Engine { common, cm, sv, .. } = &mut **engine;
    let sv_ptr: *mut () = sv as *mut Server as *mut ();
    let mut view = host_view(common, cm, sv_ptr, models_ptr);
    RE_RegisterModel(
        qs,
        frame,
        Arc::make_mut(&mut sim.published),
        &mut view,
        cvars,
        models,
        img_state,
        sky_view,
        sky,
        world_effects,
        name,
    )
}

/// Loads a BSP through `RE_LoadWorldMap`, builds a refdef at a spawn point, and
/// drives one `R_RenderView` — the R4 world feasibility spike.
///
/// `R_RenderView` runs `R_RotateForViewer`/`R_SetupFrustum` against the ABI
/// `viewParms_t`, then publishes the PVS origin, frustum, and vis bounds into
/// `frame.view` (the `ViewParms` placeholder the world walk reads). The two
/// view types are not yet unified (#51), so the harness only forces
/// `areamask_modified` to make `R_MarkLeaves` re-mark this first frame.
pub fn load_world_and_render(host: &mut UiHost, map: &str) -> WorldSpikeReport {
    // ---- load the BSP --------------------------------------------------
    {
        let UiHost {
            engine,
            models,
            cvars,
            sim,
            img_state,
            frame,
            qs,
            sky_view,
            sky,
            world_effects,
            ..
        } = &mut *host;
        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut view = host_view(common, cm, sv_ptr, models_ptr);
        RE_LoadWorldMap(
            qs,
            frame,
            Arc::make_mut(&mut sim.published),
            &mut view,
            cvars,
            models,
            img_state,
            sky_view,
            sky,
            world_effects,
            map,
        );
    }

    let loaded = host.sim.published.world.is_some();
    println!(
        "world_spike: RE_LoadWorldMap(\"{map}\") -> {}",
        if loaded { "loaded" } else { "NOT LOADED" }
    );

    // A spawn origin from the stored entity lump, bumped to eye height.
    let eye = host
        .sim
        .published
        .world
        .as_ref()
        .and_then(|w| find_spawn_origin(&w.entity_string))
        .map(|o| [o[0], o[1], o[2] + 40.0])
        .unwrap_or([0.0, 0.0, 0.0]);
    println!("world_spike: eye at {:?}", eye);

    let mut r = WorldSpikeReport {
        loaded,
        eye,
        total_draw_surfs: 0,
        world: 0,
        world_face: 0,
        world_grid: 0,
        world_triangles: 0,
        world_flare: 0,
        world_skip: 0,
        face: 0,
        triangles: 0,
        poly: 0,
        other: 0,
        visible_leaves: 0,
    };

    // ---- one R_RenderView ----------------------------------------------
    // `draw_surfs` and `frame_data` share a lifetime (the `SurfaceGeometry`
    // polygon arm can borrow the event stream), so the tally runs inside this
    // block. The `WorldSpikeReport` it fills owns no borrow.
    {
        let mut draw_surfs: Vec<DrawSurf<SurfaceGeometry>> = Vec::new();
        let frame_data = FrameData { events: Vec::new() };
        let mut entities: Vec<trRefEntity_t> = Vec::new();
        let mut dlights: Vec<dlight_t> = Vec::new();
        // The loaded world's fog volumes, copied into the ABI `fog_t` the
        // frontend fog-num math reads. The spike adds no entities, so the list
        // only feeds `R_RenderView`'s fog tagging.
        let fogs: Vec<fog_t> = host
            .sim
            .published
            .world
            .as_ref()
            .map(|w| w.fogs.iter().map(|f| f.to_fog_t()).collect())
            .unwrap_or_default();
        let mut scratch = TrMainScratch {
            pre_trans_ent_matrix: [0.0; 16],
        };
        // SAFETY: `srfTerrain_t`/`trRefdef_t` are frozen `#[repr(C)]` POD (scalars,
        // fixed arrays and raw pointers whose all-zero value is null). `R_TerrainInit`
        // below overwrites `land_scape` (both its fields) with a null-landscape
        // terrain surface, which makes `R_AddTerrainSurfaces` return early, so
        // neither the terrain surface nor `land` is read past the cvar check.
        let mut land_scape: srfTerrain_t = unsafe { core::mem::zeroed() };
        let refdef: trRefdef_t = unsafe { core::mem::zeroed() };
        let land = CmLandScape::empty();

        let mut parms = zeroed_view_parms();
        parms.viewportWidth = SCREEN_WIDTH as c_int;
        parms.viewportHeight = SCREEN_HEIGHT as c_int;
        parms.fovX = 90.0;
        parms.fovY = 90.0 * SCREEN_HEIGHT / SCREEN_WIDTH;
        parms.ori.origin = eye;
        parms.ori.axis = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        parms.pvsOrigin = eye;

        // `R_RenderView` publishes `frame.view` (PVS origin, frustum, vis
        // bounds) from its own `R_RotateForViewer`/`R_SetupFrustum`. The fov
        // args come straight off `parms` - `R_SetupFrustum` does not change
        // them.
        let fov_x = parms.fovX;
        let fov_y = parms.fovY;

        let UiHost {
            engine,
            models,
            cvars,
            sim,
            frame,
            ..
        } = &mut *host;
        let assets = Arc::make_mut(&mut sim.published);
        // Force `R_MarkLeaves` to re-mark this frame regardless of the leftover
        // view cluster.
        frame.refdef.areamask_modified = true;

        // Bump the per-scene counters, the render-side stand-in for the
        // oracle's `tr.frameSceneNum++`/`tr.sceneCount++` in `RE_RenderScene`
        // (ruling 3 keeps that trap-time fn off `FrameState`). `R_RenderView`
        // stamps `view.frameSceneNum` from this value.
        // Source: oracle/codemp/renderer/tr_scene.cpp:829-830
        frame.frame_scene_num += 1;
        frame.scene_count += 1;
        let frame_scene_num = frame.frame_scene_num;

        let models_ptr: *mut RenderModels = &mut *models;
        let Engine { common, cm, sv, .. } = &mut **engine;
        let sv_ptr: *mut () = sv as *mut Server as *mut ();
        let mut engine_view = host_view(common, cm, sv_ptr, models_ptr);

        // Register the terrain cvars (`r_drawTerrain`, siblings) and init the
        // null-landscape terrain surface. `R_Init`'s ui subset skips this, so
        // the harness runs it before the first `R_AddTerrainSurfaces` read.
        R_TerrainInit(&mut engine_view, cvars, assets, &mut land_scape);
        let distance_cull = assets.distance_cull;

        // The ui background render has no live Ghoul2 state, so it threads an
        // empty owned system (design point 2).
        let mut g2_system = Ghoul2System::default();
        // The spike walks the world once, so its marks live and die with this
        // call (W2-F4).
        let mut walk_scratch = WorldWalkScratch::default();
        if let Some(world) = assets.world.as_ref() {
            walk_scratch.set_world(world);
        }
        let mut view = zeroed_view_parms();
        R_RenderView(
            &parms,
            frame_scene_num,
            0,
            &mut view,
            &mut engine_view,
            assets,
            cvars,
            frame,
            &mut walk_scratch,
            &mut g2_system,
            &frame_data,
            &refdef,
            0,
            fov_x,
            fov_y,
            0,
            &mut dlights,
            &fogs,
            distance_cull,
            &land_scape,
            &land,
            0,
            &mut entities,
            &mut scratch,
            models,
            &mut draw_surfs,
        );

        // ---- tally (inside the block, while `frame_data` lives) --------
        r.total_draw_surfs = draw_surfs.len();
        for ds in &draw_surfs {
            match ds.surface {
                SurfaceGeometry::World(w) => {
                    r.world += 1;
                    match w {
                        WorldSurfaceRef::Face(_) => r.world_face += 1,
                        WorldSurfaceRef::Grid(_) => r.world_grid += 1,
                        WorldSurfaceRef::Triangles(_) => r.world_triangles += 1,
                        WorldSurfaceRef::Flare(_) => r.world_flare += 1,
                        WorldSurfaceRef::Skip(_) => r.world_skip += 1,
                    }
                }
                SurfaceGeometry::Face(_) => r.face += 1,
                SurfaceGeometry::Triangles { .. } => r.triangles += 1,
                SurfaceGeometry::Poly { .. } => r.poly += 1,
                // The world spike loads no MD3 entity, so this arm never fires;
                // it folds into `other` for exhaustiveness.
                SurfaceGeometry::Md3(_) => r.other += 1,
                SurfaceGeometry::Ghoul2(_) => r.other += 1,
                SurfaceGeometry::Other => r.other += 1,
            }
        }

        // The visible-leaf tally reads the walk marks, which W2-F4 moved out of
        // the world and onto `walk_scratch`, so it runs inside this block.
        if let Some(w) = assets.world.as_ref() {
            let vis_count = walk_scratch.vis_count;
            r.visible_leaves = w
                .nodes
                .iter()
                .enumerate()
                .filter(|(i, n)| n.contents != -1 && walk_scratch.node_visframe[*i] == vis_count)
                .count();
        }
    }

    r
}

/// Scans the BSP entity lump for a spawn origin. Prefers
/// `info_player_deathmatch`, falls back to `info_player_start`. A plain token
/// scan is enough for the harness.
pub fn find_spawn_origin(entities: &str) -> Option<[f32; 3]> {
    for want in ["info_player_deathmatch", "info_player_start"] {
        for block in entities.split('{') {
            if !block.contains(want) {
                continue;
            }
            let Some(oi) = block.find("\"origin\"") else {
                continue;
            };
            let rest = &block[oi + "\"origin\"".len()..];
            let start = rest.find('"')? + 1;
            let end = rest[start..].find('"')? + start;
            let vals: Vec<f32> = rest[start..end]
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if vals.len() == 3 {
                return Some([vals[0], vals[1], vals[2]]);
            }
        }
    }
    None
}

/// A `SkyState` at rest (`tr_sky`'s file-scope statics, zeroed).
fn empty_sky() -> SkyState {
    empty_sky_state()
}
