# Idiom slice 7 — g_object + g_exphysics (DEC-31)

> **Status: EXECUTED — merged to master (idiom era, DEC-31; slice 7 branch `idiom/g-object-exphysics`).**

The two small physics files (g_object.rs 344 lines / 4 fns,
g_exphysics.rs 300 lines / 1 fn). Branch: `idiom/g-object-exphysics`.
Gates: referee 8/8 at every commit (G_RunObject drives tape asteroids;
G_RunExPhys drives dropped-flag/limb physics).

Standing rulings apply. Slice-specific shapes:

- **c1 — g_object**: inline `crate::` paths -> file-top imports (~30),
  plain `_VectorCopy` -> assignments, `ctx.entity_mut(id)` write bursts
  cluster onto held borrows (ctx-read RHS hoisted first, the
  slice-6 c4 pattern). `DoImpact`'s qboolean arg stays (g_active's
  slice owns that flip).
- **c2 — g_exphysics**: same sweep; `autoKill: qboolean -> bool`
  (5 call sites, all literals); local `hasFirstCollision` -> bool;
  `while i < numG2Bolts` -> for loop. The `g2Bolts: *mut c_int` +
  `numG2Bolts` pair keeps Raven arity (every current caller passes
  null/0; the bolt branch stays faithful, not dropped). Fn-wide
  unsafe narrows to the zeroed inits and the `g2Bolts` reads.
