pub use glam;

pub mod math {
    use glam::{Mat4, Vec2, Vec3, Vec4};

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Ray3D {
        pub origin: Vec3,
        pub direction: Vec3,
    }

    impl Ray3D {
        pub fn new(origin: Vec3, direction: Vec3) -> Self {
            Self {
                origin,
                direction: direction.normalize_or_zero(),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ViewportRect {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }

    impl ViewportRect {
        pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self {
                x,
                y,
                width,
                height,
            }
        }

        pub fn pixel_to_ndc(&self, pixel_pt: Vec2) -> Vec2 {
            let rel_x = (pixel_pt.x - self.x) / self.width.max(1.0);
            let rel_y = (pixel_pt.y - self.y) / self.height.max(1.0);
            Vec2::new(rel_x * 2.0 - 1.0, 1.0 - rel_y * 2.0)
        }

        pub fn ndc_to_pixel(&self, ndc_pt: Vec2) -> Vec2 {
            let rel_x = (ndc_pt.x + 1.0) * 0.5;
            let rel_y = (1.0 - ndc_pt.y) * 0.5;
            Vec2::new(self.x + rel_x * self.width, self.y + rel_y * self.height)
        }
    }

    pub fn viewport_to_world_2d(
        viewport_pt: Vec2,
        viewport: ViewportRect,
        inv_view_proj: Mat4,
    ) -> Vec2 {
        let ndc = viewport.pixel_to_ndc(viewport_pt);
        let world_pos = inv_view_proj.project_point3(Vec3::new(ndc.x, ndc.y, 0.0));
        Vec2::new(world_pos.x, world_pos.y)
    }

    pub fn world_to_viewport_2d(world_pt: Vec2, viewport: ViewportRect, view_proj: Mat4) -> Vec2 {
        let clip = view_proj * Vec4::new(world_pt.x, world_pt.y, 0.0, 1.0);
        if clip.w.abs() < 1e-6 {
            return Vec2::ZERO;
        }
        let ndc = Vec2::new(clip.x / clip.w, clip.y / clip.w);
        viewport.ndc_to_pixel(ndc)
    }

    pub fn viewport_to_ray(
        viewport_pt: Vec2,
        viewport: ViewportRect,
        inv_view_proj: Mat4,
    ) -> Ray3D {
        let ndc = viewport.pixel_to_ndc(viewport_pt);
        let p_near = inv_view_proj.project_point3(Vec3::new(ndc.x, ndc.y, 0.0));
        let p_far = inv_view_proj.project_point3(Vec3::new(ndc.x, ndc.y, 1.0));
        Ray3D::new(p_near, p_far - p_near)
    }
}
