use glam::Vec3;
#[derive(Debug, Copy, Clone)]
pub struct Directional {
    pub albedo: Vec3,
    pub direction: Vec3,
    pub intensity: f32,
}
#[derive(Debug, Copy, Clone)]
pub struct Positional {
    pub albedo: Vec3,
    pub position: Vec3,
    pub intensity: f32,
}
#[derive(Debug, Copy, Clone)]
pub struct SphericalPositional {
    pub albedo: Vec3,
    pub position: Vec3,
    pub radius: f32,
    pub intensity: f32,
}
#[derive(Debug, Copy, Clone)]
pub struct Spot {
    pub albedo: Vec3,
    pub position: Vec3,
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub enum Light {
    Directional(Directional),
    Positional(Positional),
    SphericalPositional(SphericalPositional),
}

pub trait LightSource {
    fn albedo(&self) -> Vec3;
    fn direction(&self, point: Vec3) -> Vec3;
    fn distance(&self, point: Vec3) -> f32;
    fn intensity(&self) -> f32;
}

impl LightSource for Directional {
    fn direction(&self, point: Vec3) -> Vec3 {
        self.direction
    }

    fn distance(&self, point: Vec3) -> f32 {
        1.
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }
    
    fn albedo(&self) -> Vec3 {
        self.albedo
    }
}

impl LightSource for SphericalPositional {
    fn direction(&self, point: Vec3) -> Vec3 {
        (point - self.position).normalize()
    }

    fn distance(&self, point: Vec3) -> f32 {
        (point - self.position).length()
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }
    fn albedo(&self) -> Vec3 {
        self.albedo
    }
}

impl LightSource for Positional {
    fn direction(&self, point: Vec3) -> Vec3 {
        (point - self.position).normalize()
    }

    fn distance(&self, point: Vec3) -> f32 {
        (point - self.position).length()
    }

    fn intensity(&self) -> f32 {
        self.intensity
    }
    fn albedo(&self) -> Vec3 {
        self.albedo
    }
}

impl LightSource for Light {
    fn direction(&self, point: Vec3) -> Vec3 {
        match *self {
            Light::Directional(l) => l.direction(point),
            Light::Positional(l) => l.direction(point),
            Light::SphericalPositional(l) => l.direction(point),
        }
    }

    fn distance(&self, point: Vec3) -> f32 {
        match *self {
            Light::Directional(l) => l.distance(point),
            Light::Positional(l) => l.distance(point),
            Light::SphericalPositional(l) => l.distance(point),
        }
    }

    fn intensity(&self) -> f32 {
        match *self {
            Light::Directional(l) => l.intensity(),
            Light::Positional(l) => l.intensity(),
            Light::SphericalPositional(l) => l.intensity(),
        }
    }
    fn albedo(&self) -> Vec3 {
        match *self {
            Light::Directional(l) => l.albedo(),
            Light::Positional(l) => l.albedo(),
            Light::SphericalPositional(l) => l.albedo(),
        }
    }
}
