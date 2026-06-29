# ABI Port Workflow

This workflow ports one boundary file at a time. Each worker owns exactly one
Rust file and must either validate the existing args/output types or replace the
stubbed `()` payloads with ABI-shaped types.

## Goal

For every enum-backed syscall or `vmMain` call under `src/boundary`, record the
real transport contract:

- the Rust `Args` type
- the Rust `Output` type
- the encode/decode transport, when the file owns one
- Raven comments that explain the call, where available
- Raven source locations for the enum value, args, output, and transport side

Function-table ABIs remain out of scope for this workflow until we explicitly
start the SP game/function-table pass.

## Assignment Unit

Assign exactly one row from `docs/abi-port-manifest.tsv` to exactly one worker.
The worker may read any Raven source file, but may only edit the assigned Rust
file unless the row is an enum file and the edit is strictly documentation for
that enum file.

Workers are not alone in the codebase. They must not revert unrelated edits, and
they must adapt to nearby changes already made by other workers.

## Source Hierarchy

Use the enum declaration for the ABI number and Raven comments, but do not stop
there. Args and output usually come from the wrapper, switch arm, or engine call
site.

For syscalls, prefer sources in this order:

1. Module trap wrapper implementation, such as `*_syscalls.c`.
2. Module trap wrapper declarations, such as `*_local.h`.
3. Engine syscall switch, such as `sv_game.cpp`, `cl_cgame.cpp`, `cl_ui.cpp`, or
   the SP `vmachine.cpp` path.
4. Enum header comments, such as `g_public.h`, `cg_public.h`, or `ui_public.h`.
5. Call sites only when wrapper/switch evidence is missing or ambiguous.

For `vmMain` calls, prefer sources in this order:

1. Module `vmMain` switch arm, such as `g_main.c`, `cg_main.c`, or `ui_main.c`.
2. Engine `VM_Call(...)` call sites.
3. Shared structs used through `gSharedBuffer` or equivalent shared buffers.
4. Enum header comments.

## Source Map

| Surface | Enum source | Module-side args/output | Engine-side args/output |
| --- | --- | --- | --- |
| MP game imports | `oracle/oracle/codemp/game/g_public.h` | `oracle/oracle/codemp/game/g_syscalls.c`, `oracle/oracle/codemp/game/g_local.h` | `oracle/oracle/codemp/server/sv_game.cpp` |
| MP game exports | `oracle/oracle/codemp/game/g_public.h` | `oracle/oracle/codemp/game/g_main.c` | `oracle/oracle/codemp/server/*.cpp`, `oracle/oracle/codemp/icarus/*.cpp`, `oracle/oracle/codemp/qcommon/RoffSystem.cpp` |
| MP cgame imports | `oracle/oracle/codemp/cgame/cg_public.h` | `oracle/oracle/codemp/cgame/cg_syscalls.c`, `oracle/oracle/codemp/cgame/cg_local.h` | `oracle/oracle/codemp/client/cl_cgame.cpp` |
| MP cgame exports | `oracle/oracle/codemp/cgame/cg_public.h` | `oracle/oracle/codemp/cgame/cg_main.c` | `oracle/oracle/codemp/client/*.cpp`, `oracle/oracle/codemp/qcommon/RoffSystem.cpp`, Ghoul2 callers |
| MP UI imports | `oracle/oracle/codemp/ui/ui_public.h` | `oracle/oracle/codemp/ui/ui_syscalls.c`, `oracle/oracle/codemp/ui/ui_local.h`, `oracle/oracle/codemp/ui/ui_shared.h` | `oracle/oracle/codemp/client/cl_ui.cpp` |
| MP UI exports | `oracle/oracle/codemp/ui/ui_public.h` | `oracle/oracle/codemp/ui/ui_main.c` | `oracle/oracle/codemp/client/*.cpp`, `oracle/oracle/codemp/client/cl_scrn.cpp` |
| SP cgame imports | `oracle/oracle/code/cgame/cg_public.h` | `oracle/oracle/code/cgame/cg_syscalls.c`, `oracle/oracle/code/cgame/cg_local.h` | `oracle/oracle/code/client/cl_cgame.cpp`, `oracle/oracle/code/client/vmachine.cpp` |
| SP cgame exports | `oracle/oracle/code/client/vmachine.h` | `oracle/oracle/code/cgame/cg_main.cpp` | `oracle/oracle/code/client/*.cpp`, `oracle/oracle/code/server/*.cpp`, `oracle/oracle/code/ghoul2/*.cpp` |
| SP UI imports | `oracle/oracle/code/ui/ui_public.h` | enum-only pass for now; do not port function-table ABI here | `oracle/oracle/code/client/cl_ui.cpp` |
| SP UI exports | none for enum ABI | no enum-backed `uiExport_t`; function-table ABI is out of scope | no enum-backed `VM_Call` surface |

## Worker Prompt

Use this prompt when delegating a manifest row or a small non-overlapping group
of rows. Do not summarize a row down to only the Rust file name. Paste the full
generated assignment/evidence lines for the worker's scope exactly as generated,
including:

- Rust target file and line number
- call name
- enum/comment source file and line number
- args source file and line number
- output source file and line number
- transport/switch source file and line number
- any notes already present in the generated output

If a generated line has no proven args or output source yet, include that
absence in the pasted scope so the worker can either prove it or keep the TODO.

```text
You own only the ABI boundary rows pasted below:
<full generated manifest/evidence lines for this worker scope>

Task:
Validate or create the Args and Output types for each pasted call. Preserve the
ABI integer enum. Keep or add Raven comments where useful. Add source references
for:
- enum value source
- args source
- output source
- transport/switch source

Use the pasted lines as the initial scope of truth, then verify them against the
Raven source hierarchy in docs/abi-port-workflow.md. If the source evidence is
ambiguous, leave the Rust payload as `()` with `//TODO: Port args` or
`//TODO: Port output`, and add a comment naming the ambiguous Raven locations.

Do not edit files outside the pasted scope. Do not revert unrelated changes. Run
formatting on assigned files if needed. Report:
- files changed
- args evidence
- output evidence
- remaining TODOs
```

## Acceptance Checklist

For each assigned file:

- [ ] The enum discriminant remains unchanged.
- [ ] `Args` matches the actual transport inputs, not just the header comment.
- [ ] `Output` matches the actual return or out-pointer semantics.
- [ ] Pointer, shared-buffer, `PASSFLOAT`, `VMA`, and string-buffer behavior is
      documented where used.
- [ ] Raven comments are preserved where they clarify behavior.
- [ ] Args and output each have source file and line references.
- [ ] Existing typed MP game calls are validated against Raven, not trusted
      blindly.
- [ ] Stubbed calls keep `//TODO: Port args` or `//TODO: Port output` until both
      sides of the ABI are proven.
- [ ] `cargo build` still passes after integration.

## Commit Cadence

Commit per file or per tiny non-overlapping batch. A good commit message format:

```text
Port ABI payload for <SURFACE> <CALL_NAME>
```

For validation-only commits:

```text
Validate ABI payload for <SURFACE> <CALL_NAME>
```
