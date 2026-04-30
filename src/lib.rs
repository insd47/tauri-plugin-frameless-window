mod builder;
mod commands;
pub mod error;
mod popup;

use tauri::{
    plugin::{Builder, TauriPlugin},
    Wry,
};

pub use builder::WebviewWindowBuilderExt;
pub use popup::{is_popup_window_label, PopupBuilder, PopupExt, PopupHandle};

pub fn init() -> TauriPlugin<Wry> {
    Builder::<Wry>::new("frameless-window")
        .invoke_handler(tauri::generate_handler![
            commands::open_popup,
            commands::close_popup,
            commands::is_popup_sheet,
            commands::show_popup,
        ])
        .setup(|app, _api| {
            popup::init(app);
            Ok(())
        })
        .build()
}
