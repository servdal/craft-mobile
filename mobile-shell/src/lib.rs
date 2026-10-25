#![forbid(unsafe_code)]
mod guard;
pub use craft_mobile_files as files;
pub use eframe;
pub use egui;
pub use guard::guarded;
pub use serde_json;
use std::sync::{Arc, Mutex, PoisonError};
pub type Actions = Arc<Mutex<Vec<(String, serde_json::Value)>>>;
pub fn pick(actions: &Actions, save: bool, name: &str) {
    let q = actions.clone();
    files::request(
        if save {
            files::Mode::Save
        } else {
            files::Mode::Open
        },
        name,
        move |path| {
            if let Some(path) = path {
                q.lock().unwrap_or_else(PoisonError::into_inner).push((
                    if save { "file.saveAs" } else { "file.open" }.into(),
                    serde_json::json!({"path":path.to_string_lossy()}),
                ));
            }
        },
    );
}
pub trait Editor {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame);
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame);
    fn input(&mut self, _raw: &mut egui::RawInput) {}
    fn command(&mut self, id: &str, p: serde_json::Value) -> Result<(), String>;
}
struct Mobile<E> {
    editor: E,
    actions: Actions,
    browser: Option<files::Browser>,
    status: String,
    title: &'static str,
    extension: &'static str,
    show_desktop_tools: bool,
}
impl<E: Editor> eframe::App for Mobile<E> {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let actions =
            std::mem::take(&mut *self.actions.lock().unwrap_or_else(PoisonError::into_inner));
        for (id, p) in actions {
            if id == "mobile.photosSaved" {
                self.status = "Image saved to Photos.".into();
                continue;
            }
            if id == "mobile.photoImported" {
                let path = p["path"].as_str().unwrap_or_default();
                let command = match self.extension {
                    "pcraft" => Some("file.open"),
                    "vectorcraft" => Some("file.place"),
                    "docx" | "xlsx" | "pptx" => Some("insert.picture"),
                    _ => None,
                };
                self.status = match command {
                    Some(command) => match guarded(|| self.editor.command(command, serde_json::json!({"path":path,"link":false}))) {
                        Ok(()) => "Photo imported.".into(),
                        Err(error) => format!("Photo copied to Documents, but the editor could not insert it: {error}"),
                    },
                    None => "Photo copied to Documents/Imports. This editor does not yet insert photos from the mobile toolbar.".into(),
                };
                continue;
            }
            if let Err(e) = guarded(|| self.editor.command(&id, p)) {
                self.status = e;
            }
        }
        self.editor.logic(ctx, frame);
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.editor.input(raw)
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let bounds = ui.ctx().content_rect();
        let mut content = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("mobile-safe-area")
                .max_rect(bounds),
        );
        content.set_clip_rect(bounds);
        let ui = &mut content;
        let naturally_compact = mobile_ui::compact_size(
            bounds.size(),
            false,
            mobile_ui::is_phone(ui.ctx()),
        );
        let compact = naturally_compact && !self.show_desktop_tools;
        mobile_ui::set_phone(ui.ctx(), compact);
        egui::Panel::top("mobile-file-actions").show(ui, |ui| {
            if compact {
                ui.horizontal(|ui| {
                    ui.strong(self.title.trim_end_matches(" Mobile"));
                    if ui.add_sized([56.0, mobile_ui::TOUCH_HEIGHT], egui::Button::new("New")).clicked() {
                        if let Err(e) = guarded(|| self.editor.command("file.new", serde_json::json!({}))) {
                            self.status = e;
                        }
                    }
                    if ui.add_sized([64.0, mobile_ui::TOUCH_HEIGHT], egui::Button::new("Open")).clicked() {
                        pick(&self.actions, false, "");
                    }
                    ui.menu_button("Menu", |ui| {
                        if mobile_ui::button(ui, "Choose from Photos").clicked() {
                            let actions = self.actions.clone();
                            files::request(files::Mode::Photos, "", move |path| {
                                if let Some(path) = path {
                                    actions.lock().unwrap_or_else(PoisonError::into_inner).push((
                                        "mobile.photoImported".into(),
                                        serde_json::json!({"path":path.to_string_lossy()}),
                                    ));
                                }
                            });
                            ui.close();
                        }
                        if mobile_ui::button(ui, "Save to Photos").clicked() {
                            let actions = self.actions.clone();
                            files::request(files::Mode::SavePhotos, "", move |path| {
                                if path.is_some() {
                                    actions.lock().unwrap_or_else(PoisonError::into_inner).push(("mobile.photosSaved".into(), serde_json::json!({})));
                                }
                            });
                            ui.close();
                        }
                        if mobile_ui::button(ui, "Save As File").clicked() {
                            pick(&self.actions, true, &format!("Untitled.{}", self.extension));
                            ui.close();
                        }
                        if mobile_ui::button(ui, "Export to Files").clicked() {
                            files::request(files::Mode::Export, "", |_| {});
                            ui.close();
                        }
                        ui.separator();
                        if mobile_ui::button(ui, "Show desktop tools").clicked() {
                            self.show_desktop_tools = true;
                            ui.close();
                        }
                    });
                });
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.strong(self.title);
                    if ui.button("New").clicked() {
                    if let Err(e) =
                        guarded(|| self.editor.command("file.new", serde_json::json!({})))
                    {
                        self.status = e;
                    }
                }
                if ui.button("Open").clicked() {
                    pick(&self.actions, false, "");
                }
                if ui.button("Photos").clicked() {
                    let actions = self.actions.clone();
                    files::request(files::Mode::Photos, "", move |path| {
                        if let Some(path) = path {
                            actions
                                .lock()
                                .unwrap_or_else(PoisonError::into_inner)
                                .push((
                                    "mobile.photoImported".into(),
                                    serde_json::json!({"path":path.to_string_lossy()}),
                                ));
                        }
                    });
                }
                if ui.button("Save to Photos").clicked() {
                    let actions = self.actions.clone();
                    files::request(files::Mode::SavePhotos, "", move |path| {
                        if path.is_some() {
                            actions
                                .lock()
                                .unwrap_or_else(PoisonError::into_inner)
                                .push(("mobile.photosSaved".into(), serde_json::json!({})));
                        }
                    });
                }
                if ui.button("Export to Files").clicked() {
                    files::request(files::Mode::Export, "", |_| {});
                }
                if ui.button("Save As").clicked() {
                    pick(&self.actions, true, &format!("Untitled.{}", self.extension));
                }
                if naturally_compact && ui.button("Phone layout").clicked() {
                    self.show_desktop_tools = false;
                }
                });
            }
            if !self.status.is_empty() {
                ui.label(&self.status);
                if ui.small_button("Dismiss").clicked() {
                    self.status.clear();
                }
            }
        });
        self.editor.ui(ui, frame);
        if let Some(b) = &mut self.browser {
            b.show(ui.ctx());
        }
    }
}
pub fn run<E: Editor + 'static>(
    title: &'static str,
    extension: &'static str,
    create: impl FnOnce(&eframe::CreationContext<'_>, Actions) -> E + 'static,
) -> eframe::Result<()> {
    if !matches!(std::env::consts::OS, "ios" | "android") {
        return Err(eframe::Error::AppCreation(
            "This mobile host requires an iOS or Android target. Use the matching packaging helper.".into(),
        ));
    }
    let _ = log::set_logger(&STARTUP_LOG);
    log::set_max_level(log::LevelFilter::Warn);
    let mut gpu = eframe::WgpuConfiguration::default();
    if let eframe::egui_wgpu::WgpuSetup::CreateNew(setup) = &mut gpu.wgpu_setup {
        setup.device_descriptor = Arc::new(|adapter| {
            let supported = adapter.limits();
            let mut required = eframe::wgpu::Limits::default();
            // Metal Simulator advertises 15 varyings rather than WebGPU's default 16.
            required.max_inter_stage_shader_variables = required
                .max_inter_stage_shader_variables
                .min(supported.max_inter_stage_shader_variables);
            required.max_texture_dimension_2d = 8192.min(supported.max_texture_dimension_2d);
            eframe::wgpu::DeviceDescriptor {
                label: Some("Craft Mobile"),
                required_limits: required,
                ..Default::default()
            }
        });
    }
    let actions = Actions::default();
    #[cfg(debug_assertions)]
    if std::env::var_os("CRAFT_MOBILE_QA_OPEN").is_some() {
        pick(&actions, false, "");
    }

    #[cfg(debug_assertions)]
    if std::env::var_os("CRAFT_MOBILE_QA_PHOTOS").is_some() {
        let actions = actions.clone();
        files::request(files::Mode::Photos, "", move |path| {
            if let Some(path) = path {
                actions
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push((
                        "mobile.photoImported".into(),
                        serde_json::json!({"path":path.to_string_lossy()}),
                    ));
            }
        });
    }
    #[cfg(target_os = "android")]
    let android_app = ANDROID_APP
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    eframe::run_native(
        title,
        eframe::NativeOptions {
            #[cfg(target_os = "android")]
            android_app,
            #[cfg(target_os = "android")]
            run_and_return: false,
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: gpu,
            ..Default::default()
        },
        Box::new(move |cc| {
            let browser = files::Browser::new();
            let status = browser.as_ref().err().cloned().unwrap_or_default();
            Ok(Box::new(Mobile {
                editor: create(cc, actions.clone()),
                actions,
                browser: browser.ok(),
                status,
                title,
                extension,
                show_desktop_tools: false,
            }))
        }),
    )
}

struct StartupLog;
impl log::Log for StartupLog {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Warn
    }
    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let message = format!("{}: {}\n", record.level(), record.args());
        eprint!("{message}");
        if let Ok(root) = files::documents() {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(root.join(".startup.log"))
            {
                let _ = file.write_all(message.as_bytes());
            }
        }
    }
    fn flush(&self) {}
}
static STARTUP_LOG: StartupLog = StartupLog;

#[cfg(target_os = "android")]
pub use android_activity::AndroidApp;
#[cfg(target_os = "android")]
static ANDROID_APP: Mutex<Option<AndroidApp>> = Mutex::new(None);
#[cfg(target_os = "android")]
pub fn init_android(app: AndroidApp) {
    files::init_android(app.clone());
    *ANDROID_APP.lock().unwrap_or_else(PoisonError::into_inner) = Some(app);
}
