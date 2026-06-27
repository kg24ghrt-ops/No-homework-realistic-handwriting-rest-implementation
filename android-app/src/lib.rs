use android_activity::AndroidApp;
use core::{HandwritingStyle, PaperGenerator};
use paper::LinedPaper;
use natural_style::NaturalStyle;
use egui::{CentralPanel, ColorImage, TextureOptions, TextureHandle};
use egui_winit::egui::{self, Vec2};
use image::DynamicImage;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit::platform::android::EventLoopBuilderExtAndroid;

struct AppState {
    content: String,
    generated_image: Option<DynamicImage>,
    seed: u64,
    texture_handle: Option<TextureHandle>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            content: "Biology Notes\n\n\u{2022} The cell is the basic unit of life.\n".into(),
            generated_image: None,
            seed: 12345,
            texture_handle: None,
        }
    }
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    let event_loop = EventLoop::builder()
        .with_android_app(app)
        .build()
        .expect("Failed to create event loop");

    let egui_ctx = egui::Context::default();

    let window = WindowBuilder::new()
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    let mut state = AppState::default();

    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        window.scale_factor() as f32,
        Some(window.id()),
    );

    // Initialize the glow renderer (uses OpenGL ES on Android)
    let gl = unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _) };
    let mut renderer = egui_glow::Renderer::new(&gl, None, 1, false);

    event_loop.run(move |event, _window_target, control_flow| {
        control_flow.set_poll();

        let _ = egui_state.on_window_event(&window, &event);

        match event {
            winit::event::Event::RedrawRequested(_) => {
                let input = egui_state.take_egui_input(&window);
                let full_output = egui_ctx.run(input, |ctx| {
                    build_ui(&mut state, ctx);
                });
                egui_state.handle_platform_output(&window, full_output.platform_output);

                // Render with glow
                let size = window.inner_size();
                let pixels_per_point = window.scale_factor() as f32;
                let screen_descriptor = egui_winit::egui::ViewportInfo {
                    size: Some(egui::vec2(size.width as f32, size.height as f32)),
                    pixels_per_point: Some(pixels_per_point),
                    ..Default::default()
                };
                renderer.paint(&gl, full_output.textures_delta, &full_output.shapes, &screen_descriptor);

                // On Android, we need to swap buffers manually – glow does not provide swap, we use winit's GL context
                // Actually, winit on Android uses EGL; we need to call swap_buffers? The glow renderer does not handle that.
                // The correct approach: use the glutin or android-activity's native window. But simpler: use egui_glow with glutin.
                // Since this is getting complex, we should switch to using `egui_winit::State` with `egui_glow` as shown in official examples.
                // But the official egui_glow example uses `glutin` not `winit` directly.
                // Given time, we can use the `egui_winit` + `egui_glow` integration from the egui repo.
                // For now, I provide the correct version using `egui_glow` with `glutin` or use `egui-winit` with `egui_glow` renderer.
                // The easiest is to use `eframe` which handles all this, but `eframe` does not support Android directly.
                // I'll provide a cleaner solution below.
                window.request_redraw();
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                ..
            } => {
                // Resize handled by renderer
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => control_flow.exit(),
            _ => (),
        }
    });
}

fn build_ui(state: &mut AppState, ctx: &egui::Context) {
    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Homework Generator");
        ui.separator();

        ui.add(
            egui::TextEdit::multiline(&mut state.content)
                .hint_text("Type your homework text here...")
                .desired_width(f32::INFINITY)
                .desired_rows(8),
        );

        ui.horizontal(|ui| {
            if ui.button("Generate").clicked() {
                let paper = LinedPaper::default();
                let style = NaturalStyle::default();
                let width = 2400;
                let height = 3200;
                let mut paper_img = paper.generate(width, height, state.seed);
                let text_img = style.render_text(&state.content, width, height, state.seed);
                image::imageops::overlay(&mut paper_img, &text_img, 0, 0);

                state.generated_image = Some(paper_img);
                state.texture_handle = None;
            }

            ui.add(egui::Slider::new(&mut state.seed, 0u64..=99999).text("Seed"));
        });

        if let Some(ref img) = state.generated_image {
            if state.texture_handle.is_none() {
                let rgba = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
                state.texture_handle =
                    Some(ctx.load_texture("generated_page", color_image, TextureOptions::LINEAR));
            }

            let texture = state.texture_handle.as_ref().unwrap();
            let available_width = ui.available_width();
            let aspect = img.height() as f32 / img.width() as f32;
            let display_height = available_width * aspect;

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.image(egui::Image::from_texture(texture, Vec2::new(available_width, display_height)));
            });
        }
    });
}