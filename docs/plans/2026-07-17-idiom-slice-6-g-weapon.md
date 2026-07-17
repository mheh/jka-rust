# Idiom slice 6 — g_weapon (DEC-31)

The weapon-fire surface (g_weapon.rs, 6,234 lines, ~90 fns). Branch:
`idiom/g-weapon`. Gates: referee 8/8 at every commit (the ffa1 tape
cycles weapons; every fire path and impact event is on tape).

Standing rulings apply. Slice-specific shapes:

- **c1 — the deferred bool flip**: `CanDamage -> bool` and
  `G_RadiusDamage -> bool` (defined in g_combat.rs; flip deferred here
  by the slice-4 plan). 18 call sites across 11 files; `qtrue/qfalse`
  returns and `!= 0` call-site comparisons collapse.
- **c2 — mechanical file sweep**: inline `crate::` paths -> file-top
  imports (~236 sites), plain-copy `_VectorCopy`/`VectorCopy` ->
  `[f32;3]` assignments, `__teidN` scripted temps -> named ids
  (`te_id` matching Raven's `tent`). w_saber/w_force keep their
  `__teid` temps for their own slices.
- **c3 — alt-fire bool**: `altFire`/`alt_fire: qboolean -> bool` across
  the WP_Fire* family + FireWeapon/FireVehicleWeapon dispatch, per the
  CreateMissile precedent (slice 2). ~16 external call sites.
- **c4 — raw-pocket idiomization**: the big fns (WP_DisruptorMainFire /
  WP_DisruptorAltFire, DEMP2_AltRadiusDamage, WP_LobFire,
  WP_FireConcussionAlt, FireVehicleWeapon, emplaced_gun_update) —
  borrow collapse, accessor conversion, dead-guard drops. Mega-fn
  splits allowed under the slice-2 ruling (`Split from` note + oracle
  line-range cite) where a phase is naturally input-only.

Out of scope: cross-file w_saber/w_force interiors; ghoul2 bolt-math
call shapes (stay as-is at the trap seam).
