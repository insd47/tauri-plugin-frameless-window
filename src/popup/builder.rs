use crate::error::{Error, Result};
use crate::builder::WebviewWindowBuilderExt;
use crate::popup::blocking;
use crate::popup::types::{PopupHandle, PopupState};
use crate::popup::{popup_window_label, POPUP_ROUTE_PREFIX};
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Wry};
use tokio::sync::oneshot;
use url::form_urlencoded::Serializer;

pub struct PopupBuilder {
    app: AppHandle<Wry>,
    route: String,
    args: Vec<(String, String)>,
    parent: Option<WebviewWindow<Wry>>,
    blocking: bool,
}

impl PopupBuilder {
    pub(crate) fn new(app: AppHandle<Wry>, route: String) -> Self {
        Self {
            app,
            route,
            args: Vec::new(),
            parent: None,
            blocking: false,
        }
    }

    pub fn args<I, K, V>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.args.extend(
            args.into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    pub fn parent(mut self, parent: WebviewWindow<Wry>) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn blocking(mut self) -> Self {
        self.blocking = true;
        self
    }

    #[allow(dead_code)]
    pub async fn open(self) -> Result<bool> {
        Ok(self.open_detached()?.wait().await)
    }

    pub fn open_detached(self) -> Result<PopupHandle> {
        let window_label = popup_window_label();
        let (tx, rx) = oneshot::channel::<bool>();

        {
            let state = self.app.state::<PopupState>();
            state
                .pending
                .lock()
                .map_err(|e| Error::Lock(format!("popup state: {e}")))?
                .insert(window_label.clone(), tx);
        }

        let url = popup_url(&self.route, &self.args);
        let popup = match self.build_window(&window_label, url) {
            Ok(window) => window,
            Err(err) => {
                let _ = remove_pending_popup(&self.app, &window_label);
                return Err(err.into());
            }
        };

        let _ = popup.hide();

        let is_blocking = self.blocking;
        let parent_label = self.parent.as_ref().map(|p| p.label().to_string());
        let app = self.app.clone();
        let wait_label = window_label.clone();

        if is_blocking {
            if let Some(parent) = &self.parent {
                blocking::disable_parent(parent, &popup);
            }
        }

        let completion = spawn(async move {
            let resolved = rx.await.unwrap_or(false);

            if is_blocking {
                if let Some(ref label) = parent_label {
                    if let Some(parent) = app.get_webview_window(label) {
                        blocking::enable_parent(&parent);
                    }
                }
            }

            if let Some(window) = app.get_webview_window(&wait_label) {
                let _ = window.destroy();
            }

            resolved
        });

        Ok(PopupHandle {
            window_label,
            completion,
        })
    }

    fn build_window(&self, label: &str, url: String) -> tauri::Result<WebviewWindow> {
        let mut builder =
            WebviewWindowBuilder::frameless(&self.app, label, WebviewUrl::App(url.into()))
                .maximizable(false)
                .minimizable(false)
                .resizable(false)
                .center()
                .inner_size(320.0, 720.0);

        if let Some(parent) = &self.parent {
            if !(self.blocking && cfg!(target_os = "macos")) {
                builder = builder.parent(parent)?;
            }
        }

        builder.build()
    }
}

fn popup_url(route: &str, args: &[(String, String)]) -> String {
    let mut url = format!("{POPUP_ROUTE_PREFIX}/{route}");

    if args.is_empty() {
        return url;
    }

    let query = Serializer::new(String::new())
        .extend_pairs(
            args.iter()
                .map(|(key, value)| (key.as_str(), value.as_str())),
        )
        .finish();

    url.push('?');
    url.push_str(&query);
    url
}

fn remove_pending_popup(app: &AppHandle<Wry>, id: &str) -> Result<()> {
    let state = app.state::<PopupState>();
    state
        .pending
        .lock()
        .map_err(|e| Error::Lock(format!("popup state: {e}")))?
        .remove(id);
    Ok(())
}
