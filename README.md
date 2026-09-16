<p align="center">
  <img src="assets/icon.svg" alt="trioxus logo" width="160" height="160" />
</p>

<h1 align="center">trioxus</h1>

<p align="center">
  <strong>Dioxus-facing declarative graphics facade over tricore across Web and Desktop.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/trioxus"><img src="https://img.shields.io/crates/v/trioxus.svg" alt="Crates.io" /></a>
  <a href="https://docs.rs/trioxus"><img src="https://docs.rs/trioxus/badge.svg" alt="docs.rs" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg" alt="License" /></a>
</p>

---

## Overview

**Trioxus** is the declarative graphics component facade for the [Dioxus](https://dioxuslabs.com/) ecosystem. It provides the single `<TrioxusCanvas>` component, reactive lifecycle orchestration, and host embedding adapters over [`tricore`](https://github.com/techton7/tricore) — the pure Rust, host-agnostic WGPU graphics engine.

---

## Architecture: Core & Facade Division

`trioxus` deliberately leaves raw GPU pipelines, camera math, and offscreen render targets to `tricore`, focusing entirely on idiomatic Dioxus ergonomics:

```text
[ Dioxus Component Tree (RSX) ]
              │
              ▼
    [ <TrioxusCanvas> ] (trioxus facade)
              │
    ┌─────────┴────────────────────────┐
    ▼                                  ▼
[ Web: <canvas> ]            [ Native: Blitz <object> ]
(WASM WebGPU/WebGL2)         (Vello / CustomWidgetAttr)
    │                                  │
    └──────────────────┬───────────────┘
                       ▼
            [ tricore Engine Core ]
            - WGPU Device & Queue Lifecycle
            - Host-Neutral Viewport & Camera Math
            - GPU Instanced 2D Batching (Lines, Béziers, Quads)
            - 3D Scene Primitives & PBR Shaders
            - Offscreen Targets & Readback Staging
```

---

## Features

- **Single `<TrioxusCanvas>` API:** Write your 2D/3D graphics once in Dioxus RSX; runs identically in browser Web and Blitz Native Desktop.
- **Zero-Friction Engine Re-export:** `trioxus` transparently re-exports `tricore`, giving full access to camera math, 2D batching structs, and scene types without redundant imports.
- **DPR-Aware Host Coordinate Bridge:** Automatically maps browser/desktop pointer events into host-neutral `tricore` viewport spaces (`screen_to_world`, `world_to_screen`).
- **Reactive Invalidation:** Integrate smoothly with Dioxus Signals without locking up the high-frequency GPU rendering loop.

---

## Quick Start

Add `trioxus` to your `Cargo.toml`:

```toml
[dependencies]
dioxus = { version = "0.7", features = ["web"] }
trioxus = "0.1"
```

### Usage in RSX

```rust
use dioxus::prelude::*;
use trioxus::{TrioxusCanvas, use_trioxus_runtime};

#[component]
pub fn App() -> Element {
    let mut runtime = use_trioxus_runtime();

    rsx! {
        div { class: "w-screen h-screen relative",
            TrioxusCanvas {
                id: "main-canvas",
                class: "w-full h-full",
            }
            button {
                class: "absolute top-4 left-4 bg-blue-600 text-white p-2 rounded",
                onclick: move |_| runtime.request_render(),
                "Request Render"
            }
        }
    }
}
```

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
