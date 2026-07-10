// Minimal Win32 declarations for the out-of-scope oracle Tokenizer.cpp
// (fixture-generator only). Tokenizer.cpp's CParseFile wraps the Win32 file API
// (CreateFile/ReadFile/SetFilePointer/CloseHandle) — but ibi-gen feeds source
// through AddParseStream (a memory buffer), never AddParseFile, so this whole
// path is dead code. It still has to *compile and link*, so provide inert
// definitions. None of these are ever called.
#ifndef ICARUS_ORACLE_WIN32STUB_H
#define ICARUS_ORACLE_WIN32STUB_H

#include <cstring>

typedef void*          HANDLE;
typedef unsigned long  DWORD;
typedef unsigned long* LPDWORD;
typedef void*          LPVOID;
typedef const void*    LPCVOID;
typedef int            BOOL;
typedef unsigned long  COLORREF;

struct SECURITY_ATTRIBUTES { DWORD nLength; void* lpSecurityDescriptor; BOOL bInheritHandle; };

#define GENERIC_READ         0x80000000UL
#define FILE_SHARE_READ      0x00000001UL
#define FILE_SHARE_WRITE     0x00000002UL
#define OPEN_EXISTING        3UL
#define FILE_ATTRIBUTE_NORMAL 0x00000080UL
#define FILE_BEGIN           0UL
#define FILE_CURRENT         1UL
#define FILE_END             2UL
#define RGB(r,g,b) ((COLORREF)(((unsigned char)(r))|((unsigned short)((unsigned char)(g))<<8)|(((DWORD)(unsigned char)(b))<<16)))

// MSVC case-insensitive compares Tokenizer.cpp / Interpreter.cpp call -> POSIX.
#include <strings.h>
static inline int stricmp(const char *a, const char *b) { return strcasecmp(a, b); }
static inline int strnicmp(const char *a, const char *b, size_t n) { return strncasecmp(a, b, n); }

static inline HANDLE CreateFile(const char*, DWORD, DWORD, void*, DWORD, DWORD, HANDLE) { return (HANDLE)-1; }
static inline BOOL   ReadFile(HANDLE, LPVOID, DWORD, LPDWORD n, void*) { if (n) *n = 0; return 0; }
static inline DWORD  SetFilePointer(HANDLE, long, LPDWORD, DWORD) { return 0; }
static inline BOOL   CloseHandle(HANDLE) { return 1; }

#endif
