# Idiom slice 5 — g_utils (DEC-31)

> **Status: EXECUTED — merged to master (idiom era, DEC-31; slice 5 branch `idiom/g-utils`).**

The shared toolbox (g_utils.rs, 2,752 lines, 66 fns). Branch:
`idiom/g-utils`. Gates: referee 8/8 at every commit (spawns, temp
events, sounds and use-targets fire on every tape frame).

Standing rulings apply. Slice-specific shapes:

- **c1 — the returns flip**: `G_Spawn(ctx) -> EntityId` (40 sites) and
  `G_TempEntity(ctx, ...) -> EntityId` (84 sites), per the
  CreateMissile/LaunchItem precedent. Callers' ptr->id round-trips
  collapse; raw-region callers derive the pointer at the use site
  (`ctx.entity_mut(id) as *mut gentity_t`). Compile-error-driven
  caller sweep; two-line collapse scripted, stragglers by hand.
- **c2 — toolbox internals**: borrow collapse, plain-copy assignments,
  inline-path imports, family-internal qboolean -> bool across the 66
  fns; G_UseTargets/G_Sound/G_SetOrigin/spawn-string helpers keep
  Raven names and exact behavior.

Out of scope: G_FreeEntity's Option<EntityId> param shape is already
idiomatic; the classname *mut c_char field flip stays deferred (would
ripple the spawn-string table).
