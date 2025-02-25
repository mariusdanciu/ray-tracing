use core::f32;

use glam::{vec3, vec4, Mat4, Vec3, Vec3Swizzles, Vec4Swizzles};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit},
};

use super::geometry;

#[derive(Debug, Clone, Copy, Default)]
pub struct Cone {
    pub position: Vec3,
    pub radius: f32,
    pub height: f32,
    pub rotation_axis: Vec3,
    pub material_index: usize,
    transform: Mat4,
    inv_transform: Mat4,
}

impl Cone {
    pub fn new(
        position: Vec3,
        radius: f32,
        height: f32,
        rotation_axis: Vec3,
        material_index: usize,
    ) -> Object3D {
        Object3D::Cone(
            Cone {
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

impl Intersection for Cone {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        let rd3 = (self.inv_transform
            * vec4(ray.direction.x, ray.direction.y, ray.direction.z, 0.))
        .xyz();
        let ro3 = (self.inv_transform * vec4(ray.origin.x, ray.origin.y, ray.origin.z, 1.)).xyz();

        let rd = rd3.xy();
        let ro = ro3.xy();

        let a = rd.dot(rd) - rd3.z * rd3.z;
        let b = 2.0 * (ro.dot(rd) - ro3.z * rd3.z);
        let c = ro.dot(ro) - ro3.z * ro3.z;

        let disc = b * b - 4.0 * a * c;

        let mut cone: Option<RayHit> = None;

        if disc > 0.0 {
            let t1 = (-b - disc.sqrt()) / (2.0 * a);

            let h_t = ro3 + rd3 * t1;

            let valid_t1 = h_t.z > 0. && h_t.z < 1.;

            if valid_t1 {
                let poi = ray.origin + ray.direction * t1;
                let n = vec3(h_t.x, h_t.y, -(h_t.x * h_t.x + h_t.y * h_t.y).sqrt());

                let normal = (self.transform * vec4(n.x, n.y, n.z, 0.0))
                    .xyz()
                    .normalize();

                let u = (h_t.y / h_t.x).atan(); // atan2 covers edge cases

                let v = h_t.z * 2. + 1.;

                cone = Some(RayHit {
                    distance: t1,
                    point: poi,
                    normal,
                    material_index: self.material_index,
                    u: u,
                    v: v,
                });
            }
        }

        let t1 = (ro3.z - 1.) / -rd3.z;

        let h_t1 = ro3 + rd3 * t1;

        let valid_t1 = h_t1.xy().dot(h_t1.xy()) < 1.;

        let mut t = 0.;

        if valid_t1 {
            t = t1;

            if let Some(cs) = cone {
                if cs.distance < t1 {
                    return cone;
                }
            }

            let h_t = ro3 + rd3 * t;
            let n = vec3(0., 0., 1.);

            let normal = (self.transform * vec4(n.x, n.y, n.z, 0.0)).xyz();
            return Some(RayHit {
                distance: t,
                point: ray.origin + ray.direction * t,
                normal,
                material_index: self.material_index,
                u: h_t.x,
                v: h_t.y,
            });
        }

        cone
    }
}
