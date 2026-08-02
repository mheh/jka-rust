/*
 * syscall.c - the C-variadic half of the standalone probe module.
 *
 * The engine hands a loadable module a variadic syscall pointer
 * (`intptr_t (*)(intptr_t, ...)`). Stable Rust can neither define nor correctly
 * call a C-variadic function on Apple arm64, where va-args land on the stack,
 * so the forward lives here. This is the same reason
 * `tools/cgame-referee/shim/src/trampoline.c` and qcommon's
 * `game_syscall_trampoline.c` exist.
 *
 * The probe never receives a call THROUGH this file. It only sends, so there is
 * no trampoline here, just the outbound forward.
 */
#include <stdint.h>

static intptr_t (*g_engine_syscall)(intptr_t, ...) = 0;

void probe_set_engine_syscall(void *fn) {
    g_engine_syscall = (intptr_t (*)(intptr_t, ...))fn;
}

/* Forwards one 16-word frame as a variadic call. Extra words past what the trap
 * reads are ignored by the variadic callee, the way vm.cpp:363-377 forwards. */
intptr_t probe_engine_call(const intptr_t *args) {
    if (!g_engine_syscall)
        return 0;
    return g_engine_syscall(args[0], args[1], args[2], args[3], args[4],
                            args[5], args[6], args[7], args[8], args[9],
                            args[10], args[11], args[12], args[13], args[14],
                            args[15]);
}
