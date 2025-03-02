use core::f32;

use glam::{mat3, vec2, vec3, vec4, Vec3, Vec3Swizzles, Vec4Swizzles};
use rand::rngs::ThreadRng;
use sdl2::libc::MNT_ASYNC;

use crate::light::LightSource;
use crate::objects::Object3D;
use crate::ray::{Ray, RayHit, RayMarchingHit};
use crate::scene::Scene;
use crate::utils::geometry;

static MAX_STEPS: usize = 255;
static MAX_DISTANCE: f32 = 40.;
static HIT_PRECISION: f32 = 0.001;
static INV_PI: f32 = 1. / f32::consts::PI;

#[derive(Debug, Clone)]
pub struct RayMarching<'a> {
    pub scene: &'a Scene,
}

impl<'a> RayMarching<'a> {
    pub fn sdfs(&self, ray: &Ray, t: f32) -> (usize, RayMarchingHit) {
        let mut min_dist = f32::MAX;
        let mut obj_idx = 0usize;

        let mut albedo = Vec3::ZERO;
        let mut tray = Ray::new();

        for i in self.scene.sdfs.iter() {
            let idx = *i;
            let obj = self.scene.objects[idx];

            match obj {
                Object3D::Cuboid(s) => {
                    let d = s.sdf(&self.scene, &ray, t, &obj);
                    if d.distance < min_dist {
                        min_dist = d.distance;
                        albedo = d.albedo;
                        tray = d.transformed_ray;
                        obj_idx = idx;
                    }
                }
                Object3D::Sphere(s) => {
                    let d = s.sdf(&self.scene, ray, t, &obj);
                    if d.distance < min_dist {
                        min_dist = d.distance;
                        albedo = d.albedo;
                        tray = d.transformed_ray;
                        obj_idx = idx;
                    }
                }
                Object3D::Plane(s) => {
                    let d = s.sdf(&self.scene, ray, t, &obj);
                    if d.distance < min_dist {
                        min_dist = d.distance;
                        albedo = d.albedo;
                        tray = d.transformed_ray;
                        obj_idx = idx;
                    }
                }

                Object3D::Cylinder(s) => {
                    let d = s.sdf(&self.scene, ray, t, &obj);
                    if d.distance < min_dist {
                        min_dist = d.distance;
                        albedo = d.albedo;
                        tray = d.transformed_ray;
                        obj_idx = idx;
                    }
                }

                Object3D::Union(s) => {
                    let d = s.sdf(self.scene, ray, t);

                    if d.distance < min_dist {
                        min_dist = d.distance;
                        albedo = d.albedo;
                        obj_idx = s.second;
                        tray = d.transformed_ray;
                    }
                }

                Object3D::Substraction(s) => {
                    let d = s.sdf(self.scene, ray, t);

                    if d.distance < min_dist {
                        min_dist = d.distance;
                        albedo = d.albedo;
                        obj_idx = s.second;
                        tray = d.transformed_ray;
                    }
                }
                _ => {}
            }
        }

        (
            obj_idx,
            RayMarchingHit {
                distance: min_dist,
                albedo,
                transformed_ray: tray,
            },
        )
    }

    fn normal(&self, p: Vec3) -> Vec3 {
        let k = 0.5773 * 0.0005;
        let e = vec2(1., -1.);

        let xyy = vec3(e.x, e.y, e.y);
        let yyx = vec3(e.y, e.y, e.x);
        let yxy = vec3(e.y, e.x, e.y);
        let xxx = vec3(e.x, e.x, e.x);

        let r_xyy = Ray {
            origin: p,
            direction: xyy,
        };
        let r_yyx = Ray {
            origin: p,
            direction: yyx,
        };
        let r_yxy = Ray {
            origin: p,
            direction: yxy,
        };
        let r_xxx = Ray {
            origin: p,
            direction: xxx,
        };
        (xyy * self.sdfs(&r_xyy, k).1.distance
            + yyx * self.sdfs(&r_yyx, k).1.distance
            + yxy * self.sdfs(&r_yxy, k).1.distance
            + xxx * self.sdfs(&r_xxx, k).1.distance)
            .normalize()
    }

    fn occlusion(&self, pos: Vec3, nor: Vec3) -> f32 {
        let mut occ = 0.0f32;
        let mut sca = 1.0f32;
        for i in 0..3 {
            let hr = 0.02 + 0.025 * (i * i) as f32;
            //let aopos = nor * hr + pos;
            let dd = self.sdfs(
                &Ray {
                    origin: pos,
                    direction: nor,
                },
                hr,
            );
            occ += -(dd.1.distance - hr) * sca;
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

    pub fn march_ray(&self, ray: &Ray) -> Option<(usize, RayMarchingHit)> {
        let mut t = 0.0;

        // March the ray
        let mut i = 0;
        while i < MAX_STEPS {
            if t > MAX_DISTANCE {
                break;
            }

            let (obj_idx, rh) = self.sdfs(ray, t);
            t += rh.distance;
            if rh.distance < HIT_PRECISION {
                return Some((obj_idx, RayMarchingHit{
                    distance: t,
                    albedo: rh.albedo,
                    transformed_ray: rh.transformed_ray,
                }));
            }
            i += 1;
        }
        None
    }

    pub fn albedo(&self, ray: &Ray, rnd: &mut ThreadRng) -> Vec3 {
       // let (hit, t, obj_idx, mut albedo, r) = self.march_ray(ray);
        if let Some((obj_idx, rh)) = self.march_ray(ray) {
            let hit = ray.origin + ray.direction * rh.distance;

            let n = self.normal(hit);

            let obj = self.scene.objects[obj_idx];
            let mat_idx = obj.material_index();
            let mat = self.scene.materials[mat_idx];
            let mut albedo = rh.albedo;

            //let n = (obj.transform().1*vec4(n.x, n.y, n.z, 0.0)).xyz().normalize();
            if let Some(t1) = mat.texture {
                // let hit1 = r.origin + r.direction * t;
                // let u = ((hit1.x * hit1.x + hit1.y * hit1.y) / (hit1.z)).atan();
                // let v = (hit1.y / hit1.x).atan();
                // let tex = &self.scene.textures[t1];
                // albedo = tex.from_uv(v* INV_PI, u* INV_PI);

                let n1: Vec3 = (obj.transform().1 * vec4(n.x, n.y, n.z, 0.0))
                    .xyz()
                    .normalize();
                let hit1 = rh.transformed_ray.origin + rh.transformed_ray.direction * rh.distance;

                let tex = &self.scene.textures[t1];
                albedo = geometry::tri_planar_mapping(hit1, n1, 0.8, 0.5, tex);
            }

            let rayhit = RayHit {
                distance: rh.distance,
                point: hit,
                normal: n,
                material_index: mat_idx,
                u: 0.0,
                v: 0.0,
            };

            let mut color = self.light(&ray, &rayhit, albedo);
            let occ = self.occlusion(hit, n);

            color *= occ;
            // color *= geometry::fog(color, t, vec3(0., 0., 0.), 0.05); //(-0.05 * t).exp();
            color *= 1.0 - geometry::smooth_step(1.0, 20.0, rh.distance);
            return color;
        }

        self.scene.ambient_color
    }
}
