#![forbid(unsafe_code)]
use cadcraft_engine::Session;
use cadcraft_ui_egui::{CadApp, Services};
use craft_mobile_shell::{self as mobile, eframe, egui, serde_json};
struct Editor(CadApp);
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
        let id = match id {
            "file.new" => "new",
            "file.open" => "open",
            "file.saveAs" => "saveas",
            other => other,
        };
        self.0.run(id, p).map(|_| ())
    }
}
fn main() -> eframe::Result<()> {
    install_io();
    mobile::run("CADCraft Mobile", "dxf", |_cc, _actions| {
        let services = Services { pick_open: None, pick_save: None };
        let session = Session::new();
        Editor(CadApp::new(session, services))
    })
}

pub fn install_io() {
    cadcraft_engine::cmd::file::set_io(cadcraft_engine::cmd::file::IoHooks {
        read: |b, name| cadcraft_io::read(b, name).map_err(|e| e.to_string()),
        write: |d, name| cadcraft_io::write(d, name).map_err(|e| e.to_string()),
        plot: Some(|d, space, opts| cadcraft_io::plotter::plot(d, space, opts).map_err(|e| e.to_string())),
    });
    cadcraft_engine::cmd::file::set_write_as(|d, name, o| {
        cadcraft_io::write_with(d, name, &cadcraft_io::SaveOptions { version: o.version, binary: o.binary }).map_err(|e| e.to_string())
    });
    // AutoLISP's entget, entmake and entmod see entities as DXF groups.
    cadcraft_engine::cmd::file::set_entity_codec(cadcraft_engine::cmd::file::EntityCodec {
        groups: cadcraft_io::entity_groups,
        entity: cadcraft_io::entity_from_groups,
    });
}
