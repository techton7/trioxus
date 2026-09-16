# Handoff: Sovereign Pure-Rust WGPU Graphics Architecture (`trioxus`) & Cross-Platform Roadmap

## 1. Supervisory Communication Contract

1. Use **English only**.
2. Avoid overclaiming. Clearly distinguish between:
   - **Current repo facts** (what is currently committed, implemented, and verified)
   - **Target architectural blueprint** (the agreed-upon specification and design)
   - **Validation actually executed** (live commands, runtime tests, and visual proof)
3. Zero synthetic mocks. Zero hardcoded geometric constants claiming runtime fidelity.
4. Maintain full transparency on runtime verification status (`[contract-implemented]` vs `[runtime-proven]`).

---

## 2. Strategic Pivot: From Three.js Wrapper to Sovereign Pure-Rust WGPU Engine

### 2.1 The Repudiation of the Three.js Wrapper
The initial vision of `trioxus` was a Rust wrapper around Three.js (analogous to React Three Fiber). This strategy is **permanently discarded** for the following reasons:
1. **Blitz Native (`dioxus-native`) Incompatibility**:
   - Three.js is bound to the JavaScript runtime, `window.document`, and browser WebGL/WebGPU contexts.
   - Blitz is a pure Rust web engine powered by Vello/WGPU. It has no JavaScript engine (no V8, no JSC, no SpiderMonkey).
   - Running Three.js on Blitz would necessitate bundling an embedded JS runtime (QuickJS/Boa), completely destroying Blitz's lightweight native architecture.
2. **FFI & Ownership Impedance Mismatch**:
   - Even on Web (`wasm32`), wrapping Three.js through `wasm-bindgen` introduces costly FFI boundary serialization, GC pinning, and breaks Rust's type-safety and ownership semantics.
3. **The Solution**:
   - A **100% Pure Rust + WGPU codebase**.
   - First-class cross-platform parity across both Web (WebGPU/WebGL2) and Desktop (Blitz Native via Metal/Vulkan/DirectX).

### 2.2 Rejection of WGPU Ingestion into `oxidase` (The Sovereign Track Decision)
The question was evaluated whether `oxidase` (the cross-platform DOM abstraction layer) should acquire and manage WGPU resources for `trioxus`. The conclusion is an emphatic **NO**. `trioxus` must proceed on its own **Sovereign Track**:

1. **Domain Boundary & Single Responsibility**:
   - `oxidase::dom` exists strictly to abstract `window.document` (Web) and `blitz_dom::BaseDocument` (Native).
   - In both W3C and Rust standards, DOM and GPU pipelines are completely decoupled. A DOM tree should have zero knowledge of render pipelines or GPU command encoders.
2. **Dependency Pollution Protection**:
   - `oxidase` is the core foundation for `monoxus` (headless UI components like Select, Dialog, Popover, Tabs).
   - Pulling `wgpu` into `oxidase` would force heavy GPU compilation toolchains (`naga`, `wgpu-hal`, `wgpu-core`, `ash`, `metal`) onto lightweight UI components.
3. **Blitz Architectural Reality**:
   - In Blitz, `BaseDocument` does NOT own a WGPU device.
   - WGPU is exclusively passed down through the window renderer to **`blitz_dom::Widget`** via lifecycle callbacks (`can_create_surfaces`, `paint`).
   - Blitz is architected specifically for widgets to handle their own GPU offscreen rendering.
4. **Cooperative Interface**:
   - `trioxus` owns all WGPU lifecycles directly.
   - `oxidase::dom` is consumed by `trioxus` solely as an optional utility for DOM/viewport measurement (`bounding_client_rect`, DPR).

---

## 3. Ecosystem Topology: The Tri-Repo Symbiosis

The ecosystem forms a cohesive trinity where each repository maintains strict single responsibility:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                          techton7/nodoxus                               │
│  - Reactive Node Graph Topology, Ports, State Machine                   │
│  - Hybrid Frustum Culling (QuadTree in memory, ~30-50 nodes in DOM)    │
│  - Figma Ghost Input (Inline text editing via temporary <input>)        │
└───────────────────────┬─────────────────────────▲───────────────────────┘
                        │ Delegates GPU Rendering  │ Drives Motion
                        ▼                         │
