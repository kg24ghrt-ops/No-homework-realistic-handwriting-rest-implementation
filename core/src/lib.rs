use image::DynamicImage;

pub trait HandwritingStyle {
    /// Renders the given text onto a transparent canvas of size (width, height).
    /// All randomness must be deterministically derived from `seed`.
    fn render_text(&self, text: &str, width: u32, height: u32, seed: u64) -> DynamicImage;
}

pub trait PaperGenerator {
    /// Generates a paper background image of the given size, seeded.
    fn generate(&self, width: u32, height: u32, seed: u64) -> DynamicImage;
}