use core::f32;

use glam::{vec3, vec4, Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit},
};

use super::geometry;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cylinder {
    pub position: Vec3,
    pub radius: f32,
    pub height: f32,
    pub rotation_axis: Vec3,
    pub material_index: usize,
    transform: Mat4,
    inv_transform: Mat4,
}

impl Cylinder {
    pub fn new(
        position: Vec3,
        height: f32,
        rotation_axis: Vec3,
        radius: f32,
        material_index: usize,
    ) -> Object3D {
        Object3D::Cylinder(
            Cylinder {
                position,
                radius,
                height,
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
            * Mat4::from_rotation_z(self.rotation_axis.z * geometry::DEGREES)
            * Mat4::from_scale(vec3(self.radius, self.radius, self.height));
        self.transform = t;
        self.inv_transform = t.inverse();
        *self
    }
}

impl Intersection for Cylinder {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        let rd3 = (self.inv_transform
            * vec4(ray.direction.x, ray.direction.y, ray.direction.z, 0.))
        .xyz();
        let ro3 = (self.inv_transform * vec4(ray.origin.x, ray.origin.y, ray.origin.z, 1.)).xyz();

        let rd = rd3.xy();
        let ro = ro3.xy();

        let a = rd.dot(rd);
        let b = 2.0 * ro.dot(rd);
        let c = ro.dot(ro) - 1.;

        let disc = b * b - 4.0 * a * c;

        if disc > 0.0 {
            let t1 = (-b - disc.sqrt()) / (2.0 * a);

            let h_t1 = ro3 + rd3 * t1;

            let valid_t1 = h_t1.z.abs() < 0.5;

            if valid_t1 {
                let n = vec3(h_t1.x, h_t1.y, 0.0);
                let normal = (self.transform * vec4(n.x, n.y, n.z, 0.0))
                    .xyz()
                    .normalize();
                let poi = ray.origin + ray.direction * t1;

                let u = (h_t1.y / h_t1.x).atan(); // atan2 covers edge cases

                let v = h_t1.z * 2.;

                return Some(RayHit {
                    distance: t1,
                    point: poi,
                    normal,
                    material_index: self.material_index,
                    u: u,
                    v: v,
                });
            }
        }

        let t1 = (ro3.z - 0.5) / -rd3.z;

        let t2 = (ro3.z + 0.5) / -rd3.z;

        let h_t1 = ro3 + rd3 * t1;
        let h_t2 = ro3 + rd3 * t2;

        let valid_t1 = h_t1.xy().dot(h_t1.xy()) < 1.;
        let valid_t2 = h_t2.xy().dot(h_t2.xy()) < 1.;

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
        let n = vec3(0., 0., h_t.z);

        let normal = (self.transform * vec4(n.x, n.y, n.z, 0.0))
            .xyz()
            .normalize();
        Some(RayHit {
            distance: t,
            point: ray.origin + ray.direction * t,
            normal,
            material_index: self.material_index,
            u: h_t.x,
            v: h_t.y,
        })
    }
}
