# B1 — cvar / cmd ground-truth dossier

Scope: `Cvar_*` and `Cmd_*`/`Cbuf_*` semantics, MP (`oracle/codemp/qcommon/{cvar,cmd_common,cmd_pc}.cpp`)
and SP (`oracle/code/qcommon/{cvar,cmd}.cpp`). Global census (who owns
`cvar_vars`, `cmd_functions`, the tokenizer scratch buffer, etc.) is already
done in `docs/dossiers/A2-state-ownership.md` §1b–1c — this doc cites that
census rather than repeating it, and focuses on *behavior*: ordering,
overflow, flag semantics, and the module↔engine seam.

---

## 1. Cvar semantics

### 1.1 `Cvar_Get` flag merging / latching rules

Identical logic MP (`codemp/qcommon/cvar.cpp:188-280`) and SP
(`code/qcommon/cvar.cpp:224-311`), modulo the storage differences in §1.7:

- If the cvar **doesn't exist**, it's allocated fresh with `flags` as given
  (MP `cvar.cpp:256-280`, SP `cvar.cpp:291-311`).
- If it **does exist**, `Cvar_Get` never touches `var->string`/`value`/
  `integer` — only bookkeeping fields change (MP `cvar.cpp:208-251`, SP
  `cvar.cpp:243-286`):
  - `var->flags |= flags` unconditionally (MP `cvar.cpp:223`, SP
    `cvar.cpp:258`) — flags only ever accumulate via `Get`, never clear.
  - **CVAR_USER_CREATED transition**: if the existing cvar was
    user-created (typed at console before any C code declared it) and the
    new `Get` call does *not* pass `CVAR_USER_CREATED` and `var_value[0]`
    is non-empty, the `CVAR_USER_CREATED` flag is stripped, the
    `resetString` is replaced with the C-code-supplied default, and
    `cvar_modifiedFlags |= flags` fires (MP `cvar.cpp:212-221`, SP
    `cvar.cpp:247-256`) — this is how a cvar the user `set` before the
    engine registered it "becomes" a real cvar with a proper default.
  - `resetString` is set once if empty; if non-empty and a *different*
    non-empty value is passed, a warning prints (`Com_DPrintf` in MP
    `cvar.cpp:230`, promoted to `Com_Printf` in SP `cvar.cpp:265` — SP's
    warning is not developer-gated).
  - If a **latched value is pending** (`var->latchedString` set), `Get`
    immediately applies it via `Cvar_Set2(var_name, s, qtrue)` (forced) —
    MP `cvar.cpp:234-241`, SP `cvar.cpp:268-276`. This is the only place a
    latched value becomes live outside of `Cvar_Set2`'s own forced path:
    the *next* `Cvar_Get` call (i.e., typically the next `Cvar_Register`
    at module (re)init, or a subsystem restart) is what actually commits a
    latched cvar — not a frame tick or `Cbuf_Execute`.
- Both mark `Cvar_Get: NULL parameter` as `ERR_FATAL` (MP `cvar.cpp:192-194`,
  SP `cvar.cpp:227-229`), and both silently rewrite an invalid name (one
  containing `\`, `"`, or `;`) to the literal string `"BADNAME"` rather than
  failing the call (MP `cvar.cpp:196-199` / `Cvar_ValidateString` at
  `cvar.cpp:62-76`; SP `cvar.cpp:231-234` / `cvar.cpp:36-50`). Value-string
  validation is `#if 0`'d out in both (MP `cvar.cpp:201-206`, SP
  `cvar.cpp:236-241`) — a value containing `\`/`"`/`;` passes through
  unchecked.

### 1.2 `Cvar_Set2` branches

Byte-identical control flow MP `cvar.cpp:287-395` / SP `cvar.cpp:318-419`:

1. Name validated the same way as `Get` (→ `"BADNAME"` on failure).
2. If the cvar doesn't exist: `NULL` if `value` is `NULL` (a "reset to
   default on a cvar that was never registered" no-op); otherwise
   `Cvar_Get` is called with `CVAR_USER_CREATED` when `!force`, or flags
   `0` when `force` — i.e. a forced `Set` on a nonexistent cvar creates it
   **without** the user-created flag.
3. If `!value`, `value` defaults to `var->resetString` (this is how
   `Cvar_Reset` — `Cvar_Set2(name, NULL, qfalse)` — works, MP
   `cvar.cpp:437-439`).
4. `cvar_modifiedFlags |= var->flags` fires **unconditionally** at this
   point in both engines (MP `cvar.cpp:329`, SP `cvar.cpp:353`) — even for
   values that turn out identical to the current one, even for a value
   that gets rejected two lines later by `CVAR_ROM`/`CVAR_INIT`/cheat
   protection. `cvar_modifiedFlags` is therefore a coarse "someone tried to
   touch a cvar with these flag bits" signal, not "a cvar's live value
   changed." (MP has one extra early-out not present in SP: MP
   `cvar.cpp:325-327` returns before the `cvar_modifiedFlags` OR if
   `value` string-equals `var->string`; SP has no such early identical-value
   check before line 353, so **SP sets `cvar_modifiedFlags` even on a true
   no-op `Set` to the current value**, MP does not.)
5. **`!force` path** (console/script-originated, `Cvar_Command`/`Cvar_Set_f`
   etc. all call with `force=qfalse`):
   - `CVAR_ROM` → print "is read only", return unchanged (checked *before*
     `CVAR_INIT`/`CVAR_LATCH`/cheat, so ROM wins over all of them).
   - `CVAR_INIT` → print "is write protected", return unchanged.
   - `CVAR_LATCH` → do NOT touch `var->string`; store into
     `var->latchedString` (freeing any prior pending latch first), bump
     `modified`/`modificationCount`, print "will be changed upon
     restarting" — but only if the new value differs from the existing
     latch (or, if no latch pending, from the live value); an identical
     resubmission is a no-op that returns early without bumping
     `modificationCount` (MP `cvar.cpp:345-364`).
   - `CVAR_CHEAT` with `cvar_cheats->integer == 0` → print "is cheat
     protected", return unchanged.
6. **`force` path**: any pending `latchedString` is discarded (freed,
   nulled) — a forced set overrides and cancels a latch outright rather
   than merging with it.
7. Final identical-value check (`strcmp(value, var->string)`) — a genuine
   no-op returns without bumping `modified`/`modificationCount` in either
   engine. Otherwise: free old `string`, `CopyString` new one, recompute
   `value`/`integer` via `atof`/`atoi`, bump `modified`/`modificationCount`.

### 1.3 CVAR_ROM / CVAR_USERINFO / CVAR_SERVERINFO / CVAR_ARCHIVE — who checks what

- **CVAR_ROM**: checked only in `Cvar_Set2`'s `!force` branch (§1.2 step 5)
  — code can still force-set a ROM cvar (`Cvar_Set`/internal calls all pass
  `force=qtrue`; only console-originated `Cvar_Command`/`Cvar_Set_f` pass
  `qfalse`). `Cvar_Restart_f` explicitly preserves ROM/INIT/NORESTART cvars
  (MP `cvar.cpp:772`, SP `cvar.cpp:767`) "so some inter-module communication
  will get broken (com_cl_running, etc)" if it didn't.
