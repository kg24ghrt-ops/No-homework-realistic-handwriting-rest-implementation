use core::PaperGenerator;
use image::{DynamicImage, RgbImage, Rgb};
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

pub struct LinedPaper {
    line_color: Rgb<u8>,
    line_spacing: u32,
    margin_left: u32,
    line_jitter_amplitude: f32,
    grain_intensity: f64,
}

impl Default for LinedPaper {
    fn default() -> Self {
        LinedPaper {
            line_color: Rgb([200, 210, 220]),
            line_spacing: 80,
            margin_left: 120,
            line_jitter_amplitude: 0.8,
            grain_intensity: 0.04,
        }
    }
}

impl PaperGenerator for LinedPaper {
    fn generate(&self, width: u32, height: u32, seed: u64) -> DynamicImage {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let mut img = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));

        // Draw horizontal lines with slight jitter
        let mut y = 150; // starting y for first line
        while y < height {
            let mut x = self.margin_left;
            while x < width {
                let jitter = rng.gen_range(-self.line_jitter_amplitude..=self.line_jitter_amplitude) as i32;
                let new_y = (y as i32 + jitter).clamp(0, height as i32 - 1) as u32;
                if new_y < height {
                    img.put_pixel(x, new_y, self.line_color);
                }
                x += 1;
            }
            y += self.line_spacing;
        }

        // Draw left margin line (solid, faint)
        for y in 0..height {
            if self.margin_left < width {
                img.put_pixel(self.margin_left, y, Rgb([220, 0, 0])); // red margin line
            }
        }

        // Apply Perlin noise grain
        let perlin = Perlin::new(seed as u32);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let noise_val = perlin.get([x as f64 / 8.0, y as f64 / 8.0]) * self.grain_intensity;
            let r = (pixel[0] as f64 / 255.0 + noise_val).clamp(0.0, 1.0);
            let g = (pixel[1] as f64 / 255.0 + noise_val).clamp(0.0, 1.0);
            let b = (pixel[2] as f64 / 255.0 + noise_val).clamp(0.0, 1.0);
            *pixel = Rgb([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]);
        }

        DynamicImage::ImageRgb8(img)
    }
}