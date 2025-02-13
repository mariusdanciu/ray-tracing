use glam::{vec4, Mat4, Vec3, Vec4Swizzles};

use crate::ray::{Ray, RayHit};
/**
 * Moller Trumbore intersection
 */
pub fn triangle_intersection(
    ray: &Ray,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    material_index: usize,
) -> Option<RayHit> {
    let e1 = v2 - v1;
    let e2 = v3 - v1;
    let ray_cross_e2 = ray.direction.cross(e2);
    let det = e1.dot(ray_cross_e2);

    if det > -f32::EPSILON && det < f32::EPSILON {
        return None; // This ray is parallel to this triangle.
    }
    let back_facing = det < f32::EPSILON;

    let inv_det = 1.0 / det;
    let s = ray.origin - v1;
    let u = inv_det * s.dot(ray_cross_e2);
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let s_cross_e1 = s.cross(e1);

    let v = inv_det * ray.direction.dot(s_cross_e1);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = inv_det * e2.dot(s_cross_e1);

    if t > f32::EPSILON {
        let hit_point = ray.origin + ray.direction * t;

        let mut normal = (v2 - v1).cross(v3 - v1).normalize();
        if back_facing {
            normal = -normal;
        }

        return Some(RayHit {
            distance: t,
            point: hit_point,
            normal,
            material_index,
            u,
            v,
        });
    } else {
        // This means that there is a line intersection but not a ray intersection.
        return None;
    }
}

