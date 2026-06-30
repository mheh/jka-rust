//! MP qcommon platform API dispatch.
//!
//! Source: `oracle/oracle/codemp/qcommon/platform.h:1-22`
//!
//! NOTE: Platform compatibility aliases (`LPCTSTR`, `LPCSTR`, `DWORD`, `UINT`,
//! `HANDLE`, `COLORREF`, `BYTE`) moved to `crate::shared::platform`.

// Simple header file to dispatch to the relevant platform API headers
// #ifndef _PLATFORM_H
// #define _PLATFORM_H

// #if defined(_XBOX)
// #include <xtl.h>
// TODO: Port xtl.h platform API surface.
// #endif

// #if defined(_WINDOWS)
// #include <windows.h>
// TODO: Port windows.h platform API surface.
// #endif

// #if defined (__linux__)
// Moved to `crate::shared::platform`.
// #endif
// #endif
