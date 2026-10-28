#![forbid(unsafe_code)]
use std::{
    collections::VecDeque,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex, OnceLock, PoisonError},
    task::{Context, Poll, Waker},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Save,
    Folder,
    Export,
    Photos,
    SavePhotos,
}
pub struct Request {
    pub mode: Mode,
    pub name: String,
    pub title: String,
    pub extensions: Vec<String>,
    pub initial: Option<PathBuf>,
    pub answer: Box<dyn FnOnce(Option<PathBuf>) + Send>,
}
pub type Queue = Arc<Mutex<VecDeque<Request>>>;
static QUEUE: OnceLock<Queue> = OnceLock::new();
pub fn queue() -> Queue {
    QUEUE.get_or_init(Default::default).clone()
}
pub fn request(
    mode: Mode,
    name: impl Into<String>,
    answer: impl FnOnce(Option<PathBuf>) + Send + 'static,
) {
    queue()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push_back(Request {
            mode,
            name: name.into(),
            title: "Documents".into(),
            extensions: vec![],
            initial: None,
            answer: Box::new(answer),
        });
}
/// Present the platform's destination picker after an editor has saved a local copy.
/// The editor never receives a path outside its sandbox; iOS/Android copy the finished
/// document to the user-selected Files/Documents provider.
pub fn export(path: PathBuf, answer: impl FnOnce(Option<PathBuf>) + Send + 'static) {
    queue()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push_back(Request {
            mode: Mode::Export,
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Document")
                .into(),
            title: "Export to Files".into(),
            extensions: vec![],
            initial: Some(path),
            answer: Box::new(answer),
        });
}
pub fn documents() -> Result<PathBuf, String> {
    if let Some(path) = craft_mobile_platform::documents_directory()? {
        std::fs::create_dir_all(&path).map_err(|error| error.to_string())?;
        return path.canonicalize().map_err(|error| error.to_string());
    }
    let home = std::env::var_os("HOME").ok_or("Application sandbox is unavailable")?;
    let p = PathBuf::from(home).join("Documents");
    std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    p.canonicalize().map_err(|e| e.to_string())
}
pub fn confined(path: &Path, write: bool) -> Result<PathBuf, String> {
    confined_in(&documents()?, path, write)
}
fn confined_in(root: &Path, path: &Path, write: bool) -> Result<PathBuf, String> {
    let resolved = if write {
        let parent = path
            .parent()
            .ok_or("Missing folder")?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        parent.join(path.file_name().ok_or("Missing file name")?)
    } else {
        path.canonicalize().map_err(|e| e.to_string())?
    };
    if !resolved.starts_with(root) {
        return Err("Choose a file inside this application's Documents folder".into());
    }
    if write && path.exists() {
        let actual = path.canonicalize().map_err(|e| e.to_string())?;
        if !actual.starts_with(root) {
            return Err("File points outside Documents".into());
        }
    }
    Ok(resolved)
}
pub fn read(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(confined(Path::new(path), false)?).map_err(|e| e.to_string())
}
pub fn write(path: &str, data: &[u8]) -> Result<(), String> {
    let p = confined(Path::new(path), true)?;
    atomic_write(&p, data).map_err(|e| e.to_string())
}
fn atomic_write(p: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp = p.with_file_name(format!(
        ".mobile-{}-{}.tmp",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp)?;
    let result = (|| {
        f.write_all(data)?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, &p)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}
fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && !s.chars().any(|c| c == '/' || c == '\\' || c.is_control())
        && s.len() <= 240
}
pub struct Browser {
    root: PathBuf,
    folder: PathBuf,
    pending: Option<Request>,
    name: String,
    error: String,
    replace: bool,
    native_pending: bool,
}
impl Browser {
    pub fn new() -> Result<Self, String> {
        let root = documents()?;
        let projects = root.join("Projects");
        if !projects.exists() {
            std::fs::create_dir_all(&projects).map_err(|error| error.to_string())?;
        }
        Ok(Self {
            folder: root.clone(),
            root,
            pending: None,
            name: String::new(),
            error: String::new(),
            replace: false,
            native_pending: false,
        })
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        if self.pending.is_none() {
            self.pending = queue()
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .pop_front();
            if let Some(p) = &mut self.pending {
                if matches!(p.mode, Mode::Photos | Mode::SavePhotos) {
                    p.extensions = ["png", "jpg", "jpeg", "heic", "heif"]
                        .into_iter()
                        .map(str::to_owned)
                        .collect();
                }
                if let Some(dir) = &p.initial {
                    if let Ok(actual) = confined(dir, false) {
                        if actual.is_dir() {
                            self.folder = actual;
                        }
                    }
                }
                self.name = Path::new(&p.name)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .into();
                self.error.clear();
                self.replace = false;
                #[cfg(any(target_os = "ios", target_os = "android"))]
                if matches!(p.mode, Mode::Open | Mode::Photos)
                    || (p.mode == Mode::Export && p.initial.is_some())
                {
                    let opened = if p.mode == Mode::Photos {
                        craft_mobile_platform::photos_open()
                    } else if p.mode == Mode::Export {
                        let Some(path) = p.initial.as_deref() else {
                            self.error = "Missing document to export".into();
                            return;
                        };
                        craft_mobile_platform::export(path)
                    } else {
                        craft_mobile_platform::open()
                    };
                    match opened {
                        Ok(()) => self.native_pending = true,
                        Err(e) => self.error = e,
                    }
                }
            }
        }
        if self.native_pending {
            if let Some(result) = craft_mobile_platform::poll() {
                self.native_pending = false;
                match result {
                    craft_mobile_platform::Selection::Selected(path) => {
                        match confined(&path, false) {
                            Ok(path) => {
                                if let Some(request) = self.pending.take() {
                                    (request.answer)(Some(path));
                                }
                                return;
                            }
                            Err(error) => self.error = error,
                        }
                    }
                    craft_mobile_platform::Selection::Cancelled => {
                        if let Some(request) = self.pending.take() {
                            (request.answer)(None);
                        }
                        return;
                    }
                    craft_mobile_platform::Selection::Failed(error) => self.error = error,
                }
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
                return;
            }
        }
        let Some(req) = &self.pending else { return };
        let mode = req.mode;
        let title = req.title.clone();
        let extensions = req.extensions.clone();
        let mut chosen = None;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("mobile-documents")).show(ctx, |ui| {
            ui.set_max_width((ctx.content_rect().width() - 30.0).clamp(240.0, 600.0));
            ui.heading(title);
            ui.label(if mode == Mode::Export {
                "Choose a saved document to export to Files."
            } else if mode == Mode::SavePhotos {
                "Choose a saved PNG, JPEG or HEIC image to add to Photos."
            } else {
                "Documents saved in this app."
            });
            if matches!(mode, Mode::Open | Mode::Photos) && ui.button("Browse Files…").clicked() {
                match craft_mobile_platform::open() {
                    Ok(()) => self.native_pending = true,
                    Err(e) => self.error = e,
                }
            }
            if matches!(mode, Mode::Open | Mode::Photos) && ui.button("Browse Photos…").clicked()
            {
                match craft_mobile_platform::photos_open() {
                    Ok(()) => self.native_pending = true,
                    Err(e) => self.error = e,
                }
            }
            ui.label(
                self.folder
                    .strip_prefix(&self.root)
                    .unwrap_or(Path::new("Documents"))
                    .display()
                    .to_string(),
            );
            if self.folder != self.root && ui.button("Up").clicked() {
                if let Some(p) = self.folder.parent() {
                    self.folder = p.to_path_buf();
                }
            }
            let entries = std::fs::read_dir(&self.folder)
                .map(|xs| xs.filter_map(Result::ok).take(2000).collect::<Vec<_>>());
            match entries {
                Ok(mut files) => {
                    files.sort_by_key(|e| e.file_name());
                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .show(ui, |ui| {
                            for entry in files {
                                let path = entry.path();
                                let name = entry.file_name().to_string_lossy().into_owned();
                                if name.starts_with('.') {
                                    continue;
                                }
                                if path.is_file()
                                    && !extensions.is_empty()
                                    && !extensions.iter().any(|x| x == "*")
                                    && !path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                                        extensions.iter().any(|x| x.eq_ignore_ascii_case(e))
                                    })
                                {
                                    continue;
                                }
                                if path.is_dir() {
                                    if ui.button(format!("▸ {name}")).clicked() {
                                        match confined(&path, false) {
                                            Ok(p) => self.folder = p,
                                            Err(e) => self.error = e,
                                        }
                                    }
                                } else if mode != Mode::Folder && ui.button(&name).clicked() {
                                    if mode == Mode::Save {
                                        self.name = name;
                                        self.replace = false
                                    } else {
                                        match confined(&path, false) {
                                            Ok(p) => chosen = Some(p),
                                            Err(e) => self.error = e,
                                        }
                                    }
                                }
                            }
                        });
                }
                Err(e) => {
                    ui.label(e.to_string());
                }
            }
            if mode == Mode::Save {
                if ui.text_edit_singleline(&mut self.name).changed() {
                    self.replace = false;
                }
                if ui
                    .button(if self.replace {
                        "Confirm replace"
                    } else {
                        "Save"
                    })
                    .clicked()
                {
                    if !valid_name(&self.name) {
                        self.error = "Enter a file name without a folder path".into()
                    } else {
                        let p = self.folder.join(&self.name);
                        if p.exists() && !self.replace {
                            self.replace = true;
                            self.error = "This file exists. Confirm replace to overwrite it.".into()
                        } else {
                            match confined(&p, true) {
                                Ok(p) => chosen = Some(p),
                                Err(e) => self.error = e,
                            }
                        }
                    }
                }
            }
            if mode == Mode::Folder && ui.button("Choose this folder").clicked() {
                chosen = Some(self.folder.clone());
            }
            ui.colored_label(egui::Color32::from_rgb(185, 45, 45), &self.error);
            cancel = ui.button("Cancel").clicked();
        });
        if matches!(mode, Mode::Export | Mode::SavePhotos)
            && let Some(path) = chosen.take()
        {
            let operation = if mode == Mode::SavePhotos {
                craft_mobile_platform::photos_save(&path)
            } else {
                craft_mobile_platform::export(&path)
            };
            match operation {
                Ok(()) => self.native_pending = true,
                Err(e) => self.error = e,
            }
        }
        if cancel || chosen.is_some() {
            if let Some(req) = self.pending.take() {
                (req.answer)(chosen);
            }
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}
// Async dialog facade used by PdfCraft. It queues a real in-app document picker; no modal OS loop.
#[derive(Clone, Default)]
pub struct AsyncFileDialog {
    name: String,
    title: String,
    extensions: Vec<String>,
    initial: Option<PathBuf>,
}
impl AsyncFileDialog {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_title(mut self, s: impl AsRef<str>) -> Self {
        self.title = s.as_ref().into();
        self
    }
    pub fn set_file_name(mut self, s: impl AsRef<str>) -> Self {
        self.name = s.as_ref().into();
        self
    }
    pub fn set_directory(mut self, p: impl AsRef<Path>) -> Self {
        self.initial = Some(p.as_ref().to_path_buf());
        self
    }
    pub fn add_filter(mut self, _label: impl AsRef<str>, ext: &[impl AsRef<str>]) -> Self {
        self.extensions.extend(
            ext.iter()
                .map(|s| s.as_ref().trim_start_matches('.').to_string()),
        );
        self
    }
    fn pick(self, mode: Mode) -> PickFuture {
        let state = Arc::new(Mutex::new((false, None, None::<Waker>)));
        let next = state.clone();
        queue()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push_back(Request {
                mode,
                name: self.name,
                title: if self.title.is_empty() {
                    "Documents".into()
                } else {
                    self.title
                },
                extensions: self.extensions,
                initial: self.initial,
                answer: Box::new(move |value| {
                    let mut s = next.lock().unwrap_or_else(PoisonError::into_inner);
                    s.0 = true;
                    s.1 = value.map(FileHandle);
                    if let Some(w) = s.2.take() {
                        w.wake();
                    }
                }),
            });
        PickFuture(state)
    }
    pub fn pick_file(self) -> PickFuture {
        self.pick(Mode::Open)
    }
    pub fn save_file(self) -> PickFuture {
        self.pick(Mode::Save)
    }
    pub fn pick_folder(self) -> PickFuture {
        self.pick(Mode::Folder)
    }
    pub async fn pick_files(self) -> Option<Vec<FileHandle>> {
        self.pick_file().await.map(|p| vec![p])
    }
}
pub struct PickFuture(Arc<Mutex<(bool, Option<FileHandle>, Option<Waker>)>>);
impl Future for PickFuture {
    type Output = Option<FileHandle>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut s = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if s.0 {
            Poll::Ready(s.1.take())
        } else {
            s.2 = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
#[derive(Clone, Debug)]
pub struct FileHandle(PathBuf);
impl FileHandle {
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn file_name(&self) -> String {
        self.0
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
    pub async fn read(&self) -> std::io::Result<Vec<u8>> {
        std::fs::read(confined(&self.0, false).map_err(std::io::Error::other)?)
    }
    pub async fn write(&self, data: &[u8]) -> std::io::Result<()> {
        write(&self.0.to_string_lossy(), data).map_err(std::io::Error::other)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_names_reject_traversal() {
        for s in ["", ".", "..", "../doc", "a/b", "a\\b", "a\0b"] {
            assert!(!valid_name(s));
        }
        assert!(valid_name("Drawing 1.dxf"));
    }
    #[test]
    fn async_picker_waits_for_user() {
        let future = AsyncFileDialog::new().pick_file();
        let req = queue()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pop_front()
            .unwrap();
        (req.answer)(None);
        assert!(future.0.lock().unwrap().0);
    }
}

#[cfg(test)]
mod filesystem_tests {
    use super::*;
    #[test]
    fn confinement_and_atomic_replace() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("craft-mobile-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir)?;
        let root = dir.canonicalize()?;
        let path = root.join("sample.txt");
        atomic_write(&path, b"before")?;
        atomic_write(&path, b"after")?;
        assert_eq!(std::fs::read(&path)?, b"after");
        assert!(confined_in(&root, &root.join("../outside"), true).is_err());
        #[cfg(unix)]
        {
            let link = root.join("escape");
            std::os::unix::fs::symlink(std::env::temp_dir(), &link)?;
            assert!(confined_in(&root, &link, false).is_err());
            assert!(confined_in(&root, &link.join("outside"), true).is_err());
        }
        assert!(atomic_write(&root, b"must not replace directory").is_err());
        assert!(path.exists());
        std::fs::remove_dir_all(dir)?;
        Ok(())
    }
}

#[cfg(target_os = "android")]
pub use craft_mobile_platform::init_android;
