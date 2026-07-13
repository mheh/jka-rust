//! Port of `oracle/codemp/game/g_mem.c` — the game-tier bump allocator.
//!
//! State-threading resolution (supersedes the mega-pass park): Raven's
//! file-scope `static memoryPool`/`allocPoint` become `GameWorld` fields
//! (STATE-D6), and `G_Alloc` reaches them through the `ctx.world` channel the
//! rest of the module already threads — no global/`static mut` (porting-rules
//! §B3). `G_InitMemory`/`Svcmd_GameMem_f` already took `ctx`.
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

/// Raven `G_Alloc` — the game-tier bump allocator over the module island's
/// `memoryPool`/`allocPoint` (STATE-D6: Raven's file-scope `static memoryPool`
/// becomes a `GameWorld` field reached through `ctx.world`). Threading `ctx`
/// supersedes the mega-pass state-threading park (the pool fields already
/// existed on `GameWorld`; only the accessor was missing).
///
/// Source: `oracle/codemp/game/g_mem.c:16-33`
pub fn G_Alloc(ctx: &mut GameContext, size: c_int) -> *mut c_void {
    use crate::g_main::{G_Error, G_Printf};

    // Raven `#define POOLSIZE (256 * 1024)` — `g_mem.c` file-local.
    // Source: `oracle/codemp/game/g_mem.c:11`
    const POOLSIZE: c_int = 262144; // 256 * 1024

    unsafe {
        let world = &mut *ctx.world_raw();

        if world.cvars.g_debugAlloc.integer != 0 {
            G_Printf(
                ctx,
                cstr(&format!(
                    "G_Alloc of {} bytes ({} left)\n",
                    size,
                    POOLSIZE - world.allocPoint - ((size + 31) & !31)
                ))
                .as_ptr(),
            );
        }

        if world.allocPoint + size > POOLSIZE {
            G_Error(
                ctx,
                cstr(&format!("G_Alloc: failed on allocation of {size} bytes\n")).as_ptr(),
            );
            return core::ptr::null_mut();
        }

        let p = world.memoryPool.as_mut_ptr().add(world.allocPoint as usize) as *mut c_void;
        world.allocPoint += (size + 31) & !31;
        p
    }
}

/// Raven `G_InitMemory`.
///
/// Source: `oracle/codemp/game/g_mem.c:35-37`
pub fn G_InitMemory(ctx: &mut GameContext) {
    // Raven: allocPoint = 0;
    unsafe {
        (*ctx.world_raw()).allocPoint = 0;
    }
}

/// Raven `Svcmd_GameMem_f`.
///
/// Source: `oracle/codemp/game/g_mem.c:39-41`
pub fn Svcmd_GameMem_f(ctx: &mut GameContext) {
    use crate::g_main::G_Printf;
    // Raven: G_Printf( "Game memory status: %i out of %i bytes allocated\n", allocPoint, POOLSIZE );
    let poolsize: c_int = 262144; // POOLSIZE = 256 * 1024
    let msg = format!(
        "Game memory status: {} out of {} bytes allocated\n",
        unsafe { (*ctx.world_raw()).allocPoint },
        poolsize
    );
    let msg_cstr = cstr(&msg);
    G_Printf(ctx, msg_cstr.as_ptr());
}