┌────────────────────────────────────────┐ ┌──────┴───────────────────────┐
│           techton7/trioxus             │ │        techton7/kinetoxus    │
│  - Sovereign WGPU Scene Engine         │ │  - Pure Rust Motion Engine   │
│  - <TrioxusCanvas> Component           │ │  - Penner Easing & Springs   │
│  - 2D Batch Bezier/Quad Shaders        │ │  - Multi-Target Tweener:     │
│  - 3D Geometry, GLTF & Camera Math     │ │    * DOM Signals             │
│  - screen_to_world / world_to_screen   │ │    * Direct WGPU [f32; N]    │
└────────────────────────────────────────┘ └──────────────────────────────┘
```

---

## 4. Trioxus Architectural Blueprint

### 4.1 Unified Consumer Component API (`<TrioxusCanvas>`)
End users interact with a single declarative component. Platform branching is strictly encapsulated inside `trioxus`:

```rust
// Consumer usage across all platforms:
rsx! {
    TrioxusCanvas {
        id: "main-canvas",
        scene: my_scene,
        camera: my_camera,
        class: "w-full h-full",
    }
}
```

### 4.2 Dual-Backend WGPU Surface Binding

#### 1. Web Track (`wasm32`):
- Employs standard HTML `<canvas id="{id}">`.
- Initializes `wgpu::Instance` on mount (`onmounted`).
- Creates surface using `wgpu::SurfaceTarget::Canvas`.
- Handles high-DPI scaling via `window.device_pixel_ratio` and `ResizeObserver`.
- Supports WebGPU with automatic WebGL2 fallback.

#### 2. Native Desktop Track (`blitz-dom::Widget`):
- Employs Blitz `<object "data": custom_widget_attr>`.
- Implements `blitz_dom::Widget` on `TrioxusWidget`:
  ```rust
  impl blitz_dom::Widget for TrioxusWidget {
      fn can_create_surfaces(&mut self, render_ctx: &mut dyn anyrender::RenderContext) {
          // Borrow Blitz's existing WGPU Device & Queue directly
          if let Some(ctx) = render_ctx.renderer_specific_context() {
              if let Ok(handle) = ctx.downcast::<wgpu_context::DeviceHandle>() {
                  self.renderer.attach_device(handle.device.clone(), handle.queue.clone());
              }
          }
      }

      fn destroy_surfaces(&mut self) {
          self.renderer.detach_device();
      }

      fn paint(
          &mut self,
          render_ctx: &mut dyn anyrender::RenderContext,
          _styles: &ComputedStyles,
          width: u32,
          height: u32,
          scale: f64,
      ) -> anyrender::Scene {
          let mut scene = anyrender::Scene::new();
          if let Some(resource_id) = self.renderer.render_offscreen(render_ctx, width, height, scale) {
              scene.fill(
                  peniko::Fill::NonZero,
                  peniko::kurbo::Affine::IDENTITY,
                  anyrender::PaintRef::Resource(peniko::ImageBrush {
                      image: resource_id,
                      sampler: peniko::ImageSampler::default(),
                  }),
                  None,
                  &peniko::kurbo::Rect::from_origin_size((0.0, 0.0), (width as f64, height as f64)),
              );
          }
          scene
      }
  }
  ```
- Renders to an offscreen `wgpu::Texture`, registers it via `render_ctx.register_wgpu_texture(...)`, and composites it into Blitz's Vello scene using `ImageBrush`.

### 4.3 Camera & Viewport Math (Single Source of Truth)
To prevent 1-pixel drift between HTML overlay nodes and GPU-rendered lines, `trioxus` owns all coordinate conversions:
- `screen_to_world(screen_pt: Vec2) -> Vec2`
- `world_to_screen(world_pt: Vec2) -> Vec2`
- `screen_to_ray(screen_pt: Vec2) -> Ray3D` (for 3D object picking)

Coordinates account for canvas bounding box offset, Pan offset, Zoom scale, and DPR scale factor.

### 4.4 2D High-Speed Batch Renderer (for Nodoxus)
`nodoxus` relies on `trioxus` to draw thousands of dynamic connections and background grids:
- **GPU Parametric Bezier Instancing (Blinc Model)**:
  - Each cubic Bezier curve is submitted as an instance: `[p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, width: f32, color: u32]`.
  - Vertex shader dynamically computes Bezier polynomials $B(t)$ and normal tangents to extrude curve geometry on the GPU.
  - Fragment shader applies Signed Distance Field (SDF) anti-aliasing.
  - Zero CPU vertex tessellation overhead.
- **Instanced Quad Batching (LOD Proxy Quads)**:
  - When zoomed out, `nodoxus` unmounts DOM nodes; `trioxus` renders proxy node rectangles in a single draw call via instanced quads.

---

## 5. Development Milestones & Tracer Bullet Verification

| Milestone | Deliverables | Verification Criteria |
| :--- | :--- | :--- |
| **M1: Cross-Platform Spike** | Basic `TrioxusCanvas` rendering a colored spinning cube/quad. | `[runtime-proven]` on both Web (`dx serve --web`) and Blitz Native (`cargo run`). |
| **M2: Camera & Math Core** | 2D Pan/Zoom camera, 3D Orbit camera, `screen_to_world` / `world_to_screen` inversion. | Unit tests for matrix math + live mouse cursor coordinate tracking test. |
| **M3: 2D Batch Pipeline** | GPU instanced cubic Bezier shader, grid shader, instanced quad buffer. | Stress test rendering 10,000 active curves at 60+ FPS. |
| **M4: 3D Scene Core & Blinc GLTF** | PBR pipeline, standard primitives (Cube, Sphere), `.glb`/`.gltf` model parsing. | Successful loading and interactive rendering of standard GLTF assets. |

---

## 6. Immediate Action Items

1. Update `trioxus/Cargo.toml` dependencies:
   - Add `wgpu`, `bytemuck`, `glam` (math).
   - Add platform-conditional dependencies:
     - `wasm-bindgen`, `web-sys` for `wasm32`.
     - `blitz-dom`, `dioxus-native`, `anyrender`, `peniko` for native desktop.
2. Author M1 prototype in `trioxus/src/`:
   - `src/components/canvas.rs`: Platform-abstracted `<TrioxusCanvas>`.
   - `src/renderer/mod.rs`: Shared WGPU rendering logic.
   - `src/platform/web.rs`: Web `<canvas>` integration.
   - `src/platform/native.rs`: Blitz `CustomWidgetAttr` integration.
3. Build and verify M1 example on Desktop and Web.
