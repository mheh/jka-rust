// trmodel-oracle — deterministic host control surface for the dumpers.
#ifndef TRMODEL_ORACLE_HOST_H
#define TRMODEL_ORACLE_HOST_H

// Register a fixture path as living in a pure PAK with the given checksum, so
// FS_FileIsInPAK(path) returns 1 and stamps `checksum` (else -1) — the ruling-59
// 1/-1 convention that drives iPAKFileCheckSum + DumpNonPure. Path is lowercased.
void host_pak_add(const char *lc_path, int checksum);

// Set a loader cvar's integer value (sv_pure / r_modelpoolmegs). Auto-registers.
void host_cvar_set(const char *name, int value);

// Count of successful FS_ReadFile disk loads — proves cache miss (disk) vs hit
// (served from CachedModels, zero reads). Reset by the dumper between phases.
extern int host_fs_reads;

#endif
