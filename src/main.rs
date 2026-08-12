mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod ray_intersect;
mod texture;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use nalgebra_glm::{dot, normalize, Vec3};
use std::f32::consts::PI;
use std::time::Duration;

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use crate::texture::Texture;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BACKGROUND_COLOR: u32 = 0x071426;
const FOV: f32 = PI / 3.0;
const ROTATION_SPEED: f32 = PI / 60.0;

#[derive(Clone, Copy, PartialEq)]
enum TextureMode {
    None,
    Lava,
    Electric,
}

struct Textures {
    lava: Texture,
    electric: Texture,
}

fn base_color(intersect: &Intersect, mode: TextureMode, textures: &Textures) -> Color {
    if !intersect.material.textured {
        return intersect.material.diffuse;
    }

    match mode {
        TextureMode::None => intersect.material.diffuse,
        TextureMode::Lava => textures.lava.sample(intersect.uv.0, intersect.uv.1),
        TextureMode::Electric => textures.electric.sample(intersect.uv.0, intersect.uv.1),
    }
}

fn shade(
    intersect: &Intersect,
    light: &Light,
    mode: TextureMode,
    textures: &Textures,
) -> Color {
    let light_direction = (light.position - intersect.point).normalize();
    let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);
    let color = base_color(intersect, mode, textures);

    color * (diffuse_intensity * intersect.material.albedo * light.intensity)
}

fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    mode: TextureMode,
    textures: &Textures,
) -> Color {
    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(hit) = object.ray_intersect(ray_origin, ray_direction) {
            if closest.is_none_or(|current| hit.distance < current.distance) {
                closest = Some(hit);
            }
        }
    }

    closest.map_or(Color::from_hex(BACKGROUND_COLOR), |hit| {
        shade(&hit, light, mode, textures)
    })
}

fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
    mode: TextureMode,
    textures: &Textures,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let perspective_scale = (FOV / 2.0).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = ((2.0 * x as f32) / width - 1.0)
                * aspect_ratio
                * perspective_scale;
            let screen_y = (-(2.0 * y as f32) / height + 1.0) * perspective_scale;

            let camera_ray = normalize(&Vec3::new(screen_x, screen_y, -1.0));
            let world_ray = camera.basis_change(&camera_ray);
            let color = cast_ray(&camera.eye, &world_ray, objects, light, mode, textures);

            framebuffer.set_current_color(color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn create_scene() -> Vec<Box<dyn RayIntersect>> {
    let snow = Material::textured(Color::new(225, 235, 245));
    let black = Material::solid(Color::new(20, 20, 25));
    let orange = Material::solid(Color::new(240, 100, 20));
    let brown = Material::solid(Color::new(90, 45, 20));
    let ground = Material::solid(Color::new(65, 90, 115));

    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();
    let mut add = |x, y, z, size, material| {
        objects.push(Box::new(Cube::new(Vec3::new(x, y, z), size, material)));
    };

    add(0.0, -8.4, 0.0, 12.0, ground);
    add(0.0, -1.2, 0.0, 2.4, snow);
    add(0.0, 0.9, 0.0, 1.8, snow);
    add(0.0, 2.5, 0.0, 1.4, snow);

    add(-0.28, 2.72, 0.78, 0.18, black);
    add(0.28, 2.72, 0.78, 0.18, black);
    add(0.0, 2.43, 0.82, 0.25, orange);

    for (x, y) in [(-0.24, 2.18), (0.0, 2.12), (0.24, 2.18)] {
        add(x, y, 0.76, 0.12, black);
    }

    for y in [1.35, 0.90, 0.45] {
        add(0.0, y, 0.97, 0.20, black);
    }

    for side in [-1.0_f32, 1.0] {
        add(side * 1.00, 1.10, 0.0, 0.30, brown);
        add(side * 1.30, 1.25, 0.0, 0.30, brown);
        add(side * 1.60, 1.40, 0.0, 0.30, brown);
        add(side * 1.85, 1.65, 0.0, 0.25, brown);
    }

    for x in [-0.90, -0.45, 0.0, 0.45, 0.90] {
        add(x, 3.425, 0.0, 0.45, black);
    }

    for y in [3.875, 4.325] {
        for x in [-0.45, 0.0, 0.45] {
            add(x, y, 0.0, 0.45, black);
        }
    }

    objects
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut window = Window::new(
        "Muneco de nieve: 1 Sin textura | 2 Lava | 3 Electricidad",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("No se pudo crear la ventana");

    let objects = create_scene();
    let textures = Textures {
        lava: Texture::load("assets/lava.png"),
        electric: Texture::load("assets/electric.png"),
    };
    let light = Light::new(
        Vec3::new(-6.0, 7.0, 8.0),
        Color::new(255, 255, 255),
        1.25,
    );
    let mut camera = Camera::new(
        Vec3::new(7.0, 4.5, 10.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let mut texture_mode = TextureMode::None;
    let mut must_render = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let requested_mode = if window.is_key_pressed(Key::Key1, KeyRepeat::No) {
            Some(TextureMode::None)
        } else if window.is_key_pressed(Key::Key2, KeyRepeat::No) {
            Some(TextureMode::Lava)
        } else if window.is_key_pressed(Key::Key3, KeyRepeat::No) {
            Some(TextureMode::Electric)
        } else {
            None
        };

        if let Some(mode) = requested_mode {
            if mode != texture_mode {
                texture_mode = mode;
                must_render = true;
            }
        }

        for (key, yaw, pitch) in [
            (Key::Left, ROTATION_SPEED, 0.0),
            (Key::Right, -ROTATION_SPEED, 0.0),
            (Key::Up, 0.0, -ROTATION_SPEED),
            (Key::Down, 0.0, ROTATION_SPEED),
        ] {
            if window.is_key_down(key) {
                camera.orbit(yaw, pitch);
                must_render = true;
            }
        }

        if must_render {
            render(
                &mut framebuffer,
                &objects,
                &camera,
                &light,
                texture_mode,
                &textures,
            );
            must_render = false;
        }

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .expect("No se pudo actualizar la ventana");

        std::thread::sleep(Duration::from_millis(16));
    }
}

