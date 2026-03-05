# voxel-raymarcher-rust

Voxel raymarching experiment in Rust (`raylib` + `glam`), with chunked terrain generation and CPU ray marching.

## Run

```bash
cargo run
```

For a faster executable when profiling:

```bash
cargo build --release
./target/release/voxel-raymarcher-rust
```

## Controls

- `W/S/A/D`: move
- `Space` / `Left Ctrl`: up/down
- `Q/E`: yaw rotate
- `Y/H`: pitch adjust
- `T/G`: viewplane distance
- `Left Shift`: faster movement/rotation
- `M`: toggle orbit/fly mode
- `Tab` (fly mode): toggle mouse-look capture
- `-` / `=`: draw distance down/up
- `,` / `.`: DDA step budget down/up
- `[` / `]`: FOV down/up
- `Backspace`: reset draw distance, step budget, and FOV
- `F1`: render scale `1x` (native)
- `F2`: render scale `1/2x`
- `F3`: render scale `1/4x`
- `F4`: render scale `1/8x`
- `F5`: render scale `1/16x`
- `F6`: render scale `1/32x`
- `F7` / `F8`: chunk generation budget down/up
- `R`: reset camera
- `Esc`: quit

UI overlay (top-right) shows FPS, draw budget settings, and render counters:
- rays cast/hit
- voxel traversal steps
- empty chunk skips
- render scale + internal render resolution
- chunk generation budget
- simulation/raymarch/upload/frame timings

The `+/-` overlay buttons are clickable when mouse-look is unlocked.

## Profiling (Linux perf + inferno)

This repo already used `perf` + `inferno` for flamegraphs.

```bash
# allow non-root perf events (temporary; adjust for your system)
sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'

cargo build --release
perf record --call-graph dwarf ./target/release/voxel-raymarcher-rust
perf report
perf script | inferno-collapse-perf > out.perf-folded
inferno-flamegraph < out.perf-folded > flamegraph.svg
```

## Archive Notes

- Initial repo commit: `9c9bebb` on `2024-06-02 00:42:24 -0500` (scaffold/docs only).
- First Rust code commit: `ffaaa9e` on `2024-06-02 01:59:37 -0500`.
- Main initial implementation window: June 2, 2024 (`ffaaa9e` -> `34fb45d`).
- Related Python prototype: `pyvoxels`, initial commit `45b4f7e` on `2024-06-01 19:24:52 +0900`.
- Based on structure/timeline, this Rust project was very likely ported from the Python prototype, then optimized for better performance.
