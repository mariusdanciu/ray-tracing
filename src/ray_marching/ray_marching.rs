
use glam::{vec2, vec3, Vec3};
use rand::rngs::ThreadRng;

use crate::light::LightSource;
use crate::objects::Object3D;
use crate::ray::{Ray, RayHit};
use crate::scene::Scene;
use crate::utils::geometry;

static MAX_STEPS: usize = 255;
static MAX_DISTANCE: f32 = 40.;
static HIT_PRECISION: f32 = 0.001;

#[derive(Debug, Clone)]
pub struct RayMarching<'a> {
    pub scene: &'a Scene,
}

impl<'a> RayMarching<'a> {
    pub fn sdfs(&self, p: Vec3) -> (f32, usize, Vec3) {
        let mut min_dist = f32::MAX;
        let mut obj_idx = 0usize;

        let mut albedo = Vec3::ZERO;

        for i in self.scene.sdfs.iter() {
            let idx = *i;
            let obj = self.scene.objects[idx];

            match obj {
                Object3D::Cuboid(s)  => {
                    let d = s.sdf(&self.scene, p, &obj);
                    if d.0 < min_dist {
                        min_dist = d.0;
                        albedo = d.1;
                        obj_idx = idx;
                    }
                }
                Object3D::Sphere(s)  => {
                    let d = s.sdf(&self.scene, p, &obj);
                    if d.0 < min_dist {
                        min_dist = d.0;
                        albedo = d.1;
                        obj_idx = idx;
                    }
                }
                Object3D::Plane(s) => {
                    let d = s.sdf(&self.scene, p, &obj);
                    if d.0 < min_dist {
                        min_dist = d.0;
                        albedo = d.1;
                        obj_idx = idx;
                    }
                }

                Object3D::Cylinder(s)  => {
                    let d = s.sdf(&self.scene, p, &obj);
                    if d.0 < min_dist {
                        min_dist = d.0;
                        obj_idx = idx;
                        albedo = d.1
                    }
                }

                Object3D::Union(s) => {
                    let d = s.sdf(self.scene, p);

                    if d.0 < min_dist {
                        min_dist = d.0;
                        albedo = d.1;
                        obj_idx = s.second;
                    }
                }

                Object3D::Substraction(s) => {
                    let d = s.sdf(self.scene, p);

                    if d.0 < min_dist {
                        min_dist = d.0;
                        albedo = d.1;
                        obj_idx = s.second;
                    }
                }
                _ => {}
            }
        }

        (min_dist, obj_idx, albedo)
    }

    fn normal(&self, p: Vec3) -> Vec3 {
        let k = 0.5773 * 0.0005;
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

    fn occlusion(&self, pos: Vec3, nor: Vec3) -> f32 {
        let mut occ = 0.0f32;
        let mut sca = 1.0f32;
        for i in 0..4 {
            let hr = 0.02 + 0.025 * (i * i) as f32;
            let aopos = nor * hr + pos;
            let dd = self.sdfs(aopos);
            occ += -(dd.0 - hr) * sca;
            sca *= 0.85;
        }
        return 1.0 - occ.clamp(0.0, 1.0);
    }

    pub fn light(&self, ray: &Ray, hit: &RayHit, albedo: Vec3) -> Vec3 {
        let mut l_acc = Vec3::ZERO;
        if let Some(material) = self.scene.materials.get(hit.material_index) {
            for l in &self.scene.lights {
                let phong = ray.blinn_phong(&hit, l, albedo, material);
                let light_dis = l.distance(hit.point);
                l_acc += (phong / (light_dis * light_dis)) * l.albedo() * l.intensity();

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

    pub fn march_ray(&self, ray: Ray) -> (bool, f32, usize, Vec3) {
        let mut t = 0.0;

        // March the ray
        let mut i = 0;
        while i < MAX_STEPS {
            if t > MAX_DISTANCE {
                break;
            }
            let (h, obj_idx, albedo) = self.sdfs(ray.origin + ray.direction * t);

            t += h;
            if h < HIT_PRECISION {
                return (true, t, obj_idx, albedo);
            }
            i += 1;
        }
        (false, t, 0, Vec3::ZERO)
    }

    pub fn albedo(&self, ray: Ray, rnd: &mut ThreadRng) -> Vec3 {
        let (hit, t, obj_idx, albedo) = self.march_ray(ray);

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

            let mut color = self.light(&ray, &rayhit, albedo);
            let occ = self.occlusion(hit, n);

            color *= occ;
            color *= ( -0.05*t ).exp();
            color *= 1.0 - geometry::smooth_step(5.0, 30.0, t);
            return color;
        }

        self.scene.ambient_color
    }
}
