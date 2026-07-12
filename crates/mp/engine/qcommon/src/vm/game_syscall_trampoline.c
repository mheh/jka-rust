/*
 * game_syscall_trampoline.c — the C-variadic shim half of the SEAM-D11 inbound
 * raw syscall trampoline (skeleton-findings resolution 1, 2026-07-03).
 *
 * Stable Rust cannot DEFINE a C-variadic function (`c_variadic` is
 * nightly-only), and a fixed-arity workaround breaks for foreign variadic
 * callers on arm64 macOS (stack-passed va-args) — so the variadic definition
 * lives here, built by this crate's `cc` build script. It unpacks the va_list
 * into a flat intptr_t array exactly the way the oracle's own VM_DllSyscall
 * does — `int args[16]; args[0] = arg;` then 15 `va_arg` words ("For speed, we
 * just grab 15 arguments ... the extra is thrown away") — and forwards to the
 * Rust `extern "C-unwind"` half (`game_syscall_trampoline_words`,
 * src/vm/trampoline.rs), our `currentVM->systemCall( args )` equivalent.
 *
 * Source: oracle/codemp/qcommon/vm.cpp:363-377 (VM_DllSyscall; array
 * unpack `:366-375`, forward `:377`; the 15-arg rationale `:358-360`).
 */
#include <stdarg.h>
#include <stdint.h>

/* The Rust half (src/vm/trampoline.rs): reads the slot's injected EngineSlot
 * and dispatches. `extern "C-unwind"` on the Rust side so a Com_Error panic can
 * unwind back through this shim's frame (SEAM-D12). */
extern intptr_t game_syscall_trampoline_words(const intptr_t *args);

intptr_t game_syscall_trampoline(intptr_t arg, ...) {
    /* Mirrors vm.cpp:366-375: 16 words total — the command word + 15 grabbed
     * arguments, extras thrown away by the callee. */
    intptr_t args[16];
    va_list ap;
    int i;

    args[0] = arg;

    va_start(ap, arg);
    for (i = 1; i < (int)(sizeof(args) / sizeof(args[0])); i++)
        args[i] = va_arg(ap, intptr_t);
    va_end(ap);

    return game_syscall_trampoline_words(args);
}
