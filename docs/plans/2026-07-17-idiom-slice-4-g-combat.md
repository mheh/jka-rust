# Idiom slice 4 — g_combat (DEC-31)

> **Status: EXECUTED — merged to master (idiom era, DEC-31; slice 4 branch `idiom/g-combat`).**

The centerpiece slice: g_combat.rs (6,348 lines, 47 fns) rewritten
idiomatically. Branch: `idiom/g-combat`. Gates: referee 8/8 at every
commit — the duel1-combat and ffa1 tapes cover every damage tick,
death, and respawn.

Standing rulings (all settled, no new user decisions expected):
- Raven names stay; splits allowed with "Split from `<fn>`" + oracle
  line-range cites; kind matches for item reads; family-internal
  qboolean → bool; borrow collapse without bare scope braces;
  evaluation order preserved exactly.
- **G_Damage aliasing ruling**: suicide aliases targ/attacker/inflictor
  (g_cmds.c:1193) — sequential re-acquire discipline throughout, NO
  entity_pair_mut / split borrows.
- The oracle file has no gotos; splits are for size, not labels.
- world.globals.death_anim_i (player_die's rotating anim counter,
  folded 5ed6e1b0) stays world-owned.

## Landing plan

1. c1 — small/medium fns outside the mega-fns: ObjectDie, G_HeavyMelee,
   G_GetHitLocation, ExplodeDeath, Score/AddScore, TossClientWeapon/
   Items, LookAtKiller, GibEntity, BodyRid, body_die, CheckAlmost*,
   G_InKnockDown, alert/obit family, CheckArmor, G_ApplyKnockback,
   RaySphereIntersections, G_Knockdown, vehicle-killer helpers,
   CanDamage, G_RadiusDamage, G_DamageFromKiller.
2. c2 — dismemberment + hit-location family: G_CheckSpecialDeathAnim,
   G_PickDeathAnim, G_GetDismemberLoc/Bolt, Limb*, G_Dismember,
   Dismemberment*, G_GetHitQuad, G_GetHitLocFromSurfName,
   G_CheckForDismemberment, G_LocationBasedDamageModifier.
3. c3 — player_die (g_combat.c mega-fn ~1,070 lines): split into
   phase helpers with noted cites (vehicle-exit/spectator prep, scoring
   + obituary, dueling wrap-up, death anim + events, body handling).
   Exact split boundaries decided at the code face.
4. c4 — G_Damage (~1,280 lines): split into noted phase helpers
   (godmode/shield/absorb gates, knockback, armor, team/self checks,
   damage application + death handoff). Sequential re-acquire
   discipline throughout.
