use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    objects::{Material, Object3D},
    scene::Light,
    utils::geometry,
};

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

    fn reflect_vec(&self, vec: Vec3, normal: Vec3) -> Vec3 {
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
        let ambience = material.ambience * color;
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

    fn moller_trumbore_intersection(
        &self,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        material_index: usize,
    ) -> Option<RayHit> {
        let e1 = v2 - v1;
        let e2 = v3 - v1;
        let ray_cross_e2 = self.direction.cross(e2);
        let det = e1.dot(ray_cross_e2);

        if det > -f32::EPSILON && det < f32::EPSILON {
            return None; // This ray is parallel to this triangle.
        }
        let back_facing = det < f32::EPSILON;

        let inv_det = 1.0 / det;
        let s = self.origin - v1;
        let u = inv_det * s.dot(ray_cross_e2);
        if u < 0.0 || u > 1.0 {
            return None;
        }

        let s_cross_e1 = s.cross(e1);

        let v = inv_det * self.direction.dot(s_cross_e1);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        // At this stage we can compute t to find out where the intersection point is on the line.
        let t = inv_det * e2.dot(s_cross_e1);

        if t > f32::EPSILON {
            let hit_point = self.origin + self.direction * t;

            let mut normal = (v2 - v1).cross(v3 - v1).normalize();
            if back_facing {
                normal = -normal;
            }

            return Some(RayHit {
                distance: t,
                point: hit_point,
                normal,
                material_index,
                u,
                v,
            });
        } else {
            // This means that there is a line intersection but not a ray intersection.
            return None;
        }
    }

    pub fn hit(&self, obj: &Object3D, time: f32) -> Option<RayHit> {
        match obj {
            Object3D::Sphere {
                position,
                rotation_axis,
                radius,
                material_index,
                transform,
                inv_transform,
            } => self.sphere_intersection(
                *position,
                *transform,
                *inv_transform,
                radius,
                *material_index,
            ),

            Object3D::Triangle {
                v1,
                v2,
                v3,
                material_index,
            } => self.moller_trumbore_intersection(*v1, *v2, *v3, *material_index),

            Object3D::Box {
                position,
                rotation_axis,
                transform,
                inv_transform,
                dimension,
                material_index,
            } => self.box_intersection(
                *dimension,
                *transform,
                *inv_transform,
                *material_index,
                time,
            ),

            Object3D::Plane {
                normal,
                point,
                max_dist,
                material_index,
            } => self.plane_intersection(*normal, *point, *max_dist, *material_index),

            Object3D::Cylinder {
                radius,
                height,
                rotation_axis,
                transform,
                inv_transform,
                material_index,
            } => self.cylinder_intersection(
                *radius,
                *height,
                *transform,
                *inv_transform,
                *material_index,
            ),
        }
    }

    fn cylinder_intersection(
        &self,
        ra: f32,
        height: f32,
        transform: Mat4,
        inv_transform: Mat4,
        material_index: usize,
    ) -> Option<RayHit> {
        let rd3 =
            (inv_transform * vec4(self.direction.x, self.direction.y, self.direction.z, 0.)).xyz();
        let ro3 = (inv_transform * vec4(self.origin.x, self.origin.y, self.origin.z, 1.)).xyz();

        let rd = rd3.xy();
        let ro = ro3.xy();

        let a = rd.dot(rd);
        let b = 2.0 * ro.dot(rd);
        let c = ro.dot(ro) - ra * ra;

        let disc = b * b - 4.0 * a * c;

        let half = height / 2.;
        if disc > 0.0 {
            let t1 = (-b - disc.sqrt()) / (2.0 * a);

            let h_t1 = ro3 + rd3 * t1;

            let valid_t1 = h_t1.z.abs() < half;

            if valid_t1 {
                let n = vec3(h_t1.x, h_t1.y, 0.0).normalize();
                let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz();
                return Some(RayHit {
                    distance: t1,
                    point: self.origin + self.direction * t1,
                    normal,
                    material_index,
                    ..Default::default()
                });
            }
        }

        //let n1 = vec3(0., 0., 1.);
        //let denom1 = rd3.dot(n1);

        // if denom1.abs() < 1e-6 {
        //     return None;
        // }

        //t.at(2) = (bckRay.m_point1.GetElement(2) - 1.0) / -v.GetElement(2);
        //t.at(3) = (bckRay.m_point1.GetElement(2) + 1.0) / -v.GetElement(2);

        let t1 = (ro3.z - half) / -rd3.z;

        //let n2 = vec3(0., 0., -1.);
        //let denom2 = rd3.dot(n2);
        // if denom2.abs() < 0. {
        //     return None;
        // }
        let t2 = (ro3.z + half) / -rd3.z;

        let h_t1 = ro3 + rd3 * t1;
        let h_t2 = ro3 + rd3 * t2;

        let valid_t1 = h_t1.xy().dot(h_t1.xy()) < ra * ra;
        let valid_t2 = h_t2.xy().dot(h_t2.xy()) < ra * ra;

        let mut t = 0.;

        if valid_t1 && valid_t2 {
            if t1 < t2 {
                t = t1;
            } else {
                t = t2;
            };
        } else if valid_t1 {
            t = t1;
        } else if valid_t2 {
            t = t2;
        } else {
            return None;
        }

        let h_t = ro3 + rd3 * t;
        let n = vec3(0., 0., h_t.z).normalize();

        let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz();
        return Some(RayHit {
            distance: t,
            point: self.origin + self.direction * t,
            normal,
            material_index,
            ..Default::default()
        });
    }

    fn cylinder_intersection2(
        &self,
        ra: f32,
        height: f32,
        pa: Vec3,
        pb: Vec3,
        transform: Mat4,
        inv_transform: Mat4,
        material_index: usize,
    ) -> Option<RayHit> {
        let ro = self.origin;
        let rd = self.direction;

        let ba = pb - pa;
        let oc = ro - pa;

        // Dot product of ba with itself, representing the squared length of the cylinder's axis
        let baba = ba.dot(ba);
        // Dot product of ba with ray direction (rd), representing alignment of ray with cylinder's axis
        let bard = ba.dot(rd);
        // Dot product of ba with oc, representing alignment of the cylinder's axis with the vector to the ray origin
        let baoc = ba.dot(oc);

        // Quadratic coefficients for solving ray-cylinder intersection
        let k2 = baba - bard * bard;
        let k1 = baba * oc.dot(rd) - baoc * bard;
        let k0 = baba * oc.dot(oc) - baoc * baoc - ra * ra * baba;

        let h = k1 * k1 - k2 * k0;
        if h < 0.0 {
            return None;
        }
        let h = h.sqrt();
        let t = (-k1 - h) / k2; // The "t" value where the intersection occurs.

        // Checking intersection with the body of the cylinder.
        let y = baoc + t * bard;
        if y > 0.0 && y < baba {
            let hit_point = self.origin + self.direction * t;
            return Some(RayHit {
                distance: t,
                point: hit_point,
                normal: (oc + t * rd - ba * y / baba) / ra,
                material_index,
                ..Default::default()
            });
        }

        // Checking intersection with the end caps of the cylinder.
        let j = if y < 0.0 { 0.0 } else { baba };
        let t = (j - baoc) / bard;

        if (k1 + k2 * t).abs() >= h {
            return None;
        }

        let hit_point = self.origin + self.direction * t;

        let normal = ba * y.signum() / baba.sqrt();
        Some(RayHit {
            distance: t,
            point: hit_point,
            normal,
            material_index,
            ..Default::default()
        })
    }

    fn box_intersection(
        &self,
        box_size: Vec3,
        transform: Mat4,
        inv_transform: Mat4,
        material_index: usize,
        time: f32,
    ) -> Option<RayHit> {
        let mut ray_dir = self.direction;
        let mut ray_origin = self.origin;

        ray_dir = (inv_transform * vec4(ray_dir.x, ray_dir.y, ray_dir.z, 0.)).xyz();
        ray_origin = (inv_transform * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.)).xyz();

        let h_box_size = box_size;

        let b_min = -h_box_size;
        let b_max = h_box_size;

        let inv = 1.0 / ray_dir;

        let t_min = (b_min - ray_origin) * inv;
        let t_max = (b_max - ray_origin) * inv;

        let t_enter = t_min.min(t_max);
        let t_exit = t_min.max(t_max);

        let t_near = t_enter.x.max(t_enter.y).max(t_enter.z);
        let t_far = t_exit.x.min(t_exit.y).min(t_exit.z);

        if t_near > t_far || t_far < 0.0 {
            return None; // no intersection
        }

        let a = -ray_dir.signum() * geometry::step(vec3(t_near, t_near, t_near), t_enter);

        let normal = (transform * vec4(a.x, a.y, a.z, 0.0)).xyz();

        let hit_point = self.origin + self.direction * t_near;

        let opos = (inv_transform * vec4(hit_point.x, hit_point.y, hit_point.z, 1.0)).xyz();
        let onor = a;

        let u_v =
            onor.x.abs() * (opos.yz()) + onor.y.abs() * (opos.zx()) + onor.z.abs() * (opos.xy());

        Some(RayHit {
            distance: t_near,
            point: hit_point,
            normal,
            material_index,
            u: u_v.x,
            v: u_v.y,
        })
    }

    fn sphere_intersection(
        &self,
        position: Vec3,
        transform: Mat4,
        inv_transform: Mat4,
        radius: &f32,
        material_index: usize,
    ) -> Option<RayHit> {
        let mut ray_dir = self.direction;
        let mut ray_origin = self.origin;

        // Move the ray in object space.
        ray_dir = (inv_transform * vec4(ray_dir.x, ray_dir.y, ray_dir.z, 0.)).xyz();
        ray_origin = (inv_transform * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.)).xyz();

        // (bx^2 + by^2 + bz^2)t^2 + (2(axbx + ayby + azbz))t + (ax^2 + ay^2 + az^2 - r^2) = 0
        // where
        // a = ray origin
        // b = ray direction
        // r = radius
        // t = hit distance

        let a = ray_dir.dot(ray_dir);
        let b = 2. * ray_origin.dot(ray_dir);
        let c = ray_origin.dot(ray_origin) - radius * radius;

        let disc = b * b - 4. * a * c;

        if disc < 0.0 {
            return None;
        }

        let t0 = (-b + disc.sqrt()) / (2.0 * a);
        let t1 = (-b - disc.sqrt()) / (2.0 * a);

        let hit_point = self.origin + self.direction * t1;

        let n = (hit_point - position).normalize();

        // Move the normal in world space
        let normal = (transform * vec4(n.x, n.y, n.z, 0.0)).xyz();

        Some(RayHit {
            distance: t1,
            point: hit_point,
            normal,
            material_index,
            ..Default::default()
        })
    }

    pub fn plane_intersection(
        &self,
        normal: Vec3,
        p: Vec3,
        max_dist: Option<Vec2>,
        material_index: usize,
    ) -> Option<RayHit> {
        let denom = self.direction.dot(normal);

        if denom.abs() < 1e-6 {
            return None;
        }

        let t = (p - self.origin).dot(normal) / denom;

        if t < 0. {
            return None;
        }
        let hit_point = self.origin + self.direction * t;

        if let Some(Vec2 { x, y }) = max_dist {
            if hit_point.z.abs() > y || hit_point.x.abs() > x {
                return None;
            }
        }

        Some(RayHit {
            distance: t,
            point: hit_point,
            normal,
            material_index,
            u: hit_point.x * 0.1,
            v: hit_point.z * 0.1,
        })
    }
}
