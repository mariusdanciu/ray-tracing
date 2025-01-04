use glam::{Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::ray::{Ray, RayHit};

pub struct Scene {
    pub light_dir: Vec3,
    pub ambient_color: Vec3,
    pub objects: Vec<Object3D>,
    pub materials: Vec<Material>,
    pub difuse: bool,
    pub max_ray_bounces: u8,
}


#[derive(Debug, Copy, Clone)]
pub enum Object3D {
    Sphere {
        position: Vec3,
        radius: f32,
        material_index: usize,
    },

    Triangle {
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        material_index: usize,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum MaterialType {
    Reflective {
        roughness: f32,
    },
    Refractive {
        transparency: f32,
        refraction_index: f32,
    },
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub albedo: Vec3,
    pub kind: MaterialType,
    pub emission_power: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ZERO,
            kind: MaterialType::Reflective { roughness: 1.0 },
            emission_power: 0.0,
        }
    }
}

impl Material {
    fn fresnel(&self, incident: Vec3, normal: Vec3, index: f32) -> f64 {
        let i_dot_n = incident.dot(normal) as f64;
        let mut eta_i = 1.0;
        let mut eta_t = index as f64;
        if i_dot_n > 0.0 {
            eta_i = eta_t;
            eta_t = 1.0;
        }

        let sin_t = eta_i / eta_t * (1.0f64 - i_dot_n * i_dot_n).max(0.0).sqrt();
        if sin_t > 1.0 {
            //Total internal reflection
            return 1.0;
        } else {
            let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
            let cos_i = cos_t.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            return (r_s * r_s + r_p * r_p) / 2.0;
        }
    }
}

impl Object3D {
    pub fn new_sphere(origin: Vec3, radius: f32, material_index: usize) -> Object3D {
        Object3D::Sphere {
            position: origin,
            radius,
            material_index,
        }
    }
    pub fn new_triangle(v1: Vec3, v2: Vec3, v3: Vec3, material_index: usize) -> Object3D {
        Object3D::Triangle {
            v1,
            v2,
            v3,
            material_index,
        }
    }
}

impl Scene {
    pub fn to_rgba(c: Vec4) -> (u8, u8, u8, u8) {
        (
            (c.x * 255.) as u8,
            (c.y * 255.) as u8,
            (c.z * 255.) as u8,
            (c.w + 255.) as u8,
        )
    }
 
    fn trace_ray(& self, ray: Ray) -> Option<RayHit> {
        if self.objects.is_empty() {
            return None;
        }

        let mut closest_object = self.objects[0];
        let mut closest_t = f32::MIN;
        let mut closest_index: usize = usize::MAX;

        for (i, obj) in self.objects.iter().enumerate() {
            if let Some(t) = ray.compute_distance(&ray, &obj) {
                
                if t < 0. && t > closest_t {
                    closest_t = t;
                    closest_object = *obj;
                    closest_index = i;
                }
            }
        }

        if closest_index == usize::MAX {
            return None;
        }

        ray.hit(closest_object, ray, closest_t, closest_index, &self.materials)
    }

    fn color(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light: Vec3,
        contribution: Vec3,
    ) -> Vec3 {
        if depth >= self.max_ray_bounces {
            return light;
        }
        if let Some(hit) = self.trace_ray(ray) {
            match hit.material.kind {
                MaterialType::Reflective { roughness } => {
                    let mut ll = light;
                    if !self.difuse {
                        let light_angle = hit.normal.dot(-self.light_dir).max(0.0);
                        ll += hit.material.albedo * light_angle;
                    } else {
                        ll += hit.material.albedo * hit.material.emission_power;
                    }
                    let r = ray.reflection_ray(hit, roughness, rnd);
                    self.color(r, rnd, depth + 1, ll, contribution * hit.material.albedo)
                }
                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                } => {
                    let mut refraction_color = Vec3::ZERO;
                    let kr = hit
                        .material
                        .fresnel(ray.direction, hit.normal, refraction_index)
                        as f32;

                    if kr < 1.0 {
                        if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                            refraction_color = self.color(
                                refraction_ray,
                                rnd,
                                depth + 1,
                                light + hit.material.albedo * hit.material.emission_power,
                                contribution * hit.material.albedo,
                            );
                        }
                    }

                    let reflection_ray = Ray {
                        origin: hit.point + hit.normal * 0.0001,
                        direction: ray.reflect(-hit.normal),
                    };

                    let reflection_color = self.color(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        light + hit.material.albedo * hit.material.emission_power,
                        contribution * hit.material.albedo,
                    );

                    let mut color = reflection_color * kr + refraction_color * (1.0 - kr);
                    color = color * transparency * hit.material.albedo;
                    color
                }
            }
        } else {
            light + self.ambient_color * contribution
        }
    }

    pub fn pixel(& self, ray: Ray, rnd: &mut ThreadRng) -> Vec4 {
        let mut light = Vec3::ZERO; // BLACK

        let contribution = Vec3::ONE;

        light = self.color(ray, rnd, 0, light, contribution);

        vec4(light.x, light.y, light.z, 1.)
    }

}