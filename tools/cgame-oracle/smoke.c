/* cgame-oracle smoke harness - proves the built dylib loads and answers.
 *
 * Mirrors crates/cgame/tests/abi_smoke.rs (and referee-oracle's phase-1
 * oracle_smoke): dlopen the module, dlsym dllEntry + vmMain, install a stub
 * syscall through dllEntry, then call vmMain with a command word no export
 * carries. Raven's vmMain default arm (cg_main.c:354-358) routes
 * `CG_Error("vmMain: unknown command %i")` back out as a CG_ERROR (=1) syscall
 * and then returns -1 - so one drive exercises dlopen, the dllEntry handshake,
 * the outbound syscall path, and the dispatch fall-through.
 *
 * The real engine longjmps out of CG_ERROR; our stub just records it and
 * returns, which lets vmMain run on to its `return -1`.
 */
#include <dlfcn.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

/* cgame_public.h: CG_PRINT = 0, CG_ERROR next. */
#define CG_ERROR 1
/* A command word no MpCgameExport variant carries - routes to the default arm. */
#define UNKNOWN_COMMAND 0x7fff

static int saw_cg_error = 0;

/* The mock engine's systemCall. cgame's trap_Error does `syscall(CG_ERROR, fmt)`;
 * CG_Error formats first, so the fmt arg is the finished "unknown command 32767"
 * text. varargs carry the pointer at natural width on LP64. */
static intptr_t stub_syscall(intptr_t cmd, ...) {
	if (cmd == CG_ERROR) {
		va_list ap;
		va_start(ap, cmd);
		const char *msg = va_arg(ap, const char *);
		va_end(ap);
		fprintf(stderr, "[cgame-smoke] CG_ERROR arrived: %s\n", msg ? msg : "(null)");
		if (!msg || !strstr(msg, "unknown command")) {
			fprintf(stderr, "error: CG_ERROR did not carry the default-arm message\n");
			return 0;
		}
		saw_cg_error = 1;
		return 0; /* engine longjmps here; returning exercises the fall-through */
	}
	fprintf(stderr, "error: unexpected syscall %ld during unknown-command drive\n", (long)cmd);
	return 0;
}

typedef void (*dllEntry_t)(intptr_t (*)(intptr_t, ...));
typedef intptr_t (*vmMain_t)(int, intptr_t, intptr_t, intptr_t, intptr_t,
                             intptr_t, intptr_t, intptr_t, intptr_t, intptr_t,
                             intptr_t, intptr_t, intptr_t);

int main(int argc, char **argv) {
	if (argc < 2) {
		fprintf(stderr, "usage: %s <path-to-dylib>\n", argv[0]);
		return 2;
	}
	void *h = dlopen(argv[1], RTLD_NOW | RTLD_LOCAL);
	if (!h) {
		fprintf(stderr, "error: dlopen failed: %s\n", dlerror());
		return 1;
	}
	dllEntry_t dllEntry = (dllEntry_t)dlsym(h, "dllEntry");
	vmMain_t vmMain = (vmMain_t)dlsym(h, "vmMain");
	if (!dllEntry || !vmMain) {
		fprintf(stderr, "error: dlsym failed (dllEntry=%p vmMain=%p)\n",
		        (void *)dllEntry, (void *)vmMain);
		return 1;
	}

	dllEntry(stub_syscall);
	intptr_t r = vmMain(UNKNOWN_COMMAND, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

	if (!saw_cg_error) {
		fprintf(stderr, "error: vmMain did not route CG_ERROR for the unknown command\n");
		return 1;
	}
	if (r != -1) {
		fprintf(stderr, "error: vmMain default arm returned %ld, expected -1\n", (long)r);
		return 1;
	}
	fprintf(stderr, "[cgame-smoke] OK - unknown command -> CG_ERROR -> vmMain returned -1\n");
	return 0;
}
