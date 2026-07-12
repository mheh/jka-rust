//! Raven `CRagDollUpdateParams` — the ragdoll solver's per-call parameter
//! block + VM-callback hook set, reimplemented per §F17 (porting-rules) as an
//! enum-tagged struct (`G2SV-D8`).
//!
//! `CRagDollUpdateParams` is a virtual-method C++ class (`G2_gore.h:94-124`),
//! distinct from the already-ported plain-data `sharedRagDollUpdateParams_t`.
//! MP instantiates only the base class directly — `CRagDollUpdateParams
//! rduParams;` (`oracle/codemp/game/sv_game.cpp:1539`), no subclass anywhere
//! in `codemp/` — so per §F17 ("closed virtual hierarchies → enums") the
//! class collapses to [`RagDollUpdateParams`] with a single
//! [`RagDollUpdateKind::Server`] variant whose four hooks are the base
//! class's own no-op bodies. SP (`code/`) has two real subclasses
//! (`CGameRagDollUpdateParams` et al., `code/game/g_main.cpp:1296`) that
//! *do* override these hooks with live logic; that is a future DEC-04 diff
//! (MP first, SP as diff, porting-rules §F20) and is not represented here.
//!
//! Dropped, `_DEBUG`-only surface (no roster row, not a stub): `virtual void
//! DebugLine(const vec3_t p1, const vec3_t p2, bool bbox)` (`G2_gore.h:126`)
//! compiles only under `#ifdef _DEBUG`; the WinDed DEDICATED Release config
//! this doc targets defines `NDEBUG` (`docs/subsystems/ghoul2-server.md`
//! "Raven ground truth" build config), so `_DEBUG` is off and `DebugLine`
//! never compiles in this build.
//!
//! Frozen struct + enum shape verbatim from `docs/subsystems/ghoul2-server.md`
//! `## Seam definition`.
//! Type definition source: `oracle/codemp/ghoul2/G2_gore.h:94-124`

use mp_qshared::shared::vec3_t;

use crate::gore::srag_doll_effector_collision::SRagDollEffectorCollision;

/// The sole MP variant of the closed virtual hierarchy (§F17): MP never
/// subclasses `CRagDollUpdateParams`, so there is exactly one kind whose
/// hooks are the base class's own no-op bodies. SP's two subclasses would add
/// variants here as a future DEC-04 diff.
///
/// Raven: (no enum in the original — the §F17 collapse of the closed
/// zero-subclass hierarchy).
/// Type definition source: `oracle/codemp/ghoul2/G2_gore.h:94-124`
pub enum RagDollUpdateKind {
    Server,
}

/// Raven `CRagDollUpdateParams` (`G2_gore.h:94-124`) reimplemented as a
/// §F17 enum-tagged struct: the six data members verbatim, plus `kind` for
/// virtual-hook dispatch (`G2SV-D8`).
///
/// Raven: (no comment on the class itself; see the per-field/per-method
/// comments below).
/// Type definition source: `oracle/codemp/ghoul2/G2_gore.h:94-124`
pub struct RagDollUpdateParams {
    /// Raven `vec3_t angles` (`G2_gore.h:97`).
    pub angles: vec3_t,
    /// Raven `vec3_t position` (`G2_gore.h:98`).
    pub position: vec3_t,
    /// Raven `vec3_t scale` (`G2_gore.h:99`).
    pub scale: vec3_t,
    /// Raven `vec3_t velocity` (`G2_gore.h:100`).
    pub velocity: vec3_t,
    /// Raven `int me; //index!` (`G2_gore.h:101-102`) — `//CServerEntity
    /// *me;` above it is a stale commented-out declaration; the live member
    /// is the entity index.
    pub me: i32,
    /// Raven `int settleFrame` (`G2_gore.h:103`).
    pub settle_frame: i32,
    /// Which virtual-hook body this instance dispatches to (§F17; not a
    /// Raven field — the C++ vtable pointer's port-time replacement).
    pub kind: RagDollUpdateKind,
}

impl RagDollUpdateParams {
    /// Raven `virtual void EffectorCollision(const SRagDollEffectorCollision
    /// &data)` (`G2_gore.h:106-109`) — base body is empty (`//assert(0)`
    /// commented out, "for now I am just doing nothing"). In MP every call
    /// site is itself commented out (`G2_bones.cpp:3054,3083,3178,3215`,
    /// e.g. `//params->EffectorCollision(args);`), so the hook is reached by
    /// no live MP call; it is transcribed anyway per the doc's four-hook
    /// enum shape (`G2SV-D8`).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_gore.h:106-109`
    pub fn effector_collision(&mut self, _data: &SRagDollEffectorCollision) {
        match self.kind {
            // assert(0); // you probably meant to override this
            RagDollUpdateKind::Server => {}
        }
    }

    /// Raven `virtual void RagDollBegin()` (`G2_gore.h:110-113`) — base body
    /// is empty.
    ///
    /// Source: `oracle/codemp/ghoul2/G2_gore.h:110-113`
    pub fn rag_doll_begin(&mut self) {
        match self.kind {
            // assert(0); // you probably meant to override this
            RagDollUpdateKind::Server => {}
        }
    }

    /// Raven `virtual void RagDollSettled()` (`G2_gore.h:114-117`) — base
    /// body is empty. Called live server-side from `G2_RagDoll`
    /// (`G2_bones.cpp:2497,2505`, `params->RagDollSettled();`), so the
    /// no-op `Server` dispatch genuinely matches to nothing at those call
    /// sites (`G2SV-D8`).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_gore.h:114-117`
    pub fn rag_doll_settled(&mut self) {
        match self.kind {
            // assert(0); // you probably meant to override this
            RagDollUpdateKind::Server => {}
        }
    }

    /// Raven `virtual void Collision()` (`G2_gore.h:119-123`) — base body is
    /// empty; the trailing Raven comment ("we had a collision, uhh I guess
    /// call SetRagDoll RP_DEATH_COLLISION") notes intended-but-never-added
    /// override behavior. No MP call site exists (grep:
    /// `oracle/codemp/ghoul2/`).
    ///
    /// Source: `oracle/codemp/ghoul2/G2_gore.h:119-123`
    pub fn collision(&mut self) {
        match self.kind {
            // assert(0); // you probably meant to override this
            // we had a collision, uhh I guess call SetRagDoll RP_DEATH_COLLISION
            RagDollUpdateKind::Server => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_params() -> RagDollUpdateParams {
        RagDollUpdateParams {
            angles: [0.0, 0.0, 0.0],
            position: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            velocity: [0.0, 0.0, 0.0],
            me: 0,
            settle_frame: 0,
            kind: RagDollUpdateKind::Server,
        }
    }

    /// All four base-class hooks are empty in Raven (`G2_gore.h:106-123`); the
    /// sole MP `Server` variant must leave every data member untouched.
    #[test]
    fn server_hooks_are_true_no_ops() {
        let mut params = base_params();
        params.rag_doll_begin();
        params.rag_doll_settled();
        params.collision();

        assert_eq!(params.angles, [0.0, 0.0, 0.0]);
        assert_eq!(params.position, [0.0, 0.0, 0.0]);
        assert_eq!(params.scale, [1.0, 1.0, 1.0]);
        assert_eq!(params.velocity, [0.0, 0.0, 0.0]);
        assert_eq!(params.me, 0);
        assert_eq!(params.settle_frame, 0);
    }
}
