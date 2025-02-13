use core::f32;

use glam::{vec3, vec4, Mat4, Vec3Swizzles, Vec4Swizzles};

use crate::ray::{Ray, RayHit};

pub fn cylinder_intersection(
    ray: &Ray,
    ra: f32,
    height: f32,
    transform: Mat4,
    inv_transform: Mat4,
    material_index: usize,
) -> Option<RayHit> {
    let rd3 = (inv_transform * vec4(ray.direction.x, ray.direction.y, ray.direction.z, 0.)).xyz();
    let ro3 = (inv_transform * vec4(ray.origin.x, ray.origin.y, ray.origin.z, 1.)).xyz();

    let rd = rd3.xy();
    let ro = ro3.xy();

    let a = rd.dot(rd);
    let b = 2.0 * ro.dot(rd);
    let c = ro.dot(ro) - ra * ra;

    let disc = b * b - 4.0 * a * c;

    let half = height / 2.;
    if disc > 0.0 {
        let t1 = (-b - disc.sqrt()) / (2.0 * a);

        let h_t1 = ro3 + rd3 * t1;

        let valid_t1 = h_t1.z.abs() < half;

        if valid_t1 {
            let n = vec3(h_t1.x, h_t1.y, 0.0).normalize();
            let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz();
            let poi = ray.origin + ray.direction * t1;

            let u = (h_t1.y / h_t1.x).atan(); // atan2 covers edge cases

            let v = h_t1.z*2.;

            return Some(RayHit {
                distance: t1,
                point: poi,
                normal,
                material_index,
                u: u,
                v: v,
            });
        }
    }

    let t1 = (ro3.z - half) / -rd3.z;

    let t2 = (ro3.z + half) / -rd3.z;

    let h_t1 = ro3 + rd3 * t1;
    let h_t2 = ro3 + rd3 * t2;

    let valid_t1 = h_t1.xy().dot(h_t1.xy()) < ra * ra;
    let valid_t2 = h_t2.xy().dot(h_t2.xy()) < ra * ra;

    let mut t = 0.;

    if valid_t1 && valid_t2 {
        if t1 < t2 {
            t = t1;
        } else {
            t = t2;
        };
    } else if valid_t1 {
        t = t1;
    } else if valid_t2 {
        t = t2;
    } else {
        return None;
    }

    let h_t = ro3 + rd3 * t;
    let n = vec3(0., 0., h_t.z).normalize();

    let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz();
    return Some(RayHit {
        distance: t,
        point: ray.origin + ray.direction * t,
        normal,
        material_index,
        u: h_t.x,
        v: h_t.y,
    });
}
