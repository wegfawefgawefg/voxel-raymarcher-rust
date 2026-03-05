# Optimization Plan

## Goals

- Improve single-thread CPU raymarch performance before introducing `rayon`.
- Keep code maintainable and avoid monolithic hot-path spaghetti.
- Add measurable runtime counters so each optimization can be validated.

## Optimization Backlog

1. Material packing (`u16` IDs + palette) instead of storing full RGBA per voxel.
2. Lazy chunk voxel allocation (`Option<Vec<MaterialId>>`) so untouched chunks consume near-zero memory.
3. Chunk metadata for fast skip decisions:
   - generated
   - non-air voxel count
   - has transparency
4. Ray-vs-world AABB intersection to clamp each ray to relevant distance.
5. DDA voxel traversal instead of fixed step ray marching.
6. Chunk-first empty skip (jump to next chunk boundary when current chunk is known empty).
7. Runtime render counters in overlay:
   - rays cast
   - hit rays
   - voxels visited
   - empty chunk skips
8. Keep 3D chunk grid (already vertical+horizontal chunking).
9. Generate terrain once per `(chunk_x, chunk_z)` column to avoid duplicate per-`chunk_y` work.
10. Cache current chunk metadata while traversing DDA voxels (avoid per-voxel chunk meta lookup).

## Current Status

- [x] Material packing (`u16` material IDs + palette).
- [x] Lazy chunk allocation (`Option<Vec<MaterialId>>`).
- [x] Chunk metadata (`generated`, `non_air_voxels`, `has_transparency`).
- [x] Ray-vs-world AABB culling/intersection per ray.
- [x] DDA voxel traversal in renderer.
- [x] Empty-chunk jump-to-boundary skip.
- [x] Runtime render counters in overlay.
- [x] Kept 3D chunk grid.
- [x] Terrain generated once per `(chunk_x, chunk_z)` column.
- [x] Cached chunk-meta lookup per traversed chunk in DDA loop.
- [ ] Heightmap-assisted skip (deferred intentionally).

## Notes on Heightmap-Assisted Skip

Deferred for now. The world now includes transparent structures and potentially non-heightmap geometry,
so a pure terrain heightmap accelerator risks becoming special-case logic. We can add a terrain-only
heightfield accelerator later if profiling shows it is still worth the extra complexity.

## Validation

- Build: `cargo check`
- Runtime sanity:
  - verify visual parity for opaque/transparent objects
  - verify overlay counters update
  - compare FPS and counters before/after changes
