use crate::color::Color;
use nalgebra_glm::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub albedo: f32,
    pub textured: bool,
}

impl Material {
    pub fn solid(diffuse: Color) -> Self {
        Self {
            diffuse,
            albedo: 1.0,
            textured: false,
        }
    }

    pub fn textured(fallback: Color) -> Self {
        Self {
            diffuse: fallback,
            albedo: 1.0,
            textured: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub material: Material,
    pub uv: (f32, f32),
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect>;
}

