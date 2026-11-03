//! Host platform adapter bindings (isolated adapters for Web and Blitz Native).

pub mod handle;
pub use handle::*;

#[cfg(target_arch = "wasm32")]
pub mod web {
    //! Web browser canvas adapter.
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    //! Desktop Blitz CustomWidget adapter.
}