- **CVAR_USERINFO**: not read inside cvar.cpp itself — it's a *marker*
  consumed by `Cvar_InfoString(CVAR_USERINFO)` (§1.5) and by
  `cvar_modifiedFlags & CVAR_USERINFO` at the client tier (MP
  `client/cl_main.cpp:2250-2253`, SP `client/cl_main.cpp:811-813`) to decide
  when to push a `userinfo "..."` reliable command to the server.
- **CVAR_SERVERINFO**: same pattern, server side — `SV_Frame`'s "update
  infostrings if anything has been changed" block rebuilds `CS_SERVERINFO`
  from `Cvar_InfoString(CVAR_SERVERINFO)` and clears the bit (MP
  `server/sv_main.cpp:886-889`, SP `server/sv_main.cpp:530-533`).
- **CVAR_ARCHIVE**: read by `Cvar_WriteVariables` (writes `seta name
  "value"` lines to the config file for every archived cvar — MP
  `cvar.cpp:660-680`, SP `cvar.cpp:659-676`), gated by
  `cvar_modifiedFlags & CVAR_ARCHIVE` at the write call site (`Com_WriteConfiguration`-style path, MP `qcommon/common.cpp:1483-1486`, SP
  `qcommon/common.cpp:1166-1169`) so the config file is only rewritten when
  something archived actually changed. `client/cl_ui.cpp:754,758` (MP) sets
  `cvar_modifiedFlags |= CVAR_ARCHIVE` by hand (not via `Cvar_Set2`) purely
  to force a config write after the CD-key buffer is populated — a
  deliberate "flag it dirty without going through the cvar API" escape
  hatch.

### 1.4 `cvar_modifiedFlags` consumers (full map)

| Setter | Reader/clearer | Effect |
|---|---|---|
| `Cvar_Get` (MP `cvar.cpp:220`, SP `cvar.cpp:255`) — only on the USER_CREATED→real transition | — | signals a newly-"real" cvar's flags to the below |
| `Cvar_Set2` (MP `cvar.cpp:329`, SP `cvar.cpp:353`) — every call, unconditionally | — | coarse "cvar touched" signal, see §1.2 step 4 |
| `client/cl_ui.cpp:754,758` (MP, CD-key path) — manual `\|=` | — | force a config rewrite |
| — | `server/sv_main.cpp:886-889` (MP) / `:530-533` (SP) — `SV_Frame`, every server frame | rebuild `CS_SERVERINFO`/`CS_SYSTEMINFO` configstrings from `Cvar_InfoString`/`_Big`, clear bit |
| — | `server/sv_init.cpp:768,772` (MP) / `:451,454` (SP) — `SV_SpawnServer` | clear stale SERVERINFO/SYSTEMINFO bits at map (re)start so the first post-spawn `SV_Frame` doesn't redundantly rebroadcast |
| — | `client/cl_main.cpp:2250-2253` (MP) / `:811-813` (SP) — per-frame client update, skipped while `cl_paused` | push `userinfo "<info string>"` as a reliable command to the server, clear bit |
| — | `qcommon/common.cpp:1483-1486` (MP) / `:1166-1169` (SP) — config-write path | if `CVAR_ARCHIVE` bit set, rewrite the config file and clear it; otherwise skip the write entirely |

Net effect: `cvar_modifiedFlags` is a single global bitmask multiplexing
four independent "something changed, go re-publish" signals
(serverinfo/systeminfo/userinfo/archive-dirty), each read+cleared by a
different subsystem on its own cadence (per-server-frame, per-client-frame,
per-config-write). There is no per-cvar dirty tracking at this layer beyond
the per-cvar `modified`/`modificationCount` fields (which exist for the
`vmCvar_t` mirroring protocol, §1.6, not for this broadcast mechanism).

### 1.5 `Cvar_InfoString` construction

