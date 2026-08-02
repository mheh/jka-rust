// cin-oracle Win32 shim, force-included ahead of every translation unit.
//
// `cl_cin.cpp` pulls in `client.h` and `snd_local.h`, which call a few MSVC and
// Win32 names that macOS does not have. The harness supplies them so the oracle
// source compiles unedited.
#ifndef CIN_ORACLE_WIN_SHIM_H
#define CIN_ORACLE_WIN_SHIM_H

#include <ctype.h>
#include <stddef.h>
#include <strings.h>

// MSVC spells the case-insensitive compare with an `i`. The ASCII behaviour
// matches, and every name the cinematic code compares is ASCII.
#define strnicmp strncasecmp

typedef const char *LPCSTR;
typedef char *LPSTR;

// The harness clock. host.cpp holds it at a fixed value, so every dump is
// reproducible.
extern "C" unsigned int timeGetTime(void);

// MSVC lowercases in place and returns the same buffer.
static inline char *strlwr(char *s)
{
	for (char *p = s; *p; p++) {
		*p = (char)tolower((unsigned char)*p);
	}
	return s;
}

static inline void OutputDebugString(const char *) {}

// The MSVC `min` macro over two longs. A narrow overload keeps <algorithm>
// intact, which a macro would break.
static inline long min(long a, long b) { return a < b ? a : b; }

#endif // CIN_ORACLE_WIN_SHIM_H
