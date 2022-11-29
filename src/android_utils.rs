use winit::dpi::PhysicalSize;
use crate::MobAppHelper;

impl MobAppHelper for winit::platform::android::activity::AndroidApp {
    fn show_keyboard(&self) {self.show_soft_input(false)}
    fn hide_keyboard(&self) {self.hide_soft_input(false)}

    fn screen_size(&self) -> Option<PhysicalSize<i32>> {
        self.native_window()
            .map(|nw| PhysicalSize::new(nw.width(), nw.height()))
    }
}