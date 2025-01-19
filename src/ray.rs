use glam::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};

use crate::objects::Object3D;

pub static EPSILON: f32 = 0.0001_f32;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material_index: usize,
    pub u: f32,
    pub v: f32,
}

impl Default for RayHit {
    fn default() -> Self {
        Self {
            distance: f32::MIN,
            point: Default::default(),
            normal: Default::default(),
            material_index: Default::default(),
            u: 0.,
            v: 0.,
        }
    }
}

impl Ray {
    pub fn reflect(&self, normal: Vec3) -> Vec3 {
        self.direction - (2. * (self.direction.dot(normal))) * normal
    }

    pub fn reflection_ray(&self, hit: RayHit, roughness: f32, rnd: &mut ThreadRng) -> Ray {
        let dir: Vec3;
        if roughness < 1. {
            dir = self
                .reflect(
                    hit.normal
                        + roughness
                            * vec3(
                                rnd.gen_range(-0.5..0.5),
                                rnd.gen_range(-0.5..0.5),
                                rnd.gen_range(-0.5..0.5),
                            ),
                )
                .normalize();
        } else {
            let rnd = vec3(
                rnd.gen_range(-1.0..1.0),
                rnd.gen_range(-1.0..1.0),
                rnd.gen_range(-1.0..1.0),
            );

            dir = -(hit.normal + rnd).normalize();
        }
        Ray {
            origin: hit.point + hit.normal * 0.0001,
            direction: dir,
        }
    }

    pub fn refraction_ray(&self, hit: RayHit, refraction_index: f32) -> Option<Ray> {
        let mut normal = hit.normal;
        let mut eta_t = refraction_index;
        let mut eta_i = 1.0;
        let mut c1 = self.direction.dot(hit.normal);

        if c1 < 0.0 {
            c1 = -c1;
        } else {
            normal = -normal;
            eta_i = eta_t;
            eta_t = 1.;
        }
        let eta = eta_i / eta_t;

        let k = 1. - eta * eta * (1. - c1 * c1);
        if k < 0. {
            return None;
        }

        let c2 = k.sqrt();
        let direction = eta * self.direction + normal * (eta * c1 - c2);

        Some(Ray {
            origin: hit.point + EPSILON * normal,
            direction: direction,
        })
    }

    fn moller_trumbore_intersection(
        &self,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        material_index: usize,
    ) -> Option<RayHit> {
        let e1 = v2 - v1;
        let e2 = v3 - v1;
        let ray_cross_e2 = self.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None; // This ray is parallel to this triangle.
        }
        let back_facing = det > f32::EPSILON;

        let inv_det = 1.0 / det;
        let s = self.origin - v1;
        let u = inv_det * s.dot(ray_cross_e2);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let s_cross_e1 = s.cross(e1);

        let v = inv_det * self.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);

        if t < f32::EPSILON {
            let hit_point = self.origin + self.direction * t;

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

    pub fn hit(&self, obj: &Object3D) -> Option<RayHit> {
        match obj {
            Object3D::Sphere {
                position,
                radius,
                material_index,
            } => self.sphere_intersection(position, radius, *material_index),

            Object3D::Triangle {
                v1,
                v2,
                v3,
                material_index,
            } => self.moller_trumbore_intersection(*v1, *v2, *v3, *material_index),
        }
    }

    fn sphere_intersection(
        &self,
        position: &Vec3,
        radius: &f32,
        material_index: usize,
    ) -> Option<RayHit> {
        // (bx^2 + by^2 + bz^2)t^2 + (2(axbx + ayby + azbz))t + (ax^2 + ay^2 + az^2 - r^2) = 0
        // where
        // a = ray origin
        // b = ray direction
        // r = radius
        // t = hit distance

        let origin = self.origin - *position;

        let a = self.direction.dot(self.direction);
        let b = 2. * origin.dot(self.direction);
        let c = origin.dot(origin) - radius * radius;

        let disc = b * b - 4. * a * c;

        if disc < 0.0 {
            return None;
        }

        // closest to ray origin
        let t0 = (-b + disc.sqrt()) / (2.0 * a);
        let t1 = (-b - disc.sqrt()) / (2.0 * a);
        
        let t = t1;
        

        let hit_point = self.origin + self.direction * t;

        let normal = (hit_point - *position).normalize();

        Some(RayHit {
            distance: t,
            point: hit_point,
            normal,
            material_index,
            ..Default::default()
        })
    }
}
