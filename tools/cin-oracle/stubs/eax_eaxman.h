// snd-oracle stub for Creative's `eaxman.h` (the EAX Manager COM object).
//
// DEC-57.4 drops the EAX arm. Raven reaches this object only after the OpenAL
// arm loads `EAXMan.dll`, which the harness never does, so the methods below are
// declarations with aborting bodies.
//
// build.sh copies this file to the literal name `eax\eaxman.h`.
#ifndef SND_ORACLE_EAXMAN_H
#define SND_ORACLE_EAXMAN_H

#include "eax\eax.h"

#define EM_OK 0
#define EMFLAG_LOADFROMMEMORY 1
#define EMFLAG_LOCKPOSITION 2

typedef struct _EMPOINT {
	float fX;
	float fY;
	float fZ;
} EMPOINT;

// host.cpp prints the name and aborts.
void snd_oracle_al_unreachable(const char *name);

struct EAXMANAGER {
	HRESULT Release() { snd_oracle_al_unreachable("EAXManager::Release"); return -1; }
	HRESULT LoadDataSet(const char *, unsigned long) { snd_oracle_al_unreachable("EAXManager::LoadDataSet"); return -1; }
	HRESULT FreeDataSet(unsigned long) { snd_oracle_al_unreachable("EAXManager::FreeDataSet"); return -1; }
	HRESULT GetSourceID(const char *, long *) { snd_oracle_al_unreachable("EAXManager::GetSourceID"); return -1; }
	HRESULT GetSourceNumInstances(long, long *) { snd_oracle_al_unreachable("EAXManager::GetSourceNumInstances"); return -1; }
	HRESULT GetSourceInstancePos(long, long, EMPOINT *) { snd_oracle_al_unreachable("EAXManager::GetSourceInstancePos"); return -1; }
	HRESULT GetSourceDynamicAttributes(long, EMPOINT *, long *, float *, long *, float *, float *, EMPOINT *, unsigned long) { snd_oracle_al_unreachable("EAXManager::GetSourceDynamicAttributes"); return -1; }
	HRESULT GetListenerDynamicAttributes(long, EMPOINT *, long *, unsigned long) { snd_oracle_al_unreachable("EAXManager::GetListenerDynamicAttributes"); return -1; }
	HRESULT GetEnvironmentName(long, char *, unsigned long) { snd_oracle_al_unreachable("EAXManager::GetEnvironmentName"); return -1; }
	HRESULT GetEnvironmentAttributes(long, EAXREVERBPROPERTIES *) { snd_oracle_al_unreachable("EAXManager::GetEnvironmentAttributes"); return -1; }
};

typedef struct EAXMANAGER *LPEAXMANAGER;
typedef HRESULT (*LPEAXMANAGERCREATE)(LPEAXMANAGER *);

// Raven calls the Win32 loader directly at snd_dma.cpp:5330-5419. The harness
// returns a null module so the EAX manager never appears.
static inline HINSTANCE LoadLibrary(const char *) { return 0; }
static inline void *GetProcAddress(HINSTANCE, const char *) { return 0; }
static inline void FreeLibrary(HINSTANCE) {}

#endif // SND_ORACLE_EAXMAN_H
