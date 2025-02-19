use std::time::Instant;

use glam::{Vec3, Vec4Swizzles};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit},
};

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub v1: Vec3,
    pub v2: Vec3,
    pub v3: Vec3,
    pub material_index: usize,
}

impl Triangle {
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3, material_index: usize) -> Object3D {
        Object3D::Triangle(Triangle {
            v1,
            v2,
            v3,
            material_index,
        })
    }
}

impl Intersection for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        let edge_ab = self.v2 - self.v1;
        let edge_ac = self.v3 - self.v1;

        let normal = edge_ab.cross(edge_ac);

        let denom = ray.direction.dot(normal);

        if denom.abs() < 1e-6 {
            return None;
        }

        let t = (self.v1 - ray.origin).dot(normal) / denom;

        let hit_point = ray.origin + ray.direction * t;

        let edge_bc = self.v3 - self.v2;
        let edge_ca = self.v1 - self.v3;

        let a_to_hit = hit_point - self.v1;
        let b_to_hit = hit_point - self.v2;
        let c_to_hit = hit_point - self.v3;

        let a_test = edge_ab.cross(a_to_hit);
        let b_test = edge_bc.cross(b_to_hit);
        let c_test = edge_ca.cross(c_to_hit);

        let a_n = a_test.dot(normal) > 0.;
        let b_n = b_test.dot(normal) > 0.;
        let c_n = c_test.dot(normal) > 0.;

        let inside = a_n && b_n && c_n;
        let back_face = !a_n && !b_n && !c_n;

        if inside || back_face {
            let n = normal;

            let sign = -ray.direction.dot(n).signum();

            // Triangle ABP
            let abp_area = edge_ab.cross(a_to_hit);
            let u = abp_area.dot(n) ;

            // Triangle CAP
            let cap_area = edge_ca.cross(c_to_hit);
            let v = cap_area.dot(n);

            return Some(RayHit {
                distance: t,
                point: hit_point,
                normal: (sign * n).normalize(),
                material_index: self.material_index,
                u,
                v,
            });
        }

        return None;

        // Moller - Trombore
        // -----------------
        // let e1 = self.v2 - self.v1;
        // let e2 = self.v3 - self.v1;
        // let ray_cross_e2 = ray.direction.cross(e2);
        // let det = e1.dot(ray_cross_e2);

        // if det > -f32::EPSILON && det < f32::EPSILON {
        //     return None; // This ray is parallel to this triangle.
        // }
        // let back_facing = det < f32::EPSILON;

        // let inv_det = 1.0 / det;
        // let s = ray.origin - self.v1;
        // let u = inv_det * s.dot(ray_cross_e2);
        // if u < 0.0 || u > 1.0 {
        //     return None;
        // }

        // let s_cross_e1 = s.cross(e1);

        // let v = inv_det * ray.direction.dot(s_cross_e1);
        // if v < 0.0 || u + v > 1.0 {
        //     return None;
        // }

        // // At this stage we can compute t to find out where the intersection point is on the line.
        // let t = inv_det * e2.dot(s_cross_e1);

        // if t > f32::EPSILON {
        //     let hit_point = ray.origin + ray.direction * t;

        //     let mut normal = (self.v2 - self.v1).cross(self.v3 - self.v1).normalize();
        //     if back_facing {
        //         normal = -normal;
        //     }

        //     return Some(RayHit {
        //         distance: t,
        //         point: hit_point,
        //         normal,
        //         material_index: self.material_index,
        //         u,
        //         v,
        //     });
        // } else {
        //     // This means that there is a line intersection but not a ray intersection.
        //     return None;
        // }
    }
}
