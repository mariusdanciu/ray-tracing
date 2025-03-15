use glam::Vec3;
use rand::rngs::ThreadRng;

use crate::ray::Ray;



pub struct Scene {

}

impl Scene {
    pub fn pixel(&self, ray: Ray, rnd: &mut ThreadRng) -> Vec3 {
        Vec3::ZERO
    }
}