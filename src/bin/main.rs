use hello_world_android_egui::Event;

fn main() {
    #[cfg(debug_assertions)]
    simple_logger::init().unwrap();
    let event_loop = winit::event_loop::EventLoopBuilder::<Event>::with_user_event().build();
    hello_world_android_egui::main(event_loop, None);
}
