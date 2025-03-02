use glam::{Vec2, Vec3};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit, RayMarchingHit},
    scene::Scene,
};

use super::geometry;

#[derive(Debug, Clone, Copy)]
pub struct Substraction {
    pub first: usize,
    pub second: usize,
}

impl Substraction {
    pub fn new(first: usize, second: usize) -> Object3D {
        Object3D::Substraction(Substraction { first, second })
    }

    pub fn material_index(&self, scene: &Scene) -> usize {
        scene.objects[self.first].material_index()
    }

    pub fn sdf(&self, scene: &Scene, ray: &Ray, t: f32) -> RayMarchingHit {
        let p = ray.origin + ray.direction * t;
        let o1 = scene.objects[self.first];
        let o2 = scene.objects[self.second];

        let h1 = o1.sdf(scene, ray, t, &o1);
        let h2 = o2.sdf(scene, ray, t, &o2);

        let d = (-h2.distance).max(h1.distance);

        RayMarchingHit::new(d, scene.materials[o1.material_index()].albedo, h2.transformed_ray)
    }
}

impl Intersection for Substraction {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        None
    }
}
