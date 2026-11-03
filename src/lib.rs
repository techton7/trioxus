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

// Re-export tricore as the foundational graphics engine
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

    #[test]
    fn test_tricore_reexport_accessible() {
        let cube = tricore::Mesh3D::unit_cube();
        assert_eq!(cube.vertices.len(), 24);
    }
}
