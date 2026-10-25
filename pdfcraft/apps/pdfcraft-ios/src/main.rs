#![forbid(unsafe_code)]
use craft_mobile_shell::{self as mobile, eframe, egui, serde_json};
use pdfcraft_ui_egui::{PdfCraftApp, SaveTarget};
struct Editor(PdfCraftApp);
impl mobile::Editor for Editor {
    fn logic(&mut self, c: &egui::Context, f: &mut eframe::Frame) {
        eframe::App::logic(&mut self.0, c, f);
    }
    fn ui(&mut self, u: &mut egui::Ui, f: &mut eframe::Frame) {
        eframe::App::ui(&mut self.0, u, f);
    }
    fn command(&mut self, id: &str, p: serde_json::Value) -> Result<(), String> {
        match id {
            "file.open" => {
                self.0.open_path(p["path"].as_str().ok_or("Missing path")?);
                Ok(())
            }
            "file.save" | "file.saveAs" => {
                self.0.save_override = Some(p["path"].as_str().ok_or("Missing path")?.into());
                let saved = self.0.save_active(SaveTarget::As);
                self.0.save_override = None;
                if saved { Ok(()) } else { Err("PDF could not be saved; inspect the document notice".into()) }
            }
            "file.new" => {
                self.0.execute("create.blank");
                Ok(())
            }
            _ => Err("Command is unavailable".into()),
        }
    }
}
fn main() -> eframe::Result<()> {
    mobile::run("PdfCraft Mobile", "pdf", |_cc, _actions| {
        let mut app = PdfCraftApp::new();
        app.system_clipboard = false;
        Editor(app)
    })
}
