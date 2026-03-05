# SDF World Gen Plan

## Current Baseline

- World size is `256^3` voxels (`0..255` per axis).
- Terrain currently comes from perlin-based column heights.
- Floor is at `y = 255` in the current coordinate convention.

## Goals

- Add interesting large-scale features with signed distance fields (SDF):
  - spheres
  - torus shapes
  - rotated boxes
  - subtraction blends (CSG difference)
- Keep terrain + SDF composition deterministic and debuggable.
- Keep CPU cost bounded so interactive camera remains smooth.

## Phase 1: SDF Infrastructure

1. Add a compact `SdfPrimitive` enum:
   - `Sphere { center, radius, material }`
   - `Torus { center, major, minor, material }`
   - `Box { center, half_extents, rotation, material }`
2. Add `SdfOp` enum:
   - `Add`
   - `Subtract`
3. Add a tiny scene container:
   - `SdfScene { nodes: Vec<SdfNode> }`
4. Add helpers for world-space SDF evaluation:
   - `distance_sphere`
   - `distance_torus`
   - `distance_oriented_box`

## Phase 2: Terrain + SDF Composition

1. Keep terrain heightmap as base.
2. For each voxel in a generation column/chunk:
   - evaluate base terrain occupancy
   - evaluate SDF nodes
   - apply CSG order (`Add` then `Subtract` or explicit sequence)
3. Resolve final material by last positive contributor (or explicit priority).

## Phase 3: Performance Controls

1. Add coarse bounding boxes per SDF node.
2. Skip SDF evaluation when voxel is outside all relevant bounds.
3. Optionally cache affected chunk ranges for each node at scene build time.
4. Add overlay counters:
   - SDF samples
   - SDF bounds rejects

## Phase 4: Authoring and Iteration

1. Add a simple hardcoded scene preset list:
   - `Classic` (terrain-only)
   - `Crater` (sphere subtraction)
   - `Archipelago` (multiple sphere adds)
   - `DonutField` (torus adds/subtracts)
2. Add hotkeys to cycle preset and regenerate around camera.

## Integration Notes

- Start with generation-time SDF stamping (not per-ray SDF tracing).
- Reuse existing asynchronous terrain pipeline:
  - worker computes column shape inputs
  - main thread commits voxels
- If generation cost rises, move voxel fill for SDF-heavy chunks to worker output buffers and commit chunk data in bulk.

## Risks

- CSG ordering can be confusing without strict rules.
- Rotated box math can be bug-prone if transforms are inconsistent.
- Too many SDF nodes can dominate generation CPU without bounds checks.

## Milestone Sequence

1. Phase 1 + one `Sphere(Subtract)` crater demo.
2. Phase 2 composition with terrain materials.
3. Phase 3 counters and bounds acceleration.
4. Phase 4 preset switching and polish.
