use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("Watch error: {0}")]
    WatchError(String),
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Lock error")]
    LockError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub path: String,
    pub event_type: String,
    pub timestamp: String,
}

pub struct FileWatcher {
    watchers: Mutex<HashMap<String, WatcherHandle>>,
    events: Arc<Mutex<Vec<FileEvent>>>,
}

struct WatcherHandle {
    _watcher: RecommendedWatcher,
    stop_sender: Sender<()>,
}

impl FileWatcher {
    pub fn new() -> Self {
        FileWatcher {
            watchers: Mutex::new(HashMap::new()),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_recent_events(&self, limit: usize) -> Vec<FileEvent> {
        match self.events.lock() {
            Ok(events) => {
                let total = events.len();
                let start = if total > limit { total - limit } else { 0 };
                events[start..].to_vec()
            }
            Err(_) => Vec::new(),
        }
    }

    pub fn start_watching(&self, path: &str) -> Result<(), WatcherError> {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(WatcherError::PathNotFound(path.display().to_string()));
        }

        let path_str = path.display().to_string();

        {
            let watchers = self.watchers.lock().map_err(|_| WatcherError::LockError)?;
            if watchers.contains_key(&path_str) {
                return Err(WatcherError::WatchError(
                    "Already watching this path".to_string(),
                ));
            }
        }

        let (stop_tx, stop_rx) = channel();
        let (event_tx, event_rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = event_tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| WatcherError::WatchError(e.to_string()))?;

        watcher
            .watch(&path, RecursiveMode::Recursive)
            .map_err(|e| WatcherError::WatchError(e.to_string()))?;

        let events = self.events.clone();

        thread::spawn(move || loop {
            if stop_rx.try_recv().is_ok() {
                break;
            }

            if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(500)) {
                let event_type = match event.kind {
                    EventKind::Create(_) => "create",
                    EventKind::Modify(_) => "modify",
                    EventKind::Remove(_) => "remove",
                    EventKind::Access(_) => "access",
                    EventKind::Other => "other",
                    _ => "unknown",
                };

                for path in event.paths {
                    let file_event = FileEvent {
                        path: path.display().to_string(),
                        event_type: event_type.to_string(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };
                    eprintln!("[WATCHER] {} - {}", file_event.event_type, file_event.path);

                    if let Ok(mut ev) = events.lock() {
                        ev.push(file_event);
                        if ev.len() > 1000 {
                            ev.remove(0);
                        }
                    }
                }
            }
        });

        let mut watchers = self.watchers.lock().map_err(|_| WatcherError::LockError)?;
        watchers.insert(
            path_str,
            WatcherHandle {
                _watcher: watcher,
                stop_sender: stop_tx,
            },
        );

        Ok(())
    }

    pub fn stop_watching(&self, path: &str) -> Result<(), WatcherError> {
        let mut watchers = self.watchers.lock().map_err(|_| WatcherError::LockError)?;

        if let Some(handle) = watchers.remove(path) {
            let _ = handle.stop_sender.send(());
            Ok(())
        } else {
            Err(WatcherError::WatchError(
                "Path not being watched".to_string(),
            ))
        }
    }

    pub fn get_watched_paths(&self) -> Result<Vec<String>, WatcherError> {
        let watchers = self.watchers.lock().map_err(|_| WatcherError::LockError)?;
        Ok(watchers.keys().cloned().collect())
    }

    pub fn stop_all(&self) -> Result<(), WatcherError> {
        let mut watchers = self.watchers.lock().map_err(|_| WatcherError::LockError)?;
        for (_, handle) in watchers.drain() {
            let _ = handle.stop_sender.send(());
        }
        Ok(())
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}
