// snd-oracle stub for Creative's `eax.h` (EAX 4.0).
//
// DEC-57.4 drops the EAX arm. Raven gates it on the `s_bEAX` flag, which only
// becomes true inside the OpenAL arm, so the harness compiles the declarations
// and never runs them. The struct layouts below follow the public EAX 4.0 SDK
// so the oracle source parses. No golden depends on them.
//
// build.sh copies this file to the literal name `eax\eax.h`.
#ifndef SND_ORACLE_EAX_H
#define SND_ORACLE_EAX_H

#include "openal\al.h"

// Windows COM surface the EAX headers assume. macOS has none, so the harness
// declares the small part Raven names.
#ifndef SND_ORACLE_GUID_DEFINED
#define SND_ORACLE_GUID_DEFINED
typedef struct _GUID {
	unsigned long Data1;
	unsigned short Data2;
	unsigned short Data3;
	unsigned char Data4[8];
} GUID;
typedef long HRESULT;
typedef void *HINSTANCE;
#define SUCCEEDED(hr) ((HRESULT)(hr) >= 0)
#define FAILED(hr) ((HRESULT)(hr) < 0)
#endif

#define EAX_MAX_FXSLOTS 4

// Property-set selectors. The values are never observed.
#define EAXFXSLOT_ALLPARAMETERS 3
#define EAXFXSLOT_LOADEFFECT 1
#define EAXFXSLOT_VOLUME 2
#define EAXFXSLOT_LOCK 3
#define EAXFXSLOT_FLAGS 4
#define EAXFXSLOT_LOCKED 1
#define EAXFXSLOTFLAGS_ENVIRONMENT 0x00000001
#define EAXCONTEXT_PRIMARYFXSLOTID 2
#define EAXREVERB_ALLPARAMETERS 2
#define EAXREVERB_ENVIRONMENT 3
#define EAXREVERB_ROOM 6
#define EAXREVERB_REFLECTIONSPAN 16
#define EAXREVERB_REVERBPAN 19
#define EAX_ENVIRONMENT_UNDERWATER 22
#define EAXSOURCE_OBSTRUCTIONPARAMETERS 5
#define EAXSOURCE_OCCLUSIONPARAMETERS 6
#define EAXSOURCE_EXCLUSION 7
#define EAXSOURCE_ACTIVEFXSLOTID 24
#define EAXSOURCE_FLAGS 20
#define EAXSOURCE_DEFAULTOBSTRUCTION 8
#define EAXSOURCE_DEFAULTOBSTRUCTIONLFRATIO 9
#define EAXSOURCE_DEFAULTOCCLUSION 10
#define EAXSOURCE_DEFAULTOCCLUSIONLFRATIO 11
#define EAXSOURCE_DEFAULTOCCLUSIONROOMRATIO 12
#define EAXSOURCE_DEFAULTOCCLUSIONDIRECTRATIO 13

// Windows compares GUIDs with these operators, and snd_dma.cpp:5372 uses them.
static inline bool operator==(const GUID &a, const GUID &b)
{
	if (a.Data1 != b.Data1 || a.Data2 != b.Data2 || a.Data3 != b.Data3) {
		return false;
	}
	for (int i = 0; i < 8; i++) {
		if (a.Data4[i] != b.Data4[i]) {
			return false;
		}
	}
	return true;
}
static inline bool operator!=(const GUID &a, const GUID &b) { return !(a == b); }

typedef struct _EAXVECTOR {
	float x;
	float y;
	float z;
} EAXVECTOR;

typedef struct _EAXREVERBPROPERTIES {
	unsigned long ulEnvironment;
	float flEnvironmentSize;
	float flEnvironmentDiffusion;
	long lRoom;
	long lRoomHF;
	long lRoomLF;
	float flDecayTime;
	float flDecayHFRatio;
	float flDecayLFRatio;
	long lReflections;
	float flReflectionsDelay;
	EAXVECTOR vReflectionsPan;
	long lReverb;
	float flReverbDelay;
	EAXVECTOR vReverbPan;
	float flEchoTime;
	float flEchoDepth;
	float flModulationTime;
	float flModulationDepth;
	float flAirAbsorptionHF;
	float flHFReference;
	float flLFReference;
	float flRoomRolloffFactor;
	unsigned long ulFlags;
} EAXREVERBPROPERTIES;

typedef struct _EAXFXSLOTPROPERTIES {
	GUID guidLoadEffect;
	long lVolume;
	long lLock;
	unsigned long ulFlags;
} EAXFXSLOTPROPERTIES;

typedef struct _EAXOBSTRUCTIONPROPERTIES {
	long lObstruction;
	float flObstructionLFRatio;
} EAXOBSTRUCTIONPROPERTIES;

typedef struct _EAXOCCLUSIONPROPERTIES {
	long lOcclusion;
	float flOcclusionLFRatio;
	float flOcclusionRoomRatio;
	float flOcclusionDirectRatio;
} EAXOCCLUSIONPROPERTIES;

typedef struct _EAXACTIVEFXSLOTS {
	GUID guidActiveFXSlots[EAX_MAX_FXSLOTS];
} EAXACTIVEFXSLOTS;

typedef ALenum (*EAXSet)(const GUID *, ALuint, ALuint, ALvoid *, ALuint);
typedef ALenum (*EAXGet)(const GUID *, ALuint, ALuint, ALvoid *, ALuint);

#endif // SND_ORACLE_EAX_H
