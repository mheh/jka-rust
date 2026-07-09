//! SP qcommon platform API dispatch.
//!
//! Source: `oracle/code/qcommon/platform.h:1-17`
//!
//! NOTE: Platform compatibility aliases (`LPCTSTR`, `LPCSTR`, `DWORD`, `UINT`,
//! `HANDLE`, `COLORREF`, `BYTE`) moved to `crate::shared::platform`.

// Simple header file to dispatch to the relevant platform API headers
// #ifndef _PLATFORM_H
// #define _PLATFORM_H

// #if defined(_XBOX)
// #include <xtl.h>
// NOTE: Raven included <xtl.h> here for Xbox platform APIs.
// #endif

// #ifdef _WIN32
// #define WIN32_LEAN_AND_MEAN 1
// #endif

// #if defined(_WINDOWS)
// #include <windows.h>
// NOTE: Raven included <windows.h> here for Windows platform APIs.
// #endif

// #endif
