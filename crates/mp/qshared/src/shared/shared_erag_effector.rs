#![allow(non_camel_case_types)]

/// Raven `sharedERagEffector` ragdoll effector bone bit flags.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:867-894`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum sharedERagEffector {
    RE_MODEL_ROOT = 0x00000001,   // "model_root"
    RE_PELVIS = 0x00000002,       // "pelvis"
    RE_LOWER_LUMBAR = 0x00000004, // "lower_lumbar"
    RE_UPPER_LUMBAR = 0x00000008, // "upper_lumbar"
    RE_THORACIC = 0x00000010,     // "thoracic"
    RE_CRANIUM = 0x00000020,      // "cranium"
    RE_RHUMEROUS = 0x00000040,    // "rhumerus"
    RE_LHUMEROUS = 0x00000080,    // "lhumerus"
    RE_RRADIUS = 0x00000100,      // "rradius"
    RE_LRADIUS = 0x00000200,      // "lradius"
    RE_RFEMURYZ = 0x00000400,     // "rfemurYZ"
    RE_LFEMURYZ = 0x00000800,     // "lfemurYZ"
    RE_RTIBIA = 0x00001000,       // "rtibia"
    RE_LTIBIA = 0x00002000,       // "ltibia"
    RE_RHAND = 0x00004000,        // "rhand"
    RE_LHAND = 0x00008000,        // "lhand"
    RE_RTARSAL = 0x00010000,      // "rtarsal"
    RE_LTARSAL = 0x00020000,      // "ltarsal"
    RE_RTALUS = 0x00040000,       // "rtalus"
    RE_LTALUS = 0x00080000,       // "ltalus"
    RE_RRADIUSX = 0x00100000,     // "rradiusX"
    RE_LRADIUSX = 0x00200000,     // "lradiusX"
    RE_RFEMURX = 0x00400000,      // "rfemurX"
    RE_LFEMURX = 0x00800000,      // "lfemurX"
    RE_CEYEBROW = 0x01000000,     // "ceyebrow"
}
