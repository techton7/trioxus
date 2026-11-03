//! Reactive render lifecycle and invalidation hooks.

use dioxus::prelude::*;

use crate::platform::CanvasHostContext;

/// Hook providing reactive invalidation and render requesting for `<TrioxusCanvas>`.
pub fn use_trioxus_runtime() -> TrioxusRuntime {
    use_hook(TrioxusRuntime::new)
}

/// Hook providing reactive invalidation attached to a specific host context.
pub fn use_trioxus_runtime_with_host(host: CanvasHostContext) -> TrioxusRuntime {
    use_hook(|| TrioxusRuntime::with_host(host))
}

/// Runtime handle for coordinating frame lifecycles, requesting redraws,
/// and dispatching redraw notifications without dirtying the host document.
#[derive(Clone, Copy)]
pub struct TrioxusRuntime {
    render_requested: Signal<u64>,
    redraw_pending: Signal<bool>,
    host: Signal<Option<CanvasHostContext>>,
}

impl TrioxusRuntime {
    /// Creates a new unattached runtime instance.
    pub fn new() -> Self {
        Self {
            render_requested: Signal::new(0),
            redraw_pending: Signal::new(false),
            host: Signal::new(None),
        }
    }

    /// Creates a runtime instance attached to a host context.
    pub fn with_host(host: CanvasHostContext) -> Self {
        Self {
            render_requested: Signal::new(0),
            redraw_pending: Signal::new(false),
            host: Signal::new(Some(host)),
        }
    }

    /// Attaches a canvas host context to this runtime.
    pub fn attach_host(&self, host: CanvasHostContext) {
        let mut host_signal = self.host;
        *host_signal.write() = Some(host);
    }

    /// Retrieves the attached canvas host context, if any.
    pub fn host(&self) -> Option<CanvasHostContext> {
        self.host.read().clone()
    }

    /// Requests a new frame to be rendered.
    ///
    /// Increments the frame counter, marks redraw as pending,
    /// and invokes `request_redraw` on the attached host handle if present.
    pub fn request_render(&self) {
        let mut render_requested = self.render_requested;
        let mut redraw_pending = self.redraw_pending;
        *render_requested.write() += 1;
        *redraw_pending.write() = true;
        if let Some(host) = self.host.read().as_ref() {
            host.request_redraw();
        }
    }

    /// Current frame counter.
    pub fn frame_count(&self) -> u64 {
        *self.render_requested.read()
    }

    /// Returns the reactive signal for render requests.
    /// Sub-VDOM render loops or subscribers can read or track this signal.
    pub fn render_signal(&self) -> Signal<u64> {
        self.render_requested
    }

    /// Queries whether a redraw request is currently pending.
    pub fn is_redraw_pending(&self) -> bool {
        *self.redraw_pending.read()
    }

    /// Completes the currently pending frame, marking redraw as no longer pending.
    pub fn complete_frame(&self) {
        let mut redraw_pending = self.redraw_pending;
        *redraw_pending.write() = false;
    }

    /// Notifies that a redraw is needed without dirtying or mutating the host document.
    ///
    /// This triggers the canvas runtime render request and host redraw notification
    /// without causing Dioxus to re-evaluate the outer document tree.
    pub fn notify_redraw(&self) {
        self.request_render();
    }
}

impl Default for TrioxusRuntime {
    fn default() -> Self {
        Self::new()
    }
}
