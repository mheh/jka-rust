# The Two-Island State Model (STATE-D1 visualization)

Session artifact for the A2 (state-ownership) design session, 2026-07-02.
Shows how one owned `Engine` (engine side) and one owned `GameWorld` (module
side) interact across the raw ABI seam without fighting the borrow checker,
using MP's hardest reentrancy case (Chain A: server frame -> game module ->
trap_LinkEntity -> engine world-sector mutation).

## The model

```
     ENGINE SIDE (one owned Engine)         MODULE SIDE (one owned GameWorld)
     ------------------------------         ---------------------------------

main()
 |
 +-> com_frame(&mut Engine)
      |
      +-> sv_frame(&mut Engine)
           |
           |  loop {
           |    engine.sv.time += msec;   <-- fields used BETWEEN calls only;
           |                                  no borrow held ACROSS the call
           |    vm_call(GAME_RUN_FRAME) ----------.
           |  }                                    |
           |                                       v
           |                        g_run_frame(&mut GameWorld, svc)
           |                                       |
           |                          for id in 0..num_entities {
           |                            // EntityId re-borrow discipline:
           |                            // borrow world.entities[id] briefly,
           |                            // release before any nested call
           |                            g_run_mover(&mut world, id, svc)
           |                          }            |
           |                                       v
           |                        trap LinkEntity(raw *mut ent)
           |                                       |
           |     ===================== THE RAW SEAM =====================
           |     =  LocateGameData registered base+stride with engine   =
           |     =  at init. Engine reads/writes entity memory ONLY     =
           |     =  through these raw pointers (unsafe, rules D11).     =
           |     =  Borrow checker never sees the aliasing, by design.  =
           |     =  Identical contract in NativeDll AND Static builds.  =
           |     ========================================================
           |                                       |
           v                                       |
      dispatcher  <--------------------------------'
      sv_link_entity(&mut engine.sv.world_sectors, ent_view)
           |
           v
      mutates world-sector lists
      (engine-owned struct -- DISJOINT from GameWorld)
```

## Why the naive alternative fails

```
NAIVE mega-struct:   All { engine, game_world }

  g_run_frame(&mut all)              <-- holds &mut all (entity loop)
       |
       +-> trap link_entity(&mut all)  <-- needs &mut all AGAIN
                                           => COMPILE ERROR: the borrow
                                           checker cannot see through the
                                           indirect call that the two uses
                                           touch disjoint fields
```

## The three load-bearing tricks

1. **Reborrow-threading** — `&mut Engine` flows *down* the call chain; every
   caller keeps only locals across nested calls. This is the oracle's own
   shape: `SV_Frame` holds `timeResidual` locals while `VM_Call` runs
   (`oracle/codemp/server/sv_main.cpp:909-915`).
2. **EntityId re-borrow discipline** — module logic passes `(world, id)` and
   re-indexes per access instead of carrying `&mut entity` across calls
   (rules §B5; the GP2 arena precedent).
3. **The raw seam** — engine<->module entity aliasing is `unsafe` pointer
   arithmetic behind the `SharedGameData` abstraction in every native
   transport, identical to the oracle's `sv.gentities` contract
   (`oracle/codemp/server/sv_game.cpp:327-335`), confined per §D11.
   Rust's guarantees hold *within* each island; the seam is the one audited
   crossing.

Corollaries:
- No `RefCell`, no effect queues (queues would break same-frame
  link-then-query parity, e.g. `G_TouchTriggers` -> `trap_EntitiesInBox`
  observing entities linked earlier in the same frame).
- Seam dispatchers/entrypoints use `extern "C-unwind"` so `com_error` panics
  can traverse live C frames when hosting real mod DLLs (STATE-D3 session
  note; Raven throws C++ exceptions through the same frames today).
- Multi-world (STATE-D2 session note): `GameWorld` is a value and the engine
  holds *a* seam registration — multiple worlds/registrations are
  structurally possible later without redesign.
