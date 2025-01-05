use glam::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};

use crate::scene::{Material, Object3D};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub material: Material,
}

impl Ray {
    pub fn reflect(&self, normal: Vec3) -> Vec3 {
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

    pub fn reflection_ray(&self, hit: RayHit, roughness: f32, rnd: &mut ThreadRng) -> Ray {
        let dir: Vec3;
        if roughness < 1. {
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
            let sphere_random = vec3(
                rnd.gen_range(-1.0..1.0),
                rnd.gen_range(-1.0..1.0),
                rnd.gen_range(-1.0..1.0),
            )
            .normalize();

            dir = -(hit.normal + sphere_random).normalize();
        }
        Ray {
            origin: hit.point + hit.normal * 0.0001,
            direction: dir,
        }
    }

    pub fn refraction_ray(&self, hit: RayHit, refraction_index: f32) -> Option<Ray> {
        self.transmission(hit, 0.001, refraction_index)
    }

    fn moller_trumbore_intersection(&self, ray: &Ray, v1: Vec3, v2: Vec3, v3: Vec3) -> Option<(f32, bool)> {
        let e1 = v2 - v1;
        let e2 = v3 - v1;
        let ray_cross_e2 = ray.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);
        

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None; // This ray is parallel to this triangle.
        }
        let back_facing = det > f32::EPSILON;

        let inv_det = 1.0 / det;
        let s = ray.origin - v1;
        let u = inv_det * s.dot(ray_cross_e2);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let s_cross_e1 = s.cross(e1);

        let v = inv_det * ray.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);

        if t < f32::EPSILON {
            return Some((t, back_facing));
        } else {
            // This means that there is a line intersection but not a ray intersection.
            return None;
        }
    }
    pub fn compute_distance(&self, ray: &Ray, obj: &Object3D) -> Option<(f32, bool)> {
        match obj {
            Object3D::Sphere {
                position, radius, ..
            } => {
                // (bx^2 + by^2 + bz^2)t^2 + (2(axbx + ayby + azbz))t + (ax^2 + ay^2 + az^2 - r^2) = 0
                // where
                // a = ray origin
                // b = ray direction
                // r = radius
                // t = hit distance

                let origin = ray.origin - *position;

                let a = ray.direction.dot(ray.direction);
                let b = 2. * origin.dot(ray.direction);
                let c = origin.dot(origin) - radius * radius;

                let disc = b * b - 4. * a * c;

                if disc < 0.0 {
                    return None;
                }

                // closest to ray origin
                let t = (-b + disc.sqrt()) / (2.0 * a);

                Some((t, false))
            }

            Object3D::Triangle { v1, v2, v3, .. } => {
                self.moller_trumbore_intersection(ray, *v1, *v2, *v3)
            }
        }
    }

    pub fn hit(
        &self,
        obj: Object3D,
        ray: Ray,
        distance: f32,
        materials: &Vec<Material>,
        back_facing: bool
    ) -> Option<RayHit> {
        match obj {
            Object3D::Sphere {
                position,
                radius: _,
                material_index,
            } => {
                let origin = ray.origin - position; // translation
                let hit_point = origin + ray.direction * distance;

                let normal = hit_point.normalize();

                let material = materials[material_index];
                Some(RayHit {
                    point: hit_point + position, // translation cancel
                    normal,
                    material,
                })
            }
            Object3D::Triangle {
                v1,
                v2,
                v3,
                material_index,
            } => {
                let hit_point = ray.origin + ray.direction * distance;

                let mut normal = (v2 - v1).cross(v3 - v1).normalize();
                if back_facing {
                    normal = -normal; 
                }

                let material = materials[material_index];
                Some(RayHit {
                    point: hit_point,
                    normal,
                    material,
                })
            }
        }
    }
}
