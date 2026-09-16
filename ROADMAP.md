# Trioxus Roadmap

This document is the working roadmap for `trioxus`.

`trioxus` is the Dioxus-facing graphics component facade over [`tricore`](https://github.com/techton7/tricore) — the pure Rust, host-agnostic WGPU graphics engine.

## Mission

- Build `trioxus` as a declarative Dioxus graphics facade and host-embedding layer over `tricore`.
- Support both Web and Blitz Native behind one consumer-facing `<TrioxusCanvas>` component.
- Map DOM and host pointer events into `tricore` host-neutral viewport coordinates.
- Expose motion-ready handles from `tricore` to `kinetoxus` using standard motion verbs: `set`, `from`, `to`, and `from_to`.

## Strategic Decisions

1. `trioxus` is **not** a Three.js wrapper and should not depend on a JavaScript scene runtime.
2. `tricore` owns the core WGPU graphics runtime, camera/viewport math, 2D batching, 3D scene, and offscreen targets.
3. `trioxus` owns Dioxus component bindings, reactive lifecycle orchestration, and Web/Native platform embedding.
4. `oxidase` remains a browser and DOM bridge utility, not a GPU owner.
5. `kinetocore` remains the headless interpolation, easing, spring, and timeline contract.
6. `kinetoxus` becomes the Dioxus-facing motion facade and should support both signal targets and `tricore` handles.

## Ownership Map

| Project | Owns | Must not become |
|---|---|---|
| `kinetocore` | Pure motion math, interpolation traits, easing, springs, timelines, generic target contracts | A Dioxus crate or a renderer |
| `kinetoxus` | Dioxus-facing motion ergonomics, hooks, signal adapters, unified verbs for signal and non-signal targets | The graphics engine |
| `tricore` | Host-agnostic WGPU device/queue lifecycle, viewport math, 2D batching, 3D scene, offscreen targets, readback | A Dioxus crate or a UI framework |
| `trioxus` | Dioxus `<TrioxusCanvas>` component, Web/Native host embedding, DOM event to viewport bridging | A monolithic raw WGPU engine |
| `nodoxus` | Graph topology, node interactions, virtualization, overlay UI, graph-specific state | A low-level renderer |
| `oxidase` | Browser interop, watchers, DOM measurement, optional browser-side helpers | A WGPU abstraction layer |

## Recommended Consumer Experience

Application authors should ideally experience motion through one Dioxus-oriented language even when the targets differ:

```text
kinetoxus::to(signal_target, value, duration)
kinetoxus::to(camera_handle, pose, duration)
kinetoxus::from_to(uniform_handle, from, to, duration)
```

That does **not** require `trioxus` to surrender renderer ownership. The intended shape is:

- `trioxus` exposes motion-ready handles and controllers.
- `kinetocore` defines the low-level motion contract.
- `kinetoxus` provides the ergonomic facade for Dioxus applications.

## Success Criteria

`trioxus` should be considered directionally successful when it can satisfy all of the following:

1. The same `TrioxusCanvas` consumer API works on Web and Blitz Native.
2. Camera math is owned inside `trioxus` and serves as the single source of truth for screen/world conversions.
3. A `kinetoxus` animation can drive a `trioxus` camera or scene handle without forcing full-component signal diffing every frame.
4. `nodoxus` can eventually consume `trioxus` for 2D GPU-backed lines, grids, and proxy quads.
5. The rendering core remains pure Rust and does not require a JavaScript scene engine to function.

## Two Delivery Tracks

### Track A: Fast PoC

This is the shortest path to useful dogfooding and should happen first.

1. Land a Web-first `TrioxusCanvas` spike using a real HTML `<canvas>`.
2. Initialize WGPU, clear the surface, and render a single visible primitive.
3. Add one camera or transform handle that can be animated from a Dioxus app.
4. Prove `kinetoxus::set`, `from`, `to`, and `from_to` against that handle.
5. Keep the first demo intentionally small: one camera motion, one transform motion, one resize path.

### Track B: Structural Foundation

This is the recommended long-term route and should start as soon as the PoC proves the shape.

1. Freeze the crate boundaries and motion contracts before feature growth.
2. Add explicit platform backends for Web and Blitz Native.
3. Build camera, coordinate, and rendering abstractions before broad scene features.
4. Introduce 2D batching before ambitious 3D surface area expansion.
5. Add `kinetoxus` adapters at the handle level, not by routing every frame through generic component signals.

## Roadmap Phases

| Phase | Goal | Deliverables | Proof target |
|---|---|---|---|
| 0 | Identity reset | Align README, package metadata, and architectural docs with the pure Rust + WGPU direction | `[contract-implemented]` |
| 1 | Web-first proof of life | `TrioxusCanvas`, mount lifecycle, DPR-aware resize handling, and one rendered primitive | `[runtime-proven]` on Web |
| 2 | Cross-platform surface abstraction | Native Blitz widget path with offscreen texture composition and a shared renderer entry point | `[runtime-proven]` on Web and Blitz Native |
| 3 | Camera and math core | `screen_to_world`, `world_to_screen`, `screen_to_ray`, pan/zoom/orbit controllers, and matrix tests | `[contract-implemented]` + `[runtime-proven]` live cursor alignment |
| 4 | Motion integration surface | Motion-ready handles for camera, transforms, and uniform-like numeric targets; `kinetoxus` demos using `set/from/to/from_to` | `[runtime-proven]` without signal-only bottlenecks |
| 5 | 2D renderer for `nodoxus` | Batched lines, cubic bezier pipeline, background grid, and zoomed-out proxy quads | `[runtime-proven]` stress scene |
| 6 | 3D scene core | Basic meshes, transforms, materials, lights, picking, and scene lifecycle cleanup | `[runtime-proven]` interactive sample scene |
| 7 | Asset and ecosystem lane | GLTF loading, camera demos, nodoxus integration samples, and end-user examples | `[runtime-proven]` end-to-end demos |

## Phase Details

### Phase 0 - Identity Reset

- Replace outdated Three.js-centric positioning in docs and metadata.
- Establish this repository as the graphics runtime layer, not as a JS bridge.
- Publish the architectural boundary with `kinetocore`, `kinetoxus`, `nodoxus`, and `oxidase`.

### Phase 1 - Web-First Proof of Life

- Add a minimal `TrioxusCanvas` component backed by a mounted `<canvas>`.
- Handle resize and device-pixel-ratio updates correctly.
- Render a single triangle, quad, or cube with deterministic startup and teardown.
- Keep the first renderer path small enough to reason about without prematurely designing a full scene graph.

### Phase 2 - Cross-Platform Surface Abstraction

- Add a native Blitz embedding path using a widget or equivalent offscreen rendering slot.
- Share renderer setup and frame submission logic across platforms wherever the APIs genuinely overlap.
- Keep platform ownership explicit where it does not.

### Phase 3 - Camera and Coordinate Contract

- Introduce camera structs and controller handles for:
  - 2D pan and zoom
  - 3D orbit or free camera
- Expose precise conversion APIs that higher-level systems can trust.
- Validate the contract with both unit-level matrix checks and live coordinate inspection.

### Phase 4 - Motion Integration Surface

- Expose direct handles for animatable renderer-owned state.
- Ensure those handles can be targeted by `kinetocore` contracts and by `kinetoxus` ergonomics.
- Make the first official motion demos about real `trioxus` concerns:
  - camera glide
  - transform interpolation
  - shader or uniform-style numeric changes

### Phase 5 - 2D High-Volume Rendering

- Implement the 2D lane needed by `nodoxus`.
- Prioritize:
  - instanced lines
  - cubic bezier rendering
  - grid rendering
  - zoomed-out proxy quads for large graphs
- Treat this as a first-class workload, not a side feature.

### Phase 6 - 3D Scene Core

- Add foundational scene primitives only after the renderer and camera contracts are stable.
- Start with a small, explicit feature set:
  - transforms
  - mesh primitives
  - materials
  - lighting
  - picking hooks
- Do not chase broad engine parity before the basic lifecycle is solid.

### Phase 7 - Asset and Ecosystem Integration

- Add asset ingestion only after the rendering and motion contracts are proven.
- Integrate with future `nodoxus` and `kinetoxus` demos from the application side.
- Prefer a few trustworthy end-to-end examples over a wide but shallow feature list.

## Suggested Module Layout

```text
src/
  canvas/
  renderer/
  platform/
    web.rs
    native.rs
  camera/
  scene/
  motion/
  math/
```

The exact file layout can evolve, but the ownership boundaries should remain explicit: platform code stays platform-specific, camera math stays renderer-owned, and motion surfaces expose handles rather than bundling UI hooks into the rendering core.

## Non-Goals for Early Releases

- Full React Three Fiber parity
- A compatibility wrapper over the Three.js API surface
- Editor tooling before renderer proof exists
- General-purpose physics beyond motion interpolation needs
- Premature abstraction over every possible render backend

## Immediate Next Actions

1. Keep this roadmap as the reference document for the pivot.
2. Land Phase 0 documentation alignment.
3. Build the smallest real Web WGPU spike for `TrioxusCanvas`.
4. Define one camera or transform handle shape that `kinetoxus` can animate.
5. Use that path as the first dogfood bridge before expanding scene breadth.
