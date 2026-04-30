use crate::error::Result;
use crate::popup::builder::PopupBuilder;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;
use tauri::{Manager, Wry};

pub(crate) struct PopupState {
    pub(crate) pending: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<bool>>>>,
    pub(crate) blocking_data: Arc<Mutex<HashMap<String, BlockingData>>>,
    pub(crate) sheet_labels: Arc<Mutex<HashSet<String>>>,
}

impl Default for PopupState {
    fn default() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            blocking_data: Arc::new(Mutex::new(HashMap::new())),
            sheet_labels: Arc::new(Mutex::new(HashSet::new())),
        }
    }
}

#[cfg(target_os = "windows")]
pub(crate) struct BlockingData {}

#[cfg(target_os = "macos")]
pub(crate) struct BlockingData {
    pub(crate) parent_nswindow: SendRetained,
    pub(crate) popup_nswindow: SendRetained,
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub(crate) struct BlockingData {}

#[cfg(target_os = "macos")]
pub(crate) struct SendRetained(pub(crate) objc2::rc::Retained<objc2::runtime::AnyObject>);

#[cfg(target_os = "macos")]
unsafe impl Send for SendRetained {}
#[cfg(target_os = "macos")]
unsafe impl Sync for SendRetained {}

pub struct PopupHandle {
    #[allow(dead_code)]
    pub(crate) window_label: String,
    pub(crate) completion: JoinHandle<bool>,
}

impl PopupHandle {
    #[allow(dead_code)]
    pub fn label(&self) -> &str {
        &self.window_label
    }

    pub async fn wait(self) -> bool {
        self.completion.await.unwrap_or(false)
    }
}

pub trait PopupExt {
    fn popup(&self, route: &str) -> PopupBuilder;
    async fn resolve_popup(&self, label: &str, resolved: bool) -> Result<()>;
}

impl<T: Manager<Wry> + Sync + Send> PopupExt for T {
    fn popup(&self, route: &str) -> PopupBuilder {
        PopupBuilder::new(self.app_handle().clone(), route.to_string())
    }

    async fn resolve_popup(&self, label: &str, resolved: bool) -> Result<()> {
        let state = self.state::<PopupState>();
        let sender = state
            .pending
            .lock()
            .map_err(|e| crate::error::Error::Lock(format!("popup state: {e}")))?
            .remove(label);

        if let Some(sender) = sender {
            let _ = sender.send(resolved);
        } else {
            log::debug!("Popup {} was already resolved or missing.", label);
        }

        Ok(())
    }
}
