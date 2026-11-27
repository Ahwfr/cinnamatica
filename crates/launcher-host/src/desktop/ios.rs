//! UIKit owns presentation; the importing worker waits without blocking the main thread.
use std::{
    ffi::{CStr, c_char, c_void},
    path::PathBuf,
    sync::mpsc,
    time::Duration,
};

unsafe extern "C" {
    fn CinnamaticaPickDocuments(
        max_bytes: u64,
        multiple: bool,
        context: *mut c_void,
        complete: unsafe extern "C" fn(*mut c_void, *const c_char),
    );
    fn pthread_main_np() -> i32;
}

#[derive(serde::Deserialize)]
struct Reply {
    #[serde(default)]
    paths: Vec<PathBuf>,
    #[serde(default)]
    cancelled: bool,
    error: Option<String>,
}

unsafe extern "C" fn completed(context: *mut c_void, json: *const c_char) {
    // The allocation stays owned by the native operation, even if the receiver times
    // out. A late cancellation therefore cannot dereference freed worker state.
    let sender =
        unsafe { Box::from_raw(context.cast::<mpsc::SyncSender<Result<Reply, String>>>()) };
    let result = if json.is_null() {
        Err("Document picker returned no result".into())
    } else {
        unsafe { CStr::from_ptr(json) }
            .to_str()
            .map_err(|e| e.to_string())
            .and_then(|text| serde_json::from_str(text).map_err(|e| e.to_string()))
    };
    let _ = sender.send(result);
}

pub(super) fn pick_documents(
    multiple: bool,
    max_bytes: Option<u64>,
) -> Result<Vec<PathBuf>, String> {
    if unsafe { pthread_main_np() } != 0 {
        return Err("Open the file picker from the import worker".into());
    }
    let (sender, receiver) = mpsc::sync_channel::<Result<Reply, String>>(1);
    let context = Box::into_raw(Box::new(sender)).cast::<c_void>();
    unsafe {
        CinnamaticaPickDocuments(
            max_bytes.unwrap_or(64 * 1024 * 1024),
            multiple,
            context,
            completed,
        )
    };
    let reply = receiver
        .recv_timeout(Duration::from_secs(240))
        .map_err(|_| {
            "File selection timed out; close the picker before trying again".to_owned()
        })??;
    if let Some(error) = reply.error {
        return Err(error);
    }
    if reply.cancelled {
        return Ok(Vec::new());
    }
    Ok(reply.paths)
}
