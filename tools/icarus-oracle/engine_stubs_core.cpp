// Core engine-service stubs for the BlockStream + Q3_Registers golden dumpers.
// These two units touch almost no engine surface: the Zone allocator (behind
// class operator-new and ICARUS_Malloc), Com_Printf/Com_Error, and the
// developer-gated Q3_DebugPrint. Console/debug output is routed to stderr so it
// never contaminates the golden stdout stream (and mirrors the MP-dedicated
// reality that com_developer defaults to 0, suppressing Q3_DebugPrint entirely).
#include "../qcommon/exe_headers.h"

#include <cstdio>
#include <cstdlib>
#include <cstdarg>

// Zone allocator. Raven's TAG_ICARUS class allocs are always value-initialised
// (icarus.md § State ownership, ruling 20); calloc reproduces the zeroed
// operator-new (Z_Malloc(...,qtrue)) exactly, and is a safe superset for the
// non-zeroed ICARUS_Malloc path (member bytes are always overwritten by memcpy).
void *Z_Malloc(int iSize, memtag_t, qboolean, int) { return calloc(1, iSize ? iSize : 1); }
void  Z_Free(void *ptr) { free(ptr); }

void QDECL Com_Printf(const char *fmt, ...)
{
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}

void QDECL Com_Error(int code, const char *fmt, ...)
{
	va_list ap; va_start(ap, fmt);
	fprintf(stderr, "Com_Error(%d): ", code); vfprintf(stderr, fmt, ap);
	va_end(ap);
	exit(3);
}

// Developer-gated verbose/warning/error log (Q3_Interface.cpp:638-643). In the
// dedicated build com_developer==0 gates it off; route to stderr regardless.
void Q3_DebugPrint(int, const char *fmt, ...)
{
	va_list ap; va_start(ap, fmt); vfprintf(stderr, fmt, ap); va_end(ap);
}
