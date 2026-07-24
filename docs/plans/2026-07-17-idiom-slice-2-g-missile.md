# Idiom slice 2 — g_missile (DEC-31)

> **Status: EXECUTED — merged to master (idiom era, DEC-31; slice 2 branch `idiom/g-missile`).**

Second idiom-era slice: the projectile subsystem (`g_missile.rs`, 10 fns,
1,418 lines) rewritten idiomatically on the slice-1 exemplar patterns.
Branch: `idiom/g-missile`. Gates: referee 8/8 (duel1-combat + ffa1 tapes
carry dense projectile traffic) at every commit.

## Settled shapes (user sit-down 2026-07-17)

- **Raven names stay** (DEC-31); signatures idiomize with mechanical
  caller adaptation, per the slice-1 `gentity_t.item` precedent:
  - `CreateMissile(... , owner: EntityId, altFire: bool) -> EntityId`
    (was `-> *mut gentity_t`, `altFire: qboolean`); callers in
    `g_weapon.rs` + NPC AI files adapt at the touch point only.
- **Mega-fn split ruling (new, applies era-wide): splits are allowed,
  and every extracted helper carries an explicit "Split from
  `<Raven fn>`" doc note plus the exact oracle line-range cite.**

  ```rust
  /// Split from `G_MissileImpact` — saber-block branch.
  /// Source: `oracle/codemp/game/g_missile.c:520-610`
  fn missile_blocked_by_saber(...)
  ```

  `G_MissileImpact` (~650 lines) is the first application; the public
  Raven-named fn stays the entry point.
- Slice-1 patterns apply throughout: entity-borrow collapse without
  bare scope braces (NLL), family-internal `qboolean` → `bool`,
  evaluation order preserved exactly, wire-visible values byte-stable
  (impact events, bounce trajectories, snap_vector calls, RNG order).
- DEC-29 pool-client reads (`(*ctx.entity(x).client).ps...`) stay raw;
  the file-local Stage-1 `ent_resolve*` helpers retire where handle
  flow makes them dead.

## Landing plan

1. c1: signature flips (`CreateMissile` → `EntityId`/`bool`) + caller
   adaptation + Stage-1 scaffolding retire where free.
2. c2: small fns (G_ReflectMissile, G_DeflectMissile, G_BounceMissile,
   G_ExplodeMissile, G_RunStuckMissile, G_BounceProjectile,
   G_MissileBounceEffect).
3. c3: `G_MissileImpact` with noted splits.
4. c4: `G_RunMissile` + leftovers sweep.
