use glam::Vec3;

use rand::rngs::ThreadRng;

use crate::light::LightSource;
use crate::objects::{Material, MaterialType};
use crate::ray::{Ray, RayHit, EPSILON};
use crate::scene::Scene;

#[derive(Debug, Clone)]
pub struct RayTracing<'a> {
    pub scene: &'a Scene,
}

impl<'a> RayTracing<'a> {

    pub fn albedo(&self, ray: Ray, rnd: &mut ThreadRng) -> Vec3 {
        let light = Vec3::ZERO; // BLACK

        let contribution = Vec3::ONE;
        if self.scene.diffuse {
            self.color_diffuse(ray, rnd, 0, light, contribution)
        } else {
            self.color(ray, rnd, 0, light, contribution)
        }
    }

    pub fn light(
        &self,
        ray: &Ray,
        hit: &RayHit,
        albedo: Vec3,
        material: &Material,
        obj_index: usize,
    ) -> Vec3 {
        let mut l_acc = Vec3::ZERO;
        for l in &self.scene.lights {
            let k = ray.blinn_phong(&hit, l, albedo, material);
            let light_dis = l.distance(hit.point);
            l_acc += (k / (light_dis * light_dis)) * l.albedo() * l.intensity();
        }
        if self.scene.shadow_casting {
            for l in &self.scene.lights {
                if let Some((hit, idx)) = self.trace_ray(Ray {
                    origin: hit.point + EPSILON * hit.normal,
                    direction: -l.direction(hit.point),
                }) {
                    if idx != obj_index {
                        // in the shadow
                        l_acc *= 0.5;
                    }
                }
            }
        }
        l_acc.powf(0.4166) // Gamma correction
                           // l_acc
    }

    fn trace_ray(&self, ray: Ray) -> Option<(RayHit, usize)> {
        if self.scene.objects.is_empty() {
            return None;
        }

        let mut closest_t = f32::MAX;

        let mut closest_hit: Option<(RayHit, usize)> = None;

        for (idx, obj) in self.scene.objects.iter().enumerate() {
            if let Some(t) = ray.hit(&obj) {
                if t.distance > 0. && t.distance < closest_t {
                    closest_hit = Some((t, idx));
                    closest_t = t.distance;
                }
            }
        }

        closest_hit
    }

    fn color_diffuse(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light_color: Vec3,
        contribution: Vec3,
    ) -> Vec3 {
        if depth >= self.scene.max_ray_bounces {
            return light_color;
        }
        if let Some((hit, obj_index)) = self.trace_ray(ray) {
            let material = self.scene.materials[hit.material_index];
            let mut albedo = material.albedo;

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    if let Some(idx) = material.texture {
                        albedo = self.scene.textures[idx].from_uv(hit.u, hit.v);
                    }

                    let p_light = light_color + material.emission_power * albedo;

                    let r = ray.reflection_ray(
                        hit,
                        roughness,
                        rnd,
                        self.scene.diffuse,
                        self.scene.enable_accumulation,
                    );

                    let reflected_col =
                        self.color_diffuse(r, rnd, depth + 1, p_light, contribution * albedo);

                    reflected_col
                }
                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                    reflectivity,
                } => {
                    let mut refraction_color = Vec3::ZERO;
                    let kr =
                        material.fresnel(ray.direction, hit.normal, refraction_index, reflectivity)
                            as f32;

                    if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                        refraction_color = self.color(
                            refraction_ray,
                            rnd,
                            depth + 1,
                            light_color,
                            contribution * albedo,
                        );
                    }

                    let reflection_ray = Ray {
                        origin: hit.point + EPSILON * hit.normal,
                        direction: ray.reflect(hit.normal),
                    };

                    let p_light = light_color + material.emission_power * albedo;
                    let reflection_color = self.color_diffuse(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        p_light,
                        contribution * albedo,
                    );

                    let color =
                        reflection_color * kr + refraction_color * (1.0 - kr) * transparency;
                    color
                }
            }
        } else {
            light_color + self.scene.ambient_color * contribution
        }
    }

    pub fn color(
        &self,
        ray: Ray,
        rnd: &mut ThreadRng,
        depth: u8,
        light_color: Vec3,
        contribution: Vec3,
    ) -> Vec3 {
        if depth >= self.scene.max_ray_bounces {
            return light_color;
        }
        if let Some((hit, obj_index)) = self.trace_ray(ray) {
            let material = self.scene.materials[hit.material_index];
            let mut albedo = material.albedo;

            match material.kind {
                MaterialType::Reflective { roughness } => {
                    if let Some(idx) = material.texture {
                        albedo = self.scene.textures[idx].from_uv(hit.u, hit.v);
                    }

                    let p_light = self.light(&ray, &hit, albedo, &material, obj_index);

                    let r = ray.reflection_ray(
                        hit,
                        roughness,
                        rnd,
                        self.scene.diffuse,
                        self.scene.enable_accumulation,
                    );

                    let reflected_col =
                        self.color(r, rnd, depth + 1, p_light, contribution * albedo);

                    p_light * (roughness) + p_light * reflected_col * (1. - roughness)
                }

                MaterialType::Refractive {
                    transparency,
                    refraction_index,
                    reflectivity,
                } => {
                    let mut refraction_color = Vec3::ZERO;
                    let kr =
                        material.fresnel(ray.direction, hit.normal, refraction_index, reflectivity)
                            as f32;

                    if let Some(refraction_ray) = ray.refraction_ray(hit, refraction_index) {
                        refraction_color = self.color(
                            refraction_ray,
                            rnd,
                            depth + 1,
                            light_color,
                            contribution * albedo,
                        );
                    }

                    let reflection_ray = Ray {
                        origin: hit.point + EPSILON * hit.normal,
                        direction: ray.reflect(hit.normal),
                    };

                    let p_light = self.light(&ray, &hit, albedo, &material, obj_index);

                    let reflection_color = self.color(
                        reflection_ray,
                        rnd,
                        depth + 1,
                        p_light,
                        contribution * albedo,
                    );

                    let color =
                        reflection_color * kr + refraction_color * (1.0 - kr) * transparency;

                    color * albedo
                }
            }
        } else {
            light_color + self.scene.ambient_color * contribution
        }
    }
}
