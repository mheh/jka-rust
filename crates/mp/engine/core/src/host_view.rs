//! The ruling-43 split constructor: build the live `EngineHostView` from
//! `&mut Engine` field borrows + type-erased slots, and the one-time boot
//! installation of the `SV_*`/renderer hook fields (host-seam restructure,
//! user ruling 2026-07-11).

use mp_engine_client::{cgame_system_calls_shim, ui_system_calls_shim, ClientDispatchCtx};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::opaque_slots;
use mp_engine_qcommon::vm::{arm_cgame_slot, arm_game_slot, arm_ui_slot};
use mp_engine_server::{game_system_calls_shim, GameDispatchCtx};

use crate::engine::Engine;

/// Split-borrow the engine island into the live world view (ruling 43): plain
/// field reborrows for the qcommon-owned state, `as_raw` slot wraps for the
/// islands qcommon cannot name. `cl` and `re` are NULL slots on dedicated
/// (`None`) — the null-build client hooks never cast them.
pub fn engine_host_view(engine: &mut Engine) -> EngineHostView<'_> {
    let cl_raw = match engine.cl.as_mut() {
        Some(cl) => cl as *mut _ as *mut (),
        None => core::ptr::null_mut(),
    };
    let re_raw = match engine.re.as_mut() {
        Some(re) => re as *mut _ as *mut (),
        None => core::ptr::null_mut(),
    };
    EngineHostView {
        sv: opaque_slots::Server::from_raw(&mut engine.sv as *mut _ as *mut ()),
        cl: opaque_slots::Client::from_raw(cl_raw),
        bot: opaque_slots::BotLib::from_raw(&mut engine.bot as *mut _ as *mut ()),
        rm: opaque_slots::RenderModels::from_raw(&mut engine.render_models as *mut _ as *mut ()),
        re: opaque_slots::Renderer::from_raw(re_raw),
        rmg: opaque_slots::RmManager::from_raw(&mut engine.rmg as *mut _ as *mut ()),
        g2: opaque_slots::Ghoul2System::from_raw(&mut engine.g2 as *mut _ as *mut ()),
        common: &mut engine.common,
        cm: &mut engine.cm,
    }
}

/// One-time boot installation of the mandatory hook fields (Raven's link-time
/// `SV_*`/`RE_*`/`R_*` resolution): the client/sound tier already carries the
/// null-build defaults from `EngineHooks::null_dedicated()`; the server and
/// renderer tiers install here. Runs in `main()` right after `Engine::new()`,
/// before `com_init`.
///
/// Also arms the three module syscall slots with their dispatch notes (user
/// ruling 2026-07-12; DEC-55 for the client pair) — the engine-side `currentVM`
/// dual: every pointer is a field address of the one boxed `Engine` (stable for
/// the process lifetime), written once here and read by the matching shim on
/// every module trap. Each note is leaked to process lifetime, exactly like
/// Raven's file-scope statics it replaces.
///
/// The client pair arms unconditionally. On dedicated the `cl`/`re` pointers
/// are NULL, and no cgame or ui module ever loads, so the armed pair is never
/// entered.
pub fn install_engine_hooks(engine: &mut Engine) {
    mp_engine_server::hook_install::install_engine_hooks(&mut engine.common.hooks);
    mp_renderer::hook_install::install_engine_hooks(&mut engine.common.hooks);

    let cl_raw = match engine.cl.as_mut() {
        Some(cl) => cl as *mut _ as *mut (),
        None => core::ptr::null_mut(),
    };
    let re_raw = match engine.re.as_mut() {
        Some(re) => re as *mut _ as *mut (),
        None => core::ptr::null_mut(),
    };
    let note: &'static mut GameDispatchCtx = Box::leak(Box::new(GameDispatchCtx {
        common: &mut engine.common as *mut _,
        cm: &mut engine.cm as *mut _,
        sv: &mut engine.sv as *mut _ as *mut (),
        cl: cl_raw,
        bot: &mut engine.bot as *mut _ as *mut (),
        rm: &mut engine.render_models as *mut _ as *mut (),
        re: re_raw,
        rmg: &mut engine.rmg as *mut _ as *mut (),
        g2: &mut engine.g2 as *mut _ as *mut (),
        icarus: &mut engine.icarus as *mut _,
        nav: &mut engine.nav as *mut _,
        roff: &mut engine.roff as *mut _,
    }));
    arm_game_slot(
        note as *mut GameDispatchCtx as *mut core::ffi::c_void,
        game_system_calls_shim,
    );

    // One note serves both client slots: cgame and ui differ only in which
    // dispatcher the armed shim enters (DEC-55.1). `cl` and `re` point at the
    // `Option` FIELDS, both still `None` here, so the client's own seating step
    // may run after this arming.
    let client_note: &'static mut ClientDispatchCtx = Box::leak(Box::new(ClientDispatchCtx {
        common: &mut engine.common as *mut _,
        cm: &mut engine.cm as *mut _,
        sv: &mut engine.sv as *mut _ as *mut (),
        cl: &mut engine.cl as *mut _,
        bot: &mut engine.bot as *mut _ as *mut (),
        rm: &mut engine.render_models as *mut _,
        re: &mut engine.re as *mut _,
        rmg: &mut engine.rmg as *mut _,
        g2: &mut engine.g2 as *mut _,
        roff: &mut engine.roff as *mut _,
    }));
    let client_ctx = client_note as *mut ClientDispatchCtx as *mut core::ffi::c_void;
    arm_cgame_slot(client_ctx, cgame_system_calls_shim);
    arm_ui_slot(client_ctx, ui_system_calls_shim);
}
