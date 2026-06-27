use core::HandwritingStyle;
use fontdue::{Font, FontSettings};
use image::{DynamicImage, RgbaImage, Rgba};
use rand::{Rng, SeedableRng};
use rand::distributions::Distribution;  // for .sample()
use rand_distr::Normal;                 // the Normal distribution
use rand_xoshiro::Xoshiro256StarStar;

pub struct NaturalStyle {
    font: Font,
    pixel_size: f32,
    line_height: f32,
    left_margin: f32,
    ink_color: [u8; 3],
    base_slant_deg: f32,
    slant_variation: f32,
    drift_amplitude: f32,
    drift_frequency: f32,
    spacing_randomness: f32,
}

impl Default for NaturalStyle {
    fn default() -> Self {
        let font_data = include_bytes!("../fonts/Caveat-Regular.ttf");
        let font = Font::from_bytes(font_data as &[u8], FontSettings::default())
            .expect("Failed to parse font");
        NaturalStyle {
            font,
            pixel_size: 48.0,
            line_height: 75.0,
            left_margin: 150.0,
            ink_color: [30, 30, 60],
            base_slant_deg: -4.0,
            slant_variation: 1.2,
            drift_amplitude: 2.5,
            drift_frequency: 0.02,
            spacing_randomness: 0.15,
        }
    }
}

impl HandwritingStyle for NaturalStyle {
    fn render_text(&self, text: &str, width: u32, height: u32, seed: u64) -> DynamicImage {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let mut canvas = RgbaImage::new(width, height);

        let lines: Vec<&str> = text.lines().collect();
        let mut cursor_y = 200.0;

        for line in lines {
            let mut cursor_x = self.left_margin;
            let drift_phase: f32 = rng.gen_range(0.0..std::f32::consts::TAU);

            for ch in line.chars() {
                // Rasterize the character directly
                let (metrics, bitmap) = self.font.rasterize(ch, self.pixel_size);
                if bitmap.is_empty() {
                    cursor_x += metrics.advance_width;
                    continue;
                }

                // Baseline sine drift
                let drift = self.drift_amplitude
                    * (cursor_x * self.drift_frequency + drift_phase).sin();

                // Slant with per-character Gaussian variation
                let slant_angle: f32 = {
                    let base = self.base_slant_deg;
                    // Normal distribution; unwrap is safe as parameters are valid
                    let normal = Normal::new(0.0, self.slant_variation as f64).unwrap();
                    let var: f64 = normal.sample(&mut rng);
                    (base + var as f32).to_radians()
                };
                let tan_slant = slant_angle.tan();

                // Pressure (alpha multiplier)
                let pressure: f32 = rng.gen_range(0.7..1.0);

                let glyph_width = metrics.width as i32;
                let glyph_height = metrics.height as i32;
                let glyph_xmin = metrics.xmin as f32;
                let glyph_ymin = metrics.ymin as f32;

                // Place glyph bitmap with shear (slant)
                for gy in 0..glyph_height {
                    for gx in 0..glyph_width {
                        let alpha = bitmap[(gy * glyph_width + gx) as usize] as f32 / 255.0;
                        if alpha <= 0.01 {
                            continue;
                        }

                        let glyph_x = gx as f32 + glyph_xmin;
                        let glyph_y = gy as f32 + glyph_ymin;

                        let canvas_x = (cursor_x + glyph_x + glyph_y * tan_slant) as i32;
                        let canvas_y = (cursor_y + drift + glyph_y) as i32;

                        if canvas_x < 0 || canvas_y < 0 || canvas_x >= width as i32 || canvas_y >= height as i32 {
                            continue;
                        }

                        let ink_alpha = (alpha * pressure).clamp(0.0, 1.0);
                        let mut pixel = canvas.get_pixel(canvas_x as u32, canvas_y as u32).clone();
                        let dst_alpha = pixel[3] as f32 / 255.0;
                        let out_alpha = ink_alpha + dst_alpha * (1.0 - ink_alpha);
                        if out_alpha > 0.0 {
                            let ink_r = self.ink_color[0] as f32 / 255.0;
                            let ink_g = self.ink_color[1] as f32 / 255.0;
                            let ink_b = self.ink_color[2] as f32 / 255.0;
                            let blended_r = ink_r * ink_alpha + pixel[0] as f32 / 255.0 * dst_alpha * (1.0 - ink_alpha);
                            let blended_g = ink_g * ink_alpha + pixel[1] as f32 / 255.0 * dst_alpha * (1.0 - ink_alpha);
                            let blended_b = ink_b * ink_alpha + pixel[2] as f32 / 255.0 * dst_alpha * (1.0 - ink_alpha);
                            let norm = out_alpha;
                            pixel = Rgba([
                                (blended_r / norm * 255.0).clamp(0.0, 255.0) as u8,
                                (blended_g / norm * 255.0).clamp(0.0, 255.0) as u8,
                                (blended_b / norm * 255.0).clamp(0.0, 255.0) as u8,
                                (out_alpha * 255.0).clamp(0.0, 255.0) as u8,
                            ]);
                        } else {
                            pixel = Rgba([0, 0, 0, 0]);
                        }
                        canvas.put_pixel(canvas_x as u32, canvas_y as u32, pixel);
                    }
                }

                // Advance cursor with random spacing
                let extra_space: f32 = rng.gen_range(-self.spacing_randomness..self.spacing_randomness)
                    * metrics.advance_width;
                cursor_x += metrics.advance_width + extra_space;
            }
            cursor_y += self.line_height;
        }

        DynamicImage::ImageRgba8(canvas)
    }
}