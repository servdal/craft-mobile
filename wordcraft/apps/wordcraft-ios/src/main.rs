#![forbid(unsafe_code)]
use craft_mobile_shell::{self as mobile, eframe, egui, files, serde_json};
use wordcraft_engine::Session;
use wordcraft_ui_egui::{Services, WordApp};
struct Editor(WordApp);
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
    mobile::run("WordCraft Mobile", "docx", |_cc, _actions| {
        let services = Services {
            file_dialog: Some(Box::new(move |request| {
                let (tx, rx) = std::sync::mpsc::channel();
                let (mode, name) = match request {
                    wordcraft_ui_egui::FileDialogRequest::Open { .. } => (files::Mode::Open, String::new()),
                    wordcraft_ui_egui::FileDialogRequest::Save { name } => (files::Mode::Save, name),
                };
                files::request(mode, name, move |path| {
                    let _ = tx.send(Ok(path.map(|p| p.to_string_lossy().into_owned())));
                });
                rx
            })),
            ..Default::default()
        };
        Editor(WordApp::new(Session::new(wordcraft_doc::Document::new()), services))
    })
}
