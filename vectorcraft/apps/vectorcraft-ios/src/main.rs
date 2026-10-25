#![forbid(unsafe_code)]
use craft_mobile_shell::{self as mobile, eframe, egui, files, serde_json};
use vectorcraft_engine::Session;
use vectorcraft_ui_egui::{Services, VectorcraftApp};
struct Editor(VectorcraftApp);
impl mobile::Editor for Editor {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn input(&mut self, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(raw);
    }
    fn command(&mut self, id: &str, p: serde_json::Value) -> Result<(), String> {
        self.0.run(id, p).map(|_| ())
    }
}
fn main() -> eframe::Result<()> {
    mobile::run("VectorCraft Mobile", "vectorcraft", |_cc, _actions| {
        let services = Services {
            read: Some(Box::new(files::read)),
            write: Some(Box::new(files::write)),
            exists: Some(Box::new(|p| files::confined(std::path::Path::new(p), false).is_ok())),
            ..Default::default()
        };
        Editor(VectorcraftApp::new(Session::new(), services))
    })
}
