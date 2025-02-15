use glam::{vec4, Mat4, Vec3, Vec4Swizzles};

use crate::ray::{Ray, RayHit};

pub fn sphere_intersection(
    ray: &Ray,
    position: Vec3,
    transform: Mat4,
    inv_transform: Mat4,
    radius: &f32,
    material_index: usize,
) -> Option<RayHit> {
    let mut ray_dir = ray.direction;
    let mut ray_origin = ray.origin;

    // Move the ray in object space.
    ray_dir = (inv_transform * vec4(ray_dir.x, ray_dir.y, ray_dir.z, 0.)).xyz();
    ray_origin = (inv_transform * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.)).xyz();

    // (bx^2 + by^2 + bz^2)t^2 + (2(axbx + ayby + azbz))t + (ax^2 + ay^2 + az^2 - r^2) = 0
    // where
    // a = ray origin
    // b = ray direction
    // r = radius
    // t = hit distance

    let a = ray_dir.dot(ray_dir);
    let b = 2. * ray_origin.dot(ray_dir);
    let c = ray_origin.dot(ray_origin) - radius * radius;

    let disc = b * b - 4. * a * c;

    if disc < 0.0 {
        return None;
    }

    let t0 = (-b + disc.sqrt()) / (2.0 * a);
    let t1 = (-b - disc.sqrt()) / (2.0 * a);

    let hit_point = ray.origin + ray.direction * t1;

    let n = hit_point - position;

    // Move the normal in world space
    let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz().normalize();

    Some(RayHit {
        distance: t1,
        point: hit_point,
        normal,
        material_index,
        ..Default::default()
    })
}
