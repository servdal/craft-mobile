#![forbid(unsafe_code)]
use craft_mobile_shell::{self as mobile, eframe, egui, serde_json};
use gridcraft_engine::Session;
use gridcraft_ui_egui::{Services, SheetApp};
struct Editor(SheetApp);
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
    mobile::run("GridCraft Mobile", "xlsx", |_cc, _actions| {
        let services = Services { ..Default::default() };
        let mut session = Session::new();
        if session.documents().is_empty() {
            session.new_workbook();
        }
        Editor(SheetApp::new(session, services))
    })
}
