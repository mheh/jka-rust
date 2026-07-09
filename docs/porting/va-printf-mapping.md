# va / printf / Com_sprintf mapping — pass-3 porter reference

Mechanical conversion rules for Raven's `va()`, `Com_sprintf()`, and the
`Q_str*` string ops. Apply these verbatim; do **not** invent cleverer string
handling. Ruling 18 (`docs/handoffs/jampgame-fork-discovery.md`): va/printf →
`format!`; `Com_sprintf` into `char[N]` → write-bytes+NUL; one blessed
divergence at `g_target.c:800`.

Owning types live in `crates/mp/game/src/q_shared.rs`
(`Com_sprintf` :1160, `va` :1190, `Q_strncpyz` :912, `Q_strcat` :1071).

---

## 1. `va(fmt, ...)` — rotating-scratch formatter → owned `String`

Raven `va` returns a pointer into a 2-slot rotating static buffer
(`q_shared.c:1017-1031`). The value is **consumed immediately** (passed to a
trap, copied into a field, printed) — it is never stored long-term. Port each
call to a freshly formatted owned `String`, consumed on the spot.

```c
trap_SendServerCommand( -1, va("print \"%s entered the game\\n\"", name) );
```
```rust
let s = format!("print \"{} entered the game\\n\"", cstr_to_str(name));
crate::trap::SendServerCommand(ctx.engine, -1, cstr(&s));   // s lives to the call
```

Rules:
- `va(fmt, args)` → `format!(rust_fmt, args)` producing a `String`.
- Bind it (`let s = format!(...)`) when the callee needs a `*const c_char`; pass
  a NUL-terminated view (`CString`/`c""`-style helper) whose owner outlives the
  call. Never return a pointer into a temporary.
- A single-use `va` inside a larger `Com_sprintf`/`trap_*` arg → inline the
  `format!` as the argument only if its `String` temporary lives across the call
  (it does for a plain fn-arg; it does **not** if the callee stashes the ptr).

## 2. printf-family format specifiers → Rust `format!` specs

| C spec        | Rust spec            | Note |
|---------------|----------------------|------|
| `%s`          | `{}`                 | C string → `&str`/`&CStr` (decode first) |
| `%d` / `%i`   | `{}`                 | `c_int` |
| `%u`          | `{}`                 | unsigned |
| `%c`          | `{}`                 | format the `char` (decode the byte) |
| `%f`          | `{}`                 | `f32`/`f64`; Rust prints all sig digits |
| `%x` / `%X`   | `{:x}` / `{:X}`      | hex |
| `%%`          | `%`                  | literal percent |
| `%5d`         | `{:5}`               | min width 5, right-justified |
| `%-5d`        | `{:<5}`              | left-justified |
| `%05d`        | `{:05}`              | zero-padded |
| `%5.2f`       | `{:5.2}`             | width.precision |
| `%.2f`        | `{:.2}`              | precision only |
| `%3.0f`       | `{:3.0}`             | |
| `%08x`        | `{:08x}`             | zero-padded hex |

Notes:
- Rust `{}` for floats is **not** byte-identical to C `%f` (C defaults to 6
  decimals: `%f` == `{:.6}`). If parity matters (values that hit the wire or a
  logged golden), use `{:.6}` for a bare `%f`. Where the string is only shown to
  a human (Com_Printf debug), `{}` is fine.
- Positional args stay in order; there are no dynamic (`*`) widths in the
  jampgame call set — every width/precision is a literal.

## 3. `Com_sprintf(buf, size, fmt, ...)` into a fixed `char[N]` field/local

Raven writes a formatted, NUL-terminated, size-capped string into a fixed
buffer. Port to: build the `String` with `format!`, then copy bytes + NUL into
the `[c_char; N]` target, truncating at `N-1`. Use the canonical helper below
(add it to `q_shared.rs` next to `Com_sprintf` if not present):

```rust
/// Write `src` into a fixed C char buffer, truncating to `N-1` bytes + NUL.
/// The `Com_sprintf`/`Q_strncpyz` byte-copy dual.
pub fn write_cstr_field(dest: &mut [c_char], src: &str) {
    let n = dest.len();
    if n == 0 { return; }
    let bytes = src.as_bytes();
    let copy = bytes.len().min(n - 1);
    for i in 0..copy { dest[i] = bytes[i] as c_char; }
    dest[copy] = 0;
}
```
```c
Com_sprintf( ent->soundSet, sizeof(ent->soundSet), "%s/%s", set, name );
```
```rust
write_cstr_field(&mut (*ent).soundSet, &format!("{}/{}", set, name));
```

If the target is a `*mut c_char` of caller-owned length `size` (not a struct
array), keep the raw `Com_sprintf(dest, size, ...)` seam signature and copy
through the pointer with the same truncate-at-`size-1` rule.

## 4. `Q_strncpyz` / `Q_strcat` duals

- `Q_strncpyz(dest, src, destsize)` — size-capped copy, always NUL-terminates.
  Into a fixed field: `write_cstr_field(&mut dest, src)`. Through a raw pointer:
  call the ported `crate::q_shared::Q_strncpyz(dest, src, destsize)` (:912).
- `Q_strcat(dest, size, src)` — append, size-capped. Through a raw pointer: call
  the ported `crate::q_shared::Q_strcat(dest, size, src)` (:1071). Into an owned
  `String`, use `dest.push_str(src)` then re-emit with `write_cstr_field`.
- Do not replace these with naive `strcpy`/`format!` that drops the size cap —
  the cap is behavior (parity).

## 5. `trap_SendServerCommand(client, va(...))` idiom

The dominant `va` consumer. Bind the formatted string first so it outlives the
syscall, then pass a NUL-terminated view:

```rust
let cmd = format!("cp \"{}\"", msg);
crate::trap::SendServerCommand(ctx.engine, clientNum, cstr(&cmd));
```
Never `SendServerCommand(engine, n, format!(...).as_ptr())` — the temporary
drops before the syscall reads it (dangling). Bind, then pass.

## 6. BLESSED DIVERGENCE — `g_target.c:800` (ruling 18/19)

Raven stores a `va()` rotating-buffer pointer into a **persistent** field:

```c
// oracle/oracle/codemp/game/g_target.c:800
self->activator->script_targetname = va( "newICARUSEnt%d", numNewICARUSEnts++ );
```

This is a Raven UB-adjacent bug: `script_targetname` outlives the 2-slot
rotating buffer, so a later `va` call can clobber the stored name. Do **not**
reproduce the aliasing. Diverge to an **owned** string that the field owns for
real, and leave a ≤2-line note at the site:

```rust
// DIVERGENCE (ruling 18/19): Raven stores a va() rotating-buffer ptr into the
// persistent field script_targetname (g_target.c:800) — UB-adjacent aliasing.
// We store an owned, NUL-terminated allocation instead.
let name = format!("newICARUSEnt{}", num_new_icarus_ents);
num_new_icarus_ents += 1;
(*(*self_).activator).script_targetname = ctx.new_string(&name); // owned, via G_NewString
```

Exclude this behavior from any shared differential fixture (the buffer-reuse
timing is not observable in the corrected version and would fail a naive golden).
Every other `va` site keeps the immediate-consume rule from §1.
