use core::HandwritingStyle;
use core::PaperGenerator;
use paper::LinedPaper;
use natural_style::NaturalStyle;
use image::imageops::overlay;

fn main() {
    let paper_gen = LinedPaper::default();
    let style = NaturalStyle::default();

    let seed = 12345;
    let width = 2400;  // A4 at 300 DPI (approx 8.27*300)
    let height = 3200; // 11.69*300

    let mut paper = paper_gen.generate(width, height, seed);
    let text_img = style.render_text(
        "Biology Notes\n\n• The cell is the basic unit of life.\n• Mitochondria are the powerhouses.\n• DNA is a double helix.\n\nHomework due Monday.",
        width,
        height,
        seed,
    );

    // Overlay the handwriting onto the paper (assuming text_img is RGBA)
    overlay(&mut paper, &text_img, 0, 0);

    paper.save("output.png").expect("Failed to save output.png");
    println!("Saved output.png");
}