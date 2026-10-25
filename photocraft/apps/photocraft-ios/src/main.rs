#![forbid(unsafe_code)]
use craft_mobile_shell::{self as mobile, eframe, egui, files, serde_json};
use photocraft_ui_egui::{FileDialogAnswer, FileDialogRequest, PhotocraftApp, Services};
struct Editor(PhotocraftApp);
impl mobile::Editor for Editor {
    fn logic(&mut self, c: &egui::Context, f: &mut eframe::Frame) {
        eframe::App::logic(&mut self.0, c, f);
    }
    fn ui(&mut self, u: &mut egui::Ui, f: &mut eframe::Frame) {
        eframe::App::ui(&mut self.0, u, f);
    }
    fn command(&mut self, id: &str, p: serde_json::Value) -> Result<(), String> {
        self.0.run(id, p).map(|_| ())
    }
}
fn main() -> eframe::Result<()> {
    mobile::run("PhotoCraft Mobile", "pcraft", |cc, _actions| {
        let services = Services {
            import: Some(Box::new(|name, bytes, depth| {
                mobile::guarded(|| photocraft_io::import_with_svg_group_depth(name, bytes, depth).map(|r| (r.document, r.warnings)).map_err(|e| e.to_string()))
            })),
            export: Some(Box::new(|doc, path, settings| {
                mobile::guarded(|| photocraft_io::export(doc, path, &settings.io_options()).map(|r| (r.bytes, r.warnings)).map_err(|e| e.to_string()))
            })),
            write: Some(Box::new(files::write)),
            file_dialog: Some(Box::new(|req, _frame, reply| {
                let (mode, name) = match &req {
                    FileDialogRequest::Open { .. } => (files::Mode::Open, String::new()),
                    FileDialogRequest::Save { suggested } => (files::Mode::Save, suggested.clone()),
                    FileDialogRequest::Folder { .. } => (files::Mode::Folder, String::new()),
                };
                files::request(mode, name, move |p| {
                    reply.send(p.map(|p| match req {
                        FileDialogRequest::Open { .. } => FileDialogAnswer::Paths(vec![p.to_string_lossy().into_owned()]),
                        FileDialogRequest::Save { .. } => FileDialogAnswer::SaveTo(p.to_string_lossy().into_owned()),
                        FileDialogRequest::Folder { .. } => FileDialogAnswer::FolderTo(p.to_string_lossy().into_owned()),
                    }));
                });
            })),
            ..Default::default()
        };
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), services);
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        if let Some(rs) = &cc.wgpu_render_state {
            app.set_wgpu(rs.clone());
        }
        Editor(app)
    })
}
