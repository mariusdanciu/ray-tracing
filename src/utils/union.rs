use glam::{Vec2, Vec3};

use crate::{
    objects::{Intersection, Object3D},
    ray::{Ray, RayHit},
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

    pub fn sdf(&self, scene: &Scene, p: Vec3) -> (f32, Vec3) {
        let o1 = scene.objects[self.first];
        let o2 = scene.objects[self.second];

        let (d1, c1) = o1.sdf(scene, p, &o1);
        let (d2, c2) = o2.sdf(scene, p, &o2);

        let i = geometry::interpolation(d1, d2, 0.7);
        let col = geometry::mix_vec3(c1, c2, 1. - i);

        let d = geometry::smooth_union(d1, d2, 0.7);

        (d, col)
    }
}

impl Intersection for Union {
    fn intersect(&self, ray: &Ray) -> Option<RayHit> {
        None
    }
}
