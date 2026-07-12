// roff-oracle stub — the qcommon filesystem seam ROFF calls. Real decls live in
// oracle/codemp/qcommon/qcommon.h; only FS_ReadFile / FS_FreeFile are used by
// RoffSystem.cpp (Cache). Supplied by host.cpp. oracle/ untouched.
#ifndef ROFF_ORACLE_QCOMMON_H
#define ROFF_ORACLE_QCOMMON_H

#ifdef __cplusplus
extern "C" {
#endif

int  FS_ReadFile( const char *qpath, void **buffer );   // qcommon.h — returns len, -1 on miss
void FS_FreeFile( void *buffer );                        // qcommon.h

#ifdef __cplusplus
}
#endif

#endif // ROFF_ORACLE_QCOMMON_H
