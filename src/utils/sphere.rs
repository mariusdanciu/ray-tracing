use core::f32;

use glam::{vec4, Mat4, Vec3, Vec4Swizzles};

use crate::ray::{Ray, RayHit};

pub fn sphere_intersection(
    ray: &Ray,
    transform: Mat4,
    inv_transform: Mat4,
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
    let c = ray_origin.dot(ray_origin) - 1.;

    let disc = b * b - 4. * a * c;

    if disc < 0.0 {
        return None;
    }

    let t0 = (-b + disc.sqrt()) / (2.0 * a);
    let t1 = (-b - disc.sqrt()) / (2.0 * a);

    let l = ray_origin + ray_dir * t1;

    let n = l;

    // Move the normal in world space
    let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz().normalize();

    

    let u = ((l.x * l.x + l.y * l.y) / (l.z)).atan();
    let v = (l.y / l.x).atan();

    Some(RayHit {
        distance: t1,
        point: ray.origin + ray.direction * t1,
        normal,
        material_index,
        u: v / f32::consts::PI,
        v: u / f32::consts::PI,
    })
}
