use glam::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};

use crate::light::{Light, LightSource};

use crate::objects::{Intersection, Material, Object3D};

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

    pub fn reflect_vec(&self, vec: Vec3, normal: Vec3) -> Vec3 {
        vec - (2. * (vec.dot(normal))) * normal
    }

    pub fn blinn_phong(
        &self,
        hit: &RayHit,
        light: &Light,
        color: Vec3,
        material: &Material,
    ) -> Vec3 {
        let coeff = hit.normal.dot(-light.direction(hit.point));
        let ambience = material.ambience * color ;
        let diffuse = material.diffuse * coeff.max(0.) * color;
        let half_angle = (-self.direction - light.direction(hit.point)).normalize();
        let shininess = (hit.normal.dot(half_angle))
            .max(0.)
            .powf(material.shininess);
        let specular = material.specular * shininess * color;

        ambience + diffuse + specular
    }

    pub fn phong(&self, hit: &RayHit, light: &Light, color: Vec3, material: &Material) -> Vec3 {
        let coeff = hit.normal.dot(-light.direction(hit.point));
        let ambience = material.ambience * color;
        let diffuse = material.diffuse * coeff.max(0.) * color;
        let shininess = (self
            .direction
            .dot(self.reflect_vec(-light.direction(hit.point), hit.normal)))
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
        enable_accumulation: bool,
    ) -> Ray {
        let dir: Vec3;
        if !diffuse {
            let factor = if enable_accumulation {
                roughness
                    * vec3(
                        rnd.gen_range(-0.5..0.5),
                        rnd.gen_range(-0.5..0.5),
                        rnd.gen_range(-0.5..0.5),
                    )
            } else {
                Vec3::splat(0.0)
            };

            dir = self.reflect(hit.normal + factor).normalize();
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

    pub fn hit(&self, obj: &Object3D) -> Option<RayHit> {
        match obj {
            Object3D::Sphere(s) => s.intersect(self),
            Object3D::Triangle(s) => s.intersect(self),
            Object3D::Cuboid(s) => s.intersect(self),
            Object3D::Plane(s) => s.intersect(self),
            Object3D::Cylinder(s) => s.intersect(self),
            Object3D::Cone(s) => s.intersect(self),
        }
    }
}
