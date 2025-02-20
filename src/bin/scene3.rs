use glam::{vec2, vec3, Vec3};
use ray_tracing::app::App;
use ray_tracing::camera::Camera;
use ray_tracing::objects::{Material, MaterialType, Object3D};
use ray_tracing::renderer::Renderer;
use ray_tracing::scene::{Light, Scene};
use ray_tracing::utils::{
    cone::Cone, cuboid::Cuboid, cylinder::Cylinder, errors::AppError, image::ImageUtils,
    plane::Plane, sphere::Sphere, triangle::Triangle,
};

pub fn update(s: &mut Scene, ts: f32) -> bool {
    let speed = 0.2;
    if let Some(Object3D::Cuboid(c)) = s.objects.iter_mut().find(|obj| match **obj {
        Object3D::Cuboid { .. } => true,
        _ => false,
    }) {
        c.rotation_axis.x += 2. * speed;
        c.rotation_axis.z += 4. * speed;
        c.rotation_axis.y += 2. * speed;
        c.update();
    };
    true
}

pub fn main() -> Result<(), AppError> {
    let objs = vec![Plane::new(
        vec3(0., 1., 0.),
        vec3(0., 0., 0.),
        Some(vec2(5., 5.)),
        0,
    )];

    let mut scene1 = Scene::new(
        objs,
        vec![Material {
            ambience: 0.6,
            diffuse: 0.8,
            shininess: 65.,
            specular: 2.8,
            albedo: Vec3::new(0.4, 0.4, 0.4),
            kind: MaterialType::Reflective { roughness: 0.4 },
            texture: Some(0),
            ..Default::default()
        }],
    );

    scene1 = scene1
        .with_texture(ImageUtils::load_image("./resources/chess.png")?)
        .with_texture(ImageUtils::load_image("./resources/wood.png")?)
        .with_texture(ImageUtils::load_image("./resources/stone3.jpg")?)
        .with_texture(ImageUtils::load_image("./resources/earth_clouds.jpg")?)
        .with_light(Light::Directional {
            direction: vec3(-1., -2., -5.).normalize(),
            intensity: 1.,
        });
    //scene1.ambient_color = vec3(0.4, 0.7, 1.);
    scene1.update_func = Some(update);
    scene1.diffuse = false;
    scene1.enable_accumulation = false;
    scene1.shadow_casting = false;

    let mut renderer = Renderer::new(scene1);
    let mut camera = Camera::new_with_pos(
        Vec3::new(0., 2., 5.0),
        Vec3::new(0., 0., -5.),
    );
    App::run(&mut camera, &mut renderer)
}
