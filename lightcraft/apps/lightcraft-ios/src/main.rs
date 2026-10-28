#![forbid(unsafe_code)]
use craft_mobile_shell_v37::{self as mobile, eframe, egui, serde_json};
use lightcraft_engine::Session;
use lightcraft_ui_egui::{LightcraftApp, Services};

struct Editor(LightcraftApp);

impl mobile::Editor for Editor {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) { self.0.logic(ctx); }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) { self.0.ui(ui); }
    fn input(&mut self, raw: &mut egui::RawInput) { self.0.raw_input_hook(&egui::Context::default(), raw); }
    fn command(&mut self, id: &str, p: serde_json::Value) -> Result<(), String> { self.0.run(id, p).map(|_| ()).map_err(|error| error.to_string()) }
}

fn main() -> eframe::Result<()> {
    mobile::run("LightCraft Mobile", "lccatalog", |_cc, _actions| Editor(LightcraftApp::new(Session::new(), Services::default())))
}
