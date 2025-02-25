use glam::{vec2, vec3, Vec3};
use rand::rngs::ThreadRng;

use crate::light::LightSource;
use crate::objects::{Material, Object3D};
use crate::ray::{Ray, RayHit, EPSILON};
use crate::scene::Scene;

static MAX_STEPS: usize = 300;
static MAX_DISTANCE: f32 = 100.;
static HIT_PRECISION: f32 = 0.001;

pub struct RayMarching<'a> {
    pub scene: &'a Scene,
}

impl<'a> RayMarching<'a> {
    pub fn mix(x: f32, y: f32, a: f32) -> f32 {
        x * (1. - a) + y * a
    }
    pub fn smooth_union(d1: f32, d2: f32, k: f32) -> f32 {
        let h = (0.5 + 0.5 * (d2 - d1) / k).clamp(0., 1.);
        return RayMarching::mix(d2, d1, h) - k * h * (1. - h);
    }

    pub fn sdfs(&self, p: Vec3) -> (f32, i32) {
        let mut min_dist = f32::MAX;
        let mut obj_idx = -1;

        let mut sphere_dist = f32::MAX;
        let mut plane_dist = f32::MAX;
        for (idx, obj) in self.scene.objects.iter().enumerate() {
            match obj {
                Object3D::Sphere(s) => {
                    let d = (p - s.position).length() - s.radius;
                    sphere_dist = d;
                    if d < min_dist {
                        min_dist = d;
                        obj_idx = idx as i32;
                    }
                }
                Object3D::Plane(s) => {
                    let d = (p - s.point).dot(s.normal);
                    plane_dist = d;
                    if d < min_dist {
                        min_dist = d;
                        obj_idx = idx as i32;
                    }
                }
                _ => {}
            }
        }

        //let m = sphere_dist.min(plane_dist);
        let o = RayMarching::smooth_union(sphere_dist, plane_dist, 0.5);

        (o, obj_idx)
    }

    fn normal(&self, p: Vec3) -> Vec3 {
        let k = 0.0001;
        let e = vec2(1., -1.);

        let xyy = vec3(e.x, e.y, e.y);
        let yyx = vec3(e.y, e.y, e.x);
        let yxy = vec3(e.y, e.x, e.y);
        let xxx = vec3(e.x, e.x, e.x);

        (xyy * self.sdfs(p + xyy * k).0
            + yyx * self.sdfs(p + yyx * k).0
            + yxy * self.sdfs(p + yxy * k).0
            + xxx * self.sdfs(p + xxx * k).0)
            .normalize()
    }

    pub fn light(&self, ray: &Ray, hit: &RayHit) -> Vec3 {
        let mut l_acc = Vec3::ZERO;
        if let Some(material) = self.scene.materials.get(hit.material_index) {
            for l in &self.scene.lights {
                let k = ray.blinn_phong(&hit, l, material.albedo, material);
                let light_dis = l.distance(hit.point);
                l_acc += (k / (light_dis * light_dis)) * l.albedo() * l.intensity();

                // let s = self.soft_shadow(
                //     hit.point + hit.normal * 0.01,
                //     -l.direction(hit.point),
                //     0.5,
                //     0.04,
                //     4.0,
                // );
                // l_acc *= s;
            }
        }
        l_acc.powf(0.4545)
    }

    fn soft_shadow(&self, ro: Vec3, rd: Vec3, k: f32, mint: f32, maxt: f32) -> f32 {
        let (hit, t, obj_idx) = self.march_ray(Ray {
            origin: ro,
            direction: rd,
        });

        if hit {
            return 0.3;
        }
        1.
    }

    pub fn march_ray(&self, ray: Ray) -> (bool, f32, i32) {
        let mut h = 1.;
        let mut t = 1.;
        let mut obj_idx = -1i32;
        // March the ray
        let mut i = 0;
        let mut hit = false;
        while i < MAX_STEPS {
            if t > MAX_DISTANCE {
                break;
            }
            (h, obj_idx) = self.sdfs(ray.origin + ray.direction * t);

            t += h;
            if h < HIT_PRECISION {
                hit = true;
                break;
            }
            i += 1;
        }
        (hit, t, obj_idx)
    }

    pub fn albedo(&self, ray: Ray, rnd: &mut ThreadRng) -> Vec3 {
        let (hit, t, obj_idx) = self.march_ray(ray);

        if hit {
            let hit = ray.origin + ray.direction * t;
            let n = self.normal(hit);

            let mat = self.scene.objects[obj_idx as usize].material_index();

            let rayhit = RayHit {
                distance: t,
                point: hit,
                normal: n,
                material_index: mat,
                u: 0.0,
                v: 0.0,
            };

            let mut color = self.light(&ray, &rayhit);

            return color;
        }

        self.scene.ambient_color
    }
}
