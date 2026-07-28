# Rosetta Stone — Raven C → jka-rust idiom mapping

The authoritative bilingual reference for the jampgame logic port: every
recurring Raven idiom paired with its exact, compiling Rust shape, lifted
verbatim from the worktree. Porters copy these shapes; fixers point call
sites at them. Inlined into every pass-3 packet by
`tools/closure-prototype/packets3.py` (`## EXAMPLE SYNTAX` section).

**Maintenance rule: every trial/audit finding that reveals a wrong-syntax or
shim pattern adds or amends a stanza here** — this doc, not the generator,
is the source of truth. Where a shape is tier-dependent, both forms appear
(game tier: `ctx: GameContext` / `(*ctx.world)`; bg tier: `impl PmoveContext`
methods / `self.bg` / `self.traps`). RNG access: game `(*ctx.world).bg_state.rng`,
bg `self.bg.rng`.

## EXAMPLE SYNTAX (copy these shapes)

REAL, compiling call shapes lifted verbatim from the worktree — copy them. Do NOT write Raven macro/libc spellings (`VectorCopy(a,b)`, `rand()`, `atoi(s)`) and do NOT report the mapped canonical helpers below as missing symbols.

**NEVER define a local helper/shim fn in your file — if a helper seems missing, use the mapped canonical form below or report it in `missing_symbols`.**

### traps — game tier: `crate::trap::Name(ctx.engine, <Name>Args::new(…))`
One `ctx.engine` + one `…Args::new(…)` struct in oracle arg order; args are raw pointers where Raven passed `T*` (`&mut x as *mut T` / `&x as *const T`).
```rust
// void — trap_LinkEntity(self):
trap::LinkEntity(ctx.engine, GLinkentityArgs::new(self_));

// out-param + buffers — trap_Trace(&tr, start, mins, maxs, end, num, mask):
trap::Trace(
    ctx.engine,
    GTraceArgs::new(
        &mut tr as *mut trace_t,
        &(*self_).r.currentOrigin as *const vec3_t,
        &(*self_).r.mins as *const vec3_t,
        &(*self_).r.maxs as *const vec3_t,
        &(*self_).r.currentOrigin as *const vec3_t,
        (*self_).s.number,
        CONTENTS_BODY,
    ),
);

// G2 strap (fully-qualified Args path when not in the prelude) —
// trap_G2API_GetBoltMatrix(ghoul2, mdl, bolt, &matrix, angles, pos, frame, list, scale):
trap::G2API_GetBoltMatrix(
    ctx.engine,
    mp_abi::game::syscalls::G_G2_GETBOLT::GG2GetboltArgs::new(
        (*eweb).ghoul2, 0, (*eweb).genericValue10,
        &mut boltMatrix as *mut mdxaBone_t,
        &(*eweb).s.apos.trBase as *const vec3_t,
        &(*eweb).r.currentOrigin as *const vec3_t,
        (*ctx.world).level.time,
        core::ptr::null_mut(),
        &(*eweb).modelScale as *const vec3_t,
    ),
);
```

### traps — bg tier: `self.traps.<method>(…)` (`&dyn BgTraps`, snake_case names)
```rust
self.traps.trace(
    &mut trace,
    core::ptr::addr_of!((*ps).origin) as *const vec3_t,
    core::ptr::addr_of!((*self.pm).mins) as *const vec3_t,
    core::ptr::addr_of!((*self.pm).maxs) as *const vec3_t,
    core::ptr::addr_of!(end) as *const vec3_t,
    (*ps).clientNum,
    (*self.pm).tracemask,
);
```
Byte-buffer args (`fs_read`/`fs_write`) take `*mut c_void`/`*const c_void` —
cast the `c_char` buffer at the call, don't change the buffer's type:
```rust
// C: trap_FS_Read(class_info, len, f);   char class_info[4096];
let mut class_info: [c_char; 4096] = [0; 4096];
self.traps.fs_read(class_info.as_mut_ptr() as *mut c_void, len, f);
```

