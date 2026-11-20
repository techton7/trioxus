//! # Trioxus
//!
//! **Dioxus-facing declarative graphics facade over `tricore` across Web and Desktop.**
//!
//! `trioxus` provides Dioxus component bindings, reactive lifecycle management,
//! and host embedding adapters for the underlying host-agnostic [`tricore`] graphics engine.
//!
//! ## Core Architecture
//!
//! - **Dioxus Component Facade**: Exposes `<TrioxusCanvas>` for easy declarative use in RSX.
//! - **Platform Embedding Adapters**: Integrates with Web `<canvas>` on browser targets and
//!   Blitz native custom widgets (`<object>`) on desktop targets.
//! - **Host-to-Core Coordinate Bridge**: Normalizes DOM client coordinates with DPR scaling
//!   and maps them into host-neutral `tricore` viewport spaces.
//! - **Re-exported Engine Core**: Transparently provides access to `tricore` types and math.

pub mod bridge;
pub mod components;
pub mod platform;
pub mod runtime;

#[cfg(not(feature = "tricore"))]
mod tricore_compat;
#[cfg(not(feature = "tricore"))]
pub mod tricore {
    pub use crate::tricore_compat::{glam, math};
}

// Re-export tricore as the foundational graphics engine
#[cfg(feature = "tricore")]
pub use tricore;

// High-frequency re-exports
pub use bridge::*;
pub use components::*;
pub use platform::*;
pub use runtime::*;

/// Version of the Trioxus crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[cfg(feature = "tricore")]
    #[test]
    fn test_tricore_reexport_accessible() {
        let cube = tricore::Mesh3D::unit_cube();
        assert_eq!(cube.vertices.len(), 24);
    }

    #[cfg(not(feature = "tricore"))]
    #[test]
    fn test_tricore_math_compat_accessible() {
        let viewport = tricore::math::ViewportRect::new(0.0, 0.0, 800.0, 600.0);
        assert_eq!(
            viewport.ndc_to_pixel(tricore::glam::Vec2::ZERO),
            tricore::glam::Vec2::new(400.0, 300.0)
        );
    }

    #[test]
    fn test_canvas_key_bridge_reexports_accessible() {
        let ev: crate::CanvasKeyEvent =
            crate::bridge::CanvasKeyEvent::key_down("Tab", "Tab")
                .with_modifiers(crate::CanvasModifiers::SHIFT);
        assert_eq!(ev.state, crate::CanvasKeyState::Down);
        assert!(ev.modifiers.contains(crate::bridge::CanvasModifiers::SHIFT));
    }
}
