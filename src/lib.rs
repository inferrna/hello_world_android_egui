use ::egui::FontDefinitions;
use chrono::Timelike;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use log::{error, info, warn};
use std::iter;
use std::process::exit;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::{Backends, CompositeAlphaMode, InstanceDescriptor};
use winit::event::Event::*;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event::{StartCause, WindowEvent};

#[cfg(target_os = "android")]
use winit::{
    platform::android::EventLoopBuilderExtAndroid,
};
use winit::dpi::PhysicalSize;
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;

/// A custom event type for the winit app.
#[derive(Debug, Clone, Copy)]
pub enum Event {
    RequestRedraw,
}

/// This is the repaint signal type that egui needs for requesting a repaint from another thread.
/// It sends the custom RequestRedraw event to the winit event loop.
struct ExampleRepaintSignal(std::sync::Mutex<winit::event_loop::EventLoopProxy<Event>>);

impl epi::backend::RepaintSignal for ExampleRepaintSignal {
    fn request_repaint(&self) {
        self.0
            .lock()
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to lock guard at {} line {} with error\n{}",
                    file!(),
                    line!(),
                    e
                )
            })
            .send_event(Event::RequestRedraw)
            .ok();
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    #[cfg(debug_assertions)]
    {
        std::env::set_var("RUST_BACKTRACE", "full");
        android_logger::init_once(
            android_logger::Config::default().with_max_level(log::Level::Trace.to_level_filter()),
        );
    }
    let event_loop = winit::event_loop::EventLoopBuilder::<Event>::with_user_event()
        .with_android_app(app)
        .build()
        .unwrap();
    main(event_loop);
}
pub fn main(mut event_loop: EventLoop<Event>) {
    //'Cannot get the native window, it's null and will always be null before Event::Resumed and after Event::Suspended. Make sure you only call this function between those events.', ..../winit-c2fdb27092aba5a7/418cc44/src/platform_impl/android/mod.rs:1028:13
    warn!("Winit build window at {} line {}", file!(), line!());
    let window = winit::window::WindowBuilder::new()
        .with_decorations(!cfg!(android)) /* !cfg!(android) */
        .with_resizable(!cfg!(android))
        .with_transparent(false)
        .with_title("egui-wgpu_winit example")
        .build(&event_loop)
        .unwrap_or_else(|e| {
            panic!(
                "Failed to init window at {} line {} with error\n{:?}",
                file!(),
                line!(),
                e
            )
        });
    let window = Arc::new(window);

    warn!("WGPU new instance at {} line {}", file!(), line!());

    let instance_descriptor = InstanceDescriptor{backends: Backends::VULKAN, ..Default::default()};

    let mut instance = wgpu::Instance::new(instance_descriptor);

    let mut size = window.inner_size();
    let outer_size = window.outer_size();

    warn!("outer_size = {:?}", outer_size);
    warn!("size = {:?}", size);

    warn!("Create platform at {} line {}", file!(), line!());
    // We use the egui_winit_platform crate as the platform.
    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width,
        physical_height: size.height,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    #[cfg(target_os = "android")]
    let mut platform = {
        //Just find the actual screen size on android
        event_loop.run_on_demand(|main_event, tgt| {
            tgt.set_control_flow(ControlFlow::Poll);
            warn!(
                "Got event: {:?} at {} line {}",
                &main_event,
                file!(),
                line!()
            );
            match main_event {
                NewEvents(e) => match e {
                    StartCause::ResumeTimeReached { .. } => {}
                    StartCause::WaitCancelled { .. } => {}
                    StartCause::Poll => tgt.set_control_flow(ControlFlow::Poll),
                    StartCause::Init => {}
                },
                winit::event::Event::WindowEvent {
                    window_id,
                    ref event,
                } => {
                    if let winit::event::WindowEvent::Resized(r) = event {
                        size = *r;
                        warn!(
                            "Set to new size: {:?} at {} line {}",
                            &size,
                            file!(),
                            line!()
                        );
                        tgt.exit();
                    }
                }
                DeviceEvent { .. } => {}
                UserEvent(_) => {}
                Suspended => {}
                Resumed => {
                    if let Some(primary_mon) = tgt.primary_monitor() {
                        let mut mode = primary_mon.video_modes().next().unwrap();
                        size = mode.size();
                        //test_window.;
                        warn!(
                            "Set to new size: {:?} at {} line {}",
                            &size,
                            file!(),
                            line!()
                        );
                        while let Some(new_mode) = primary_mon.video_modes().next() {
                            if mode == new_mode {
                                break;
                            }
                            warn!("Another mode: {mode}");
                        }
                    } else if let Some(other_mon) = tgt.available_monitors().next() {
                        size = other_mon.video_modes().next().unwrap().size();
                        //test_window.set_inner_size(size);
                        warn!(
                            "Set to new size: {:?} at {} line {}",
                            &size,
                            file!(),
                            line!()
                        );
                    }
                    tgt.exit();
                }
                _ => {}
            };
            platform.handle_event(&main_event);
        }).unwrap();

        //RustStdoutStderr:     `Surface` width and height must be within the maximum supported texture size. Requested was (1080, 2340), maximum extent is 2048.
        //size = PhysicalSize::new(size.width, size.height.min(2048));

        warn!("Recreate platform at {} line {}", file!(), line!());
        // We use the egui_winit_platform crate as the platform.
        Platform::new(PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        })
    };


    warn!("WGPU new surface at {} line {}", file!(), line!());
    let mut surface = unsafe { instance.create_surface(window.clone()).unwrap_or_else(|e| {
        panic!(
            "Failed to create surface at {} line {} with error\n{:?}",
            file!(),
            line!(),
            e
        )
    }) };

    warn!("instance request_adapter at {} line {}", file!(), line!());
    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap_or_else(|| panic!("Failed get adapter at {} line {}", file!(), line!()));

    warn!("Got adapter {:?}", adapter.get_info());
    warn!("adapter request_device at {} line {}", file!(), line!());

    //Make it possible to run on intel HD3000 on Linux.
    //Might be also helpful for other low-level devices.
    let mut limits = wgpu::Limits::downlevel_defaults();
    limits.max_texture_dimension_2d = 4096;
    limits.max_texture_dimension_1d = 8192;
    limits.max_compute_workgroups_per_dimension = 0;
    limits.max_compute_workgroup_size_x = 0;
    limits.max_compute_workgroup_size_y = 0;
    limits.max_compute_workgroup_size_z = 0;
    limits.max_compute_workgroup_storage_size = 0;
    limits.max_compute_invocations_per_workgroup = 0;
    limits.max_storage_buffer_binding_size = 0;
    limits.max_storage_textures_per_shader_stage = 0;
    limits.max_storage_buffers_per_shader_stage = 0;
    limits.max_dynamic_storage_buffers_per_pipeline_layout = 0;
    
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            required_features: wgpu::Features::default(),
            required_limits: limits,
            label: None,
        },
        None,
    ))
    .unwrap_or_else(|e| {
        panic!(
            "Failed to request device at {} line {} with error\n{:?}",
            file!(),
            line!(),
            e
        )
    });

    let surface_capabilities = surface.get_capabilities(&adapter);
    let surface_format = surface_capabilities.formats[0];
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::AutoNoVsync,
        desired_maximum_frame_latency: 6,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![surface_format],
    };

    warn!("surface configure at {} line {}", file!(), line!());
    surface.configure(&device, &surface_config);

    warn!("RenderPass new at {} line {}", file!(), line!());
    // We use the egui_wgpu_backend crate as the render backend.
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1);
    warn!("DemoWindows default at {} line {}", file!(), line!());

    // Display the demo application that ships with egui.
    let mut demo_app = egui_demo_lib::DemoWindows::default();
    let start_time = Instant::now();
    let mut in_bad_state = false;

    let clbk_window = window.clone();
    let loop_window = window.clone();

    platform.context().set_pixels_per_point(loop_window.scale_factor() as f32);
    platform.context().set_request_repaint_callback(move |_| clbk_window.request_redraw());

    warn!("Enter the loop");
    event_loop.run_on_demand(move |event, window_target| {
        // Pass the winit events to the platform integration.
        window_target.set_control_flow(ControlFlow::Poll);
        warn!("Got event: {:?} at {} line {}", &event, file!(), line!());
        match &event {
            UserEvent(Event::RequestRedraw) => {
                loop_window.request_redraw();
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    exit(0);
                }
                winit::event::WindowEvent::Focused(focused) => {
                    in_bad_state |= !focused;
                },
                WindowEvent::ActivationTokenDone { .. } => {}
                WindowEvent::Moved(_) => {}
                WindowEvent::Destroyed => {}
                WindowEvent::DroppedFile(_) => {}
                WindowEvent::HoveredFile(_) => {}
                WindowEvent::HoveredFileCancelled => {}
                WindowEvent::KeyboardInput { .. } => {}
                WindowEvent::ModifiersChanged(_) => {}
                WindowEvent::Ime(_) => {}
                WindowEvent::CursorMoved { .. } => {}
                WindowEvent::CursorEntered { .. } => {}
                WindowEvent::CursorLeft { .. } => {}
                WindowEvent::MouseWheel { .. } => {}
                WindowEvent::MouseInput { .. } => {},
                WindowEvent::TouchpadMagnify { .. } => {}
                WindowEvent::SmartMagnify { .. } => {}
                WindowEvent::TouchpadRotate { .. } => {}
                WindowEvent::TouchpadPressure { .. } => {}
                WindowEvent::AxisMotion { .. } => {}
                WindowEvent::Touch(_) => {}
                WindowEvent::ScaleFactorChanged { .. } => {}
                WindowEvent::ThemeChanged(_) => {}
                WindowEvent::Occluded(_) => {}
                WindowEvent::RedrawRequested => {
                    let output_frame = match surface.get_current_texture() {
                        Ok(frame) => frame,
                        Err(wgpu::SurfaceError::Outdated) => {
                            // This error occurs when the app is minimized on Windows.
                            // Silently return here to prevent spamming the console with:
                            error!("The underlying surface has changed, and therefore the swap chain must be updated");
                            in_bad_state = true;
                            return;
                        }
                        Err(wgpu::SurfaceError::Lost) => {
                            // This error occurs when the app is minimized on Windows.
                            // Silently return here to prevent spamming the console with:
                            error!("LOST surface, drop frame. Originally: \"The swap chain has been lost and needs to be recreated\"");
                            in_bad_state = true;
                            return;
                        }
                        Err(e) => {
                            error!("Dropped frame with error: {}", e);
                            return;
                        }
                    };
                    let output_view = output_frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());

                    info!("Begin new frame");
                    dbg!(loop_window.scale_factor());
                    // Begin to draw the UI frame.
                    platform.begin_frame();

                    // Draw the demo application.
                    demo_app.ui(&platform.context());
                    info!("End new frame");
                    // End the UI frame. We could now handle the output and draw the UI with the backend.
                    let full_output = platform.end_frame(Some(&loop_window));

                    platform.update_time(start_time.elapsed().as_secs_f64());

                    let paint_jobs = platform.context().tessellate(full_output.shapes, loop_window.scale_factor() as f32);

                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("encoder"),
                    });

                    // Upload all resources for the GPU.
                    let screen_descriptor = ScreenDescriptor {
                        physical_width: surface_config.width,
                        physical_height: surface_config.height,
                        scale_factor: loop_window.scale_factor() as f32
                    };
                    let tdelta: egui::TexturesDelta = full_output.textures_delta;
                    egui_rpass
                        .add_textures(&device, &queue, &tdelta)
                        .expect("add texture failed");
                    egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);
                    // Record all render passes.
                    egui_rpass
                        .execute(
                            &mut encoder,
                            &output_view,
                            &paint_jobs,
                            &screen_descriptor,
                            Some(wgpu::Color::BLACK),
                        )
                        .unwrap_or_else(|e| panic!("Failed to render pass at {} line {} with error\n{:?}", file!(), line!(), e));
                    // Submit the commands.
                    queue.submit(iter::once(encoder.finish()));

                    // Redraw egui
                    output_frame.present();
                    egui_rpass
                        .remove_textures(tdelta)
                        .expect("remove texture failed");

                    //loop_window.request_redraw();
                    platform.context().request_repaint_after(Duration::from_millis(30));
                }
            },
            Resumed => {
                if in_bad_state {
                    //https://github.com/gfx-rs/wgpu/issues/2302
                    warn!("WGPU new surface at {} line {}", file!(), line!());
                    surface =
                        instance.create_surface(loop_window.clone()).unwrap_or_else(|e| {
                            panic!(
                                "Failed to create surface at {} line {} with error\n{:?}",
                                file!(),
                                line!(),
                                e
                            )
                        });
                    warn!("surface configure at {} line {}", file!(), line!());
                    surface.configure(&device, &surface_config);
                    in_bad_state = false;
                }
            },
            Suspended => (),
            NewEvents(e) => {
                match e {
                    StartCause::ResumeTimeReached { .. } => {}
                    StartCause::WaitCancelled { .. } => {window_target.set_control_flow(ControlFlow::Poll)}
                    StartCause::Poll => {
                        window_target.set_control_flow(ControlFlow::Poll);
                    }
                    StartCause::Init => {
                        loop_window.request_redraw();
                    }
                }
            }
            DeviceEvent { .. } => {}
            AboutToWait => {
                window_target.set_control_flow(ControlFlow::Wait);
            }
            LoopExiting => {}
            MemoryWarning => {}
        }
        platform.handle_event(&event);
    }).unwrap();
}

/// Time of day as seconds since midnight. Used for clock in demo app.
pub fn seconds_since_midnight() -> f64 {
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}
