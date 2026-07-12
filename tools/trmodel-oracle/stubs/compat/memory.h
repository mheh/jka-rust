// trmodel-oracle compat shim: matcomp.c does `#include <memory.h>` (an SVR4/MSVC
// header for memcpy) that does not exist on macOS/libc++. Redirect to <cstring>.
// Build-path only (-Istubs/compat); oracle/matcomp.c is never edited.
#ifndef TRMODEL_ORACLE_COMPAT_MEMORY_H
#define TRMODEL_ORACLE_COMPAT_MEMORY_H
#include <cstring>
#endif
