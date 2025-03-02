use core::f32;

use glam::{vec3, vec4, Mat4, Vec3, Vec4Swizzles};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit, RayMarchingHit},
    scene::Scene,
};

use super::geometry;

static INV_PI: f32 = 1. / f32::consts::PI;
#[derive(Debug, Clone, Copy, Default)]
pub struct Sphere {
    pub position: Vec3,
    pub rotation_axis: Vec3,
    pub radius: f32,
    pub material_index: usize,
    pub part_of_composite: bool,
    pub transform: Mat4,
    pub inv_transform: Mat4,
}

impl Sphere {
    pub fn new(origin: Vec3, radius: f32, material_index: usize) -> Object3D {
        Object3D::Sphere(
            Sphere {
                position: origin,
                rotation_axis: Vec3::ZERO,
                radius,
                material_index,
                ..Default::default()
            }
            .update(),
        )
    }

    pub fn new_sphere_with_rotation(
        origin: Vec3,
        rotation_axis: Vec3,
        radius: f32,
        material_index: usize,
    ) -> Object3D {
        let t = Mat4::from_translation(origin)
            * Mat4::from_rotation_x(rotation_axis.x * geometry::DEGREES)
            * Mat4::from_rotation_y(rotation_axis.y * geometry::DEGREES)
            * Mat4::from_rotation_z(rotation_axis.z * geometry::DEGREES)
            * Mat4::from_scale(vec3(radius, radius, radius));
        Object3D::Sphere(Sphere {
            position: origin,
            rotation_axis,
            radius,
            material_index,
            transform: t,
            inv_transform: t.inverse(),
            part_of_composite: false,
        })
    }

    pub fn update(&mut self) -> Self {
        let t = Mat4::from_translation(self.position)
            * Mat4::from_rotation_x(self.rotation_axis.x * geometry::DEGREES)
            * Mat4::from_rotation_y(self.rotation_axis.y * geometry::DEGREES)
            * Mat4::from_rotation_z(self.rotation_axis.z * geometry::DEGREES)
            * Mat4::from_scale(vec3(self.radius, self.radius, self.radius));
        self.transform = t;
        self.inv_transform = t.inverse();
        *self
    }

    pub fn sdf(&self, scene: &Scene, ray: &Ray, t: f32, object: &Object3D) -> RayMarchingHit {
        let ray = self.transform_ray(ray);
        let p = ray.origin + ray.direction * t;
        //let p = self.inv_transform * vec4(p.x, p.y, p.z, 1.0);
        //let p = p.xyz();

        let m = object.material_index();
        let c = scene.materials[m].albedo;

        RayMarchingHit::new(p.length() - self.radius, c, ray)
    }

    pub fn transform_normal(&self, n: Vec3) -> Vec3 {
        (self.transform * vec4(n.x, n.y, n.z, 1.0)).xyz()
    }

    pub fn transform_ray(&self, n: &Ray) -> Ray {
        Ray {
            direction: (self.inv_transform * vec4(n.direction.x, n.direction.y, n.direction.z, 0.))
                .xyz(),
            origin: (self.inv_transform * vec4(n.origin.x, n.origin.y, n.origin.z, 1.)).xyz(),
        }
    }
}

impl Intersection for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        let mut ray_dir = ray.direction;
        let mut ray_origin = ray.origin;

        // Move the ray in object space.
        ray_dir = (self.inv_transform * vec4(ray_dir.x, ray_dir.y, ray_dir.z, 0.)).xyz();
        ray_origin =
            (self.inv_transform * vec4(ray_origin.x, ray_origin.y, ray_origin.z, 1.)).xyz();

        // (bx^2 + by^2 + bz^2)t^2 + (2(axbx + ayby + azbz))t + (ax^2 + ay^2 + az^2 - r^2) = 0
        // where
        // a = ray origin
        // b = ray direction
        // r = radius
        // t = hit distance

        let a = ray_dir.dot(ray_dir);
        let b = 2. * ray_origin.dot(ray_dir);
        let c = ray_origin.dot(ray_origin) - 1.;

        let disc = b * b - 4. * a * c;

        if disc < 0.0 {
            return None;
        }

        //let t0 = (-b + disc.sqrt()) / (2.0 * a);
        let t1 = (-b - disc.sqrt()) / (2.0 * a);

        let l = ray_origin + ray_dir * t1;

        let n = l;

        // Move the normal in world space
        let normal = (self.transform * vec4(n.x, n.y, n.z, 0.0))
            .xyz()
            .normalize();

        let u = ((l.x * l.x + l.y * l.y) / (l.z)).atan();
        let v = (l.y / l.x).atan();

        Some(RayHit {
            distance: t1,
            point: ray.origin + ray.direction * t1,
            normal,
            material_index: self.material_index,
            u: v * INV_PI,
            v: u * INV_PI,
        })
    }
}
