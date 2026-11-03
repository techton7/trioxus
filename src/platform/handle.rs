//! Host platform seam contracts decoupling the canvas from host window/DOM.

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use tricore::math::ViewportRect;

/// Decoupled platform handle abstracting the underlying window, canvas, or custom widget.
///
/// This trait exposes size, DPR, and redraw request capabilities to the canvas runtime
/// without leaking `web-sys`, `winit`, `triui`, or Blitz-specific dependencies.
pub trait CanvasHostHandle: 'static {
    /// Canvas framebuffer physical dimensions in device pixels `(width, height)`.
    fn physical_size(&self) -> (u32, u32);

    /// Canvas logical dimensions in CSS or display-independent pixels `(width, height)`.
    fn logical_size(&self) -> (f64, f64);

    /// Device Pixel Ratio (scale factor) of the host surface.
    fn dpr(&self) -> f64;

    /// Notifies the host environment that a redraw is requested.
    fn request_redraw(&self);

    /// Bounding rectangle of the canvas viewport in host-neutral coordinates.
    fn viewport_rect(&self) -> ViewportRect;
}

/// Reference-counted wrapper for dynamic [`CanvasHostHandle`] implementations.
///
/// Can be cloned cheaply across components and hooks.
#[derive(Clone)]
pub struct CanvasHostContext {
    handle: Arc<dyn CanvasHostHandle>,
}

impl CanvasHostContext {
    /// Creates a new host context from an implementation of [`CanvasHostHandle`].
    pub fn new<H: CanvasHostHandle>(handle: H) -> Self {
        Self {
            handle: Arc::new(handle),
        }
    }

    /// Creates a new host context from an existing `Arc<dyn CanvasHostHandle>`.
    pub fn from_arc(handle: Arc<dyn CanvasHostHandle>) -> Self {
        Self { handle }
    }

    /// Borrows the underlying dynamic [`CanvasHostHandle`].
    pub fn handle(&self) -> &dyn CanvasHostHandle {
        &*self.handle
    }

    /// Canvas framebuffer physical dimensions in device pixels `(width, height)`.
    pub fn physical_size(&self) -> (u32, u32) {
        self.handle.physical_size()
    }

    /// Canvas logical dimensions in display-independent pixels `(width, height)`.
    pub fn logical_size(&self) -> (f64, f64) {
        self.handle.logical_size()
    }

    /// Device Pixel Ratio (scale factor) of the host surface.
    pub fn dpr(&self) -> f64 {
        self.handle.dpr()
    }

    /// Notifies the host environment that a redraw is requested.
    pub fn request_redraw(&self) {
        self.handle.request_redraw()
    }

    /// Bounding rectangle of the canvas viewport in host-neutral coordinates.
    pub fn viewport_rect(&self) -> ViewportRect {
        self.handle.viewport_rect()
    }
}

impl Deref for CanvasHostContext {
    type Target = dyn CanvasHostHandle;

    fn deref(&self) -> &Self::Target {
        &*self.handle
    }
}

impl CanvasHostHandle for CanvasHostContext {
    fn physical_size(&self) -> (u32, u32) {
        self.handle.physical_size()
    }

    fn logical_size(&self) -> (f64, f64) {
        self.handle.logical_size()
    }

    fn dpr(&self) -> f64 {
        self.handle.dpr()
    }

    fn request_redraw(&self) {
        self.handle.request_redraw()
    }

    fn viewport_rect(&self) -> ViewportRect {
        self.handle.viewport_rect()
    }
}

impl PartialEq for CanvasHostContext {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.handle, &other.handle)
    }
}

impl fmt::Debug for CanvasHostContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanvasHostContext")
            .field("physical_size", &self.physical_size())
            .field("logical_size", &self.logical_size())
            .field("dpr", &self.dpr())
            .field("viewport_rect", &self.viewport_rect())
            .finish()
    }
}
