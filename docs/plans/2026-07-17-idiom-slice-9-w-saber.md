# Idiom slice 9 — w_saber (DEC-31)

The saber system (w_saber.rs, 11,875 lines, 79 fns) — the biggest
slice yet. Branch: `idiom/w-saber`. Gates: referee 8/8 at every commit
(saber combat dominates the ffa1 tape).

Standing rulings apply. Slice-specific shapes:

- **c1 — mechanical sweep**: inline `crate::` paths -> file-top
  imports (~181), `__teid11-13` -> `te_id` pattern.
- **c2 — predicate flips**: the 22 `-> qboolean` fns flip to `bool`
  (G_CanBeEnemy, G_CheckLookTarget, SaberAttacking, WP_SabersCheck-
  Lock/2, WP_GetSaberDeflectionAngle, G_ClientIdleInWorld,
  G_G2TraceCollide, G_SaberInBackAttack, G_SaberFaceCollisionCheck,
  G_SaberCollide, WP_SabersIntersect, CheckSaberDamage, CheckThrown-
  SaberDamaged, saberKnockOutOfHand, saberCheckKnockdown_*4,
  G_KickDownable, G_PrettyCloseIGuess, HasSetSaberOnly). Callers are
  almost all file-internal; externals: bg_channel `can_be_enemy` seam
  (trait stays qboolean — cast at the seam), g_cmds, g_active, and
  the 5 HasSetSaberOnly files. **WP_SaberCanBlock keeps `-> c_int`**
  (oracle returns `int`, w_saber.c:9276); only its `projectile:
  qboolean` param flips.
- **c3 — plain-copy pass**: the plain subset of 130 `_VectorCopy`
  sites -> assignments.
- **c4 — burst pass**: te/teS temp-entity write bursts -> safe
  borrows (slice-6 c4 pattern).

Out of scope: bg_saber/bg_pmove saber surfaces; ghoul2 trace internals.
