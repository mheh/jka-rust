/*
 * C-variadic half of the botlib import `Print` callback.
 *
 * Stable Rust cannot define a C-variadic fn, so the `botlib_import_t.Print`
 * slot (`void (*)(int type, char *fmt, ...)`, botlib.h:159) is filled by this
 * shim — the botlib-import twin of `game_syscall_trampoline.c`. It reproduces
 * Raven `BotImport_Print`'s formatting (`vsprintf(str, fmt, ap)` into
 * `char str[2048]`, sv_bot.cpp:267-274), then forwards the finished string to
 * the Rust receiver `bot_import_print_forward`, which owns the `PRT_*` switch
 * and `Com_Printf`/`Com_Error`.
 *
 * Divergence (porting-rules §19): `vsnprintf` bounds the write to the buffer
 * where Raven's `vsprintf` is unbounded.
 */
#include <stdarg.h>
#include <stdio.h>

extern void bot_import_print_forward(int type, const char *str);

void bot_import_print_trampoline(int type, char *fmt, ...) {
	char str[2048];
	va_list ap;

	va_start(ap, fmt);
	vsnprintf(str, sizeof(str), fmt, ap);
	va_end(ap);

	bot_import_print_forward(type, str);
}
