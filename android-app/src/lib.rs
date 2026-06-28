use android_activity::AndroidApp;
use core::{HandwritingStyle, PaperGenerator};
use paper::LinedPaper;
use natural_style::NaturalStyle;
use egui::{CentralPanel, ColorImage, TextureOptions, TextureHandle, Vec2};
use egui_wgpu::Renderer as EguiWgpuRenderer;
use image::DynamicImage;
use pollster;
use winit::event_loop::{EventLoopBuilder, ControlFlow};
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::event::{Event, WindowEvent};
use winit::window::WindowBuilder;

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
    let event_loop = EventLoopBuilder::<()>::new()
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
        Some(window.scale_factor() as f32),
        Some(2048),
    );

    // ---- wgpu setup ----
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })).unwrap();
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor::default(),
        None,
    )).unwrap();

    let mut size = window.inner_size();
    let mut scale_factor = window.scale_factor() as f32;

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .unwrap_or(surface_caps.formats[0]);
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoVsync,
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &surface_config);

    let mut egui_renderer = EguiWgpuRenderer::new(&device, surface_format, None, 1);

    fn reconfigure_surface(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        config: &mut wgpu::SurfaceConfiguration,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) {
        config.width = new_size.width;
        config.height = new_size.height;
        surface.configure(device, config);
    }

    // ---- event loop (winit 0.29: two-argument closure) ----
    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::Resumed => {
                reconfigure_surface(&surface, &device, &mut surface_config, window.inner_size());
                window.request_redraw();
            }
            Event::Suspended => {
                // resources kept; surface will be reconfigured on next Resumed
            }
            Event::WindowEvent { event: window_event, .. } => {
                let _ = egui_state.on_window_event(&window, &window_event);

                match window_event {
                    WindowEvent::RedrawRequested => {
                        let input = egui_state.take_egui_input(&window);
                        let full_output = egui_ctx.run(input, |ctx| {
                            build_ui(&mut state, ctx);
                        });
                        egui_state.handle_platform_output(&window, full_output.platform_output);

                        let paint_jobs = egui_ctx.tessellate(full_output.shapes, scale_factor);
                        let screen_descriptor = egui_wgpu::ScreenDescriptor {
                            size_in_pixels: [size.width, size.height],
                            pixels_per_point: scale_factor,
                        };

                        let output_frame = match surface.get_current_texture() {
                            Ok(frame) => frame,
                            Err(wgpu::SurfaceError::Lost) => {
                                reconfigure_surface(&surface, &device, &mut surface_config, window.inner_size());
                                window.request_redraw();
                                return;
                            }
                            Err(e) => {
                                eprintln!("Surface error: {:?}", e);
                                return;
                            }
                        };
                        let view = output_frame.texture.create_view(&Default::default());
                        let mut encoder = device.create_command_encoder(&Default::default());
                        egui_renderer.update_buffers(&device, &queue, &mut encoder, &paint_jobs, &screen_descriptor);

                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });
                            egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                        }
                        queue.submit(std::iter::once(encoder.finish()));
                        output_frame.present();

                        window.request_redraw();
                    }
                    WindowEvent::Resized(new_size) => {
                        size = new_size;
                        reconfigure_surface(&surface, &device, &mut surface_config, new_size);
                    }
                    WindowEvent::ScaleFactorChanged { scale_factor: new_sf, .. } => {
                        scale_factor = new_sf as f32;
                    }
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _ => (),
                }
            }
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

            let image = egui::Image::from_texture(texture)
                .fit_to_exact_size(Vec2::new(available_width, display_height));

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(image);
            });
        }
    });
}