# Pass-3 prep: design-session agenda + checklist (2026-07-04)

Research complete (5 dossiers, this session). Goal: ONE final porter pass
(zero-park transcription) + wiring → bootable jampgame. Everything below is
either a user ruling (A), mechanical prep (B), wiring (C), or a quick check (D).

Dossier sources: task outputs archived in session; key numbers inlined here.
Symbol manifest: `tools/closure-prototype/out/pass3/missing-symbols.json`.

## A. Design rulings needed (one session)

1. **PmoveContext + BgState** (fork 8a concrete). Facts: bg working set spans
   6 files — per-call set (pm, pml, pm_entSelf/Veh, flags) + session set
   (bgAllAnims ×16 refs, saber parse buffers, vehicle info arrays, bg_misc
   pool/itemlist). Engine surface from bg: pmove_t's 2 callbacks (ported
   already, C-shape) + trap_FS_* (5 files) + strap_G2API_* (~35 sites,
   bg_pmove) + FX + SnapVector/Trace. Entity access: bgEntity_t head-overlay
   (baseEnt/entSize, ~15 direct g_entities leak sites, BG_MySaber ×30).
   RNG: 8 bg sites; LCG (holdrand q_math.c:1432) still unported — placement
   rides this ruling.
2. **Boundary-41 channel** — collapses to 3 mechanisms (dossier classification
   14a/5b/16c/6d): (a) 14 engine-only (Com_Printf/Com_Error/indexers/straps)
   → the ruling-1 bg trap facade (Raven's own GAME_HARD_LINKED shim layer);
   (b) 5 world-only → BgState + time; (c) **16 both** (G_Damage, G_AddEvent,
   G_PlayEffect, G_Alloc…) → proposed GameCallbacks trait handed to bg in
   PmoveContext (G_Damage reachable from Pmove EVERY frame via
   PM_SlideMove→PM_VehicleImpact — cannot stub); (d) 6 vehicle-vtable slots →
   fork-7 enum, enum type must live in bg (vtable wired from
   bg_vehicleLoad.c:694-703).
3. **Entity-repr (fork 4 concrete)**: 38 stored gentity_t* fields
   (bot_state_t 11, gNPC_t 13, gentity_t 10, gclient_t 4), 175
   identity-compare sites in ai/NPC. None ABI-crossing. Rule: EntityId now
   (pass-3 porters write id compares) vs pointers-now-migrate-later.
4. **va/printf mapping table** (~899 sites; specs tame: %s/%d/%i/%f/%c + few
   widths, no dynamic). One page of mechanical rules. Special ruling:
   g_target.c:800 stores va() rotating-buffer ptr into persistent field
   script_targetname — genuine Raven UB-adjacent bug; owned String diverges
   (per §19, note at site) or emulate.
5. **bg crate placement**: bg_*.c ports currently live in crates/mp/game;
   strap wrappers skeleton'd in g_strap.rs. Move to crates/mp/bg in pass 3 or
   defer split until post-parity?

## B. Mechanical prep (after A; no decisions)

6. Symbol backfill wave from missing-symbols.json: 1,528 missing (633 macro,
   424 enum-const, 471 global-var — globals mostly = B3 state, filter to true
   consts/tables) + 732 private-needs-export (prelude/re-export fix, scripted).
   Tables: saberMoveData/transitionMove/forcePowerNeeded/weaponData/ammoData/
   bg_itemlist etc. per bg dossier §3.
7. bg signature retrofit (PmoveContext/BgState/GameCallbacks params) + packet
   regen — retarget pass-2 retrofit tooling.
8. vec3 call-site fixer: ~70 sites, root cause = porters wrote
   old-by-ref convention against reshaped q_math sigs (_VectorCopy 21,
   BG_EvaluateTrajectory 7…). Scriptable.
9. Hand-proved pmove slice (Pmove→PmoveSingle→PM_Friction/PM_SetWaterLevel)
   validating ruling 1 BEFORE tooling stamps ~250 bg signatures.
10. Porter instruction update: zero-park policy (transcribe + // PORT-NOTE
    instead of todo!()), raw-pointer deref discipline ((*ptr).field — kills
    the ~500 E0609 bucket), EntThink-family import lines, use crate::trap.
11. ICARUS: NO stub design needed (dossier: interface_export_t is
    engine-side; game sees only 11 trap_ICARUS_* wrappers + GAME_ICARUS_*
    gSharedBuffer structs, all already typed in g_public.h). Generate the 11
    wrappers; g_ICARUScb's 155 fns port as ordinary game logic.

## C. Wiring to run (post-porters)

12. Remaining ~25 Dispatch impls (GameContext currently 2/27).
13. **spawns[] table** (g_spawn.c ~200 classname→fn): needs fork-2 enum
    treatment; NOT in generated ent_fn_enums — must be added or boot dies at
    first map entity.
14. ClientConnect seam quirk: returns char* across the seam (only vmMain
    command returning a pointer) — define the shape.
15. cdylib crate-type + dllEntry/vmMain C exports.
16. Boot target: stock jampded 32-bit (favored) → i686 cross-target +
    per-arch layout asserts must pass.
17. ICARUS boot minimum: trap_ICARUS_Init callable +
    trap_SV_RegisterSharedMemory(gSharedBuffer) accepted; scripts only run on
    spawn/use triggers.

## D. Quick checks before pass-3 launch

18. The 17 unported-type params from pass-1 land (notably gentity.m_pVehicle:
    () — vehicle bodies dot into it).
19. GameGlobals () placeholders (67) — any touched by ported bodies must get
    real types in the backfill wave.
20. Verify GAME_CLIENT_CONNECT arg/return marshalling in mp_abi against
    dossier finding (C.14).

## Current state (for resume)

- Pass 2 PAUSED mid-Port: 62/94 porters done (694 filled, 1,226 re-parked),
  WIP commit deba0ff on skeleton; resume wf_1817c88f-07b (integration phase
  never ran; 2,259 errors in worktree are pre-integration state, bucketed in
  entity-repr/vec3 dossier).
- Decision needed: resume pass-2 remainder (32 porters + fixers) as-is, or
  fold its remainder into pass 3 after prep (avoids re-fixing; recommended).
