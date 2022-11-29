use winit::dpi::PhysicalSize;
use hello_world_android_egui::{Event, MobAppHelper};

struct DummyHelper {}

impl MobAppHelper for DummyHelper {
    fn show_keyboard(&self) { unimplemented!() }
    fn hide_keyboard(&self) { unimplemented!() }
    fn screen_size(&self) -> Option<PhysicalSize<i32>> { unimplemented!() }
}

fn main() {
    #[cfg(debug_assertions)]
    simple_logger::init().unwrap();
    let event_loop = winit::event_loop::EventLoopBuilder::<Event>::with_user_event().build();
    hello_world_android_egui::main::<DummyHelper>(event_loop, None);
}
