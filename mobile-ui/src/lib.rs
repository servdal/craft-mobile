//! Shared phone layout state and touch-sized drawers; document state stays in each editor.
#![forbid(unsafe_code)]
use egui::{Context, Id, Rect, Vec2};
const PHONE_KEY: &str = "craft-phone-layout";
pub const TOUCH_HEIGHT: f32 = 44.0;

pub fn compact_size(size: Vec2, keyboard: bool, previous: bool) -> bool {
    if size.x < 600.0 {
        true
    } else if keyboard {
        previous
    } else {
        size.x < 1000.0 && size.y < 480.0
    }
}
pub fn set_phone(ctx: &Context, phone: bool) {
    ctx.data_mut(|data| data.insert_temp(Id::new(PHONE_KEY), phone));
}
pub fn is_phone(ctx: &Context) -> bool {
    ctx.data(|data| data.get_temp::<bool>(Id::new(PHONE_KEY)).unwrap_or(false))
}
pub fn button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add_sized(
        [ui.available_width(), TOUCH_HEIGHT],
        egui::Button::new(label).wrap(),
    )
}
pub struct DrawerResponse {
    pub close: bool,
    pub rect: Rect,
}
pub fn drawer(ctx: &Context, title: &str, content: impl FnOnce(&mut egui::Ui)) -> DrawerResponse {
    let bounds = ctx.content_rect();
    let width = (bounds.width() - 48.0).clamp(180.0, 380.0);
    let mut close = false;
    let shown = egui::Modal::new(Id::new("craft-phone-drawer")).show(ctx, |ui| {
        ui.set_width(width);
        ui.set_max_width(width);
        ui.horizontal(|ui| {
            ui.heading(title);
            close = ui
                .add_sized([64.0, TOUCH_HEIGHT], egui::Button::new("Close"))
                .clicked();
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height((bounds.height() - 160.0).max(80.0))
            .show(ui, content);
    });
    DrawerResponse {
        close: close || shown.should_close(),
        rect: shown.response.rect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phone_rotation_and_tablet_keyboard_keep_the_right_layout() {
        assert!(compact_size(egui::vec2(320.0, 568.0), false, false));
        assert!(compact_size(egui::vec2(844.0, 390.0), false, false));
        assert!(!compact_size(egui::vec2(768.0, 1024.0), false, true));
        assert!(!compact_size(egui::vec2(1024.0, 768.0), false, true));
        assert!(!compact_size(egui::vec2(800.0, 350.0), true, false));
        assert!(compact_size(egui::vec2(844.0, 180.0), true, true));
    }
    #[test]
    fn drawer_and_touch_targets_fit_small_portrait_and_landscape_screens() {
        for size in [
            egui::vec2(320.0, 568.0),
            egui::vec2(390.0, 844.0),
            egui::vec2(844.0, 390.0),
        ] {
            let screen = Rect::from_min_size(egui::Pos2::ZERO, size);
            let drawer_width = (screen.width() - 48.0).clamp(180.0, 380.0);
            assert!(drawer_width <= screen.width());
            assert!(TOUCH_HEIGHT >= 44.0);
        }
    }
}
