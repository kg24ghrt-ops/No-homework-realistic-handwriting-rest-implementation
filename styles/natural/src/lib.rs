use core::HandwritingStyle;
use fontdue::{Font, FontSettings};
use image::{DynamicImage, RgbaImage, Rgba};
use rand::{Rng, SeedableRng};
use rand::distributions::Normal;
use rand_xoshiro::Xoshiro256StarStar;

pub struct NaturalStyle {
    font: Font,
    pixel_size: f32,
    line_height: f32,
    left_margin: f32,
    ink_color: [u8; 3],
    base_slant_deg: f32,          // degrees
    slant_variation: f32,         // std dev in degrees
    drift_amplitude: f32,
    drift_frequency: f32,
    spacing_randomness: f32,      // fraction of advance width added randomly
}

impl Default for NaturalStyle {
    fn default() -> Self {
        let font_data = include_bytes!("../fonts/Caveat-Regular.ttf");
        let font = Font::from_bytes(font_data as &[u8], FontSettings::default())
            .expect("Failed to parse font");
        NaturalStyle {
            font,
            pixel_size: 48.0,             // ~handwriting size at 300 DPI
            line_height: 75.0,
            left_margin: 150.0,
            ink_color: [30, 30, 60],      // dark blue-black
            base_slant_deg: -4.0,         // slight rightward slant
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
        let mut canvas = RgbaImage::new(width, height); // transparent background

        let lines: Vec<&str> = text.lines().collect();
        let mut cursor_y = 200.0; // start y

        for line in lines {
            let mut cursor_x = self.left_margin;
            // Random phase for sine drift per line
            let drift_phase: f32 = rng.gen_range(0.0..std::f32::consts::TAU);

            for ch in line.chars() {
                let glyph_id = self.font.lookup_glyph_index(ch);
                let (metrics, bitmap) = self.font.rasterize(glyph_id, self.pixel_size);
                if bitmap.is_empty() {
                    // skip spaces / missing glyphs by advancing cursor
                    cursor_x += metrics.advance_width;
                    continue;
                }

                // Baseline drift (sine wave)
                let drift = self.drift_amplitude
                    * (cursor_x * self.drift_frequency + drift_phase).sin();

                // Slant (rotation) -> we’ll apply shear (horizontal offset proportional to y)
                let slant_angle: f32 = {
                    let base = self.base_slant_deg;
                    let var: f32 = rng.sample(Normal::new(0.0, self.slant_variation as f64))
                        .unwrap_or(0.0) as f32;
                    (base + var).to_radians()
                };
                let tan_slant = slant_angle.tan();

                // Pressure (alpha multiplier)
                let pressure: f32 = rng.gen_range(0.7..1.0);

                // Place glyph bitmap onto canvas using shear
                let glyph_width = metrics.width as i32;
                let glyph_height = metrics.height as i32;
                let glyph_xmin = metrics.xmin as f32;
                let glyph_ymin = metrics.ymin as f32;

                for gy in 0..glyph_height {
                    for gx in 0..glyph_width {
                        let alpha = bitmap[(gy * glyph_width + gx) as usize] as f32 / 255.0;
                        if alpha <= 0.01 {
                            continue;
                        }

                        let glyph_x = gx as f32 + glyph_xmin;
                        let glyph_y = gy as f32 + glyph_ymin;

                        // Shear: shift x by glyph_y * tan_slant
                        let canvas_x = (cursor_x + glyph_x + glyph_y * tan_slant) as i32;
                        let canvas_y = (cursor_y + drift + glyph_y) as i32;

                        if canvas_x < 0 || canvas_y < 0 || canvas_x >= width as i32 || canvas_y >= height as i32 {
                            continue;
                        }

                        let ink_alpha = (alpha * pressure).clamp(0.0, 1.0);
                        // Over blending (source-over)
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
                            // Normalise
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

                // Advance cursor
                let extra_space: f32 = rng.gen_range(-self.spacing_randomness..self.spacing_randomness)
                    * metrics.advance_width;
                cursor_x += metrics.advance_width + extra_space;
            }
            cursor_y += self.line_height;
        }

        DynamicImage::ImageRgba8(canvas)
    }
}