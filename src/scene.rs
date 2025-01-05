use glam::{Vec3, Vec4};

use glam::vec4;
use rand::rngs::ThreadRng;

use crate::objects::{Material, MaterialType, Object3D};
use crate::ray::{Ray, RayHit};

pub struct Scene {
    pub light_dir: Vec3,
    pub ambient_color: Vec3,
    pub objects: Vec<Object3D>,
    pub materials: Vec<Material>,
    pub difuse: bool,
    pub max_ray_bounces: u8,
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
        let mut back_facing = false;

        for (i, obj) in self.objects.iter().enumerate() {
            if let Some((t, bf)) = ray.compute_distance(&ray, &obj) {
                
                if t < 0. && t > closest_t {
                    closest_t = t;
                    closest_object = *obj;
                    closest_index = i;
                    back_facing = bf;
                }
            }
        }

        if closest_index == usize::MAX {
            return None;
        }

        ray.hit(closest_object, ray, closest_t, &self.materials, back_facing)
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