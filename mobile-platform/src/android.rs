//! Audited JNI boundary; Java handles URI permissions and returns only private local paths.
use super::*;
use std::sync::{Mutex, PoisonError};
static APP: Mutex<Option<android_activity::AndroidApp>> = Mutex::new(None);
pub fn init_android(app: android_activity::AndroidApp) {
    *APP.lock().unwrap_or_else(PoisonError::into_inner) = Some(app);
}
use jni::{
    JavaVM,
    objects::{JObject, JString, JValue},
};
fn with_activity<T>(
    f: impl FnOnce(&mut jni::JNIEnv<'_>, &JObject<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let context = APP
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
        .ok_or("Android activity is not initialized")?;
    // SAFETY: AndroidApp supplies this live VM; the cloned handle keeps its activity reference alive.
    let vm = unsafe { JavaVM::from_raw(context.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    // SAFETY: this is the retained AndroidApp's unowned global activity reference, borrowed only during this call.
    let activity = unsafe { JObject::from_raw(context.activity_as_ptr().cast()) };
    let result = f(&mut env, &activity);
    if env.exception_check().unwrap_or(false) {
        let _ = env.exception_clear();
        return Err("Android document operation failed".into());
    }
    result
}
fn begin(method: &str, path: Option<&Path>) -> Result<(), String> {
    with_activity(|env, activity| {
        let value = if let Some(path) = path {
            let value = JObject::from(
                env.new_string(path.to_string_lossy())
                    .map_err(|e| e.to_string())?,
            );
            env.call_method(
                activity,
                method,
                "(Ljava/lang/String;)Z",
                &[JValue::Object(&value)],
            )
        } else {
            env.call_method(activity, method, "()Z", &[])
        }
        .map_err(|e| e.to_string())?;
        if value.z().map_err(|e| e.to_string())? {
            Ok(())
        } else {
            Err("Another document operation is open".into())
        }
    })
}
pub fn open() -> Result<(), String> {
    begin("openDocuments", None)
}
pub fn photos_open() -> Result<(), String> {
    begin("openPhotos", None)
}
pub fn export(path: &Path) -> Result<(), String> {
    begin("exportFile", Some(path))
}
pub fn photos_save(path: &Path) -> Result<(), String> {
    begin("savePhoto", Some(path))
}
pub fn documents() -> Result<PathBuf, String> {
    with_activity(|env, activity| {
        let value = env
            .call_method(activity, "documentsPath", "()Ljava/lang/String;", &[])
            .and_then(|v| v.l())
            .map_err(|e| e.to_string())?;
        let value = JString::from(value);
        let path: String = env.get_string(&value).map_err(|e| e.to_string())?.into();
        Ok(path.into())
    })
}
pub fn poll() -> Option<Selection> {
    match with_activity(|env, activity| {
        let value = env
            .call_method(activity, "pollResult", "()Ljava/lang/String;", &[])
            .and_then(|v| v.l())
            .map_err(|e| e.to_string())?;
        if value.is_null() {
            return Ok(None);
        }
        let value = JString::from(value);
        let json: String = env.get_string(&value).map_err(|e| e.to_string())?.into();
        Ok(Some(json))
    }) {
        Ok(None) => None,
        Ok(Some(json)) => Some(match serde_json::from_str::<serde_json::Value>(&json) {
            Ok(v) if v["error"].is_string() => {
                Selection::Failed(v["error"].as_str().unwrap_or_default().into())
            }
            Ok(v) if v["path"].is_string() => {
                Selection::Selected(v["path"].as_str().unwrap_or_default().into())
            }
            Ok(_) => Selection::Cancelled,
            Err(_) => Selection::Failed("Android returned an invalid response".into()),
        }),
        Err(error) => Some(Selection::Failed(error)),
    }
}
