extern crate sdl2;

use app::App;
use camera::Camera;
use glam::{vec3, vec4, Vec3, Vec4};
use rand::{rngs::ThreadRng, Rng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sdl2::render::Texture;

mod app;
mod camera;

#[derive(Debug, Clone)]
pub struct Scene {
    light_dir: Vec3,
    ambient_color: Vec3,
    spheres: Vec<Sphere>,
    materials: Vec<Material>,
    accumulated: Vec<Vec4>,
    frame_index: u32,
    difuse: bool,
    max_ray_bounces: u8,
}

#[derive(Debug, Copy, Clone)]
enum MaterialType {
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
    albedo: Vec3,
    kind: MaterialType,
    emission_power: f32,
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

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    position: Vec3,
    radius: f32,
    material_index: usize,
}

impl Sphere {
    fn new(origin: Vec3, radius: f32, material_index: usize) -> Sphere {
        Sphere {
            position: origin,
            radius,
            material_index,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    fn reflect(&self, normal: Vec3) -> Vec3 {
        self.direction - (2. * (self.direction.dot(normal))) * normal
    }

    fn transmission(&self, intersection: RayHit, bias: f32, index: f32) -> Option<Ray> {
        let mut ref_n = intersection.normal;
        let mut eta_t = index;
        let mut eta_i = 1.0;
        let mut i_dot_n = self.direction.dot(intersection.normal);
        if i_dot_n < 0.0 {
            //Outside the surface
            i_dot_n = -i_dot_n;
        } else {
            //Inside the surface; invert the normal and swap the indices of refraction
            ref_n = -intersection.normal;
            eta_i = eta_t;
            eta_t = 1.0;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - (eta * eta) * (1.0 - i_dot_n * i_dot_n);
        if k < 0.0 {
            None
        } else {
            Some(Ray {
                origin: intersection.point + (ref_n * bias),
                direction: (self.direction + i_dot_n * ref_n) * eta - ref_n * k.sqrt(),
            })
        }
    }

    fn reflection_ray(&self, hit: RayHit, roughness: f32, rnd: &mut ThreadRng) -> Ray {
        let mut direction = self.direction;
        if roughness < 1. {
            direction = self
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
            let sphere_random = vec3(
                rnd.gen_range(-1.0..1.0),
                rnd.gen_range(-1.0..1.0),
                rnd.gen_range(-1.0..1.0),
            )
            .normalize();

            direction = -(hit.normal + sphere_random).normalize();
        }
        Ray {
            origin: hit.point + hit.normal * 0.0001,
            direction,
        }
    }

    pub fn refraction_ray(&self, hit: RayHit, refraction_index: f32) -> Option<Ray> {
        self.transmission(hit, 0.001, refraction_index)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    object_index: usize,
    distance: f32,
    point: Vec3,
    normal: Vec3,
    material: Material,
}

#[derive(Debug, Copy, Clone)]
struct Chunk {
    size: usize,
    pixel_offset: usize,
}

impl Scene {
    fn to_rgba(c: Vec4) -> (u8, u8, u8, u8) {
        (
            (c.x * 255.) as u8,
            (c.y * 255.) as u8,
            (c.z * 255.) as u8,
            (c.w + 255.) as u8,
        )
    }

    fn trace_ray(&mut self, ray: Ray) -> Option<RayHit> {
        // (bx^2 + by^2)t^2 + (2(axbx + ayby))t + (ax^2 + ay^2 - r^2) = 0
        // where
        // a = ray origin
        // b = ray direction
        // r = radius
        // t = hit distance

        if self.spheres.is_empty() {
            return None;
        }

        let mut closest_sphere = &self.spheres[0];
        let mut closest_t = f32::MIN;
        let mut closest_index: usize = usize::MAX;

        for (i, sphere) in self.spheres.iter().enumerate() {
            let origin = ray.origin - sphere.position;

            let a = ray.direction.dot(ray.direction);
            let b = 2. * origin.dot(ray.direction);
            let c = origin.dot(origin) - sphere.radius * sphere.radius;

            let disc = b * b - 4. * a * c;

            if disc < 0.0 {
                continue;
            }

            // closest to ray origin
            let t = (-b + disc.sqrt()) / (2.0 * a);
            //let t1 = (-b + disc.sqrt()) / (2.0 * a);

            if t < 0. && t > closest_t {
                closest_t = t;
                closest_sphere = sphere;
                closest_index = i;
            }
        }

        if closest_index == usize::MAX {
            return None;
        }

        let origin = ray.origin - closest_sphere.position; // translation
        let hit_point = origin + ray.direction * closest_t;

        let normal = hit_point.normalize();

        let material = self.materials[self.spheres[closest_index].material_index];
        Some(RayHit {
            object_index: closest_index,
            distance: closest_t,
            point: hit_point + closest_sphere.position, // translation cancel
            normal,
            material,
        })
    }

    fn color(
        &mut self,
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
                    self.color(
                        r,
                        rnd,
                        depth + 1,
                        ll,
                        contribution * hit.material.albedo,
                    )
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

    fn pixel(&mut self, ray: Ray, rnd: &mut ThreadRng) -> Vec4 {
        let mut light = Vec3::ZERO; // BLACK

        let contribution = Vec3::ONE;

        light = self.color(ray, rnd, 0, light, contribution);

        /*         for i in 0..self.max_ray_bounces {
                   if let Some(hit) = self.trace_ray(r) {
                       if !self.difuse {
                           let light_angle = hit.normal.dot(-self.light_dir).max(0.0);
                           light += hit.material.albedo * light_angle;
                       } else {
                           light += hit.material.albedo * hit.material.emission_power;
                       }

                       contribution *= hit.material.albedo;

                       match hit.material.kind {
                           MaterialType::Reflective { roughness } => {
                               r = ray.reflection_ray(hit, roughness, rnd)
                           }
                           MaterialType::Refractive {
                               transparency,
                               refraction_index,
                           } => {
                               let refraction_ray = ray.refraction_ray(hit, refraction_index);
                               let reflection_ray = ray.reflect(hit.normal);

                               let kr = hit
                                   .material
                                   .fresnel(ray.direction, hit.normal, refraction_index)
                                   as f32;
                           }
                       }
                   } else {
                       light += self.ambient_color * contribution;
                       break;
                   }
               }
        */
        vec4(light.x, light.y, light.z, 1.)
    }

    fn render_chunk(
        scene: &mut Scene,
        camera: &Camera,
        rnd: &mut ThreadRng,
        chunk: Chunk,
        bytes: &mut [u8],
    ) {
        let mut i = 0;

        for pos in 0..chunk.size {
            let ray_dir = camera.ray_directions[pos + chunk.pixel_offset];

            let vcolor = scene.pixel(
                Ray {
                    origin: camera.position,
                    direction: ray_dir,
                },
                rnd,
            );

            scene.accumulated[pos] += vcolor;

            let mut accumulated = scene.accumulated[pos];
            accumulated /= scene.frame_index as f32;

            accumulated = accumulated.clamp(Vec4::ZERO, Vec4::ONE);

            let color = Scene::to_rgba(accumulated);
            bytes[i] = color.0;
            bytes[i + 1] = color.1;
            bytes[i + 2] = color.2;
            bytes[i + 3] = color.3;

            i += 4;
        }
    }

    fn render_par(
        texture: &mut Texture,
        camera: &Camera,
        scene: &mut Scene,
        time_step: f32,
        updated: bool,
        rnd: &mut ThreadRng,
    ) -> Result<(), String> {
        let w = camera.width;
        let h = camera.height;

        if updated {
            scene.accumulated = vec![Vec4::ZERO; w * h];
            scene.frame_index = 1;
        }

        let num_chunks = 10;
        let mut img: Vec<u8> = vec![0; w * h * 4];

        let img_len = img.len();
        let img_chunk_size = (img_len / (num_chunks * 4)) * 4;

        let chunks: Vec<(usize, &mut [u8])> = img.chunks_mut(img_chunk_size).enumerate().collect();

        let col: Vec<Scene> = chunks
            .into_par_iter()
            .map(|e| {
                let mut rnd = rand::thread_rng();
                let buf_len = e.1.len();

                let acc_size = buf_len / 4;

                let offset = e.0 * acc_size;

                let mut acc = vec![Vec4::ZERO; acc_size];
                acc.copy_from_slice(&scene.accumulated[offset..(offset + acc_size)]);

                let mut s = Scene {
                    light_dir: scene.light_dir,
                    ambient_color: scene.ambient_color,
                    spheres: scene.spheres.clone(),
                    materials: scene.materials.clone(),
                    accumulated: acc,
                    frame_index: scene.frame_index,
                    difuse: scene.difuse,
                    max_ray_bounces: scene.max_ray_bounces,
                };

                let chunk = Chunk {
                    size: acc_size,
                    pixel_offset: offset,
                };
                Scene::render_chunk(&mut s, camera, &mut rnd, chunk, e.1);
                s
            })
            .collect();

        let mut offset = 0;
        for c in col {
            let len = c.accumulated.len();
            scene.accumulated[offset..offset + len].copy_from_slice(c.accumulated.as_slice());
            offset += len;
        }

        texture
            .update(None, img.as_slice(), w * 4)
            .map_err(|e| e.to_string())?;

        scene.frame_index += 1;

        Ok(())
    }
}
pub fn main() -> Result<(), String> {
    let mut scene1 = Scene {
        max_ray_bounces: 5,
        frame_index: 1,
        light_dir: vec3(-1., -1., -1.).normalize(),
        ambient_color: vec3(0., 0., 0.0),
        accumulated: vec![],
        difuse: true,
        spheres: vec![
            Sphere::new(Vec3::new(0., 0., 0.), 0.5, 0),
            Sphere::new(Vec3::new(0., -100.5, 0.), 100., 1),
            Sphere::new(Vec3::new(10., 3., -14.), 10.0, 2),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0., 0.5, 0.7),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.5,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.4, 0.4, 0.4),
                kind: MaterialType::Reflective { roughness: 1.0 },
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.8, 0.5, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 10.0,
                ..Default::default()
            },
        ],
    };

    let mut scene2 = Scene {
        max_ray_bounces: 5,
        frame_index: 1,
        light_dir: vec3(-1., -1., -1.).normalize(),
        ambient_color: vec3(0.0, 0.0, 0.0),
        accumulated: vec![],
        difuse: true,
        spheres: vec![
            Sphere::new(Vec3::new(0., 0., -0.5), 0.5, 0),
            Sphere::new(Vec3::new(0., -100.5, 0.), 100., 1),
            Sphere::new(Vec3::new(0.5, 0.0, 1.0), 0.5, 2),
            Sphere::new(Vec3::new(10., 3., -14.), 10.0, 3),
        ],
        materials: vec![
            Material {
                albedo: Vec3::new(0.3, 0.0, 1.0),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 1.8,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.9, 0.9, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 0.2,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(1.0, 1.0, 1.0),
                kind: MaterialType::Refractive {
                    transparency: 1.0,
                    refraction_index: 0.97,
                },
                emission_power: 0.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.8, 0.5, 0.2),
                kind: MaterialType::Reflective { roughness: 1.0 },
                emission_power: 10.0,
                ..Default::default()
            },
        ],
    };
    App::run(&mut scene2, Scene::render_par)
}