- MP `Cvar_InfoString` (`cvar.cpp:811-845`) walks `cvar_vars`, includes a
  cvar iff `!(flags & CVAR_INTERNAL) && (flags & bit)`, appending via
  `Info_SetValueForKey` into a `static char info[MAX_INFO_STRING]` (1024
  bytes, `q_shared.h:384`) — i.e. `CVAR_INTERNAL` cvars (passwords etc,
  `q_shared.h:1800`) are unconditionally excluded from *every* info-string
  bit, not just one. `Info_SetValueForKey`'s own truncation/escaping rules
  (not read here — out of scope, it's a q_shared string-table primitive)
  apply; `Cvar_InfoString` does no escaping of its own.
- MP additionally has `Cvar_InfoString_Big` (`cvar.cpp:854-869`), identical
  but into `static char info[BIG_INFO_STRING]` (8192 bytes) via
  `Info_SetValueForKey_Big` — used exclusively for `CS_SYSTEMINFO`
  (`sv_main.cpp:892`) since systeminfo can carry more key/value pairs than
  the 1024-byte limit tolerates.
- SP `Cvar_InfoString` (`cvar.cpp:804-816`) has **no `CVAR_INTERNAL` check**
  (SP's flag set has no `CVAR_INTERNAL` bit at all, §1.7) and **no `_Big`
  variant** — SP only ever builds the 1024-byte form, including for
  systeminfo-equivalent use. This is a real MP/SP capability gap, not just
  a naming difference.
- Both cvar files also define `Cvar_InfoStringBuffer` (bounded-copy wrapper
  around `Cvar_InfoString`, MP `cvar.cpp:878-880`, SP `cvar.cpp:823-825`) —
  the version exposed across the trap seam (`trap_Cvar_InfoStringBuffer`
  doesn't exist in the trap table, but `Cvar_InfoStringBuffer` is called
  directly by engine-side UI trap handlers, e.g. `UI_CVAR_INFOSTRINGBUFFER`
  at `codemp/client/cl_ui.cpp:899`).

### 1.6 `Cvar_Update` — the `vmCvar_t` mirroring protocol

This is the seam behavior. A VM module never touches `cvar_t` directly (it
can't — `cvar_t` lives in engine memory); it holds a `vmCvar_t` (4 plain
fields: `handle`, `modificationCount`, `value`, `integer`, `string[256]`,
`q_shared.h:1817-1823` MP / `q_shared.h:1327-1333` SP) that must be
explicitly refreshed.

**Registration** (`Cvar_Register`, MP `cvar.cpp:889-899`, SP
`cvar.cpp:834-844`, called from the module side via `trap_Cvar_Register`):
1. `Cvar_Get(varName, defaultValue, flags)` — creates or merges as in §1.1.
2. `vmCvar->handle = cv - cvar_indexes` — the handle is a **raw array index**
   into the engine's `cvar_indexes[MAX_CVARS]` table, not an opaque token.
   This is why `cvar_indexes` must never move/realloc individual slots
   (§1.7) and why `Cvar_Defrag` (§1.7) only ever moves the *string
   payloads*, never the `cvar_t` structs themselves.
3. `vmCvar->modificationCount = -1` — forced sentinel guaranteeing the
   very next `Cvar_Update` call sees a mismatch and actually copies (a
   fresh cvar's real `modificationCount` starts at `1`, MP `cvar.cpp:264`,
   so `-1` can never accidentally match).
4. `Cvar_Update(vmCvar)` is called immediately, so registration always
   leaves the `vmCvar_t` populated with current values — a module never
   sees a zeroed `vmCvar_t` after registering.

**Per-call refresh** (`Cvar_Update`, MP `cvar.cpp:909-941`, SP
`cvar.cpp:854-873`, called from the module side via `trap_Cvar_Update`,
typically once per frame per registered cvar — e.g. cgame's `CG_UpdateCvars`
pattern):
1. Bounds-check `(unsigned)vmCvar->handle >= cvar_numIndexes` →
   `Com_Error(ERR_DROP, ...)` — an out-of-range handle is fatal-to-the-frame,
   not silently ignored.
2. **The whole avoid-needless-copy mechanism**: `if (cv->modificationCount
   == vmCvar->modificationCount) return;` — a single integer compare, no
   string compare, no diffing. `modificationCount` is bumped exactly once
   per successful `Cvar_Set2` value change (§1.2 step 7) or once per
   latch-armed (step 5); it is never decremented or reset except by
   `Cvar_Restart_f` zeroing the whole `cvar_t` for a user-created cvar
   being discarded (MP `cvar.cpp:794`, which also zeros `modificationCount`
   itself, but that only applies to cvars that get *removed* entirely, not
   to normal ones — see the `!cv->string` guard next).
3. `if (!cv->string) return;` guards the case where `Cvar_Restart_f` has
   `Com_Memset`'d a user-created cvar's slot to all-zero (`cvar.cpp:794`)
   out from under a module that still holds a handle into that now-dead
   slot — the comment says it plainly: "variable might have been cleared
   by a cvar_restart." The module's stale `vmCvar_t` is left as-is (last
   known value) rather than updated to garbage.
4. Length guard: `strlen(cv->string)+1 > MAX_CVAR_VALUE_STRING` →
   `Com_Error(ERR_DROP, ...)` in MP (`cvar.cpp:927-931`) — SP has no
   equivalent guard (SP `cvar.cpp:854-873` copies unconditionally via
   `Q_strncpyz(vmCvar->string, cv->string, sizeof(vmCvar->string))`, which
   truncates silently instead of erroring). This is a real MP/SP divergence
   in overflow handling for this one path.
5. Copy `string`/`value`/`integer`, update `vmCvar->modificationCount = cv->modificationCount`.

**Engine-side handler wiring** (both directions of the seam, both engines):
- MP `game` module: `trap_Cvar_Register`/`trap_Cvar_Update` are syscalls
  (`codemp/game/g_syscalls.c`, traps #6/#7 in `docs/abi-traps.md`) dispatched
  by the engine's `SV_GameSystemCalls` switch — `case G_CVAR_REGISTER` /
  `case G_CVAR_UPDATE` at `codemp/server/sv_game.cpp:529,532`, each simply
  forwarding to `Cvar_Register`/`Cvar_Update`.
- MP `cgame`: `case CG_CVAR_REGISTER` / `case CG_CVAR_UPDATE` at
  `codemp/client/cl_cgame.cpp:445,448`.
- MP `ui`: `case UI_CVAR_REGISTER` / `case UI_CVAR_UPDATE` at
  `codemp/client/cl_ui.cpp:868,872`.
- SP `game` (`jagame`, a real DLL, `GetGameAPI`/`game_import_t` hard-export
  table — no syscall marshaling, confirmed at `code/server/sv_game.cpp:527-530`:
  `import.cvar = Cvar_Get; import.cvar_set = Cvar_Set;
  import.Cvar_VariableIntegerValue = Cvar_VariableIntegerValue;
  import.Cvar_VariableStringBuffer = Cvar_VariableStringBuffer;` — note SP's
  `game_import_t` doesn't even expose `Cvar_Register`/`Cvar_Update` to the
  game module; SP game only ever reads/sets by name, it never holds a
  `vmCvar_t` handle at all).
- SP `cgame`: still uses the syscall-trampoline shape (`cgi_Cvar_Register`
  → `syscall(CG_CVAR_REGISTER, ...)`, `code/cgame/cg_syscalls.cpp:58-64`),
  dispatched by `case CG_CVAR_REGISTER` / `case CG_CVAR_UPDATE` at
  `code/client/cl_cgame.cpp:445,448` — i.e. SP cgame preserves the
  `VM_Call`-shaped seam per DEC-07 even though it's statically linked.
- SP `ui`: calls `Cvar_Register`/`Cvar_Update` **directly**, no trap/syscall
  indirection at all (`code/ui/ui_main.cpp:2657,3607` — literally
  `Cvar_Register(cv->vmCvar, ...)`), confirming DEC-07's "no real ABI
  boundary to model" for SP ui specifically (SP cgame retains the shim, SP
  ui does not — a source-level asymmetry within "statically linked SP",
  worth flagging since DEC-07 treats them as one bucket).

### 1.7 Hash table, `MAX_CVARS`, `Cvar_Realloc`/`Cvar_Defrag` pool quirk

- **MP**: `MAX_CVARS = 1224` (`cvar.cpp:10`), fixed array `cvar_t
  cvar_indexes[MAX_CVARS]` — a bump allocator (`cvar_numIndexes++`, never
  decremented; `Cvar_Restart_f`'s per-cvar removal only unlinks from the
  `cvar_vars` list and zeroes the struct in place, it never reclaims the
  index slot, `cvar.cpp:778-796`). Overflow (`cvar_numIndexes >= MAX_CVARS`)
  is `Com_Error(ERR_FATAL, "MAX_CVARS")` — a hard crash, not a graceful
  reject (`cvar.cpp:256-258`). MP additionally has a real hash table:
  `static cvar_t *hashTable[256]` (`cvar.cpp:15`), chained via `hashNext`
  (only field SP's `cvar_t` lacks, `q_shared.h:1808-1811` MP vs
  `q_shared.h:1314-1317` SP), `generateHashValue` (`cvar.cpp:41-55`, a
  lowercase-weighted-by-position sum masked to 255) — `Cvar_FindVar` is
  O(1)-ish via hash chain (`cvar.cpp:83-96`).
- **SP**: `MAX_CVARS = 1024` (`cvar.cpp:10`), same bump-allocator array, but
  overflow check is `cvar_numIndexes == MAX_CVARS` (`cvar.cpp:291`, `==`
  not `>=`) — behaviorally equivalent to MP's `>=` since the counter only
  ever increments by exactly 1 per call and is checked before every
  increment, so there's no way to skip past the boundary; not a real bug,
  just a less-defensive spelling. SP has **no hash table at all** —
  `Cvar_FindVar` (`cvar.cpp:57-67`) is a linear O(n) walk of the
  `cvar_vars` singly-linked list for every lookup (every `Get`, every
  `Set2`, every console `Cvar_Command` dispatch).
- **The pool quirk** (`Cvar_Realloc`/`Cvar_Defrag`, byte-identical MP
  `cvar.cpp:965-1018` and SP `cvar.cpp:897-950`): this is **not** a
  freelist and **not** a general-purpose allocator — it's a one-shot
  defragmentation pass. `Cvar_Defrag` walks every live cvar, sums the
  total byte length of `name+string+resetString+latchedString` (each
  `strlen+1`), does **one** `Z_Malloc` for that total, then walks the list
  again copying each string into the new contiguous block via
  `Cvar_Realloc` (which `strcpy`s into `memPool + memPoolUsed`, advances
  the cursor, frees the old individual allocation via `Cvar_FreeString`,
  repoints the field at the new location) — then frees the *previous*
  whole pool (`lastMemPool`) and swaps `lastMemPool`/`memPoolSize` to the
  new one. `Cvar_FreeString` (`cvar.cpp:26-32` MP) is pointer-range-checked
  against `[lastMemPool, lastMemPool+memPoolSize)` — a string living inside
  the current pool is a no-op "free" (the whole pool gets freed atomically
  next defrag pass instead), while a string that was individually
  `Z_Malloc`'d (any string not yet defragged, i.e. anything changed since
  the last defrag) is freed normally. This means **every string pointer in
  every `cvar_t` may or may not point into the shared pool at any given
  time**, and the only way to tell is the pointer-range check — there is no
  tag bit. Callers: MP calls `Cvar_Defrag()` once, from
  `client/cl_main.cpp:715` (`CL_MapLoading`'s big level-transition free
  pass — "Collect all the small allocations done by the cvar system" is
  literally SP's comment at the analogous site); SP calls it from
  `server/sv_init.cpp:326`, inside `SV_SpawnServer`, with the comment
  "This frees, then allocates. Make it the last thing before other
  allocations begin!" — i.e. both engines treat it as a per-map-load
  housekeeping pass, not a runtime-triggered thing.

---

## 2. Cbuf semantics

### 2.1 Buffer struct and sizes

- MP: a bespoke `cmd_t { byte *data; int maxsize; int cursize; }`
  (`cmd_common.cpp:10-18`) over `byte cmd_text_buf[MAX_CMD_BUFFER=16384]`.
- SP: reuses the network message struct, `msg_t cmd_text` over
  `byte cmd_text_buf[MAX_CMD_BUFFER=8192]`, initialized via
  `MSG_Init(&cmd_text, cmd_text_buf, sizeof(cmd_text_buf))`
  (`code/qcommon/cmd.cpp:6-9,46-49`) — `MSG_Init` is trivial (`memset` the
  struct, set `data`/`maxsize`, `qcommon/msg.cpp:18-22`), so this is a
  naming/reuse choice, not a semantic difference; `cmd_text.data`/
  `cursize`/`maxsize` are read identically to MP's `cmd_t` fields
  thereafter.

### 2.2 `Cbuf_AddText` / `Cbuf_InsertText` — exact ordering and overflow

- `Cbuf_AddText` (MP `cmd_common.cpp:68-80`, SP `cmd.cpp:58-69`): appends at
  `cursize`, **no** trailing `\n` added (per the doc comment — a caller
  that wants the text to execute as a standalone line must supply its own
  separator). Overflow check `cursize + l >= maxsize` → `Com_Printf
  ("Cbuf_AddText: overflow\n")` and the entire call is a **no-op** (text is
  dropped in full, not truncated, not partially appended) — not
  `Com_Error`, execution continues normally next frame with whatever was
  already queued.
- `Cbuf_InsertText` (MP `cmd_common.cpp:91-113`, SP `cmd.cpp:80-102`):
  inserts *before* whatever is currently queued (i.e., "run this next"),
  and unlike `AddText` it **does** append a `\n`. Implementation:
  memmove the existing buffer contents forward by `len` bytes, copy the
  new text into the now-vacated head, write `\n` at `len-1`. Overflow
  check `len + cursize > maxsize` (note: `>`, not `>=`, one-off from
  `AddText`'s check — both are conservative enough not to matter in
  practice) → `Com_Printf("Cbuf_InsertText overflowed\n")`, no-op, original
  buffer left completely untouched (the memmove/copy happens only after
  the guard passes, so a rejected insert can't corrupt the existing queue).

### 2.3 `wait` command

`Cmd_Wait_f` (MP `cmd_common.cpp:32-38`, SP `cmd.cpp:24-30`) sets the
global `cmd_wait` counter (arg 1 if given, else `1`). `Cbuf_Execute`'s main
loop (MP `cmd_common.cpp:155-201`, SP `cmd.cpp:133-183`) checks `if
(cmd_wait) { cmd_wait--; break; }` **before** parsing the next line each
iteration — so `wait` doesn't pause execution *of the line containing it*
(the rest of that semicolon-chained line, if any, still runs — see the
`bind g "cmd use rocket ; +attack ; wait ; -attack ; cmd use blaster"`
example in the doc comment: `wait` only affects what comes *after* the
`;` following it, since each `;`-delimited segment is its own
`Cmd_ExecuteString` call and `cmd_wait` is checked at the top of the outer
`while(cmd_text.cursize)` loop, i.e. between segments); the remaining
buffer contents are left queued (`break`, not consumed) and resume next
call to `Cbuf_Execute` (once per frame in the normal client/server loop),
decrementing `cmd_wait` by exactly one per frame it stays nonzero.

### 2.4 MAX_CMD_BUFFER overflow — precisely

Neither `AddText` nor `InsertText` truncates: an over-budget call is
rejected **in full** (see §2.2) and the existing buffer contents are
unaffected — there is no partial-write path anywhere in `Cbuf_*`. The only
place text gets silently *shortened* rather than rejected is MP's
`Cbuf_Execute` line-length cap, which is a different mechanism (§3.2 quirk
below) operating on lines already inside the buffer, not on buffer
capacity.

### 2.5 `Cbuf_ExecuteText` modes

Identical switch in both engines (MP `cmd_common.cpp:121-141`, SP
`cmd.cpp:110-126`), used by `trap_SendConsoleCommand` (trap #17) and
`Cmd_Stuffcmd`-style paths:
- `EXEC_NOW`: MP special-cases a non-empty `text` to call
  `Cmd_ExecuteString(text)` directly (bypassing the buffer entirely,
  synchronous, "a VM should NEVER use this" per the enum's own doc comment
  at `q_shared.h:405-406` since it can run arbitrary commands including
  ones that unload the calling VM mid-call) and falls back to
  `Cbuf_Execute()` (drain the whole existing buffer now) if `text` is
  empty/NULL. **SP's `EXEC_NOW` has no such branch** — it unconditionally
  calls `Cmd_ExecuteString(text)` (`cmd.cpp:114-116`), so passing `NULL`/""
  as `text` with `EXEC_NOW` behaves differently: MP drains the buffer, SP
  tokenizes an empty/null string (which is a safe no-op per
  `Cmd_TokenizeString`'s own `!text_in` guard, so this is latent rather
  than crashing, but it is a real behavioral divergence from MP for that
  input).
- `EXEC_INSERT`: `Cbuf_InsertText(text)` — queued to run next, not
  synchronous.
- `EXEC_APPEND`: `Cbuf_AddText(text)` — queued to run after everything
  currently buffered (the "normal case" per the enum comment).
- Any other value: `Com_Error(ERR_FATAL, "Cbuf_ExecuteText: bad exec_when")`.

---

## 3. Cmd

### 3.1 Registration: MP heap linked-list vs SP fixed array

- **MP** (`cmd_pc.cpp:1-39`): `cmd_function_t` is a heap node
  (`S_Malloc`'d — "small malloc to avoid zone fragmentation" per the source
  comment) with `next`/`name`/`function`, prepended to a `static
  cmd_function_t *cmd_functions` list head — genuinely unbounded (limited
  only by available memory). Re-registering an existing name is a no-op
  (returns without adding a duplicate node); a `Com_Printf` warning fires
  only if `function != NULL`, i.e. re-registering the *same name* with a
  `NULL` function (a "completion-only" placeholder pattern) is explicitly
  silent — "allow completion-only commands to be silently doubled"
  (`cmd_pc.cpp:25-30`).
- **SP** (`code/qcommon/cmd.cpp:270-283,465-496`): `cmd_function_t` is a
  **fixed 32-byte-name struct in a static array**,
  `cmd_functions[CMD_MAX_NUM=256]`, zero-initialized (`= {0}`). Registration
  linearly scans all 256 slots: (a) if `cmd_name` already matches a
  populated slot, same no-op/silent-double-if-NULL-function behavior as
  MP; (b) otherwise remembers the *first* slot seen with an empty
  `name[0]=='\0'` as the insertion point, but keeps scanning to completion
  (so a duplicate later in the array is still caught). **Registering
  command #257** (all 256 slots full, no empty slot found): `add == NULL`
  after the scan → `Com_Printf("Cmd_AddCommand: Too many commands
  registered\n", cmd_name)` (note: format string takes no `%s` — `cmd_name`
  is passed but never consumes it, a latent varargs-mismatch bug in the
  original source, though on typical ABIs the extra arg is simply ignored)
  and the function **returns without registering** — a silent capacity
  failure, not a crash, not an error propagated to the caller (the caller
  has no return value to check). `Cmd_RemoveCommand` (`cmd.cpp:503-514`)
  just zeroes `name[0]` in place, freeing the slot for reuse by a later
  `AddCommand` scan.
- MP has no analogous fixed cap; `Cmd_RemoveCommand` (`cmd_pc.cpp:46-66`)
  unlinks and `Z_Free`s both the name string and the node.

### 3.2 `Cmd_TokenizeString` — quoting/comment rules and the scratch-buffer hazard

Same algorithm in both engines (MP `cmd_common.cpp:398-491`, SP
`code/qcommon/cmd.cpp:365-457`), differing only in buffer sizes:

- Whitespace (`<= ' '`) is skipped between tokens.
- `//` starts a line comment — **the rest of the input is discarded
  entirely** (`return`, not "skip to newline"), so `foo bar // baz` tokenizes
  to `["foo","bar"]` but so does `foo // bar\nbaz` — the `\n`-delimited
  "next command" text after a `//` comment inside one `Cbuf_Execute` line
  is not reachable this way (comments are a tokenizer-level construct
  operating on whatever single string `Cmd_TokenizeString` was given, which
  by the time it gets there from `Cbuf_Execute` is already one `;`/`\n`-
  delimited line, per §2 — a `//` cuts off the rest of *that* line, it
  does not span the whole command buffer).
- `/* ... */` block comments are skipped in place (advance past the closing
  `*/`) without terminating tokenization, *except* inside a quoted string
  (see below) — an unterminated `/*` with no matching `*/` before end of
  input also terminates tokenization entirely (`return`).
- **Quoted strings** (`"..."`): once inside a `"`, `/`,`//`,`/*` have no
  special meaning — the token runs until the next `"` or end of input; an
  **unterminated quote runs to the end of the input string** as a single
  token (no error). A quoted token can be empty (`""`) — it still counts
  as one argv slot.
- **Unquoted tokens**: run until whitespace, `"`, `//`, or `/*` — MP has an
  extra cast quirk here (`*(const unsigned char*)text > ' '`,
  `cmd_common.cpp:466`, marked `/*eurofix*/` in the source, presumably an
  old sign-extension fix for chars ≥ 0x80) that SP spells as a plain
  `(const unsigned char *)text` cast applied once up front instead
  (`cmd.cpp:366,433`) — both end up doing unsigned comparisons, so no
  observable behavioral difference, just different code shape.
- Token count cap: `cmd_argc == MAX_STRING_TOKENS` → `return` (comment:
  "this is usually something malicious") — **MP: 1024** tokens
  (`q_shared.h:381`), **SP: 256** tokens (`q_shared.h:207`); tokens beyond
  the cap are silently dropped (not an error, not a truncated last token —
  parsing just stops).
- **Scratch buffer**: `cmd_argv[MAX_STRING_TOKENS]` + `cmd_tokenized[...]`
  are **file-static, single-instance, non-reentrant** (MP
  `cmd_common.cpp:290-292`: `cmd_tokenized[BIG_INFO_STRING(8192) +
  MAX_STRING_TOKENS(1024)]` = 9216 bytes; SP `cmd.cpp:279-283`:
  `cmd_tokenized[MAX_STRING_CHARS(1024) + MAX_STRING_TOKENS(256)]` = 1280
  bytes). Every `cmd_argv[i]` is a raw pointer *into* `cmd_tokenized` — this
  is the reentrancy hazard already flagged at the census level (A2
  §1b) confirmed live here: any code that runs **while** the caller is
  still consuming a previous `Cmd_TokenizeString` result (e.g. a command
  handler invoked from inside `Cmd_ExecuteString` that itself calls
  `Cmd_TokenizeString` again — directly, or transitively through anything
  that ends up parsing a new string, such as `Cvar_Command`'s toggle path
  which only reads `Cmd_Argv`, not a hazard, versus a hypothetical handler
  that does `Cmd_TokenizeString(some_other_string)` before returning)
  invalidates every `cmd_argv[]` pointer the outer caller was still holding.
  `Cmd_Args`/`Cmd_ArgsFrom` (MP) each own **separate** static buffers
  (`cmd_args[MAX_STRING_CHARS]` / `cmd_args[BIG_INFO_STRING]`,
  `cmd_common.cpp:337,359`) — those two are self-contained snapshots, not
  aliases into `cmd_tokenized`, so calling them doesn't compound the
  hazard, but they are themselves each single-instance statics (a second
  concurrent call to `Cmd_Args` clobbers the string a caller is still
  holding a pointer to).

**MP-only quirk — `Cbuf_Execute`'s line-length cap silently splits an
oversized line.** `Cbuf_Execute`'s inner scan for the next `;`/`\n`
separator (`cmd_common.cpp:168-176`) is **not** bounded by
`MAX_CMD_LINE=1024` while searching — only *after* finding `i` (the
separator position, or `cursize` if none found) does MP clamp
`if (i >= MAX_CMD_LINE-1) i = MAX_CMD_LINE-1;` (`cmd_common.cpp:178-180`)
before both copying into the local `line[MAX_CMD_LINE]` buffer *and*
advancing the buffer cursor by that same clamped `i`. Net effect: a single
line longer than 1023 bytes with no earlier `;`/`\n` gets executed as two
(or more) separate `Cmd_ExecuteString` calls — the first 1023 bytes run as
one command, and the remainder of what was meant to be one line is left in
the buffer and picked up as a *new* "line" on the next loop iteration
(itself re-scanned for a separator from scratch). This can turn one
intended command into garbage if the split happens mid-token. **SP has no
such cap** — its local `line` buffer is sized `MAX_CMD_BUFFER` (8192,
`cmd.cpp:137`), i.e. large enough to hold the entire buffer contents in one
line, so SP never splits a line this way (SP's cap comes only from
`MAX_CMD_BUFFER` itself, at which point the *earlier* `Cbuf_AddText`/
`InsertText` overflow check would already have rejected the text before it
ever reached the buffer, §2.2).

### 3.3 `Cmd_ExecuteString` resolution order

Identical order, both engines (MP `cmd_pc.cpp:91-145`, SP
`code/qcommon/cmd.cpp:610-662`):

1. `Cmd_TokenizeString(text)`; if zero tokens, return (no-op on
   empty/whitespace-only input).
2. **Registered command table lookup** (`Q_stricmp` against `argv[0]`,
   case-insensitive): MP walks the linked list and — on a hit — splices the
   matched node to the front (`cmd_pc.cpp:104-108`, an MRU/self-organizing
   list for future lookups); SP swaps the matched slot with slot 0
   (`cmd.cpp:624-626`, same MRU intent, array version). If the matched
   command's `function` pointer is `NULL` ("let the cgame or game handle
   it" — the completion-only-registration pattern from §3.1), the loop
   `break`s out and falls through to the remaining checks below *as if*
   the command hadn't matched at all; otherwise the function is called and
   `Cmd_ExecuteString` returns immediately — **a real registered command
   always wins over cvar-as-command and forwarding, unconditionally**.
3. **Cvar-as-command**: `Cvar_Command()` — looks up `Cmd_Argv(0)` as a cvar
   name; if found, with zero extra args it prints `"name" is:"value"
   default:"reset"` (+ latched value if pending), with one extra arg it
   sets the cvar (via `Cvar_Set2(..., force=qfalse)`, so ROM/cheat/latch
   protections in §1.2 all apply — typing a protected cvar's name at the
   console goes through the same forceless path as `set`), with a leading
   `!` in the value it toggles (integer `!v->value`/`!v->integer`) instead
   of setting literally. Returns `qtrue` if a match was found (whether or
   not the value stuck), which short-circuits everything below.
4. **Client game commands**: `if (com_cl_running && com_cl_running->integer
   && CL_GameCommand())` — only tried if a client is actually running.
5. **Server game commands**: `if (com_sv_running && com_sv_running->integer
   && SV_GameCommand())`.
6. **UI commands**: `if (com_cl_running && com_cl_running->integer &&
   UI_GameCommand())` — gated on the *client* running (not a separate "UI
   running" flag), tried *after* server commands even though UI is a
   client-side concept — this ordering (cl → sv → ui) is identical in both
   engines and is not alphabetical or "client-owned modules first," so it's
   worth preserving exactly rather than reordering for tidiness.
7. **Fallback**: unconditionally forward to the server —
   `CL_ForwardCommandToServer(text)` (MP, takes the original text) /
   `CL_ForwardCommandToServer()` (SP, no argument — reads `Cmd_Args()`
   internally instead, per its declaration at `code/qcommon/qcommon.h:681`
   vs MP's `codemp/qcommon/qcommon.h:866`). This is what makes an unknown
   console input become a chat message when connected to a server (per the
   source comment, "this will usually result in a chat message") — there
   is **no final "unknown command" error path** in `Cmd_ExecuteString`
   itself; an unrecognized, non-cvar command that isn't claimed by
   cl/sv/ui game code simply becomes server-forwarded text, and it's
   `CL_ForwardCommandToServer`'s own logic (out of scope here) that decides
   what happens if there's no connection.

---

## 4. MP/SP behavioral diffs (summary table)

| Aspect | MP | SP |
|---|---|---|
| `MAX_CVARS` | 1224 | 1024 |
| Cvar storage | hash table (256 buckets, `hashNext` chain) + linked list | linked list only, O(n) `Cvar_FindVar` |
| `cvar_t` size | `name,string,resetString,latchedString,flags,modified,modificationCount,value,integer,next,hashNext` | same minus `hashNext` |
| `CVAR_INTERNAL` flag | exists (`0x800`) | does not exist |
| `CVAR_SAVEGAME` flag | does not exist | exists (value `256`, same bit position MP uses for `CVAR_TEMP`) |
| `CVAR_TEMP` flag | `0x100` (real bit) | `0` (a no-op — ORing it changes nothing) |
| `CVAR_PARENTAL` flag | exists | does not exist |
| `cvar_cheats` seed | `Cvar_Get("sv_cheats", "0", CVAR_ROM\|CVAR_SYSTEMINFO)` | `Cvar_Get("helpUsObi", "0", CVAR_SYSTEMINFO)` — different name, **no `CVAR_ROM`** (SP's cheat-gate cvar is not ROM-protected — a forced `Cvar_Set("helpUsObi", ...)` from code can flip it, whereas MP's `sv_cheats` cannot even be force-set outside `cvar.cpp` itself since ROM is checked in `Set2`'s `!force` branch only... actually ROM only blocks the *non-forced* path in both, so this is really about console-typeability: `sv_cheats` is ROM so `set sv_cheats 1` from console is rejected; `helpUsObi` is not ROM so it can be) |
| `Cvar_InfoString` | excludes `CVAR_INTERNAL` cvars | no such exclusion (flag doesn't exist) |
| `Cvar_InfoString_Big` | exists, used for `CS_SYSTEMINFO` | does not exist |
| `Cvar_Update` overflow guard | `Com_Error(ERR_DROP, ...)` if `cv->string` won't fit `MAX_CVAR_VALUE_STRING` | none — silent truncation via `Q_strncpyz` |
| `Cvar_List_f` cheat display | shows `C` flag regardless of `sv_cheats` | **hides** cheat-flagged cvars entirely from the listing when `!cvar_cheats->integer` (`code/qcommon/cvar.cpp:730-736`, decrements `i` so the count stays accurate) |
| `Cvar_CompleteVariable`/`Next` | not present in `cvar.cpp` (MP console completion lives elsewhere) | present (`cvar.cpp:137-214`), cheat-gated (skips `CVAR_CHEAT` cvars when cheats off) |
| `MAX_CMD_BUFFER` | 16384 | 8192 |
| `MAX_CMD_LINE` (per-command-line cap) | 1024 (`cmd_common.cpp:8`), enforced — causes the oversized-line split quirk (§3.2) | none (uses full 8192 buffer as line-copy size) |
| `MAX_STRING_TOKENS` | 1024 | 256 |
| Tokenizer scratch size | 9216 B (`BIG_INFO_STRING+MAX_STRING_TOKENS`) | 1280 B (`MAX_STRING_CHARS+MAX_STRING_TOKENS`) |
| Command registry | unbounded heap linked list | fixed `cmd_functions[256]` array, silent-reject on #257 |
| `Cmd_Args` trailing space | correct — no trailing space after the last arg (`i != cmd_argc-1` check) | **bug** — condition is `i != cmd_argc` which is always true inside the `1..cmd_argc` loop, so SP's `Cmd_Args()` always appends a trailing space after every argument including the last (`code/qcommon/cmd.cpp:326-339`) |
| `Cbuf_ExecuteText(EXEC_NOW, NULL/"")` | drains the whole buffer via `Cbuf_Execute()` | tokenizes the empty/null string (safe no-op, but different from draining) |
| `game_import_t`/module wiring for cvars | syscall-marshaled for game/cgame/ui alike | SP `game`: direct hard-export function pointers, no `vmCvar_t`/handle concept exposed to game code at all; SP `cgame`: syscall-trampoline shim (VM_Call-shaped) per DEC-07; SP `ui`: calls `Cvar_Register`/`Cvar_Update` directly, no shim |
| `Cvar_Defrag` trigger site | `CL_MapLoading` (client-side, level transition) | `SV_SpawnServer` (server-side, map (re)start) |

---

## 5. Trap surface (cvar/cmd-related)

Cross-referenced against `docs/abi-traps.md` (MP `g_syscalls.c`-derived
table; the same trap *names*, minus the `Nav_`/`Bot_`/`G2_` families, are
mirrored for `cgame`/`ui` under `CG_*`/`UI_*` enum constants, not separately
tabulated in abi-traps.md but structurally identical dispatch).

| Trap | Modules using it | Engine-side handler |
|---|---|---|
| `trap_Cvar_Register` (#6) | MP game, MP cgame, MP ui | `case G_CVAR_REGISTER` `codemp/server/sv_game.cpp:529`; `case CG_CVAR_REGISTER` `codemp/client/cl_cgame.cpp:445`; `case UI_CVAR_REGISTER` `codemp/client/cl_ui.cpp:868` — all forward to `Cvar_Register` |
| `trap_Cvar_Update` (#7) | MP game, MP cgame, MP ui | `case G_CVAR_UPDATE` `sv_game.cpp:532`; `case CG_CVAR_UPDATE` `cl_cgame.cpp:448`; `case UI_CVAR_UPDATE` `cl_ui.cpp:872` — forward to `Cvar_Update` |
| `trap_Cvar_Set` (#8) | MP game, MP cgame, MP ui | `case G_CVAR_SET` `sv_game.cpp:535`; `case CG_CVAR_SET` `cl_cgame.cpp:720`(approx, see grep below); `case UI_CVAR_SET` `cl_ui.cpp:876` |
| `trap_Cvar_VariableIntegerValue` (#9) | MP game (widely, e.g. `g_main.c`, `g_cmds.c`) | `case G_CVAR_VARIABLE_INTEGER_VALUE` `sv_game.cpp:538` |
| `trap_Cvar_VariableStringBuffer` (#10) | MP game, cgame (`CG_CVAR_VARIABLESTRINGBUFFER`), ui | `case G_CVAR_VARIABLE_STRING_BUFFER` `sv_game.cpp:540`; `case CG_CVAR_VARIABLESTRINGBUFFER` `cl_cgame.cpp:723`; `case UI_CVAR_VARIABLESTRINGBUFFER` `cl_ui.cpp:883` |
| `trap_Argc` (#11) | all three MP modules (any command handler) | dispatched to `Cmd_Argc()` directly in each syscall switch (no cvar/cmd subsystem indirection beyond the function call itself) |
| `trap_Argv` (#12) | all three MP modules | dispatched to `Cmd_ArgvBuffer()` |
| `trap_SendConsoleCommand` (#17) | MP game (`g_svcmds.c`, `g_main.c`) | `SV_GameSystemCalls` → `Cbuf_ExecuteText(exec_when, text)` (§2.5) |
| UI-only extras (no numbered MP trap-table row since `docs/abi-traps.md` is generated from `g_syscalls.c`, the *game*-module trap list; these are `UI_*`-namespaced enum values in `ui_syscalls.c`/`cl_ui.cpp`, same dispatch mechanism) | `UI_CVAR_VARIABLEVALUE`, `UI_CVAR_SETVALUE`, `UI_CVAR_RESET`, `UI_CVAR_CREATE`, `UI_CVAR_INFOSTRINGBUFFER` | `cl_ui.cpp:880,887,891,895,899` → `Cvar_VariableValue`/`Cvar_SetValue`/`Cvar_Reset`/`Cvar_Get` (`UI_CVAR_CREATE` is literally a `Cvar_Get` call, letting the MP UI module define a brand-new cvar without ever holding a `vmCvar_t`)/`Cvar_InfoStringBuffer` |
| `CG_CVAR_GETHIDDENVALUE` (cgame-only, no `UI`/`G_` analogue) | MP cgame | `cl_cgame.cpp:726` — reads a cvar's value bypassing the normal `CVAR_INTERNAL`-hiding path used by `Cvar_InfoString`, presumably for HUD/UI code that legitimately needs a password-class cvar's value without exposing it via info strings |
| SP equivalents | SP `game`: no traps — `game_import_t` hard-export pointers (`Cvar_Get`/`Cvar_Set`/`Cvar_VariableIntegerValue`/`Cvar_VariableStringBuffer` assigned directly at `code/server/sv_game.cpp:527-530`; no `Cvar_Register`/`Cvar_Update`/handle concept exposed to SP game code at all — SP game never uses `vmCvar_t`). SP `cgame`: syscall-trampoline (`cgi_Cvar_Register`/`cgi_Cvar_Update`/`cgi_Cvar_Set` → `syscall(CG_CVAR_*, ...)`, `code/cgame/cg_syscalls.cpp:58-66`), dispatched by `case CG_CVAR_REGISTER`/`CG_CVAR_UPDATE` at `code/client/cl_cgame.cpp:445,448`. SP `ui`: direct calls, no trap layer (`Cvar_Register`/`Cvar_Update` called literally, `code/ui/ui_main.cpp:2657,3607`; `trap_Cvar_Set`/`trap_Cvar_VariableValue` are declared at `code/ui/ui_local.h:191-192` but are thin same-binary function calls, not a marshaled boundary, since SP ui has no separate address space) |

For the underlying reason SP `game` gets a hard-export table while SP
`cgame`/`ui` get (partial) shim treatment, see `docs/dossiers/A2-state-ownership.md`
§4.2 and DEC-07 in `docs/decisions.md` — not re-derived here.

---

## 6. TU-harness candidates per DEC-09 layer 1

`docs/decisions.md` DEC-09 layer 1 is the `tools/gp2-oracle` pattern:
compile an **unmodified** oracle `.cpp` standalone against a small stub
header (providing only the handful of externs that TU calls but doesn't
define), golden-diff its output against fixtures, and hold the Rust port to
byte-for-byte parity — no C++ toolchain needed at `cargo test` time since
goldens are pre-committed (`tools/gp2-oracle/README.md`, `run.sh`).

External symbols required by each file (grepped directly from the `.cpp`,
i.e. what a stub header would need to provide beyond libc):

**MP `cvar.cpp`**: `Com_Error`, `Com_Printf`, `Com_DPrintf`, `Com_Filter`,
`Com_sprintf`, `Com_Memset`, `CopyString`, `Q_stricmp`, `Q_strncpyz`,
`Info_SetValueForKey`, `Info_SetValueForKey_Big`, `va`, `Z_Malloc`,
`Z_Free`, `FS_Printf`, and — only for `Cvar_Init`/`Cvar_Command` — `Cmd_AddCommand`/`Cmd_Argc`/`Cmd_Argv`. All of these are either
trivial pure-string/printf-style stubs (`Com_Printf`→`vprintf`,
`Com_Error`→`longjmp`/`abort` capturing the message, `CopyString`→
`strdup`-alike, `Q_stricmp`/`Q_strncpyz`→ASCII case-fold helpers already
proven out by the gp2-oracle stubs) or can be satisfied by linking the
*actual* `cmd_pc.cpp`/`cmd_common.cpp` sources alongside (no circular
dependency the other direction — `cmd_common.cpp` needs `Cvar_Command`/
`Cvar_VariableString` from `cvar.cpp`, so the natural harness compiles both
together rather than stubbing one out of the other).
**Feasibility: easy.** No filesystem, no network, no VM, no rendering —
pure in-memory data structure code. The 256-bucket hash table and the
`Cvar_Defrag` pool-quirk (§1.7) are exactly the kind of subtle,
easy-to-get-wrong-in-a-rewrite logic this harness pattern exists to pin.

**MP `cmd_common.cpp` + `cmd_pc.cpp`**: adds `Cvar_Command`,
`Cvar_VariableString` (from `cvar.cpp` — link real source, see above),
`CL_ForwardCommandToServer`, `CL_GameCommand`, `SV_GameCommand`,
`UI_GameCommand` (four deep cross-subsystem hooks reached only from the
tail of `Cmd_ExecuteString`, §3.3 steps 4-7), `FS_ReadFile`/`FS_FreeFile`/
`COM_DefaultExtension` (only exercised by `Cmd_Exec_f`), `S_Malloc`,
`Com_Memcpy`, and the same `Com_Printf`/`Com_Error`/`Q_str*`/`va` set as
cvar.cpp. **Feasibility: moderate.** The buffer/tokenizer/registration
core (what §2 and §3.1-3.2 actually need to pin — the overflow behaviors,
the oversized-line split quirk, the quoting/comment rules, the MRU
reordering) needs only trivial stubs and no real filesystem/client/server;
a harness can stub `CL_ForwardCommandToServer`/`CL_GameCommand`/
`SV_GameCommand`/`UI_GameCommand` to return `qfalse`/no-op (never actually
reached unless a fixture deliberately types an unregistered, non-cvar
command) and `FS_ReadFile` to return failure (so `exec` fixtures just print
"couldn't exec" rather than needing real files) — none of that weakens
coverage of the actual cmd/cbuf semantics under test, since those four
hooks are call-order boundaries, not internal logic this dossier's
scope needs to verify.

**SP `cvar.cpp`**: same external-symbol shape as MP minus the hash table
(`Cmd_AddCommand`/`Cmd_Argc`/`Cmd_Argv`, `Com_Error`/`Com_Printf`/
`Com_DPrintf`/`Com_Filter`/`Com_sprintf`, `CopyString`, `Q_stricmp`/
`Q_stricmpn`/`Q_strncpyz`, `Info_SetValueForKey` — no `_Big` variant needed
since SP doesn't have one, §1.5 — `va`, `Z_Malloc`/`Z_Free`, `FS_Printf`).
**Feasibility: easy** — strictly simpler than MP's (no hash function to
get subtly wrong, no `CVAR_INTERNAL` filtering path).

**SP `cmd.cpp`**: same shape as MP's cmd group, plus `MSG_Init`/
`MSG_WriteData` in place of MP's raw `Com_Memcpy`-on-a-`cmd_t` (§2.1) —
both are trivial one-liners in the real `msg.cpp`
(`memset`+field-assignment and `memcpy`-into-`MSG_GetSpace` respectively,
`code/qcommon/msg.cpp:18-22,72-74`) and can either be stubbed directly (2
functions, ~6 lines) or the harness can link the tiny relevant slice of the
real `msg.cpp` — either way no huffman/bitpacking dependency is dragged in
since `Cbuf_*` only ever calls the byte-oriented `MSG_WriteData`, never the
bit-level write functions. **Feasibility: easy-moderate** — one extra
trivial dependency over the MP cmd group, otherwise identical
characteristics (same four cross-subsystem stub hooks, same fixed-array
registration to pin, same tokenizer to pin).

Overall: **all four groups are TU-harness-feasible** in the gp2-oracle
style; the two `cvar.cpp` files are the easiest (self-contained data
structure code, zero cross-subsystem hooks needed even for full coverage)
and the two `cmd`/`cmd_pc` groups are moderate only because
`Cmd_ExecuteString`'s tail (§3.3 steps 4-7) reaches toward client/server/ui
— which a harness sidesteps with no-op stubs without losing coverage of
anything this dossier's scope (`Cbuf_*`, tokenization, registration,
resolution *order* through step 3) needs to pin.

---

## Design forks

1. **Latched-cvar pending-value representation.** Raven stores a latch as
   `Option<String>`-shaped (`latchedString` nullable pointer) directly on
   the live cvar record, applied either by the next `Cvar_Get` (module
   re-registration) or a forced `Set` (§1.1, §1.2 step 6). A Rust port
   could keep the same shape (`pending: Option<String>` field on the cvar
   struct) or model it as a small explicit state machine
   (`CvarValue::Live(String) | CvarValue::Latched { live: String, pending:
   String }`) that makes "has a pending latch" a type-level fact instead of
   an `Option` check scattered through `Set2`/`Get`/`Restart_f`/`List_f`/
   `WriteVariables` (all five touch `latchedString` today). Both are
   faithful; this is a genuine idiom choice with no behavior implication
   either way as long as the same five call sites are covered.

2. **`cvar_modifiedFlags` as bitmask vs event/observer pattern.** Raven's
   mechanism (§1.4) is a single global `int` bitmask, set by two producers
   (`Get`'s USER_CREATED transition, `Set2` unconditionally) and cleared by
   three independent consumers on three different cadences (per-server-
   frame, per-client-frame, per-config-write). A Rust port threading state
   explicitly (porting-rules §B4) could either keep the literal bitmask
   (simplest, most faithful, but means three unrelated subsystems share
   mutable access to one flag field spread across whatever owns qcommon
   state) or replace it with three independent dirty-bits (one per
   consumer) or a small pub/sub signal each `Set2` call emits that
   consumers subscribe to. The bitmask is trivially faithful and cheap;
   the fork is whether "genuinely three independent consumers" is reason
   enough to *not* share one field, given porting-rules §A2 says port
   faithfully first and refactor after green — this fork is really "do we
   ever revisit this," not "what do we do at first port."

3. **Tokenizer scratch-buffer ownership model.** Raven's `cmd_argv`/
   `cmd_tokenized` are file-static, single-instance, and the confirmed-live
   reentrancy hazard in §3.2 means Raven's own behavior *depends on*
   non-reentrant use in practice (nothing in the shipped codebase appears
   to violate it, but nothing enforces it either). A Rust port could: (a)
   preserve the single shared buffer faithfully (matching Raven's
   accidental-safety-through-convention, simplest, but reintroduces the
   same footgun in unsafe-adjacent form, or requires a `RefCell`/similar to
   make the aliasing rules enforced-at-runtime rather than silent); (b)
   make tokenization return an owned `Vec<String>` (or a small SSO-string
   vec) per call, eliminating the hazard entirely by construction — this
   is a case where porting-rules §A1's "behavioral parity at the ABI seam"
   doesn't actually constrain the *internal* representation (nothing
   observable depends on the scratch buffer's address or lifetime, only on
   `Cmd_Argc()`/`Cmd_Argv(n)`'s return values during the window a caller
   uses them), so (b) is available without giving up parity — the open
   question is just whether the project wants the perf/alloc cost of
   per-call ownership versus a pooled/reused buffer with Rust-enforced
   (not merely conventional) non-aliasing.

4. **Info-string building approach.** Raven's `Cvar_InfoString`/`_Big`
   rebuild the *entire* string from scratch by walking all cvars every
   time they're called (§1.5) — cheap enough at Raven's cvar counts
   (hundreds, checked at most once per server/client frame when
   `cvar_modifiedFlags` indicates a change) that no incremental/cached
   approach exists in the oracle. A Rust port must decide whether to
   preserve this "rebuild on demand from the live list" approach (matches
   Raven exactly, and is the actual mechanism `cvar_modifiedFlags`-gating
   already amortizes — see §1.4) or maintain an incrementally-updated
   info-string cache per flag-bit (invalidated on `Set2`/`Get`), which
   would be observably different only in extremely contrived scenarios
   (e.g. reading `Cvar_InfoString` from two threads mid-update — not a
   concern here since Raven is single-threaded at this seam) — likely a
   non-fork in practice (rebuild-on-demand is simpler *and* faithful *and*
   not a measured bottleneck anywhere in scope), but flagged since "cache
   vs rebuild" is a natural refactor temptation for whoever ports this an
   later, and it's worth deciding once rather than per-PR.
