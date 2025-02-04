use glam::{vec3, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    objects::{Material, Object3D},
    scene::Light,
    utils::geometry,
};

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
        self.reflect_vec(self.direction, normal)
    }

    fn reflect_vec(&self, vec: Vec3, normal: Vec3) -> Vec3 {
        vec - (2. * (vec.dot(normal))) * normal
    }

    pub fn blinn_phong(
        &self,
        hit: &RayHit,
        light: &Light,
        color: Vec3,
        material: &Material,
    ) -> Vec3 {
        let coeff = hit.normal.dot(-light.direction);
        let ambience = material.ambience * color;
        let diffuse = material.diffuse * coeff.max(0.) * color;
        let half_angle = (-self.direction - light.direction).normalize();
        let shininess = (hit.normal.dot(half_angle))
            .max(0.)
            .powf(material.shininess);
        let specular = material.specular * shininess * color;

        ambience + diffuse + specular
    }
    pub fn phong(&self, hit: &RayHit, light: &Light, color: Vec3, material: &Material) -> Vec3 {
        let coeff = hit.normal.dot(-light.direction);
        let ambience = material.ambience * color;
        let diffuse = material.diffuse * coeff.max(0.) * color;
        let shininess = (self
            .direction
            .dot(self.reflect_vec(-light.direction, hit.normal)))
        .max(0.)
        .powf(material.shininess);
        let specular = material.specular * shininess * color;

        ambience + diffuse + specular
    }

    pub fn reflection_ray(
        &self,
        hit: RayHit,
        roughness: f32,
        rnd: &mut ThreadRng,
        diffuse: bool,
    ) -> Ray {
        let dir: Vec3;
        if !diffuse {
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

            dir = (hit.normal + rnd).normalize();
        }
        Ray {
            origin: hit.point + hit.normal * EPSILON,
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
            origin: hit.point - EPSILON * normal,
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
        let back_facing = det < f32::EPSILON;

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

        if t > f32::EPSILON {
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

            Object3D::Box {
                position,
                dimension,
                material_index,
            } => self.box_intersection(*dimension, *position, *material_index),
        }
    }

    fn step(&self, a: Vec3, b: Vec3) -> Vec3 {
        let x = if b.x < a.x { 0.0 } else { 1.0 };
        let y = if b.y < a.y { 0.0 } else { 1.0 };
        let z = if b.z < a.z { 0.0 } else { 1.0 };

        return vec3(x, y, z);
    }

    fn box_intersection(
        &self,
        box_size: Vec3,
        position: Vec3,
        material_index: usize,
    ) -> Option<RayHit> {
        let rotation = geometry::rotate_x_mat(-10. * std::f32::consts::PI / 180.)
            * geometry::rotate_y_mat(0. * std::f32::consts::PI / 180.);

        let ray_dir =
            (rotation * Vec4::new(self.direction.x, self.direction.y, self.direction.z, 0.0)).xyz();
        let ray_origin =
            (rotation * Vec4::new(self.origin.x, self.origin.y, self.origin.z, 1.0)).xyz();

        //let ray_dir = self.direction;
        //let ray_origin = self.origin;

        let h_box_size = box_size / 2.;

        let b_min = position - h_box_size;
        let b_max = position + h_box_size;

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
        let mut normal = self.step(vec3(t_near, t_near, t_near), t_enter);

        //normal *= -self.direction.signum();

        let hit_point = ray_origin + ray_dir * t_near;
        Some(RayHit {
            distance: t_near,
            point: hit_point,
            normal,
            material_index,
            ..Default::default()
        })
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

        let t0 = (-b + disc.sqrt()) / (2.0 * a);
        let t1 = (-b - disc.sqrt()) / (2.0 * a);

        let t = t1;

        let hit_point = self.origin + self.direction * t;

        let normal = (hit_point - *position).normalize();
        //   println!(
        //      "origin {} dir{} t0 {} t1 {} h0 {} h1{} N {} position {}",
        //      self.origin, self.direction, t0, t1, h0, h1, normal, *position
        //  );

        Some(RayHit {
            distance: t,
            point: hit_point,
            normal,
            material_index,
            ..Default::default()
        })
    }
}
