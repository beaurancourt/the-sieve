//! Tauri backend for The Sieve desktop app.
//!
//! Lets the user pick a markdown file and watches it for changes; on each
//! change, rebuilds the PDF (writing it next to the source) and emits a log
//! event to the frontend.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use the_sieve::PageSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

#[derive(Default)]
pub struct WatcherState {
    debouncer: Mutex<Option<Debouncer<notify::RecommendedWatcher>>>,
    current_path: Mutex<Option<PathBuf>>,
    page_size: Mutex<PageSize>,
}

#[derive(Serialize, Clone)]
struct LogEntry {
    timestamp: String,
    level: &'static str,
    message: String,
}

fn emit_log(app: &AppHandle, level: &'static str, message: impl Into<String>) {
    let entry = LogEntry {
        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
        level,
        message: message.into(),
    };
    let _ = app.emit("log", entry);
}

fn rebuild(path: &Path, app: &AppHandle, page_size: PageSize) {
    let start = Instant::now();
    let markdown = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            emit_log(app, "error", format!("Read failed: {e}"));
            return;
        }
    };
    let base = path.parent().map(Path::to_path_buf).unwrap_or_default();
    match the_sieve::convert_markdown_to_pdf(&markdown, &base, page_size) {
        Ok(pdf) => {
            let mut out = path.to_path_buf();
            out.set_extension("pdf");
            match std::fs::write(&out, &pdf) {
                Ok(()) => {
                    let dur = start.elapsed();
                    emit_log(
                        app,
                        "ok",
                        format!("Wrote {} ({} ms)", out.display(), dur.as_millis()),
                    );
                }
                Err(e) => emit_log(app, "error", format!("Write failed: {e}")),
            }
        }
        Err(e) => emit_log(app, "error", format!("Render failed: {e}")),
    }
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    std::fs::metadata(path).ok().map(|m| FileStamp {
        modified: m.modified().ok(),
        len: m.len(),
    })
}

fn file_stamp_changed(
    last_seen_stamp: &Mutex<Option<FileStamp>>,
    current_stamp: Option<FileStamp>,
) -> bool {
    let Some(current_stamp) = current_stamp else {
        return true;
    };

    let mut last_seen = last_seen_stamp.lock().unwrap();
    if last_seen.map(|previous| previous == current_stamp).unwrap_or(false) {
        return false;
    }

    *last_seen = Some(current_stamp);
    true
}

fn watched_file_changed(
    path: &Path,
    last_seen_stamp: &Mutex<Option<FileStamp>>,
) -> bool {
    file_stamp_changed(last_seen_stamp, file_stamp(path))
}

/// Read the current page size from managed state.
fn current_page_size(app: &AppHandle) -> PageSize {
    let ws = app.state::<WatcherState>();
    let ps = *ws.page_size.lock().unwrap();
    ps
}