### vec3 / q_shared macros → reshaped `crate::q_math` fns (inputs BY VALUE, outputs `&mut`)
Raven's `q_shared.h` `Vector*`/`DotProduct`/`CrossProduct` MACROS are reshaped functions — NEVER macro call syntax, NEVER a local shim. Note the `_` prefix on the assignment-style ones. Exact names/signatures:
- `VectorCopy(a,b)`       → `_VectorCopy(in: vec3_t, out: &mut vec3_t)`
- `VectorSubtract(a,b,c)` → `_VectorSubtract(a: vec3_t, b: vec3_t, out: &mut vec3_t)`
- `VectorAdd(a,b,c)`      → `_VectorAdd(a: vec3_t, b: vec3_t, out: &mut vec3_t)`
- `VectorScale(a,s,c)`    → `_VectorScale(in: vec3_t, scale: f32, out: &mut vec3_t)`
- `VectorMA(a,s,b,c)`     → `_VectorMA(a: vec3_t, scale: f32, b: vec3_t, out: &mut vec3_t)`
- `DotProduct(a,b)`       → `_DotProduct(a: vec3_t, b: vec3_t) -> f32`
- `CrossProduct(a,b,c)`   → `CrossProduct(a: vec3_t, b: vec3_t, cross: &mut vec3_t)`
- `VectorNormalize(v)`    → `VectorNormalize(v: &mut vec3_t) -> f32` (mutate-in-place + return length)
- `VectorNormalize2(v,o)` → `VectorNormalize2(v: vec3_t, out: &mut vec3_t) -> f32`
- `VectorLength(v)`       → `VectorLength(v: vec3_t) -> f32`  ·  `Distance(a,b)` → `Distance(a: vec3_t, b: vec3_t) -> f32`
```rust
// Raven: VectorSubtract( ent->r.currentOrigin, self->r.currentOrigin, dir );
crate::q_math::_VectorSubtract((*ent).r.currentOrigin, (*self_).r.currentOrigin, &mut dir);
// Raven: VectorCopy( ent->r.currentOrigin, muzzle );
crate::q_math::_VectorCopy((*ent).r.currentOrigin, &mut muzzle);
```
(`vec3_t` is `[f32;3]` and `Copy` — read a struct field straight in by value, no `&`. bg-tier files reach these as `mp_qshared::shared::q_math::…`.)

### float→double promotion — bare double literals promote, `f`-suffixed stay f32
C arithmetic mixing a `float` with a bare double literal (`x *= 0.75;`) promotes to double, then narrows on store — port that promotion explicitly. Literals with an `f` suffix (`0.5f`) stay pure f32. Multi-term expressions keep C's promotion points: each subexpression widens only where C does. (Precedent: PM_Friction / BG_AdjustClientSpeed / BG_G2PlayerAngles, trial-8, `bg_pmove.rs`.)
```rust
// Raven: x *= 0.75;            — bare double literal: promote, then narrow
x = (x as f64 * 0.75) as f32;
// Raven: x *= 0.5f;            — f-suffixed literal: pure f32
x *= 0.5f32;
// Raven: (-360.0 + h) * 0.75;  — f32 addition, then f64 multiply, then narrow
((-360.0f32 + h) as f64 * 0.75) as f32
```

### RNG — Raven `rand`/`random`/`crandom`/`flrand`/`irand` → the one `BgState.rng`
```rust
// Raven: respawn += crandom() * ent->random;
respawn += ((*ctx.world).bg_state.rng.crandom() * (*ent).random) as c_int;
```
(`(*ctx.world).bg_state.rng` is the tier's generator path — see the RNG section above for the full method table.)

### cstr seam — build/decode a `char*` at a syscall boundary (never a hand-rolled CString)
```rust
// Raven: Com_Printf( "%s", s );  — own the bytes in a local, pass `.as_ptr()`
crate::g_main::Com_Printf(cstr(&s).as_ptr());
// Raven: const char *n = <engine char*>;  — decode an engine string
let n: String = unsafe { cstr_to_str(name_ptr) };
```

### fn-ptr dispatch — store an `EntThink`/`EntUse`/… enum, call via `dispatch_*`
```rust
// Raven: self->think = ShieldGoSolid;
(*self_).think = Some(EntThink::ShieldGoSolid);
// Raven: if ( self->think == ShieldGoSolid ) …
if (*self_).think == Some(EntThink::ShieldGoSolid) { /* … */ }
// Raven: ent->think( ent );   (an indirect call)
if let Some(think_fn) = (*ent).think {
    crate::ent_fn_enums::dispatch_think(ctx, think_fn, ent);
}
```

### `Option<EntityId>` stored fields — assign/compare ids, never pointers (ruling 22)
```rust
// Raven: missile->parent = owner;
(*missile).parent = Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), owner));
// Raven: if ( client->hook == ent ) …
if (*client_ptr).hook == Some(ent_id((*ctx.world).g_entities.as_mut_ptr(), ent)) { /* … */ }
// Raven: if ( !ent->enemy ) …
if (*ent).enemy.is_none() { /* … */ }
// ent_id_opt(base, maybe_null_ptr) folds a nullable pointer straight to Option<EntityId>.
```

### multi-RNG-draw expressions — transcribe in SOURCE order (ruling 2026-07-28)
```rust
// Raven: f(..., 300 + (rand() & 99), ..., radius*0.05f + (crandom()*0.3f), ...);
// C leaves argument/operand evaluation order unspecified (MSVC ran right-to-left);
// ruled: the port draws in SOURCE order, left-to-right, everywhere.
let rockRand = world.bg_state.rng.rand() & 99;                     // first in source
let sizeRand = radius * 0.05 + (world.bg_state.rng.crandom() * 0.3) as f32; // second
f(..., 300 + rockRand, ..., sizeRand, ...);
```
