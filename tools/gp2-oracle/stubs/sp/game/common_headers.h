// Stub common_headers.h for compiling the UNMODIFIED SP genericparser2.cpp
// standalone (with -D_JK2EXE so trap_Z_Malloc maps to Z_Malloc). strcmpi is
// the MSVC spelling of strcasecmp.
#pragma once
#include <stdlib.h>
#include <string.h>
#include <strings.h>

#define strcmpi strcasecmp
#define stricmp strcasecmp

typedef enum { qfalse = 0, qtrue } qboolean;

#define TAG_TEXTPOOL 0
#define TAG_GP2 0

static inline void *Z_Malloc(int size, int tag, qboolean zero)
{
	(void)tag;
	return zero ? calloc(1, size) : malloc(size);
}

static inline void Z_Free(void *ptr)
{
	free(ptr);
}

// The real common_headers.h is SP's PCH and pulls the GP2 header in itself;
// genericparser2.cpp never includes its own header directly.
#include "genericparser2.h"
