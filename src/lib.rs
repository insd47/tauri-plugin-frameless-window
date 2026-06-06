mod commands;
pub mod error;
mod builder;
mod popup;

pub use builder::WebviewWindowBuilderExt;
pub use popup::{is_popup_window_label, PopupBuilder, PopupExt, PopupHandle};
use tauri::plugin::PluginApi;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, RunEvent, WindowEvent, Wry,
};

pub fn init() -> TauriPlugin<Wry> {
    Builder::<Wry>::new("frameless-window")
        .invoke_handler(tauri::generate_handler![
            commands::open_popup,
            commands::close_popup,
            commands::is_popup_sheet,
            commands::show_popup,
        ])
        .setup(setup)
        .on_event(on_event)
        .build()
}

fn setup(app: &AppHandle, _api: PluginApi<Wry, ()>) -> Result<(), Box<dyn std::error::Error>> {
    popup::init(app);
    Ok(())
}

fn on_event(app: &AppHandle, event: &RunEvent) {
    let RunEvent::WindowEvent { label, event, .. } = event else {
        return;
    };

    if !is_popup_window_label(label) {
        return;
    }

    match event {
        WindowEvent::CloseRequested { .. } => {
            log::debug!("Popup window close requested: {label}.");
        }
        WindowEvent::Destroyed => {
            let app = app.clone();
            let label = label.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(error) = app.resolve_popup(&label, false).await {
                    log::debug!("Failed to resolve destroyed popup {label}: {error}.");
                }
            });
        }
        _ => {}
    }
}
