//! Reactive render lifecycle and invalidation hooks.

use dioxus::prelude::*;

/// Hook providing reactive invalidation and render requesting for `<TrioxusCanvas>`.
pub fn use_trioxus_runtime() -> TrioxusRuntime {
    use_hook(TrioxusRuntime::new)
}

/// Runtime handle for requesting redraws and monitoring viewport changes.
#[derive(Clone, Copy)]
pub struct TrioxusRuntime {
    render_requested: Signal<u64>,
}

impl TrioxusRuntime {
    pub fn new() -> Self {
        Self {
            render_requested: Signal::new(0),
        }
    }

    /// Requests a new frame to be rendered.
    pub fn request_render(&mut self) {
        *self.render_requested.write() += 1;
    }

    /// Current frame counter.
    pub fn frame_count(&self) -> u64 {
        *self.render_requested.read()
    }
}

impl Default for TrioxusRuntime {
    fn default() -> Self {
        Self::new()
    }
}
