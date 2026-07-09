# ClientThink/ClientEndFrame/usercmd-seam parity audit (2026-07-07)

Context: produced while diagnosing live OpenJK prediction misses (client
reports constant per-snapshot misses; later refined: ~17.88-18.00 units
CONTINUOUSLY WHILE STANDING STILL, decomposing as a near-constant
(16, 8, 0..2) displacement vector). Diagnosis continues; this document
records the audited-clean surface and the referee coverage gaps found.
Read-only audit: no code was changed by the auditing agent.

## Audited byte-faithful (our file:line <-> oracle file:line)

- ClientThink_real complete (g_active.rs:2063-3769 <-> g_active.c:1939-3611):
  serverTime clamp/msec computation (rs:2202-2223 <-> c:2074-2098);
  pmove_msec clamp + pmove_fixed rounding (rs:2225-2247 <-> c:2100-2111);
  pre-Pmove ordering FL_FORCE_GESTURE -> fallingToDeath -> otherKiller ->
  useDelay -> G_AddPushVecToUcmd -> G_CheckMovingLoopingSounds
  (rs:2944-3006 <-> c:2829-2892); full pm.* setup incl. pm.cmd copy,
  tracemask, pmove_fixed|pers.pmoveFixed, animations, baseEnt/entSize
  (rs:3008-3072 <-> c:2894-2987); post-Pmove tail incl.
  BG_PlayerStateToEntityState(qfalse), SendPendingPredictableEvents,
  currentOrigin copies around LinkEntity/G_TouchTriggers/ClientImpacts
  (rs:3229-3767 <-> c:3125-3609). No post-Pmove write to
  ps.origin/velocity/viewangles for a walking player.
- pers.pmoveFixed never set on either side (oracle's userinfo parse is
  commented out, g_client.c:2171-2179).
- GAME_CLIENT_THINK dispatch passes ucmd NULL (game_context.rs:158 <->
  g_main.c:525-527); ClientThink GetUsercmd uses the clientNum parameter
  directly (g_active.rs:3802-3833 <-> g_active.c:3649-3720) — the
  ClientSpawn stride bug (fixed, g_client.rs:3250) never affected the
  per-frame path, and slot 0 was correct even pre-fix (offset 0).
- usercmd_t layout: size 28, serverTime@0 angles@4 buttons@16 weapon@20
  forcesel@21 invensel@22 generic_cmd@23 fwd/right/up@24-26 — asserted.
- ClientEndFrame (g_active.rs:3902-3968 <-> g_active.c:3794-3874): powerup
  expiry, P_WorldEffects, P_DamageFeedback, EF_CONNECTION,
  STAT_HEALTH=health, G_SetClientSound, PlayerStateToEntityState(qfalse),
  SendPendingPredictableEvents. Nothing touches ps.origin/velocity.
- BG_PlayerStateToEntityState (bg_misc.rs:1516-1643): only ps write is
  entityEventSequence (matches oracle); snap=qfalse does not snap.
- G_RunFrame client-loop ordering incl. WP_ForcePowersUpdate/
  WP_SaberPositionUpdate before G_RunClient (g_main.rs:3873-3892 <->
  g_main.c:3963-3979); level.time = levelTime direct (g_main.rs:3574).
- Cvar rows/flags: g_stepSlideFix "1" SERVERINFO, g_smoothClients "1",
  pmove_fixed/pmove_msec SYSTEMINFO; flag bit values verified.
- trap_Trace encoding (9 words incl. trailing 0,10) and JKA-MP trace_t
  layout (byte allsolid@0, byte startsolid@1, short entityNum@2,
  fraction@4, size 48).
- pmove ground path (independent sub-audit): PM_Friction, PM_Accelerate,
  PM_CmdScale, PM_WalkMove, PM_GroundTrace, PM_SlideMove,
  PM_StepSlideMove, PM_ClipVelocity, PM_CorrectAllSolid, PM_CrashLand,
  PM_CheckDuck/PM_CheckFixMins + constants: zero divergences.

## Attribution notes (superseded in part)

The audit ran while the symptom was briefly believed fixed; its
attribution (memory-init fixes: §19 byte-wise memsets / BG_AllocPad8 /
gclient_t memset-size) is retained as the best-fit *class* for the
original 3.0/6.2 sawtooth, but the symptom PERSISTS on 6cc66f7b in the
standing-still ~18-unit constant-vector form. With all movement logic
audited clean, the live defect is state/memory or seam-data, not logic.
Working hypothesis for the constant vector: snapshot ps carries a
constant nonzero velocity (~(640,320,80); x25ms = the observed (16,8,2))
that the server's own sim does not have — i.e. something clobbers or
never clears ps.velocity in the ps the engine snapshots.

## Referee coverage gaps found (actionable)

1. The harness mock trap_Trace always returns fraction 1.0 — the player
   is never grounded, so ground/walk pmove has NEVER been diffed
   in-harness (air-only coverage). A ground-returning mock would make the
   walking/spawn path bisectable offline. (crates/jampgame/tests/common/mod.rs:463-489
   documents the uninitialized-bytes caveat.)
2. Scenarios run g_synchronousClients 1 only — the async
   GAME_CLIENT_THINK ordering (the live mode) is never exercised.
