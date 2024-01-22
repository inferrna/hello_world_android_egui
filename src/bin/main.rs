#[cfg(not(target_os = "android"))]
use hello_world_android_egui::Event;
#[cfg(not(target_os = "android"))]
fn main() {
    #[cfg(debug_assertions)]
    simple_logger::init().unwrap();
    let event_loop = winit::event_loop::EventLoopBuilder::<Event>::with_user_event().build().unwrap();
    hello_world_android_egui::main(event_loop);
}

#[cfg(target_os = "android")]
fn main() {}