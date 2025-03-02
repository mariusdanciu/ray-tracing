use glam::{Vec2, Vec3};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit, RayMarchingHit},
    scene::Scene,
};

use super::geometry;

#[derive(Debug, Clone, Copy)]
pub struct Union {
    pub first: usize,
    pub second: usize,
}

impl Union {
    pub fn new(first: usize, second: usize) -> Object3D {
        Object3D::Union(Union { first, second })
    }

    pub fn material_index(&self, scene: &Scene) -> usize {
        scene.objects[self.first].material_index()
    }

    pub fn sdf(&self, scene: &Scene, ray: &Ray, t: f32) -> RayMarchingHit {
        
        let o1 = scene.objects[self.first];
        let o2 = scene.objects[self.second];

        let h1 = o1.sdf(scene, ray, t, &o1);
        let h2 = o2.sdf(scene, ray, t, &o2);

        let i = geometry::interpolation(h1.distance, h2.distance, 0.7);
        let col = geometry::mix_vec3(h1.albedo, h2.albedo, 1. - i);

        let d = geometry::smooth_union(h1.distance, h2.distance, 0.7);
        if h1.distance < h2.distance {
            return RayMarchingHit::new(d, col, h1.transformed_ray)
        }
        RayMarchingHit::new (d, col, h2.transformed_ray)
    }
}

impl Intersection for Union {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        None
    }
}
