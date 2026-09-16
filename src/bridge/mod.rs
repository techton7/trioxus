//! Coordinate and host mapping bridge connecting Dioxus DOM events to tricore viewport math.

use tricore::glam::{Mat4, Vec2};
use tricore::math::{viewport_to_world_2d, world_to_viewport_2d, ViewportRect};

/// Bridge mapping client/screen coordinates to host-neutral tricore viewport space.
pub struct CoordinateBridge {
    pub canvas_rect: ViewportRect,
    pub scale_factor: f64,
}

impl CoordinateBridge {
    pub fn new(canvas_rect: ViewportRect, scale_factor: f64) -> Self {
        Self {
            canvas_rect,
            scale_factor: if scale_factor > 0.0 { scale_factor } else { 1.0 },
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
