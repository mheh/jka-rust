# Flythrough path files

This directory holds one recorded camera path per map for `world_harness`. The
harness reads and writes `<mapstem>.fly`, where `<mapstem>` is the bsp base name.
The file for `maps/mp/duel1.bsp` is `duel1.fly`.

## Recording

1. Run the harness and free-fly with WASD and the mouse.
2. Press F5 at each pose you want on the path. The harness prints the count.
3. Press F6 to save the recording to this directory.
4. Press F8 to clear the recording and start again.

## Replay

Press F7 to toggle replay when a path file exists for the map. The `--flythrough`
flag starts replay at boot. The camera follows a closed-loop Catmull-Rom spline
through the waypoints at constant speed. A recording with fewer than four
waypoints uses linear interpolation.

## Format

The file is plain text.

- Line 1: `fly 1 <speed>` - the format version and the replay speed in world
  units per second. The recorder writes 300.
- Each later line: `x y z pitch yaw` - one waypoint as five floats, space
  separated. `pitch` and `yaw` are Raven view angles in degrees.

## Determinism

Replay time advances from the per-frame delta the harness already tracks. That
delta is real elapsed time, so poses vary across runs with GPU load. A later
image gate needs a fixed-dt mode in the harness before it can compare poses
across runs.
