use crate::color::Color;
use image::RgbImage;
use std::path::Path;

pub struct Texture {
    image: RgbImage,
}

impl Texture {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let image = image::open(path)
            .expect("No se pudo abrir la textura")
            .to_rgb8();

        Self { image }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);

        let x = (u * (self.image.width() - 1) as f32) as u32;
        let y = ((1.0 - v) * (self.image.height() - 1) as f32) as u32;
        let pixel = self.image.get_pixel(x, y);

        Color::new(pixel[0], pixel[1], pixel[2])
    }
}

