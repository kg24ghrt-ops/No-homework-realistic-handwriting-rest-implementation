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

// ── App state ──────────────────────────────────────────────────
struct AppState {
    content: String,
    generated_image: Option<DynamicImage>,
    seed: u64,
    texture_handle: Option<TextureHandle>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            content: "Biology Notes\n\n• The cell is the basic unit of life.\n".into(),
            generated_image: None,
            seed: 12345,
            texture_handle: None,
        }
    }
}

// ── Main Android entry point ───────────────────────────────────
#[no_mangle]
fn android_main(app: AndroidApp) {
    let event_loop = EventLoop::builder()
        .with_android_app(app)
        .build()
        .expect("Failed to create event loop");

    // Create the egui context early – we’ll need it for State
    let egui_ctx = egui::Context::default();

    let window = WindowBuilder::new()
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)               // winit 0.30 takes &EventLoop
        .unwrap();

    let mut state = AppState::default();

    // Correct egui_winit::State::new signature for 0.27:
    // (egui_ctx, viewport_id, display_target, pixels_per_point, window_id)
    let mut egui_state = egui_winit::State::new(
        egui_ctx.clone(),
        egui::ViewportId::ROOT,
        &window,
        window.scale_factor() as f32,
        Some(window.id()),
    );

    let mut egui_renderer = egui_winit::Renderer::new(&window, |c| {
        // Software (CPU) renderer – no GPU, no WgpuSettings needed
        Box::new(egui_wgpu::SoftwareRenderer::new(c))
    });

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
                egui_renderer.paint(&window, full_output.textures_delta, full_output.shapes);
                window.request_redraw();
            }
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => control_flow.exit(),
            _ => (),
        }
    });
}

// ── UI layout ──────────────────────────────────────────────────
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
                state.texture_handle = None; // will reload next frame
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