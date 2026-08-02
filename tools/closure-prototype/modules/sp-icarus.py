# Order mirrors Raven's Sequencer.cpp: StdAfx -> IcarusImplementation ->
# BlockStream -> Sequence -> TaskManager -> Sequencer.

SPEC = dict(
    name="sp-icarus",
    lang="c++", entry=["code/icarus/StdAfx.h", "code/icarus/IcarusInterface.h",
                       "code/icarus/IcarusImplementation.h",
                       "code/icarus/blockstream.h", "code/icarus/sequence.h",
                       "code/icarus/taskmanager.h", "code/icarus/sequencer.h"],
    includes=["code/icarus", "code/game", "code/qcommon", "code"],
    defines=["NDEBUG", "_IMMERSION"],
)
