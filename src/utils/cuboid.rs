use glam::{vec3, vec4, Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit}, scene::Scene,
};

use super::geometry;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cuboid {
    pub position: Vec3,
    pub dimension: Vec3,
    pub rotation_axis: Vec3,
    pub material_index: usize,
    transform: Mat4,
    inv_transform: Mat4,
    b_min: Vec3,
    b_max: Vec3,
}
impl Cuboid {
    pub fn new(
        position: Vec3,
        rotation_axis: Vec3,
        dimension: Vec3,
        material_index: usize,
    ) -> Object3D {
        Object3D::Cuboid(
            Cuboid {
                position,
                dimension,
                rotation_axis,
                material_index,
                ..Default::default()
            }
            .update(),
        )
    }

    pub fn update(&mut self) -> Self {
        let t = Mat4::from_translation(self.position)
            * Mat4::from_rotation_x(self.rotation_axis.x * geometry::DEGREES)
            * Mat4::from_rotation_y(self.rotation_axis.y * geometry::DEGREES)
            * Mat4::from_rotation_z(self.rotation_axis.z * geometry::DEGREES);
        self.transform = t;
        self.inv_transform = t.inverse();
        self.b_min = (Mat4::from_scale(self.dimension) * vec4(-1., -1., -1., 0.)).xyz();
        self.b_max = (Mat4::from_scale(self.dimension) * vec4(1., 1., 1., 0.)).xyz();
        *self
    }

    pub fn sdf(&self, scene: &Scene, p: Vec3, object: &Object3D) -> (f32, Vec3) {
        let p = p - self.position;
        let corner_radius = 0.1;
        let q = p.abs() - self.dimension + corner_radius;
        let m = object.material_index();
        let c = scene.materials[m].albedo;
        (q.max(Vec3::ZERO).length() + q.x.max(q.y.max(q.z)).min(0.0) - corner_radius, c)
    }
}
impl Intersection for Cuboid {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        let mut ray_dir = ray.direction;
        let mut ray_origin = ray.origin;

        ray_dir = (self.inv_transform * vec4(ray_dir.x, ray_dir.y, ray_dir.z, 0.)).xyz();
        ray_origin =
            (self.inv_transform * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.)).xyz();

        let inv = 1.0 / ray_dir;

        let t_min = (self.b_min - ray_origin) * inv;
        let t_max = (self.b_max - ray_origin) * inv;

        let t_enter = t_min.min(t_max);
        let t_exit = t_min.max(t_max);

        let t_near = t_enter.x.max(t_enter.y).max(t_enter.z);
        let t_far = t_exit.x.min(t_exit.y).min(t_exit.z);

        if t_near > t_far || t_far < 0.0 {
            return None; // no intersection
        }

        let a = -ray_dir.signum() * geometry::step(vec3(t_near, t_near, t_near), t_enter);

        let normal = (self.transform * vec4(a.x, a.y, a.z, 0.0)).xyz();

        let hit_point = ray.origin + ray.direction * t_near;

        let opos = (self.inv_transform * vec4(hit_point.x, hit_point.y, hit_point.z, 1.0)).xyz();
        let onor = a;

        let u_v =
            onor.x.abs() * (opos.yz()) + onor.y.abs() * (opos.zx()) + onor.z.abs() * (opos.xy());

        Some(RayHit {
            distance: t_near,
            point: hit_point,
            normal,
            material_index: self.material_index,
            u: u_v.x,
            v: u_v.y,
        })
    }


}
