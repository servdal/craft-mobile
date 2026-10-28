#![forbid(unsafe_code)]
use craft_mobile_shell_v37::{self as mobile, eframe, egui, serde_json};
use filmcraft_engine::Session;
use filmcraft_ui_egui::FilmcraftApp;

struct Editor(FilmcraftApp);

impl mobile::Editor for Editor {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        eframe::App::logic(&mut self.0, ctx, frame);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        eframe::App::ui(&mut self.0, ui, frame);
    }
    fn input(&mut self, raw: &mut egui::RawInput) {
        let ctx = egui::Context::default();
        eframe::App::raw_input_hook(&mut self.0, &ctx, raw);
    }
    fn command(&mut self, id: &str, p: serde_json::Value) -> Result<(), String> {
        self.0.session.execute(id, p).map(|_| ()).map_err(|error| error.to_string())
    }
}

fn main() -> eframe::Result<()> {
    mobile::run("FilmCraft Mobile", "fcproj", |_cc, _actions| Editor(FilmcraftApp::new(Session::default())))
}
