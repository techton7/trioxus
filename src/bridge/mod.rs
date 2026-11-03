//! Coordinate and host mapping bridge connecting Dioxus DOM events to tricore viewport math.

use tricore::glam::{Mat4, Vec2};
use tricore::math::{viewport_to_world_2d, world_to_viewport_2d, ViewportRect};

use crate::platform::CanvasHostHandle;

/// Pointer interaction kind received from host platform input events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasPointerKind {
    Down,
    Up,
    Move,
    Wheel { delta_x: f64, delta_y: f64 },
}

/// Pointer button involved in the pointer event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CanvasPointerButton {
    #[default]
    None,
    Primary,
    Secondary,
    Auxiliary,
}

/// Normalized canvas pointer event dispatched into canvas layout or TaffyDriver.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasPointerEvent {
    pub kind: CanvasPointerKind,
    pub local_x: f64,
    pub local_y: f64,
    pub button: CanvasPointerButton,
}

impl CanvasPointerEvent {
    /// Creates a new canvas pointer event.
    pub fn new(
        kind: CanvasPointerKind,
        local_x: f64,
        local_y: f64,
        button: CanvasPointerButton,
    ) -> Self {
        Self {
            kind,
            local_x,
            local_y,
            button,
        }
    }

    /// Converts local coordinates to physical coordinates using the specified DPR scale factor.
    pub fn local_physical(&self, scale_factor: f64) -> (f64, f64) {
        (self.local_x * scale_factor, self.local_y * scale_factor)
    }

    /// Converts local coordinates to a 2D vector for viewport and layout math.
    pub fn local_vec2(&self) -> Vec2 {
        Vec2::new(self.local_x as f32, self.local_y as f32)
    }
}

/// Bridge mapping client/screen coordinates to host-neutral tricore viewport space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateBridge {
    pub canvas_rect: ViewportRect,
    pub scale_factor: f64,
}

impl CoordinateBridge {
    /// Creates a new coordinate bridge with a canvas viewport rectangle and DPR scale factor.
    pub fn new(canvas_rect: ViewportRect, scale_factor: f64) -> Self {
        Self {
            canvas_rect,
            scale_factor: if scale_factor > 0.0 { scale_factor } else { 1.0 },
        }
    }

    /// Creates a coordinate bridge directly from any [`CanvasHostHandle`].
    pub fn from_host<H: CanvasHostHandle>(host: &H) -> Self {
        Self::new(host.viewport_rect(), host.dpr())
    }

    /// Converts host client/screen coordinates `(client_x, client_y)` to canvas-local logical coordinates.
    pub fn client_to_local_logical(&self, client_x: f64, client_y: f64) -> (f64, f64) {
        (
            client_x - self.canvas_rect.x as f64,
            client_y - self.canvas_rect.y as f64,
        )
    }

    /// Converts host client/screen coordinates `(client_x, client_y)` to canvas-local physical coordinates
    /// scaled by device pixel ratio (`scale_factor`).
    pub fn client_to_local_physical(&self, client_x: f64, client_y: f64) -> (f64, f64) {
        let (local_x, local_y) = self.client_to_local_logical(client_x, client_y);
        (local_x * self.scale_factor, local_y * self.scale_factor)
    }

    /// Converts canvas-local logical coordinates `(local_x, local_y)` back to host client coordinates.
    pub fn local_logical_to_client(&self, local_x: f64, local_y: f64) -> (f64, f64) {
        (
            local_x + self.canvas_rect.x as f64,
            local_y + self.canvas_rect.y as f64,
        )
    }

    /// Converts canvas-local physical coordinates `(phy_x, phy_y)` to canvas-local logical coordinates.
    pub fn local_physical_to_logical(&self, phy_x: f64, phy_y: f64) -> (f64, f64) {
        (phy_x / self.scale_factor, phy_y / self.scale_factor)
    }

    /// Maps a raw host client event with coordinates `(client_x, client_y)` into a normalized [`CanvasPointerEvent`].
    pub fn map_pointer_event(
        &self,
        kind: CanvasPointerKind,
        client_x: f64,
        client_y: f64,
        button: CanvasPointerButton,
    ) -> CanvasPointerEvent {
        let (local_x, local_y) = self.client_to_local_logical(client_x, client_y);
        CanvasPointerEvent {
            kind,
            local_x,
            local_y,
            button,
        }
    }

    /// Converts host client/mouse coordinates `(client_x, client_y)` to 2D world coordinates.
    pub fn screen_to_world(&self, client_pt: Vec2, inv_view_proj: Mat4) -> Vec2 {
        let local_pt = Vec2::new(
            client_pt.x - self.canvas_rect.x,
            client_pt.y - self.canvas_rect.y,
        );
        viewport_to_world_2d(local_pt, self.canvas_rect, inv_view_proj)
    }

    /// Converts 2D world coordinates to host client/mouse coordinates `(client_x, client_y)`.
    pub fn world_to_screen(&self, world_pt: Vec2, view_proj: Mat4) -> Vec2 {
        let local_pt = world_to_viewport_2d(world_pt, self.canvas_rect, view_proj);
        Vec2::new(
            local_pt.x + self.canvas_rect.x,
            local_pt.y + self.canvas_rect.y,
        )
    }
}
