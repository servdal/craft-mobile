//! Isolated UIKit ABI boundary. All editor and file-browser code remains safe Rust.
use std::path::{Path, PathBuf};
#[derive(Debug)]
pub enum Selection {
    Selected(PathBuf),
    Cancelled,
    Failed(String),
}
#[cfg(target_os = "ios")]
mod ios {
    use super::*;
    use std::ffi::{CStr, CString, c_char};
    unsafe extern "C" {
        fn craft_documents_open() -> bool;
        fn craft_documents_export(path: *const c_char) -> bool;
        fn craft_documents_poll() -> *mut c_char;
        fn craft_documents_free(value: *mut c_char);
    }
    pub fn open() -> Result<(), String> {
        // SAFETY: no borrowed arguments; native implementation dispatches UIKit work on its main queue.
        if unsafe { craft_documents_open() } {
            Ok(())
        } else {
            Err("Another Files operation is open".into())
        }
    }
    pub fn export(path: &Path) -> Result<(), String> {
        let value =
            CString::new(path.to_string_lossy().as_bytes()).map_err(|_| "Invalid file name")?;
        // SAFETY: the native function copies this valid NUL-terminated string before returning.
        if unsafe { craft_documents_export(value.as_ptr()) } {
            Ok(())
        } else {
            Err("Another Files operation is open".into())
        }
    }
    pub fn poll() -> Option<Selection> {
        // SAFETY: native returns null or one owned, NUL-terminated UTF-8 JSON allocation.
        let value = unsafe { craft_documents_poll() };
        if value.is_null() {
            return None;
        }
        // SAFETY: allocation remains live until craft_documents_free below; copy before freeing it.
        let bytes = unsafe { CStr::from_ptr(value) }.to_bytes().to_vec();
        // SAFETY: this pointer was allocated by craft_documents_poll and is freed exactly once here.
        unsafe { craft_documents_free(value) };
        Some(match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(v) => {
                if let Some(error) = v.get("error").and_then(|e| e.as_str()) {
                    Selection::Failed(error.into())
                } else if let Some(path) = v.get("path").and_then(|e| e.as_str()) {
                    Selection::Selected(path.into())
                } else {
                    Selection::Cancelled
                }
            }
            Err(_) => Selection::Failed("Files returned an invalid response".into()),
        })
    }
}
#[cfg(target_os = "ios")]
pub use ios::{export, open, poll};
#[cfg(not(target_os = "ios"))]
pub fn open() -> Result<(), String> {
    Err("Files picker requires iOS".into())
}
#[cfg(not(target_os = "ios"))]
pub fn export(_path: &Path) -> Result<(), String> {
    Err("Files export requires iOS".into())
}
#[cfg(not(target_os = "ios"))]
pub fn poll() -> Option<Selection> {
    None
}
