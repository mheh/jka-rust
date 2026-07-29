//! Which module compiled this bg copy.

/// Raven builds the bg tier three times (`QAGAME`, `CGAME`, ui) and picks
/// build arms with the preprocessor; the port compiles it once and each module
/// stamps its arm here at `BgState` construction — the DEC-36 D3 runtime-host
/// shape. bg logic whose CONTROL FLOW differs per build arm branches on this;
/// inert leaf calls stay on the callbacks (DEC-46.5) and don't need it.
///
/// - `Game`: the server game module (`QAGAME`).
/// - `Cgame`: the client game module (`CGAME`).
/// - `Ui`: the ui module.
///
/// First consumer: `PM_SlideMove`'s `#ifdef QAGAME` `PM_ClientImpact` arm
/// (`oracle/codemp/game/bg_slidemove.c:724-732`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BgHost {
    Game,
    Cgame,
    Ui,
}