#[tauri::command]
async fn pick_file(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .add_filter("Markdown", &["md", "markdown"])
        .blocking_pick_file()
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
async fn start_watching(
    path: String,
    page_size: String,
    app: AppHandle,
    state: State<'_, WatcherState>,
) -> Result<(), String> {
    let path = PathBuf::from(&path);
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    // Store the requested page size.
    let ps = parse_page_size(&page_size);
    *state.page_size.lock().unwrap() = ps;

    let app_for_cb = app.clone();
    let path_for_cb = path.clone();
    let last_seen_stamp = Arc::new(Mutex::new(file_stamp(&path)));
    let last_seen_stamp_for_cb = Arc::clone(&last_seen_stamp);
    let mut debouncer = new_debouncer(
        Duration::from_millis(200),
        move |result: DebounceEventResult| match result {
            Ok(events)
                if !events.is_empty()
                    && watched_file_changed(&path_for_cb, &last_seen_stamp_for_cb) =>
            {
                let ps = current_page_size(&app_for_cb);
                rebuild(&path_for_cb, &app_for_cb, ps);
            }
            Ok(_) => {}
            Err(e) => emit_log(&app_for_cb, "error", format!("Watch error: {e}")),
        },
    )
    .map_err(|e| e.to_string())?;

    debouncer
        .watcher()
        .watch(&path, RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;

    // Replace any previous watcher; the old one drops here.
    *state.debouncer.lock().unwrap() = Some(debouncer);
    *state.current_path.lock().unwrap() = Some(path.clone());

    emit_log(&app, "info", format!("Watching {}", path.display()));
    rebuild(&path, &app, ps);
    Ok(())
}

#[tauri::command]
async fn stop_watching(state: State<'_, WatcherState>, app: AppHandle) -> Result<(), String> {
    *state.debouncer.lock().unwrap() = None;
    let path = state.current_path.lock().unwrap().take();
    if let Some(p) = path {
        emit_log(&app, "info", format!("Stopped watching {}", p.display()));
    }
    Ok(())
}

/// Change the page size and immediately rebuild if a file is being watched.
#[tauri::command]
async fn set_page_size(
    page_size: String,
    state: State<'_, WatcherState>,
    app: AppHandle,
) -> Result<(), String> {
    let ps = parse_page_size(&page_size);
    *state.page_size.lock().unwrap() = ps;

    // If we're currently watching a file, trigger an immediate rebuild.
    let path = state.current_path.lock().unwrap().clone();
    if let Some(p) = path {
        rebuild(&p, &app, ps);
    }
    Ok(())
}

fn parse_page_size(s: &str) -> PageSize {
    match s {
        "half-letter" => PageSize::HalfLetter,
        "digest" => PageSize::Digest,
        "letter" => PageSize::Letter,
        "a4" => PageSize::A4,
        "a5" => PageSize::A5,
        _ => PageSize::HalfLetter,
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(WatcherState::default())
        .invoke_handler(tauri::generate_handler![
            pick_file,
            start_watching,
            stop_watching,
            set_page_size
        ])
        .run(tauri::generate_context!())
        .expect("error while running The Sieve desktop app");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_file_stamp_does_not_trigger_rebuild() {
        let stamp = FileStamp {
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(10)),
            len: 123,
        };
        let last_seen = Mutex::new(Some(stamp));

        assert!(!file_stamp_changed(&last_seen, Some(stamp)));
        assert_eq!(*last_seen.lock().unwrap(), Some(stamp));
    }

    #[test]
    fn changed_file_stamp_triggers_once_and_updates_state() {
        let first = FileStamp {
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(10)),
            len: 123,
        };
        let second = FileStamp {
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(20)),
            len: 456,
        };
        let last_seen = Mutex::new(Some(first));

        assert!(file_stamp_changed(&last_seen, Some(second)));
        assert_eq!(*last_seen.lock().unwrap(), Some(second));
        assert!(!file_stamp_changed(&last_seen, Some(second)));
    }

    #[test]
    fn changed_length_triggers_when_modified_time_is_unchanged() {
        let modified = SystemTime::UNIX_EPOCH + Duration::from_secs(10);
        let first = FileStamp {
            modified: Some(modified),
            len: 123,
        };
        let second = FileStamp {
            modified: Some(modified),
            len: 456,
        };
        let last_seen = Mutex::new(Some(first));

        assert!(file_stamp_changed(&last_seen, Some(second)));
        assert_eq!(*last_seen.lock().unwrap(), Some(second));
    }

    #[test]
    fn unknown_file_stamp_still_triggers_rebuild() {
        let stamp = FileStamp {
            modified: Some(SystemTime::UNIX_EPOCH + Duration::from_secs(10)),
            len: 123,
        };
        let last_seen = Mutex::new(Some(stamp));

        assert!(file_stamp_changed(&last_seen, None));
        assert_eq!(*last_seen.lock().unwrap(), Some(stamp));
    }
}
