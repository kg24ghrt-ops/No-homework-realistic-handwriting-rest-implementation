use android_activity::AndroidApp;
use core::{HandwritingStyle, PaperGenerator};
use paper::LinedPaper;
use natural_style::NaturalStyle;
use image::DynamicImage;
use std::ffi::CString;
use std::fs;

#[no_mangle]
fn android_main(app: AndroidApp) {
    // Generate a sample page
    let style = NaturalStyle::default();
    let paper = LinedPaper::default();
    let width = 2400;
    let height = 3200;
    let seed = 12345;
    let mut img = paper.generate(width, height, seed);
    let text = style.render_text("Homework Demo\n\nDone by cargo-apk", width, height, seed);
    image::imageops::overlay(&mut img, &text, 0, 0);

    // Save to external storage (simplest for testing)
    let path = "/sdcard/homework.png";
    img.save(path).ok();

    // Show a toast via JNI (optional)
    let activity = app.activity();
    let env = activity.env();
    let class = env.find_class("android/widget/Toast").unwrap();
    // … simple JNI toast omitted for brevity
}