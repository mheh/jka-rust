// fx-oracle Win32 shim, force-included ahead of every translation unit.
//
// Raven built the FX code with MSVC. A few MSVC and Win32 names have no macOS
// equivalent, so the harness supplies them and the oracle source compiles
// unedited.
#ifndef FX_ORACLE_WIN_SHIM_H
#define FX_ORACLE_WIN_SHIM_H

#include <ctype.h>
#include <stddef.h>
#include <strings.h>

// MSVC spells the case-insensitive compare with an `i`. Every name the FX code
// compares is ASCII, so the behaviour matches.
#define strnicmp strncasecmp

typedef const char *LPCSTR;
typedef char *LPSTR;

// MSVC lowercases in place and returns the same buffer. CFxScheduler::
// RegisterEffect calls this on the extension-stripped effect name.
// Source: `oracle/codemp/client/FxScheduler.cpp:265`
static inline char *strlwr(char *s)
{
	for (char *p = s; *p; p++) {
		*p = (char)tolower((unsigned char)*p);
	}
	return s;
}

static inline void OutputDebugString(const char *) {}

#endif // FX_ORACLE_WIN_SHIM_H
