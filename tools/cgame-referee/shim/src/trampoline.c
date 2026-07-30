/*
 * trampoline.c - the C-variadic half of the recorder shim.
 *
 * The engine hands the module a variadic syscall pointer
 * (`intptr_t (*)(intptr_t, ...)`). Stable Rust can neither DEFINE nor correctly
 * CALL a C-variadic function on Apple arm64 (va-args land on the stack), so both
 * the trampoline the real module calls AND the forward to the real engine live
 * here in C - the same reason qcommon's game_syscall_trampoline.c exists
 * (crates/mp/engine/qcommon/src/vm/game_syscall_trampoline.c).
 *
 * Flow: real cgame module -> shim_syscall_trampoline (this file) captures 16
 * words the way oracle VM_DllSyscall does (vm.cpp:363-377), calls the Rust
 * loggers, and forwards to the real engine syscall we stored at dllEntry.
 *
 * The test engine stub at the bottom is only referenced by tests/interpose.rs;
 * it is harmless dead weight in the real cdylib.
 */
#include <stdarg.h>
#include <stdint.h>
#include <string.h>

/* Rust half (src/lib.rs). `args` is the flat 16-word frame. */
extern void rust_log_syscall_enter(const intptr_t *args);
extern void rust_log_syscall_exit(const intptr_t *args, intptr_t ret);

/* the real engine's variadic syscall, stored at dllEntry. */
static intptr_t (*g_engine_syscall)(intptr_t, ...) = 0;

void shim_set_engine_syscall(void *fn) {
    g_engine_syscall = (intptr_t (*)(intptr_t, ...))fn;
}

/* what we hand the real module's dllEntry in place of the engine pointer. */
intptr_t shim_syscall_trampoline(intptr_t arg, ...) {
    /* Mirrors vm.cpp:366-375: command word + 15 grabbed args, extras thrown
     * away by the callee. */
    intptr_t args[16];
    va_list ap;
    int i;
    intptr_t ret;

    args[0] = arg;
    va_start(ap, arg);
    for (i = 1; i < 16; i++)
        args[i] = va_arg(ap, intptr_t);
    va_end(ap);

    rust_log_syscall_enter(args);

    /* forward AS variadic (C emits the correct arm64 va-arg call ABI). extra
     * words past what the trap reads are ignored by the variadic callee. */
    ret = g_engine_syscall(args[0], args[1], args[2], args[3], args[4], args[5],
                           args[6], args[7], args[8], args[9], args[10], args[11],
                           args[12], args[13], args[14], args[15]);

    rust_log_syscall_exit(args, ret);
    return ret;
}

/* Rust needs the address of the variadic trampoline to pass to the real
 * module's dllEntry; it cannot name a variadic fn directly, so hand it back. */
void *shim_get_trampoline(void) {
    return (void *)shim_syscall_trampoline;
}

/* ---- test-only engine stub (tests/interpose.rs) ------------------------- */
/* CG_ERROR = 1 (cgameImport_t). The oracle module's default vmMain arm routes
 * CG_Error -> syscall(CG_ERROR, msg); this stub records it and returns (the
 * real engine longjmps, letting vmMain run on to `return -1`). */

static int g_test_saw_cg_error = 0;
static char g_test_last_msg[1024];

static intptr_t test_engine_syscall(intptr_t cmd, ...) {
    if (cmd == 1 /* CG_ERROR */) {
        va_list ap;
        const char *msg;
        va_start(ap, cmd);
        msg = (const char *)va_arg(ap, intptr_t);
        va_end(ap);
        g_test_saw_cg_error = 1;
        if (msg) {
            strncpy(g_test_last_msg, msg, sizeof(g_test_last_msg) - 1);
            g_test_last_msg[sizeof(g_test_last_msg) - 1] = 0;
        }
    }
    return 0;
}

void *shim_test_engine_syscall_ptr(void) { return (void *)test_engine_syscall; }
int shim_test_saw_cg_error(void) { return g_test_saw_cg_error; }
const char *shim_test_last_msg(void) { return g_test_last_msg; }
