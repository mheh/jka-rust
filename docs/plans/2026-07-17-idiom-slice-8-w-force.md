# Idiom slice 8 — w_force (DEC-31)

> **Status: EXECUTED — merged to master (idiom era, DEC-31; slice 8 branch `idiom/w-force`).**

The force-power system (w_force.rs, 6,189 lines, 49 fns). Branch:
`idiom/w-force`. w_saber follows as its own slice (the pair is too big
for one). Gates: referee 8/8 at every commit (bots on the ffa1 tape
push/pull/jump/heal constantly).

Standing rulings apply. Slice-specific shapes:

- **c1 — mechanical sweep**: inline `crate::` paths -> file-top
  imports (~28), the 4 `__teid` scripted temps -> `tent_id`.
- **c2 — predicate flips**: the 10 `-> qboolean` fns flip to `bool`
  (WP_ForcePowerAvailable/InUse/Usable, ForceTelepathyCheckDirect-
  NPCTarget, G_InGetUpAnim, G_IsMindTricked, WP_HasForcePowers,
  G_SpecialRollGetup, CanCounterThrow, Jedi_DodgeEvasion), ~42 call
  sites across w_force/NPC_AI_Jedi/w_saber/g_client/g_weapon.
  CanCounterThrow's `pull` param and ForceTelepathy's `tookPower`
  out-param (single caller) flip with them.
- **c3 — burst/borrow pass**: tent-style temp-entity bursts and
  entity_mut one-liner runs onto held borrows where ctx-read hoisting
  allows (slice-6/7 pattern). Pool-client (`DEC-29`) and ghoul2 raw
  derefs stay.

Out of scope: w_saber (slice 9); bg_pmove force-power surfaces.
