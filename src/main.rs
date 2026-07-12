mod camera;
mod color;
mod cube;
mod framebuffer;
mod light;
mod ray_intersect;

use crate::camera::Camera;
use crate::color::Color;
use crate::cube::Cube;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::ray_intersect::{Intersect, Material, RayIntersect};
use minifb::{Key, Window, WindowOptions};
use nalgebra_glm::{dot, Vec3};
use std::f32::consts::PI;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const DAY_BACKGROUND: u32 = 0x7BC4DE;
const NIGHT_BACKGROUND: u32 = 0x040C24;

/// Calcula únicamente la componente difusa de Lambert.
fn shade(intersect: &Intersect, light: &Light) -> Color {
    let light_direction = (light.position - intersect.point).normalize();
    let diffuse_intensity = dot(&intersect.normal, &light_direction).max(0.0);

    intersect.material.diffuse
        * (diffuse_intensity * intersect.material.albedo[0] * light.intensity)
}

/// Devuelve el color del objeto más cercano alcanzado por el rayo.
fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    light: &Light,
    background_color: u32,
) -> Color {
    let mut closest: Option<Intersect> = None;

    for object in objects {
        if let Some(intersect) = object.ray_intersect(ray_origin, ray_direction) {
            let is_closer = match &closest {
                Some(current) => intersect.distance < current.distance,
                None => true,
            };

            if is_closer {
                closest = Some(intersect);
            }
        }
    }

    match closest {
        Some(intersect) => shade(&intersect, light),
        None => Color::from_hex(background_color),
    }
}

fn render(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
    background_color: u32,
) {
    let fov = PI / 3.0;
    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let scale = (fov / 2.0).tan();

    framebuffer.set_background_color(background_color);
    framebuffer.clear();

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let screen_x =
                (2.0 * (x as f32 + 0.5) / WIDTH as f32 - 1.0) * aspect_ratio * scale;
            let screen_y = (1.0 - 2.0 * (y as f32 + 0.5) / HEIGHT as f32) * scale;

            let camera_direction = Vec3::new(screen_x, screen_y, -1.0).normalize();
            let ray_direction = camera.basis_change(&camera_direction).normalize();

            let pixel_color = cast_ray(
                &camera.eye,
                &ray_direction,
                objects,
                light,
                background_color,
            );
            framebuffer.set_current_color(pixel_color.to_hex());
            framebuffer.point(x, y);
        }
    }
}

fn main() {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    framebuffer.set_background_color(DAY_BACKGROUND);

    let mut window = Window::new(
        "Muneco de nieve de cubos - Raytracer",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .expect("No se pudo crear la ventana");

    let snow = Material::new(Color::new(225, 235, 245), 0.0, [1.0, 0.0, 0.0]);
    let black = Material::new(Color::new(20, 20, 25), 0.0, [1.0, 0.0, 0.0]);
    let orange = Material::new(Color::new(240, 100, 20), 0.0, [1.0, 0.0, 0.0]);
    let brown = Material::new(Color::new(90, 45, 20), 0.0, [1.0, 0.0, 0.0]);
    let ground = Material::new(Color::new(90, 135, 160), 0.0, [1.0, 0.0, 0.0]);

    let mut objects: Vec<Box<dyn RayIntersect>> = Vec::new();

    let mut add_cube = |x: f32, y: f32, z: f32, size: f32, material: Material| {
        objects.push(Box::new(Cube::new(
            Vec3::new(x, y, z),
            size,
            material,
        )));
    };

    // Piso (también es un cubo).
    add_cube(0.0, -8.4, 0.0, 12.0, ground);

    // Cuerpo y cabeza.
    add_cube(0.0, -1.2, 0.0, 2.4, snow);
    add_cube(0.0, 0.9, 0.0, 1.8, snow);
    add_cube(0.0, 2.5, 0.0, 1.4, snow);

    // Ojos.
    add_cube(-0.28, 2.72, 0.78, 0.18, black);
    add_cube(0.28, 2.72, 0.78, 0.18, black);

    // Nariz.
    add_cube(0.0, 2.43, 0.82, 0.25, orange);

    // Boca.
    add_cube(-0.24, 2.18, 0.76, 0.12, black);
    add_cube(0.0, 2.12, 0.76, 0.12, black);
    add_cube(0.24, 2.18, 0.76, 0.12, black);

    // Botones.
    add_cube(0.0, 1.35, 0.97, 0.20, black);
    add_cube(0.0, 0.90, 0.97, 0.20, black);
    add_cube(0.0, 0.45, 0.97, 0.20, black);

    // Brazo izquierdo hecho de cubos.
    add_cube(-1.00, 1.10, 0.0, 0.30, brown);
    add_cube(-1.30, 1.25, 0.0, 0.30, brown);
    add_cube(-1.60, 1.40, 0.0, 0.30, brown);
    add_cube(-1.85, 1.65, 0.0, 0.25, brown);

    // Brazo derecho hecho de cubos.
    add_cube(1.00, 1.10, 0.0, 0.30, brown);
    add_cube(1.30, 1.25, 0.0, 0.30, brown);
    add_cube(1.60, 1.40, 0.0, 0.30, brown);
    add_cube(1.85, 1.65, 0.0, 0.25, brown);

    // Ala del sombrero.
    for x in [-0.90, -0.45, 0.0, 0.45, 0.90] {
        add_cube(x, 3.425, 0.0, 0.45, black);
    }

    // Copa del sombrero.
    for y in [3.875, 4.325] {
        for x in [-0.45, 0.0, 0.45] {
            add_cube(x, y, 0.0, 0.45, black);
        }
    }

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

    // El programa inicia de día. N alterna entre día y noche.
    let mut is_night = false;
    let mut n_was_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Detecta únicamente el instante en que se presiona N para evitar
        // que el fondo cambie muchas veces mientras la tecla sigue sostenida.
        let n_is_down = window.is_key_down(Key::N);
        if n_is_down && !n_was_down {
            is_night = !is_night;
        }
        n_was_down = n_is_down;

        if window.is_key_down(Key::Left) {
            camera.orbit(0.05, 0.0);
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-0.05, 0.0);
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, 0.05);
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, -0.05);
        }

        let background_color = if is_night {
            NIGHT_BACKGROUND
        } else {
            DAY_BACKGROUND
        };

        render(
            &mut framebuffer,
            &objects,
            &camera,
            &light,
            background_color,
        );

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .expect("No se pudo actualizar la ventana");
    }
}
