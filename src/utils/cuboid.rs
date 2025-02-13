use glam::{vec3, vec4, Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

use crate::ray::{Ray, RayHit};

use super::geometry;

pub fn box_intersection(
    ray: &Ray,
    box_size: Vec3,
    transform: Mat4,
    inv_transform: Mat4,
    material_index: usize
) -> Option<RayHit> {
    let mut ray_dir = ray.direction;
    let mut ray_origin = ray.origin;

    ray_dir = (inv_transform * vec4(ray_dir.x, ray_dir.y, ray_dir.z, 0.)).xyz();
    ray_origin = (inv_transform * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.)).xyz();

    let h_box_size = box_size;

    let b_min = -h_box_size;
    let b_max = h_box_size;

    let inv = 1.0 / ray_dir;

    let t_min = (b_min - ray_origin) * inv;
    let t_max = (b_max - ray_origin) * inv;

    let t_enter = t_min.min(t_max);
    let t_exit = t_min.max(t_max);

    let t_near = t_enter.x.max(t_enter.y).max(t_enter.z);
    let t_far = t_exit.x.min(t_exit.y).min(t_exit.z);

    if t_near > t_far || t_far < 0.0 {
        return None; // no intersection
    }

    let a = -ray_dir.signum() * geometry::step(vec3(t_near, t_near, t_near), t_enter);

    let normal = (transform * vec4(a.x, a.y, a.z, 0.0)).xyz();

    let hit_point = ray.origin + ray.direction * t_near;

    let opos = (inv_transform * vec4(hit_point.x, hit_point.y, hit_point.z, 1.0)).xyz();
    let onor = a;

    let u_v =
        onor.x.abs() * (opos.yz()) + onor.y.abs() * (opos.zx()) + onor.z.abs() * (opos.xy());

    Some(RayHit {
        distance: t_near,
        point: hit_point,
        normal,
        material_index,
        u: u_v.x,
        v: u_v.y,
    })
}
