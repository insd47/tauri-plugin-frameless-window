use crate::error::Result;
use crate::popup::{is_sheet, show_sheet, PopupExt};
use std::collections::HashMap;
use tauri::{command, Manager, Window};

#[command]
pub async fn close_popup(window: Window, resolved: bool) -> Result<()> {
    let app = window.app_handle().clone();
    let label = window.label().to_string();

    app.resolve_popup(&label, resolved).await?;
    Ok(())
}

#[command]
pub async fn open_popup(
    window: Window,
    route: String,
    args: Option<HashMap<String, String>>,
    blocking: Option<bool>,
    detach: Option<bool>,
) -> Result<bool> {
    let app = window.app_handle().clone();
    let parent = app
        .get_webview_window(window.label())
        .ok_or_else(|| "Failed to get parent window")?;

    let mut builder = app.popup(&route);

    if let Some(args) = args {
        builder = builder.args(args);
    }

    if !detach.unwrap_or(false) {
        builder = builder.parent(parent);
    }

    if blocking.unwrap_or(false) {
        builder = builder.blocking();
    }

    builder.open().await
}

#[command]
pub fn is_popup_sheet(window: Window) -> bool {
    let Some(webview_window) = window.app_handle().get_webview_window(window.label()) else {
        return false;
    };
    is_sheet(&webview_window)
}

#[command]
pub fn show_popup(window: Window) -> Result<()> {
    let Some(webview_window) = window.app_handle().get_webview_window(window.label()) else {
        return Ok(());
    };

    if is_sheet(&webview_window) {
        show_sheet(&webview_window);
    } else {
        let _ = webview_window.show();
    }

    Ok(())
}
