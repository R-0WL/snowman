use crate::ray_intersect::{Intersect, Material, RayIntersect};
use nalgebra_glm::Vec3;

const EPSILON: f32 = 1e-4;

pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub material: Material,
}

impl Cube {
    pub fn new(center: Vec3, size: f32, material: Material) -> Self {
        Self {
            center,
            size,
            material,
        }
    }

    fn texture_coordinates(&self, point: &Vec3, normal: &Vec3) -> (f32, f32) {
        let local = (point - self.center) / self.size + Vec3::new(0.5, 0.5, 0.5);

        if normal.x.abs() > 0.5 {
            let u = if normal.x > 0.0 { 1.0 - local.z } else { local.z };
            (u, local.y)
        } else if normal.y.abs() > 0.5 {
            let v = if normal.y > 0.0 { 1.0 - local.z } else { local.z };
            (local.x, v)
        } else {
            let u = if normal.z > 0.0 { local.x } else { 1.0 - local.x };
            (u, local.y)
        }
    }
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_origin: &Vec3, ray_direction: &Vec3) -> Option<Intersect> {
        let half = self.size / 2.0;
        let min_bound = self.center - Vec3::new(half, half, half);
        let max_bound = self.center + Vec3::new(half, half, half);

        let mut t_enter = f32::NEG_INFINITY;
        let mut t_exit = f32::INFINITY;
        let mut enter_normal = Vec3::zeros();
        let mut exit_normal = Vec3::zeros();

        for axis in 0..3 {
            let origin = ray_origin[axis];
            let direction = ray_direction[axis];

            if direction.abs() < EPSILON {
                if origin < min_bound[axis] || origin > max_bound[axis] {
                    return None;
                }
                continue;
            }

            let t1 = (min_bound[axis] - origin) / direction;
            let t2 = (max_bound[axis] - origin) / direction;

            let mut negative_normal = Vec3::zeros();
            negative_normal[axis] = -1.0;
            let mut positive_normal = Vec3::zeros();
            positive_normal[axis] = 1.0;

            let (near_t, far_t, near_normal, far_normal) = if t1 <= t2 {
                (t1, t2, negative_normal, positive_normal)
            } else {
                (t2, t1, positive_normal, negative_normal)
            };

            if near_t > t_enter {
                t_enter = near_t;
                enter_normal = near_normal;
            }

            if far_t < t_exit {
                t_exit = far_t;
                exit_normal = far_normal;
            }

            if t_enter > t_exit {
                return None;
            }
        }

        let (distance, normal) = if t_enter > EPSILON {
            (t_enter, enter_normal)
        } else if t_exit > EPSILON {
            (t_exit, exit_normal)
        } else {
            return None;
        };

        let point = ray_origin + ray_direction * distance;
        let uv = self.texture_coordinates(&point, &normal);

        Some(Intersect {
            point,
            normal,
            distance,
            material: self.material,
            uv,
        })
    }
}

