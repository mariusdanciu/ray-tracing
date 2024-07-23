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
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    albedo: Vec3,
    roughness: f32,
    metallic: f32,
    emission_color: Vec3,
    emission_power: f32,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: Vec3::ZERO,
            roughness: 1.0,
            metallic: 0.0,
            emission_color: Vec3::ZERO,
            emission_power: 0.0,
        }
    }
}

impl Material {
    fn reflect(&self, incident: Vec3, normal: Vec3) -> Vec3 {
        incident - (2. * (incident.dot(normal))) * normal
    }

    pub fn reflect_ray(&self, ray: Ray, hit: RayHit, rnd: &mut ThreadRng) -> Ray {
        let mut direction = ray.direction;

        if self.roughness < 1. {
            direction = self
                .reflect(
                    direction,
                    hit.normal
                        + self.roughness
                            * vec3(
                                rnd.gen_range(-0.5..0.5),
                                rnd.gen_range(-0.5..0.5),
                                rnd.gen_range(-0.5..0.5),
                            ),
                )
                .normalize();
        } else {
            let sphere_random = vec3(
                rnd.gen_range(-1.0..=1.0),
                rnd.gen_range(-1.0..=1.0),
                rnd.gen_range(-1.0..=1.0),
            )
            .normalize();

            direction = -(hit.normal + sphere_random).normalize();
        }
        Ray {
            origin: ray.origin,
            direction,
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

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    object_index: usize,
    distance: f32,
    point: Vec3,
    normal: Vec3,
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

        Some(RayHit {
            object_index: closest_index,
            distance: closest_t,
            point: hit_point + closest_sphere.position, // translation cancel
            normal,
        })
    }

    fn pixel(&mut self, ray: Ray, rnd: &mut ThreadRng) -> Vec4 {
        let mut light = Vec3::new(0., 0., 0.);

        let mut contribution = Vec3::ONE;
        let mut r = ray.clone();
        let mut factor = 1f32;

        for i in 0..5 {
            if let Some(hit) = self.trace_ray(r) {
                let material = self.materials[self.spheres[hit.object_index].material_index];

                if !self.difuse {
                    let light_angle = hit.normal.dot(-self.light_dir).max(0.0);
                    let color = material.albedo * light_angle;
                    light += color * factor;
                    factor *= 0.5;
                } else {
                    contribution *= material.albedo;
                    light += material.emission_color * material.emission_power;
                }

                r.origin = hit.point + hit.normal * 0.0001;

                r = material.reflect_ray(r, hit, rnd);
            } else {
                if !self.difuse {
                    light += self.ambient_color * factor;
                } else {
                    light += self.ambient_color * contribution;
                }
                break;
            }
        }

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
    let mut scene = Scene {
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
                roughness: 1.0,
                metallic: 0.0,
                emission_color: Vec3::new(1.0, 0.0, 1.),
                emission_power: 0.5,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.4, 0.4, 0.4),
                roughness: 1.0,
                metallic: 0.0,
                ..Default::default()
            },
            Material {
                albedo: Vec3::new(0.8, 0.5, 0.2),
                roughness: 1.,
                metallic: 0.0,
                emission_color: Vec3::new(0.8, 0.5, 0.2),
                emission_power: 10.0,
                ..Default::default()
            },
        ],
    };

    App::run(&mut scene, Scene::render_par)
}
